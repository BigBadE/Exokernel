use core::arch::naked_asm;

/// CPU context saved during context switches
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct Context {
    // Callee-saved registers (must be preserved across function calls)
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,

    // Instruction pointer (return address)
    pub rip: u64,

    // Stack pointer
    pub rsp: u64,

    // Flags register
    pub rflags: u64,

    // Segment selectors for user mode
    pub cs: u64,
    pub ss: u64,
}

impl Context {
    /// Create a new empty context
    pub const fn new() -> Self {
        Context {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: 0,
            rsp: 0,
            rflags: 0x202, // Interrupts enabled
            cs: 0,
            ss: 0,
        }
    }

    /// Create a kernel context with the given entry point and stack
    pub fn new_kernel(entry_point: u64, stack_top: u64) -> Self {
        Context {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: entry_point,
            rsp: stack_top,
            rflags: 0x202, // Interrupts enabled
            cs: 0x08,      // Kernel code segment
            ss: 0x10,      // Kernel data segment
        }
    }

    /// Create a user context with the given entry point and stack
    pub fn new_user(entry_point: u64, stack_top: u64) -> Self {
        Context {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: entry_point,
            rsp: stack_top,
            rflags: 0x202,       // Interrupts enabled
            cs: 0x1b,            // User code segment (ring 3) = 0x18 | 3
            ss: 0x23,            // User data segment (ring 3) = 0x20 | 3
        }
    }
}

/// Switch from the current context to a new context.
///
/// # Safety
/// The new context must be valid and point to executable code with a valid stack.
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(old: *mut Context, new: *const Context) {
    naked_asm!(
        // Save current context to `old`
        "mov [rdi + 0x00], r15",
        "mov [rdi + 0x08], r14",
        "mov [rdi + 0x10], r13",
        "mov [rdi + 0x18], r12",
        "mov [rdi + 0x20], rbx",
        "mov [rdi + 0x28], rbp",

        // Save return address (rip will be the instruction after call)
        "lea rax, [rip + 1f]",
        "mov [rdi + 0x30], rax",

        // Save stack pointer
        "mov [rdi + 0x38], rsp",

        // Save flags
        "pushfq",
        "pop rax",
        "mov [rdi + 0x40], rax",

        // Load new context from `new`
        "mov r15, [rsi + 0x00]",
        "mov r14, [rsi + 0x08]",
        "mov r13, [rsi + 0x10]",
        "mov r12, [rsi + 0x18]",
        "mov rbx, [rsi + 0x20]",
        "mov rbp, [rsi + 0x28]",

        // Load flags
        "mov rax, [rsi + 0x40]",
        "push rax",
        "popfq",

        // Load stack pointer
        "mov rsp, [rsi + 0x38]",

        // Jump to new instruction pointer
        "jmp [rsi + 0x30]",

        // Return point for when we switch back
        "1:",
        "ret",
    )
}

/// Jump to user mode.
///
/// # Safety
/// The context must be valid and point to user-space code.
#[unsafe(naked)]
pub unsafe extern "C" fn jump_to_usermode(
    code_selector: u64,
    data_selector: u64,
    user_rip: u64,
    user_rsp: u64,
) -> ! {
    naked_asm!(
        // Set up data segments for user mode
        "mov ax, si",           // data_selector is in rsi (2nd arg)
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",

        // Build iretq stack frame:
        // SS (user data selector)
        // RSP (user stack pointer)
        // RFLAGS
        // CS (user code selector)
        // RIP (user instruction pointer)

        "push rsi",             // SS (user data selector)
        "push rcx",             // RSP (user stack, 4th arg)
        "pushfq",               // RFLAGS
        "pop rax",
        "or rax, 0x200",        // Ensure interrupts are enabled
        "push rax",
        "push rdi",             // CS (user code selector, 1st arg)
        "push rdx",             // RIP (user entry point, 3rd arg)

        "iretq",
    )
}
