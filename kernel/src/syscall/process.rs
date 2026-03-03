//! Process syscall implementations
//!
//! In an exokernel, process_create is minimal:
//! - Kernel creates an empty address space
//! - Kernel creates a capability set for the new process
//! - Parent process loads the binary via mem_alloc_phys + mem_map syscalls
//! - Parent sets the entry point
//!
//! This keeps ELF loading in user-space (libOS), not kernel.

use exo_shared::{CapabilityHandle, ResourceDescriptor, Rights, SysError};
use x86_64::structures::paging::{PageTable, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::registers::control::Cr3;

use crate::caps;
use crate::memory;
use crate::process::{self, current_pid, ProcessId, ProcessState, Process};
use crate::println;

/// Create a new process
///
/// Creates an empty address space and returns a process capability.
/// The parent process is responsible for:
/// 1. Allocating memory (mem_alloc_phys)
/// 2. Mapping memory into the new process (mem_map with process cap)
/// 3. Loading the binary (user-space ELF loader)
/// 4. Starting the process (process_start syscall)
///
/// Args:
/// - entry_point: Initial instruction pointer (can be set later)
/// - stack_top: Initial stack pointer (can be set later)
/// - flags: ProcessFlags
///
/// Returns: Capability handle to the new process
pub fn sys_process_create(
    entry_point: u64,
    stack_top: u64,
    _flags: u64,
) -> Result<i64, SysError> {
    let parent_pid = current_pid();

    // Validate addresses if provided (0 means "set later")
    if entry_point != 0 && entry_point >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }
    if stack_top != 0 && stack_top >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    // Create the new process
    let new_pid = process::with_process_table(|table| {
        table.create_process("child")
    }).ok_or(SysError::OutOfMemory)?;

    // Set entry point and stack if provided
    if entry_point != 0 || stack_top != 0 {
        process::with_process_table(|table| {
            if let Some(proc) = table.get_mut(new_pid) {
                if entry_point != 0 {
                    proc.context.rip = entry_point;
                }
                if stack_top != 0 {
                    proc.context.rsp = stack_top;
                }
            }
        });
    }

    // Create a capability for the new process
    let resource = ResourceDescriptor::process(new_pid.as_u64());
    let rights = Rights::READ | Rights::WRITE | Rights::KILL | Rights::SUSPEND | Rights::RESUME;

    let cap_handle = caps::create_root_cap(resource, rights, parent_pid.as_u64())?;

    println!("Process {} created child process {}", parent_pid.as_u64(), new_pid.as_u64());

    Ok(cap_handle.as_raw() as i64)
}

/// Destroy a process
/// Args: process_cap
pub fn sys_process_destroy(_process_cap: u64) -> Result<i64, SysError> {
    // TODO: Implement process destruction
    Err(SysError::NotPermitted)
}

/// Start a process that was created with process_create
///
/// Args:
/// - process_cap: Capability to the process
/// - entry_point: Instruction pointer to start at
/// - stack_top: Stack pointer
///
/// The parent must have already mapped memory into the process.
pub fn sys_process_start(
    process_cap: u64,
    entry_point: u64,
    stack_top: u64,
) -> Result<i64, SysError> {
    let caller_pid = current_pid();
    let cap_handle = CapabilityHandle::from_raw(process_cap);

    // Validate the process capability
    let cap = caps::validate(cap_handle, caller_pid.as_u64(), Rights::WRITE)?;

    // Check it's a process capability
    if cap.resource.resource_type != exo_shared::ResourceType::Process {
        return Err(SysError::InvalidArgument);
    }

    let target_pid = ProcessId(cap.resource.base);

    // Validate addresses
    if entry_point >= 0x0000_8000_0000_0000 || stack_top >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    // Set up the process context and add to scheduler
    process::with_process_table(|table| {
        if let Some(proc) = table.get_mut(target_pid) {
            // Set entry point and stack
            proc.context.rip = entry_point;
            proc.context.rsp = stack_top;
            proc.context.rflags = 0x202; // IF=1 (interrupts enabled)

            // User mode segments
            proc.context.cs = 0x23; // User code (ring 3)
            proc.context.ss = 0x1b; // User data (ring 3)

            proc.state = ProcessState::Ready;
            Ok(())
        } else {
            Err(SysError::ProcessNotFound)
        }
    })?;

    // Add to scheduler
    process::scheduler::start_process(target_pid);

    println!("Process {} started by {}", target_pid.as_u64(), caller_pid.as_u64());

    Ok(0)
}

/// Get current process ID
pub fn sys_process_get_pid() -> Result<i64, SysError> {
    Ok(current_pid().as_u64() as i64)
}

/// Exit current process
pub fn sys_process_exit(exit_code: i32) -> Result<i64, SysError> {
    let pid = current_pid();

    println!("Process {} exiting with code {}", pid.as_u64(), exit_code);

    process::with_process_table(|table| {
        if let Some(proc) = table.get_mut(pid) {
            proc.terminate(exit_code);
        }
    });

    // Halt - in a real kernel we'd schedule another process
    loop {
        x86_64::instructions::hlt();
    }
}

/// Yield CPU
pub fn sys_process_yield() -> Result<i64, SysError> {
    process::scheduler::yield_now();
    Ok(0)
}

/// Set upcall handler
/// Args: handler_addr, stack_addr
pub fn sys_process_set_upcall(handler_addr: u64, stack_addr: u64) -> Result<i64, SysError> {
    use x86_64::VirtAddr;

    let pid = current_pid();

    // Validate addresses
    if handler_addr >= 0x0000_8000_0000_0000 || stack_addr >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    process::with_process_table(|table| {
        if let Some(proc) = table.get_mut(pid) {
            proc.upcall_handler = Some(VirtAddr::new(handler_addr));
            proc.upcall_stack = Some(VirtAddr::new(stack_addr));
            Ok(0)
        } else {
            Err(SysError::ProcessNotFound)
        }
    })
}

/// Set syscall handler for LibOS forwarding
///
/// When a syscall is made that the kernel doesn't recognize (not an exokernel syscall),
/// instead of returning ENOSYS, the kernel will call this handler in user-space.
///
/// The handler receives the syscall number and all 6 arguments on the stack,
/// and its return value becomes the syscall result.
///
/// This enables user-space LibOS implementations (like Linux compatibility).
///
/// Args: handler_addr, stack_addr
pub fn sys_process_set_syscall_handler(handler_addr: u64, stack_addr: u64) -> Result<i64, SysError> {
    use x86_64::VirtAddr;

    let pid = current_pid();

    // Validate addresses
    if handler_addr >= 0x0000_8000_0000_0000 || stack_addr >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    process::with_process_table(|table| {
        if let Some(proc) = table.get_mut(pid) {
            proc.syscall_handler = Some(VirtAddr::new(handler_addr));
            proc.syscall_handler_stack = Some(VirtAddr::new(stack_addr));
            println!(
                "Process {} registered syscall handler at {:#x}",
                pid.as_u64(),
                handler_addr
            );
            Ok(0)
        } else {
            Err(SysError::ProcessNotFound)
        }
    })
}
