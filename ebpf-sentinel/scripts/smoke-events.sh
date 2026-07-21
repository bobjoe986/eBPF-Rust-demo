#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
agent="${SENTINEL_AGENT:-$repo_root/target/release/sentinel-agent}"
ebpf="${SENTINEL_EBPF_OBJECT:-$repo_root/target/bpfel-unknown-none/release/sentinel-ebpf}"
output="$(mktemp)"
diagnostics="$(mktemp)"
agent_pid=""
mounted_tracefs="false"

cleanup() {
    if [[ -n "$agent_pid" ]] && kill -0 "$agent_pid" 2>/dev/null; then
        kill -INT "$agent_pid"
        wait "$agent_pid" || true
    fi
    if [[ "$mounted_tracefs" == "true" ]]; then
        umount /sys/kernel/tracing || true
    fi
    rm -f "$output" "$diagnostics"
}
trap cleanup EXIT

[[ $EUID -eq 0 ]] || { echo "run this smoke test as root" >&2; exit 1; }
[[ -x "$agent" ]] || { echo "missing agent binary: $agent" >&2; exit 1; }
[[ -f "$ebpf" ]] || { echo "missing eBPF object: $ebpf" >&2; exit 1; }

tracepoint_id=""
for candidate in \
    /sys/kernel/tracing/events/sched/sched_process_exec/id \
    /sys/kernel/debug/tracing/events/sched/sched_process_exec/id; do
    if [[ -r "$candidate" ]]; then
        tracepoint_id="$candidate"
        break
    fi
done

if [[ -z "$tracepoint_id" ]]; then
    if ! mount -t tracefs tracefs /sys/kernel/tracing; then
        echo "tracefs is unavailable and could not be mounted." >&2
        echo "On a container, run with CAP_SYS_ADMIN/--privileged and expose the host tracefs." >&2
        echo "On a host, mount it with: sudo mount -t tracefs tracefs /sys/kernel/tracing" >&2
        exit 1
    fi
    mounted_tracefs="true"
    tracepoint_id=/sys/kernel/tracing/events/sched/sched_process_exec/id
fi

[[ -r "$tracepoint_id" ]] || {
    echo "sched:sched_process_exec is not available under the mounted tracefs" >&2
    exit 1
}

"$agent" --ebpf-object "$ebpf" --output jsonl >"$output" 2>"$diagnostics" &
agent_pid=$!

for _ in {1..50}; do
    grep -q 'probe attached' "$diagnostics" && break
    kill -0 "$agent_pid" 2>/dev/null || { cat "$diagnostics" >&2; exit 1; }
    sleep 0.1
done
grep -q 'probe attached' "$diagnostics" || { echo "agent did not attach" >&2; exit 1; }

/bin/echo hello >/dev/null

for _ in {1..50}; do
    grep -q '"event":"process_exec".*"comm":"echo".*"exe":"/bin/echo"' "$output" && break
    sleep 0.1
done
grep -q '"event":"process_exec".*"comm":"echo".*"exe":"/bin/echo"' "$output"

kill -INT "$agent_pid"
wait "$agent_pid"
agent_pid=""
grep -Eq 'emitted_events=[1-9][0-9]*; dropped_events=[0-9]+' "$diagnostics"
echo "PASS: observed /bin/echo process_exec event"
cat "$diagnostics" >&2
