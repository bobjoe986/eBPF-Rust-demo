# Testing

## Static/userspace
Run where applicable:
```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The eBPF crate may require target/toolchain-specific commands; document the exact commands once the Aya template/toolchain is initialized.

## Milestone 1 commands
Prerequisites are modern x86_64 Linux (ring buffers require Linux 5.8+), BTF at
`/sys/kernel/btf/vmlinux`, root or sufficient BPF/perf capabilities, the nightly Rust toolchain,
the `bpfel-unknown-none` target, `bpf-linker`, and a mounted tracefs. The smoke script checks
`/sys/kernel/tracing` and `/sys/kernel/debug/tracing`; when neither is usable, it temporarily mounts
tracefs at `/sys/kernel/tracing` and unmounts it during cleanup.

Commands used to build and check M1:
```bash
rustup toolchain install nightly --component rust-src
cargo install bpf-linker
CARGO_TARGET_DIR=target cargo +nightly build --manifest-path crates/sentinel-ebpf/Cargo.toml \
  --target bpfel-unknown-none --release -Z build-std=core
cargo build --workspace --release
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=target cargo +nightly clippy --manifest-path crates/sentinel-ebpf/Cargo.toml \
  --target bpfel-unknown-none --release -Z build-std=core -- -D warnings
```

Run the automated M1 acceptance check after building both binaries:
```bash
sudo ./scripts/smoke-events.sh
```

If tracefs cannot be mounted, a container needs `CAP_SYS_ADMIN` (commonly `--privileged` for local
development) and access to the host tracefs. On a host it can be mounted manually:
```bash
sudo mount -t tracefs tracefs /sys/kernel/tracing
```

Run the agent manually and generate an event from another terminal:
```bash
sudo ./target/release/sentinel-agent \
  --ebpf-object target/bpfel-unknown-none/release/sentinel-ebpf \
  --output jsonl
/bin/echo hello
```

Telemetry JSONL is written only to stdout. Attachment status, malformed-event diagnostics, and
the final `emitted_events` and `dropped_events` counters are written to stderr. The kernel producer increments
`DROP_COUNT` when publishing to the ring buffer fails; host execution is never blocked.

## Smoke activity
`scripts/smoke-events.sh` currently starts the M1 agent, executes `/bin/echo hello`, and asserts a
matching process-exec JSON event. File and network activity will be added only in their milestones.

Each milestone should assert only the telemetry implemented at that milestone.

## Runtime verification
Record:
- kernel version;
- architecture;
- BTF availability;
- effective capabilities/root requirement;
- loaded BPF programs/maps;
- event count;
- dropped-event count.

## Negative tests
- malformed/truncated strings must not panic userspace;
- ring-buffer pressure must increment drop metrics rather than block host workloads;
- model endpoint unavailable must not stop collection.
