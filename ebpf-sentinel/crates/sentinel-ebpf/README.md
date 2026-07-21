# sentinel-ebpf

Aya eBPF producer for the M1 `sched_process_exec` tracepoint. It emits only bounded process metadata
through `EVENTS` and records ring-buffer publication failures in `DROP_COUNT`.

This crate is excluded from the host workspace because it builds for `bpfel-unknown-none` with a
nightly toolchain. See `docs/testing.md` for exact commands.
