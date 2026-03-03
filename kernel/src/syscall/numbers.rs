/// System call numbers
///
/// These follow a Linux-like convention for familiarity,
/// but with our own subset of calls.

pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_MMAP: u64 = 9;
pub const SYS_MUNMAP: u64 = 11;
pub const SYS_BRK: u64 = 12;
pub const SYS_EXIT: u64 = 60;
pub const SYS_GETPID: u64 = 39;

/// Debug syscall for testing
pub const SYS_DEBUG_PRINT: u64 = 1000;
