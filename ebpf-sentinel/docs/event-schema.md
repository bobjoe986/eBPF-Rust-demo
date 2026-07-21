# Event Schema

## Kernel ABI principles
- `#[repr(C)]`
- fixed-size fields only
- explicit schema version
- no `String`, `Vec`, references, or heap pointers
- conservative path/string limits
- event-specific structs start with a common header

## Proposed header
```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EventHeader {
    pub schema_version: u16,
    pub event_kind: u16,
    pub cpu: u32,
    pub timestamp_ns: u64,
    pub pid: u32,
    pub tgid: u32,
    pub uid: u32,
    pub gid: u32,
}
```

## Event kinds
Initial values should be explicit and stable:
- 1 ProcessExec
- 2 ProcessFork
- 3 ProcessExit
- 10 FileOpen
- 11 FileCreate
- 12 FileUnlink
- 13 FileRename
- 14 FileReadAggregate
- 15 FileWriteAggregate
- 20 NetConnect
- 21 NetAccept

Do not reuse numeric values after release.

## ProcessExecEvent (schema version 1)
`ProcessExecEvent` appends these fixed-size fields to `EventHeader`:

- `comm: [u8; 16]` - kernel task name, NUL-terminated when shorter;
- `exe: [u8; 256]` - executable filename reported by `sched_process_exec`, NUL-terminated and truncated when necessary.

Userspace rejects samples with the wrong size, schema version, or event kind. Byte arrays are
converted with lossy UTF-8 so malformed kernel data cannot panic the event loop.

Normalized JSONL uses this shape:
```json
{"schema_version":1,"event":"process_exec","ts_ns":123456789,"pid":4242,"tgid":4242,"uid":1000,"gid":1000,"cpu":2,"comm":"echo","exe":"/bin/echo"}
```
