//! CPU Context for context switching

use core::arch::naked_asm;

/// CPU context saved during context switches
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct Context {
    // Callee-saved registers
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,

    // Instruction pointer
    pub rip: u64,

    // Stack pointer
    pub rsp: u64,

    // Flags
    pub rflags: u64,

    // Segment selectors
    pub cs: u64,
    pub ss: u64,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: 0,
            rsp: 0,
            rflags: 0x202, // IF=1
            cs: 0x08,      // Kernel code
            ss: 0x10,      // Kernel data
        }
    }

    pub fn new_user(entry_point: u64, stack_top: u64) -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: entry_point,
            rsp: stack_top,
            rflags: 0x202, // IF=1
            cs: 0x23,      // User code (ring 3)
            ss: 0x1b,      // User data (ring 3)
        }
    }
}

/// Switch contexts between two processes
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(old: *mut Context, new: *const Context) {
    naked_asm!(
        // Save current context
        "mov [rdi + 0x00], r15",
        "mov [rdi + 0x08], r14",
        "mov [rdi + 0x10], r13",
        "mov [rdi + 0x18], r12",
        "mov [rdi + 0x20], rbx",
        "mov [rdi + 0x28], rbp",
        "lea rax, [rip + 1f]",
        "mov [rdi + 0x30], rax",
        "mov [rdi + 0x38], rsp",
        "pushfq",
        "pop rax",
        "mov [rdi + 0x40], rax",

        // Load new context
        "mov r15, [rsi + 0x00]",
        "mov r14, [rsi + 0x08]",
        "mov r13, [rsi + 0x10]",
        "mov r12, [rsi + 0x18]",
        "mov rbx, [rsi + 0x20]",
        "mov rbp, [rsi + 0x28]",
        "mov rax, [rsi + 0x40]",
        "push rax",
        "popfq",
        "mov rsp, [rsi + 0x38]",
        "jmp [rsi + 0x30]",

        "1:",
        "ret",
    )
}

/// Jump to user mode
#[unsafe(naked)]
pub unsafe extern "C" fn jump_to_usermode(
    code_selector: u64,
    data_selector: u64,
    user_rip: u64,
    user_rsp: u64,
) -> ! {
    naked_asm!(
        // Arguments: rdi=code_sel, rsi=data_sel, rdx=rip, rcx=rsp

        // Save rdx (user_rip) to r8 before any operations
        "mov r8, rdx",

        // Set up data segments
        "mov ax, si",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",

        // Build iretq frame
        "push rsi",       // SS (data selector)
        "push rcx",       // RSP (user stack)
        "pushfq",
        "pop rax",
        "or rax, 0x200",  // Enable interrupts
        "push rax",       // RFLAGS
        "push rdi",       // CS (code selector)
        "push r8",        // RIP (user entry - saved in r8)

        "iretq",
    )
}
