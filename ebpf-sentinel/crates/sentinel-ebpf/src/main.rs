#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::{
        bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_get_current_uid_gid,
        bpf_get_smp_processor_id, bpf_ktime_get_ns, bpf_probe_read_kernel_str_bytes,
    },
    macros::{map, tracepoint},
    maps::{PerCpuArray, RingBuf},
    programs::TracePointContext,
    EbpfContext,
};
use sentinel_common::{EventHeader, EventKind, ProcessExecEvent, SCHEMA_VERSION};

#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(256 * 1024, 0);

#[map]
static DROP_COUNT: PerCpuArray<u64> = PerCpuArray::with_max_entries(1, 0);

#[tracepoint]
pub fn sentinel_process_exec(ctx: TracePointContext) -> u32 {
    match try_process_exec(ctx) {
        Ok(()) => 0,
        Err(code) => code as u32,
    }
}

fn try_process_exec(ctx: TracePointContext) -> Result<(), i32> {
    let pid_tgid = bpf_get_current_pid_tgid();
    let uid_gid = bpf_get_current_uid_gid();
    let mut event = ProcessExecEvent {
        header: EventHeader {
            schema_version: SCHEMA_VERSION,
            event_kind: EventKind::ProcessExec as u16,
            cpu: unsafe { bpf_get_smp_processor_id() },
            timestamp_ns: unsafe { bpf_ktime_get_ns() },
            pid: pid_tgid as u32,
            tgid: (pid_tgid >> 32) as u32,
            uid: uid_gid as u32,
            gid: (uid_gid >> 32) as u32,
        },
        comm: bpf_get_current_comm().unwrap_or([0; 16]),
        exe: [0; 256],
    };

    // sched_process_exec stores filename as a __data_loc field after the common 8-byte header.
    let filename_loc: u32 = unsafe { ctx.read_at(8)? };
    let filename_offset = (filename_loc & 0xffff) as usize;
    let filename_ptr = unsafe { ctx.as_ptr().add(filename_offset).cast::<u8>() };
    let _ = unsafe { bpf_probe_read_kernel_str_bytes(filename_ptr, &mut event.exe) };

    if EVENTS.output::<ProcessExecEvent>(&event, 0).is_err() {
        increment_drop_count();
    }
    Ok(())
}

fn increment_drop_count() {
    if let Some(count) = DROP_COUNT.get_ptr_mut(0) {
        unsafe { *count += 1 };
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
