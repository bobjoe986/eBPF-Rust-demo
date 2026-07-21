# Architecture

## Components

### sentinel-ebpf
Kernel-resident telemetry probes written with Aya. Responsibilities are deliberately narrow: capture bounded metadata, apply cheap filters, update small BPF maps/counters, and publish events through a ring buffer.

### sentinel-common
Defines the stable kernel/userspace ABI. Structures must be fixed-size and C-compatible.

### sentinel-agent
Loads/attaches programs, consumes the ring buffer, normalizes events, performs host/container enrichment, maintains process/FD metadata where required, and exports telemetry.

### sentinel-anomaly
Consumes normalized telemetry and creates behavioral windows. It performs rules/statistical anomaly scoring first and invokes an optional local SLM only for selected windows.

## Trust boundary
The eBPF verifier and kernel interface are treated as strict constraints. Raw kernel pointers never cross into userspace. The model has no direct kernel access and cannot modify BPF policy in v0.x.

## Detection philosophy
Ground truth first, reasoning second:
1. eBPF observes activity.
2. Userspace correlates it.
3. Deterministic/statistical features identify interesting windows.
4. SLM supplies context/classification/summary.

This prevents model latency and nondeterminism from affecting collection reliability.

## Output example
```json
{"schema_version":1,"event":"process_exec","ts_ns":123456789,"pid":4242,"tgid":4242,"uid":1000,"comm":"curl","exe":"/usr/bin/curl"}
```
