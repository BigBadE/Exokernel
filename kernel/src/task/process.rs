use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use super::Context;

/// Unique process identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(pub u64);

impl ProcessId {
    /// Generate a new unique process ID
    pub fn new() -> Self {
        static NEXT_PID: AtomicU64 = AtomicU64::new(1);
        ProcessId(NEXT_PID.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the kernel process ID (always 0)
    pub const fn kernel() -> Self {
        ProcessId(0)
    }
}

impl Default for ProcessId {
    fn default() -> Self {
        Self::new()
    }
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is ready to run
    Ready,
    /// Process is currently running
    Running,
    /// Process is blocked waiting for something
    Blocked,
    /// Process has terminated
    Terminated,
}

/// A process/task in the system
pub struct Process {
    /// Unique process identifier
    pub pid: ProcessId,

    /// Process state
    pub state: ProcessState,

    /// Saved CPU context for context switching
    pub context: Context,

    /// Kernel stack for this process (used during syscalls)
    pub kernel_stack: Vec<u8>,

    /// User stack (virtual address, if user process)
    pub user_stack_top: u64,

    /// Whether this is a kernel or user process
    pub is_kernel: bool,

    /// Page table root (CR3 value)
    pub page_table_root: u64,

    /// Process name (for debugging)
    pub name: &'static str,
}

impl Process {
    /// Kernel stack size per process
    const KERNEL_STACK_SIZE: usize = 4096 * 4; // 16 KiB

    /// Create a new kernel process
    pub fn new_kernel(name: &'static str, entry_point: fn() -> !) -> Self {
        let mut kernel_stack = Vec::with_capacity(Self::KERNEL_STACK_SIZE);
        kernel_stack.resize(Self::KERNEL_STACK_SIZE, 0);

        // Stack grows downward, so top is at the end
        let stack_top = kernel_stack.as_ptr() as u64 + Self::KERNEL_STACK_SIZE as u64;

        // Get current page table (kernel processes share the kernel page table)
        let page_table_root = {
            use x86_64::registers::control::Cr3;
            let (frame, _) = Cr3::read();
            frame.start_address().as_u64()
        };

        Process {
            pid: ProcessId::new(),
            state: ProcessState::Ready,
            context: Context::new_kernel(entry_point as u64, stack_top),
            kernel_stack,
            user_stack_top: 0,
            is_kernel: true,
            page_table_root,
            name,
        }
    }

    /// Create a new user process (page table and user stack must be set up separately)
    pub fn new_user(
        name: &'static str,
        entry_point: u64,
        user_stack_top: u64,
        page_table_root: u64,
    ) -> Self {
        let mut kernel_stack = Vec::with_capacity(Self::KERNEL_STACK_SIZE);
        kernel_stack.resize(Self::KERNEL_STACK_SIZE, 0);

        Process {
            pid: ProcessId::new(),
            state: ProcessState::Ready,
            context: Context::new_user(entry_point, user_stack_top),
            kernel_stack,
            user_stack_top,
            is_kernel: false,
            page_table_root,
            name,
        }
    }

    /// Get the top of the kernel stack for this process
    pub fn kernel_stack_top(&self) -> u64 {
        self.kernel_stack.as_ptr() as u64 + self.kernel_stack.len() as u64
    }
}
