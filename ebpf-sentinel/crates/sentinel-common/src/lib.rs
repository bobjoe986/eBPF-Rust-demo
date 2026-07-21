#![no_std]

pub const SCHEMA_VERSION: u16 = 1;

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    ProcessExec = 1,
    ProcessFork = 2,
    ProcessExit = 3,
    FileOpen = 10,
    FileCreate = 11,
    FileUnlink = 12,
    FileRename = 13,
    FileReadAggregate = 14,
    FileWriteAggregate = 15,
    NetConnect = 20,
    NetAccept = 21,
}

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
