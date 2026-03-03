//! Syscall handling for the Exokernel
//!
//! This implements the minimal exokernel syscall interface:
//! - Capability management
//! - Memory binding
//! - Process/CPU management
//! - IPC
//! - I/O binding

pub mod caps;
pub mod memory;
pub mod process;
pub mod ipc;
pub mod io;

use core::arch::naked_asm;

use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

use exo_shared::{Syscall, SysError};
use crate::gdt;
use crate::println;

/// Initialize the SYSCALL/SYSRET mechanism
pub fn init() {
    let selectors = gdt::selectors();

    Star::write(
        selectors.user_code_selector,
        selectors.user_data_selector,
        selectors.kernel_code_selector,
        selectors.kernel_data_selector,
    )
    .expect("Failed to write STAR MSR");

    LStar::write(VirtAddr::new(syscall_entry as *const () as u64));

    SFMask::write(RFlags::INTERRUPT_FLAG | RFlags::DIRECTION_FLAG);

    unsafe {
        let efer = Efer::read();
        Efer::write(efer | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
}

/// Syscall entry point
#[unsafe(naked)]
pub unsafe extern "C" fn syscall_entry() {
    naked_asm!(
        // Save registers
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rcx",     // User RIP
        "push r11",     // User RFLAGS

        // Move R10 to RCX for C calling convention
        "mov rcx, r10",

        // Shuffle arguments for syscall_dispatch
        // We want: rdi=syscall_num, rsi=arg1, rdx=arg2, rcx=arg3, r8=arg4, r9=arg5
        "mov r11, r9",      // Save arg6
        "mov r9, r8",       // arg5 -> r9
        "mov r8, rcx",      // arg4 -> r8
        "mov rcx, rdx",     // arg3 -> rcx
        "mov rdx, rsi",     // arg2 -> rdx
        "mov rsi, rdi",     // arg1 -> rsi
        "mov rdi, rax",     // syscall number -> rdi

        // Enable interrupts
        "sti",

        // Call dispatcher
        "call {dispatch}",

        // Disable interrupts
        "cli",

        // Restore
        "pop r11",
        "pop rcx",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",

        "sysretq",

        dispatch = sym syscall_dispatch,
    )
}

/// Syscall dispatcher
#[unsafe(no_mangle)]
pub extern "C" fn syscall_dispatch(
    syscall_num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> i64 {
    let result = match Syscall::from_num(syscall_num) {
        // Capability syscalls
        Some(Syscall::CapGrant) => caps::sys_cap_grant(arg1, arg2, arg3, arg4),
        Some(Syscall::CapRevoke) => caps::sys_cap_revoke(arg1),
        Some(Syscall::CapDelegate) => caps::sys_cap_delegate(arg1, arg2, arg3),
        Some(Syscall::CapInspect) => caps::sys_cap_inspect(arg1, arg2),
        Some(Syscall::CapDrop) => caps::sys_cap_drop(arg1),

        // Memory syscalls
        Some(Syscall::MemAllocPhys) => memory::sys_mem_alloc_phys(arg1, arg2),
        Some(Syscall::MemMap) => memory::sys_mem_map(arg1, arg2, arg3),
        Some(Syscall::MemUnmap) => memory::sys_mem_unmap(arg1, arg2),
        Some(Syscall::MemProtect) => memory::sys_mem_protect(arg1, arg2, arg3),
        Some(Syscall::MemQuery) => memory::sys_mem_query(arg1),

        // Process syscalls
        Some(Syscall::ProcessCreate) => process::sys_process_create(arg1, arg2, arg3),
        Some(Syscall::ProcessDestroy) => process::sys_process_destroy(arg1),
        Some(Syscall::ProcessGetPid) => process::sys_process_get_pid(),
        Some(Syscall::ProcessExit) => process::sys_process_exit(arg1 as i32),
        Some(Syscall::ProcessYield) => process::sys_process_yield(),
        Some(Syscall::ProcessSetUpcall) => process::sys_process_set_upcall(arg1, arg2),
        Some(Syscall::ProcessStart) => process::sys_process_start(arg1, arg2, arg3),
        Some(Syscall::ProcessSetSyscallHandler) => process::sys_process_set_syscall_handler(arg1, arg2),

        // IPC syscalls
        Some(Syscall::IpcCreateEndpoint) => ipc::sys_ipc_create_endpoint(arg1),
        Some(Syscall::IpcSend) => ipc::sys_ipc_send(arg1, arg2, arg3, arg4, arg5),
        Some(Syscall::IpcRecv) => ipc::sys_ipc_recv(arg1, arg2, arg3, arg4, arg5),
        Some(Syscall::IpcCall) => ipc::sys_ipc_call(arg1, arg2, arg3, arg4, arg5),
        Some(Syscall::IpcReply) => ipc::sys_ipc_reply(arg1, arg2),

        // I/O syscalls
        Some(Syscall::IoBindPort) => io::sys_io_bind_port(arg1, arg2, arg3),
        Some(Syscall::IoPort) => io::sys_io_port(arg1, arg2, arg3, arg4),
        Some(Syscall::IrqBind) => io::sys_irq_bind(arg1, arg2),
        Some(Syscall::IrqAck) => io::sys_irq_ack(arg1),
        Some(Syscall::DmaAlloc) => io::sys_dma_alloc(arg1, arg2),
        Some(Syscall::DmaGetPhys) => io::sys_dma_get_phys(arg1),

        // Debug syscalls
        Some(Syscall::DebugPrint) => sys_debug_print(arg1, arg2),
        Some(Syscall::DebugDumpCaps) => sys_debug_dump_caps(),

        None => {
            // Check if this process has a syscall handler registered
            let handler_info = crate::process::with_process_table(|table| {
                if let Some(proc) = table.current() {
                    if let Some(handler) = proc.syscall_handler {
                        return Some(handler.as_u64());
                    }
                }
                None
            });

            if let Some(handler_addr) = handler_info {
                // Forward the syscall to the user-space handler
                // The handler will be called with the syscall number and arguments
                // We do this by returning a special result that the assembly code handles
                //
                // For now, we'll call the handler synchronously by using an indirect call
                // This works because we're still in the same address space
                //
                // Handler signature: extern "C" fn(num: u64, a1-a6: u64) -> i64
                let handler: extern "C" fn(u64, u64, u64, u64, u64, u64, u64) -> i64 =
                    unsafe { core::mem::transmute(handler_addr) };

                // Call the user-space handler directly
                // This works because:
                // 1. We're executing with interrupts enabled (sti was called)
                // 2. The handler is in user-space memory which is mapped
                // 3. We return to user mode after via sysret
                //
                // Note: This is a simplified approach. A production kernel would
                // set up a proper upcall frame and return to user mode.
                let result = handler(syscall_num, arg1, arg2, arg3, arg4, arg5, 0);
                return result;
            }

            println!("Unknown syscall: {:#x}", syscall_num);
            Err(SysError::InvalidSyscall)
        }
    };

    match result {
        Ok(val) => val as i64,
        Err(e) => e.as_code(),
    }
}

/// Debug print syscall
fn sys_debug_print(buf_ptr: u64, len: u64) -> Result<i64, SysError> {
    use crate::print;

    // Basic validation
    if buf_ptr >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    let buf = unsafe {
        core::slice::from_raw_parts(buf_ptr as *const u8, len as usize)
    };

    print!("[USER] ");
    for &byte in buf {
        if byte == 0 {
            break;
        }
        print!("{}", byte as char);
    }
    println!();

    Ok(len as i64)
}

/// Debug dump capabilities
fn sys_debug_dump_caps() -> Result<i64, SysError> {
    use crate::caps;

    let (used, free) = caps::with_cap_table(|t| t.stats());
    println!("Capability table: {} used, {} free", used, free);

    Ok(0)
}
