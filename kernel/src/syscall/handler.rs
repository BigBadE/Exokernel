use core::arch::naked_asm;
use crate::{println, print};
use super::numbers::*;

/// Syscall entry point (called via SYSCALL instruction)
///
/// On entry:
/// - RCX contains the return RIP
/// - R11 contains the saved RFLAGS
/// - RAX contains the syscall number
/// - RDI, RSI, RDX, R10, R8, R9 contain arguments 1-6
///
/// On exit (via SYSRET):
/// - RCX will be restored to RIP
/// - R11 will be restored to RFLAGS
/// - RAX contains the return value
#[unsafe(naked)]
pub unsafe extern "C" fn syscall_entry() {
    naked_asm!(
        // Save user stack pointer and switch to kernel stack
        // For now, we use a simple approach - save everything on the current stack
        // In a real kernel, we'd switch to a per-CPU kernel stack

        // Save callee-saved registers that we might clobber
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Save user RCX (return address) and R11 (RFLAGS)
        "push rcx",     // User RIP
        "push r11",     // User RFLAGS

        // Move R10 to RCX (syscall convention uses R10 for 4th arg, but C ABI uses RCX)
        "mov rcx, r10",

        // Call the Rust syscall dispatcher
        // Arguments are already in the right registers for the C ABI:
        // RDI = arg1, RSI = arg2, RDX = arg3, RCX = arg4 (was R10), R8 = arg5, R9 = arg6
        // RAX = syscall number (we need to pass this too)
        //
        // We'll use a different approach: pass syscall number as first arg
        // syscall_dispatch(number, arg1, arg2, arg3, arg4, arg5, arg6)
        // But we only have 6 registers, so push arg6 to stack or reorganize

        // Actually, let's use a simpler approach:
        // Pass: RAX=number, RDI=arg1, RSI=arg2, RDX=arg3, R10=arg4, R8=arg5, R9=arg6
        // We need to shuffle for C calling convention

        // Save RAX (syscall number) since we need to pass more args
        "mov r10, rax",     // Save syscall number in R10 temporarily

        // Now call: syscall_dispatch(rax=number, rdi=arg1, rsi=arg2, rdx=arg3, rcx=arg4, r8=arg5, r9=arg6)
        // But wait, we already moved R10 to RCX above. Let's reorganize.

        // Reset and do it properly:
        // At this point after the pushes:
        // R10 = syscall number (we just saved it)
        // RDI = arg1, RSI = arg2, RDX = arg3, RCX = arg4 (moved from R10 earlier), R8 = arg5, R9 = arg6

        // For our Rust function we want:
        // arg0 (RDI) = syscall number
        // arg1 (RSI) = user arg1
        // arg2 (RDX) = user arg2
        // arg3 (RCX) = user arg3
        // arg4 (R8)  = user arg4
        // arg5 (R9)  = user arg5

        // Shuffle registers
        "mov r11, r9",      // Save arg6 to R11 temporarily
        "mov r9, r8",       // arg5 -> R9
        "mov r8, rcx",      // arg4 -> R8
        "mov rcx, rdx",     // arg3 -> RCX
        "mov rdx, rsi",     // arg2 -> RDX
        "mov rsi, rdi",     // arg1 -> RSI
        "mov rdi, r10",     // syscall number -> RDI

        // Enable interrupts during syscall handling
        "sti",

        // Call the dispatcher
        "call {dispatch}",

        // Disable interrupts before returning to user mode
        "cli",

        // RAX now contains the return value

        // Restore user RFLAGS and RIP
        "pop r11",      // User RFLAGS
        "pop rcx",      // User RIP

        // Restore callee-saved registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",

        // Return to user mode
        "sysretq",

        dispatch = sym syscall_dispatch,
    )
}

/// Rust syscall dispatcher
///
/// This is called from the assembly entry point with arguments already arranged.
#[unsafe(no_mangle)]
pub extern "C" fn syscall_dispatch(
    number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> i64 {
    match number {
        SYS_EXIT => sys_exit(arg1 as i32),
        SYS_WRITE => sys_write(arg1, arg2 as *const u8, arg3),
        SYS_READ => sys_read(arg1, arg2 as *mut u8, arg3),
        SYS_BRK => sys_brk(arg1),
        SYS_MMAP => sys_mmap(arg1, arg2, arg3 as i32, arg4 as i32, arg5 as i32, 0),
        SYS_GETPID => sys_getpid(),
        SYS_DEBUG_PRINT => sys_debug_print(arg1 as *const u8, arg2),
        _ => {
            println!("Unknown syscall: {}", number);
            -1 // ENOSYS
        }
    }
}

/// Exit the current process
fn sys_exit(status: i32) -> ! {
    println!("Process exited with status: {}", status);

    // For now, just halt. In a real kernel, we'd clean up the process
    // and schedule another one.
    loop {
        x86_64::instructions::hlt();
    }
}

/// Write to a file descriptor
fn sys_write(fd: u64, buf: *const u8, count: u64) -> i64 {
    // For now, only support stdout (fd=1) and stderr (fd=2)
    if fd != 1 && fd != 2 {
        return -9; // EBADF
    }

    // Validate the buffer is in user space (simple check)
    let buf_addr = buf as u64;
    if buf_addr >= 0x0000_8000_0000_0000 {
        return -14; // EFAULT - bad address
    }

    // Read and print the buffer
    for i in 0..count {
        let byte = unsafe { *buf.add(i as usize) };
        if byte == 0 {
            break;
        }
        print!("{}", byte as char);
    }

    count as i64
}

/// Read from a file descriptor
fn sys_read(fd: u64, _buf: *mut u8, _count: u64) -> i64 {
    // For now, only support stdin (fd=0)
    if fd != 0 {
        return -9; // EBADF
    }

    // TODO: Implement actual reading from keyboard buffer
    0
}

/// Program break manipulation
fn sys_brk(addr: u64) -> i64 {
    // Simple brk implementation
    // In a real kernel, this would manage the process's heap

    static mut CURRENT_BRK: u64 = 0x0000_0010_0000_0000; // Start of heap region

    unsafe {
        if addr == 0 {
            // Query current brk
            CURRENT_BRK as i64
        } else if addr >= 0x0000_0010_0000_0000 && addr < 0x0000_0020_0000_0000 {
            // Set new brk (simple validation)
            CURRENT_BRK = addr;
            addr as i64
        } else {
            -12 // ENOMEM
        }
    }
}

/// Memory map
fn sys_mmap(
    addr: u64,
    length: u64,
    _prot: i32,
    flags: i32,
    fd: i32,
    _offset: u64,
) -> i64 {
    // Simple anonymous mmap implementation
    const MAP_ANONYMOUS: i32 = 0x20;

    if flags & MAP_ANONYMOUS == 0 && fd != -1 {
        // File-backed mmap not supported yet
        return -38; // ENOSYS
    }

    // For anonymous mappings, we need to allocate memory
    // This is a simplified implementation

    static mut MMAP_REGION: u64 = 0x0000_0030_0000_0000;

    unsafe {
        let result = if addr == 0 {
            let region = MMAP_REGION;
            MMAP_REGION += (length + 0xFFF) & !0xFFF; // Page-align
            region
        } else {
            addr
        };

        // TODO: Actually map the pages

        result as i64
    }
}

/// Get process ID
fn sys_getpid() -> i64 {
    // Return a dummy PID for now
    // In a real kernel, this would come from the current process structure
    1
}

/// Debug print syscall for testing
fn sys_debug_print(buf: *const u8, len: u64) -> i64 {
    // Validate buffer address
    let buf_addr = buf as u64;
    if buf_addr >= 0x0000_8000_0000_0000 {
        return -14; // EFAULT
    }

    print!("[USER] ");
    for i in 0..len {
        let byte = unsafe { *buf.add(i as usize) };
        if byte == 0 {
            break;
        }
        print!("{}", byte as char);
    }
    println!();

    len as i64
}
