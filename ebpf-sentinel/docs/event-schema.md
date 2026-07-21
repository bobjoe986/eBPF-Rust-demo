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
