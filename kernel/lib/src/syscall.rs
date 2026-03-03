//! High-level syscall wrappers
//!
//! This module provides safe Rust wrappers around the raw exokernel syscalls.

use exo_shared::{CapabilityHandle, SyscallNumber, SysError};
use crate::{syscall0, syscall1, syscall2, syscall3};

// =============================================================================
// Debug Syscalls
// =============================================================================

/// Debug print syscall - prints a message to the kernel console
pub fn debug_print(msg: &str) -> Result<(), SysError> {
    let ret = unsafe {
        syscall2(
            SyscallNumber::DebugPrint as u64,
            msg.as_ptr() as u64,
            msg.len() as u64,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Debug dump capabilities
pub fn debug_dump_caps() -> Result<(), SysError> {
    let ret = unsafe { syscall0(SyscallNumber::DebugDumpCaps as u64) };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

// =============================================================================
// Process Syscalls
// =============================================================================

/// Get current process ID
pub fn get_pid() -> u64 {
    let ret = unsafe { syscall0(SyscallNumber::ProcessGetPid as u64) };
    ret as u64
}

/// Exit process with exit code
pub fn exit(code: i32) -> ! {
    unsafe {
        syscall1(SyscallNumber::ProcessExit as u64, code as u64);
    }
    // Should never reach here
    loop {}
}

/// Yield CPU to scheduler
pub fn yield_now() {
    unsafe {
        syscall0(SyscallNumber::ProcessYield as u64);
    }
}

/// Create a new process
///
/// Creates an empty process with a new address space.
/// Returns a capability handle to the new process.
///
/// The caller is responsible for:
/// 1. Allocating memory for the new process
/// 2. Mapping memory into the new process's address space
/// 3. Loading the binary
/// 4. Starting the process with `process_start`
pub fn process_create(entry_point: u64, stack_top: u64, flags: u64) -> Result<CapabilityHandle, SysError> {
    let ret = unsafe {
        syscall3(
            SyscallNumber::ProcessCreate as u64,
            entry_point,
            stack_top,
            flags,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(CapabilityHandle::from_raw(ret as u64))
    }
}

/// Start a process that was created with process_create
///
/// The process must have memory mapped and be ready to execute.
pub fn process_start(
    process_cap: CapabilityHandle,
    entry_point: u64,
    stack_top: u64,
) -> Result<(), SysError> {
    let ret = unsafe {
        syscall3(
            SyscallNumber::ProcessStart as u64,
            process_cap.as_raw(),
            entry_point,
            stack_top,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Set upcall handler for async events (IRQs, etc.)
pub fn set_upcall(handler_addr: u64, stack_addr: u64) -> Result<(), SysError> {
    let ret = unsafe {
        syscall2(
            SyscallNumber::ProcessSetUpcall as u64,
            handler_addr,
            stack_addr,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Set syscall handler for LibOS syscall forwarding
///
/// When the kernel receives a syscall it doesn't recognize (not an exokernel syscall),
/// it will call this handler with the syscall number and arguments instead of
/// returning ENOSYS.
///
/// This enables user-space LibOS implementations for compatibility layers
/// (e.g., Linux syscall translation).
///
/// Handler signature: extern "C" fn(num: u64, arg1-6: u64) -> i64
pub fn set_syscall_handler(handler_addr: u64, stack_addr: u64) -> Result<(), SysError> {
    let ret = unsafe {
        syscall2(
            SyscallNumber::ProcessSetSyscallHandler as u64,
            handler_addr,
            stack_addr,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}
