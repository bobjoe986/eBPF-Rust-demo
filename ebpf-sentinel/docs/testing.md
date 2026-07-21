# Testing

## Static/userspace
Run where applicable:
```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The eBPF crate may require target/toolchain-specific commands; document the exact commands once the Aya template/toolchain is initialized.

## Smoke activity
`scripts/smoke-events.sh` should eventually generate:
- process exec and exit;
- temp file create/read/write/rename/delete;
- localhost TCP connect/accept.

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
