# AGENTS.md — eBPF Sentinel

## Mission
Build a small, reliable Linux security telemetry sensor in Rust using Aya/eBPF. The sensor observes process, file, and network behavior, correlates events in userspace, and later supplies suspicious behavioral summaries to a local small language model (SLM) for contextual analysis.

## Non-negotiable architecture
1. **Kernel/eBPF code collects only bounded telemetry.** Do not perform LLM inference, unbounded parsing, complex correlation, DNS resolution, filesystem traversal, or expensive string processing in eBPF.
2. **Userspace owns enrichment and correlation.** PID/process lineage, container context, FD/path state, time-window aggregation, baselining, anomaly scoring, and SLM invocation belong in `sentinel-agent` / `sentinel-anomaly`.
3. **Use typed shared event structures** from `sentinel-common`. Kernel/userspace layouts must be `#[repr(C)]` and avoid heap-backed types.
4. **Prefer stable kernel hooks.** Start with tracepoints where practical. Use kprobes/fentry/LSM only when a requirement cannot be met cleanly with tracepoints and document the portability tradeoff.
5. **Use a ring buffer** for kernel-to-userspace telemetry. Track dropped/reservation-failed events with counters.
6. **Never capture file contents, packet payloads, environment contents, secrets, or credentials by default.** Metadata first.
7. **Fail open for host workloads.** Sensor failures must not intentionally block normal host activity in v0.x.

## Repository map
- `crates/sentinel-common` — shared event ABI, event kinds, bounded structs.
- `crates/sentinel-ebpf` — Aya eBPF programs and maps.
- `crates/sentinel-agent` — loader, attachment, ring-buffer consumer, enrichment, JSON output.
- `crates/sentinel-anomaly` — correlation, baselines, scoring, optional local SLM client.
- `docs/architecture.md` — architecture and trust boundaries.
- `docs/event-schema.md` — event ABI and normalized userspace schema.
- `docs/milestones.md` — required incremental delivery plan.
- `docs/testing.md` — test strategy and acceptance gates.

## Required development sequence
Do **not** implement all telemetry sources at once.

### Vertical Slice 1 — Process execution
Implement one complete path first:
`process exec hook -> Event -> RingBuf -> userspace -> normalized JSON`.

Acceptance gate:
- builds cleanly;
- eBPF program loads on the supported test kernel;
- executing `/bin/echo hello` produces one sensible process event;
- PID/TGID/UID/comm and executable identity are represented where available;
- malformed/partial data does not panic userspace.

### Vertical Slice 2 — Process lifecycle
Add fork and exit. Add process lineage state in userspace, not in the kernel unless minimal state is clearly justified.

### Vertical Slice 3 — File metadata
Add file open/create/unlink/rename metadata. Do not start with every `read(2)`/`write(2)` call.

### Vertical Slice 4 — File read/write
Add read/write only after filtering/aggregation exists. Prefer aggregating counts/bytes per `(process, file, time window)` rather than emitting every call. If FD-to-path state is needed, clearly handle close/dup/fork lifecycle or document limitations.

### Vertical Slice 5 — Network
Add outbound connect and inbound accept with address family, protocol, IPs, ports, PID/TGID, and timestamp. Packet payload capture is out of scope.

### Vertical Slice 6 — Correlation/anomaly scoring
Implement deterministic features before SLM integration: rare parent-child pair, unusual destination, sensitive path access, new executable, burst behavior, and cross-domain sequences.

### Vertical Slice 7 — Local SLM
Call a local inference endpoint only for pre-correlated suspicious windows. The model is an analyst/enrichment layer, not the primary high-throughput detector.

## eBPF coding rules
- Keep programs verifier-friendly and bounded.
- No recursion.
- No unbounded loops.
- Avoid large stack allocations; keep the BPF stack limit in mind.
- Prefer fixed-size arrays and bounded copies.
- Check every helper/map/ring-buffer operation that can fail.
- Treat kernel pointers as unsafe and minimize unsafe regions.
- Do not assume a field offset across kernels without BTF/CO-RE-compatible reasoning.
- Add comments explaining verifier-sensitive code, not obvious Rust syntax.
- Emit the smallest event needed; enrich later.
- Add event versioning before changing a shared ABI incompatibly.

## Rust userspace rules
- Stable Rust where possible; use the toolchain required by Aya for the eBPF target only.
- `cargo fmt` and `cargo clippy` must pass for userspace crates.
- No `unwrap()`/`expect()` in long-running event loops except for provably impossible states with comments.
- Use structured errors (`anyhow` at application boundaries is acceptable; typed errors for reusable libraries).
- Use `tracing` for diagnostics. Never mix operational logs with telemetry JSON on stdout.
- stdout: normalized telemetry when `--output jsonl` is selected.
- stderr/log sink: diagnostics.
- Gracefully handle Ctrl-C/SIGTERM and detach/close resources.

## Event ABI rules
Every kernel event begins with a common header containing at minimum:
- schema version;
- event kind;
- monotonic timestamp;
- PID;
- TGID;
- UID;
- GID where practical;
- CPU id where useful.

Use bounded byte arrays for `comm`, paths, and addresses. Userspace converts them to safe strings/IP types.

## Privacy/security defaults
Do not collect:
- file contents;
- command environment values;
- authentication tokens;
- TLS plaintext;
- packet payloads;
- arbitrary process memory.

Command-line arguments are optional and must be explicitly enabled if implemented later.

## Performance targets for v0.1
These are engineering targets, not guaranteed kernel limits:
- negligible overhead when relevant events are idle;
- no unbounded userspace queue growth;
- bounded channel/ring capacities;
- visible dropped-event counters;
- backpressure strategy documented;
- filtering before SLM submission.

## Testing requirements
For every vertical slice:
1. unit-test pure userspace parsing/correlation;
2. add an integration smoke script that generates known Linux activity;
3. verify expected telemetry;
4. verify unrelated activity does not crash the sensor;
5. document required privileges/kernel features.

Do not claim a hook is portable until tested or documented against the supported kernel matrix.

## Supported scope for initial development
Target modern x86_64 Linux with BTF enabled. Keep ARM64 in mind, but do not block v0.1 on it. Container/Kubernetes support comes after host-mode correctness.

## Container/Kubernetes direction
Eventually run the sensor as a privileged/node-level component (for Kubernetes, normally a DaemonSet) with the minimum host access required for BPF. Run SLM inference as a separate process/service or container. Do not place one model per monitored application container.

## Local SLM contract
The anomaly layer should send compact behavioral summaries such as:
```json
{
  "window_seconds": 10,
  "process_chain": ["nginx", "sh", "curl"],
  "file_activity": [{"path":"/etc/passwd","op":"read","count":1}],
  "network": [{"direction":"outbound","dst":"203.0.113.10:443"}],
  "signals": ["rare_child_process", "new_destination"]
}
```
Do not stream raw syscall firehoses into the model.

## Codex task behavior
Before modifying code:
1. read this file;
2. read the relevant file under `docs/`;
3. inspect existing implementation and tests;
4. make the smallest coherent change that completes the requested slice.

After modifying code:
1. run formatting;
2. run applicable tests;
3. run clippy where possible;
4. report exactly what was tested and what could not be tested;
5. update docs when architecture/ABI/requirements changed.

Never silently weaken a test or remove telemetry to make CI pass. Explain kernel/verifier/environment limitations explicitly.
