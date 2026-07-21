# eBPF Sentinel

A Rust/Aya Linux security sensor for process, file, and network telemetry, with userspace correlation and an optional local small-language-model analysis layer.

## Intended data path

```text
Linux kernel
  -> Aya eBPF programs
  -> BPF ring buffer
  -> sentinel-agent
  -> normalization + enrichment
  -> sentinel-anomaly
  -> deterministic scoring
  -> optional local SLM
  -> JSONL/security findings
```

## Initial build order
1. Process exec end-to-end.
2. Fork/exit.
3. File open/create/unlink/rename.
4. Filtered/aggregated file read/write.
5. TCP connect/accept.
6. Correlation and anomaly scoring.
7. Local SLM enrichment.

See `AGENTS.md` and `docs/milestones.md` before asking Codex to implement features.

## Milestone 1

The implemented vertical slice attaches an Aya tracepoint to `sched:sched_process_exec`, publishes
the versioned fixed-size `ProcessExecEvent` through a ring buffer, and emits normalized JSONL from
`sentinel-agent`. Ring-buffer publication failures are counted in the `DROP_COUNT` map.

Build, privilege, runtime, smoke-test, and verification commands are documented in
[`docs/testing.md`](docs/testing.md). File and network hooks are intentionally not present.

## Recommended first Codex task

> Read AGENTS.md and docs/architecture.md. Implement Milestone 1 only: a minimal Aya process-exec vertical slice that emits a versioned shared event through a ring buffer and prints normalized JSONL in userspace. Add a smoke-test script and document all commands used. Do not add file/network hooks yet.
