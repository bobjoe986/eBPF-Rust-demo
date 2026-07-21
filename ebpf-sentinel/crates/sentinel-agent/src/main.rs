use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{anyhow, Context, Result};
use aya::{maps::RingBuf, programs::TracePoint, Ebpf};
use clap::{Parser, ValueEnum};
use sentinel_common::{EventKind, ProcessExecEvent, SCHEMA_VERSION};
use serde::Serialize;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Output {
    Jsonl,
}

#[derive(Debug, Parser)]
#[command(about = "Aya process-exec telemetry agent")]
struct Args {
    #[arg(
        long,
        default_value = "target/bpfel-unknown-none/release/sentinel-ebpf"
    )]
    ebpf_object: PathBuf,
    #[arg(long, value_enum, default_value_t = Output::Jsonl)]
    output: Output,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct NormalizedExec {
    schema_version: u16,
    event: &'static str,
    ts_ns: u64,
    pid: u32,
    tgid: u32,
    uid: u32,
    gid: u32,
    cpu: u32,
    comm: String,
    exe: String,
}

fn bounded_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn normalize(bytes: &[u8]) -> Result<NormalizedExec> {
    if bytes.len() != size_of::<ProcessExecEvent>() {
        return Err(anyhow!(
            "unexpected event size: got {}, expected {}",
            bytes.len(),
            size_of::<ProcessExecEvent>()
        ));
    }

    // Ring-buffer samples are byte-aligned; read_unaligned avoids imposing Rust alignment on them.
    let event = unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast::<ProcessExecEvent>()) };
    if event.header.schema_version != SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported schema version {}",
            event.header.schema_version
        ));
    }
    if event.header.event_kind != EventKind::ProcessExec as u16 {
        return Err(anyhow!(
            "unsupported event kind {}",
            event.header.event_kind
        ));
    }

    Ok(NormalizedExec {
        schema_version: event.header.schema_version,
        event: "process_exec",
        ts_ns: event.header.timestamp_ns,
        pid: event.header.pid,
        tgid: event.header.tgid,
        uid: event.header.uid,
        gid: event.header.gid,
        cpu: event.header.cpu,
        comm: bounded_string(&event.comm),
        exe: bounded_string(&event.exe),
    })
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut ebpf = Ebpf::load_file(&args.ebpf_object)
        .with_context(|| format!("loading {}", args.ebpf_object.display()))?;
    let program: &mut TracePoint = ebpf
        .program_mut("sentinel_process_exec")
        .context("missing sentinel_process_exec program")?
        .try_into()?;
    program.load().context("loading process-exec tracepoint")?;
    program.attach("sched", "sched_process_exec").context(
        "attaching sched:sched_process_exec (tracefs must be mounted at \
             /sys/kernel/tracing or /sys/kernel/debug/tracing)",
    )?;

    let mut ring = RingBuf::try_from(ebpf.take_map("EVENTS").context("missing EVENTS map")?)?;
    let running = Arc::new(AtomicBool::new(true));
    let signal_flag = Arc::clone(&running);
    ctrlc::set_handler(move || signal_flag.store(false, Ordering::Relaxed))?;

    eprintln!("sentinel-agent: process-exec probe attached");
    let mut emitted_events = 0_u64;
    while running.load(Ordering::Relaxed) {
        let mut poll_fd = libc::pollfd {
            fd: std::os::fd::AsRawFd::as_raw_fd(&ring),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: poll_fd points to one initialized pollfd for the duration of this call.
        let polled = unsafe { libc::poll(&mut poll_fd, 1, 250) };
        if polled < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::Interrupted {
                return Err(error).context("polling EVENTS ring buffer");
            }
        }
        while let Some(sample) = ring.next() {
            match normalize(&sample) {
                Ok(event) => {
                    println!("{}", serde_json::to_string(&event)?);
                    emitted_events += 1;
                }
                Err(error) => eprintln!("sentinel-agent: discarded malformed event: {error:#}"),
            }
        }
    }

    let drops = aya::maps::PerCpuArray::<_, u64>::try_from(
        ebpf.take_map("DROP_COUNT")
            .context("missing DROP_COUNT map")?,
    )?
    .get(&0, 0)?
    .iter()
    .sum::<u64>();
    eprintln!("sentinel-agent: stopped; emitted_events={emitted_events}; dropped_events={drops}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_common::{EventHeader, COMM_LEN, EXE_LEN};

    fn bytes_of(event: &ProcessExecEvent) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref(event).cast::<u8>(),
                size_of::<ProcessExecEvent>(),
            )
        }
    }

    #[test]
    fn normalizes_exec_and_lossy_strings() {
        let mut comm = [0; COMM_LEN];
        comm[..4].copy_from_slice(b"echo");
        let mut exe = [0; EXE_LEN];
        exe[..10].copy_from_slice(b"/bin/echo\0");
        let event = ProcessExecEvent {
            header: EventHeader {
                schema_version: SCHEMA_VERSION,
                event_kind: EventKind::ProcessExec as u16,
                cpu: 2,
                timestamp_ns: 42,
                pid: 10,
                tgid: 11,
                uid: 1000,
                gid: 1001,
            },
            comm,
            exe,
        };

        let normalized = normalize(bytes_of(&event)).unwrap();
        assert_eq!(normalized.comm, "echo");
        assert_eq!(normalized.exe, "/bin/echo");
        assert_eq!(normalized.ts_ns, 42);
    }

    #[test]
    fn rejects_truncated_and_unknown_events() {
        assert!(normalize(&[0; 3]).is_err());

        let event = ProcessExecEvent {
            header: EventHeader {
                schema_version: SCHEMA_VERSION + 1,
                event_kind: EventKind::ProcessExec as u16,
                cpu: 0,
                timestamp_ns: 0,
                pid: 0,
                tgid: 0,
                uid: 0,
                gid: 0,
            },
            comm: [0; COMM_LEN],
            exe: [0; EXE_LEN],
        };
        assert!(normalize(bytes_of(&event)).is_err());
    }
}
