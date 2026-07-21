# Milestones

## M0 — Toolchain/bootstrap
- initialize Aya-compatible workspace/toolchains;
- verify target kernel has required BPF/BTF support;
- document build/run commands.

## M1 — Process exec vertical slice
- shared header + ProcessExec event;
- one process-exec hook;
- ring buffer;
- userspace loader/consumer;
- JSONL output;
- event/drop counters;
- smoke test.

## M2 — Fork/exit and lineage
- fork + exit telemetry;
- userspace process table;
- parent/child correlation;
- stale-state cleanup.

## M3 — File metadata lifecycle
- open/create/unlink/rename;
- path metadata where reliably available;
- explicit truncation indicator for bounded paths;
- tests using temp files.

## M4 — Read/write aggregation
- no per-syscall firehose by default;
- aggregate operations/bytes by process+file+window;
- define FD/path lifecycle limitations or implement close/dup/fork handling;
- configurable include/exclude paths.

## M5 — Network
- TCP connect/accept;
- IPv4 + IPv6;
- normalized host-byte-order ports in userspace;
- no payload capture.

## M6 — Behavioral correlation
Create time-window features such as:
- rare parent-child process;
- new executable/path;
- sensitive path access;
- new destination/port;
- web-service -> shell -> network sequence;
- burst of file writes followed by execution.

## M7 — Local SLM enrichment
- define versioned inference request/response schema;
- local-only endpoint configuration;
- timeout/circuit breaker;
- model failure never interrupts collection;
- only suspicious aggregated windows reach the model.

## M8 — Container/Kubernetes awareness
- cgroup/container identification;
- Kubernetes metadata enrichment in userspace;
- DaemonSet deployment design;
- least-privilege capability/host-mount review.
