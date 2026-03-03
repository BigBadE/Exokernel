//! Process management
//!
//! High-level process management utilities built on top of the raw syscalls.

use exo_shared::{CapabilityHandle, SysError};
use crate::syscall;
use crate::elf::{self, ElfError};

/// Process handle - wrapper around a process capability
#[derive(Debug, Clone, Copy)]
pub struct Process {
    cap: CapabilityHandle,
}

impl Process {
    /// Create a process handle from a capability
    pub fn from_cap(cap: CapabilityHandle) -> Self {
        Self { cap }
    }

    /// Get the underlying capability
    pub fn cap(&self) -> CapabilityHandle {
        self.cap
    }

    /// Spawn a new process from ELF data
    pub fn spawn(elf_data: &[u8]) -> Result<Self, SpawnError> {
        let cap = elf::spawn(elf_data)?;
        Ok(Self { cap })
    }

    /// Create an empty process (for manual setup)
    pub fn create_empty() -> Result<Self, SysError> {
        let cap = syscall::process_create(0, 0, 0)?;
        Ok(Self { cap })
    }

    /// Start a process with given entry point and stack
    pub fn start(&self, entry_point: u64, stack_top: u64) -> Result<(), SysError> {
        syscall::process_start(self.cap, entry_point, stack_top)
    }
}

/// Error spawning a process
#[derive(Debug, Clone, Copy)]
pub enum SpawnError {
    /// ELF parsing/loading error
    Elf(ElfError),
    /// Syscall error
    Sys(SysError),
}

impl From<ElfError> for SpawnError {
    fn from(e: ElfError) -> Self {
        SpawnError::Elf(e)
    }
}

impl From<SysError> for SpawnError {
    fn from(e: SysError) -> Self {
        SpawnError::Sys(e)
    }
}

// =============================================================================
// Current Process Utilities
// =============================================================================

/// Get current process ID
pub fn current_pid() -> u64 {
    syscall::get_pid()
}

/// Exit the current process
pub fn exit(code: i32) -> ! {
    syscall::exit(code)
}

/// Yield CPU to other processes
pub fn yield_now() {
    syscall::yield_now()
}

/// Set upcall handler for this process
pub fn set_upcall_handler(handler: u64, stack: u64) -> Result<(), SysError> {
    syscall::set_upcall(handler, stack)
}
