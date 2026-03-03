//! Process Management
//!
//! Processes in the exokernel are minimal - just an address space,
//! a capability table, and CPU state. All abstractions are in user-space.

pub mod context;
pub mod scheduler;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use x86_64::VirtAddr;

use crate::caps::CapabilityTable;

pub use context::Context;

/// Process ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ProcessId(pub u64);

impl ProcessId {
    pub const KERNEL: Self = Self(0);

    pub fn new() -> Self {
        static NEXT_PID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_PID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
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
    /// Ready to run
    Ready,
    /// Currently running
    Running,
    /// Blocked on IPC receive
    BlockedRecv,
    /// Blocked on IPC send
    BlockedSend,
    /// Terminated
    Terminated,
}

/// A process in the exokernel
pub struct Process {
    /// Process ID
    pub pid: ProcessId,

    /// Process state
    pub state: ProcessState,

    /// CPU context
    pub context: Context,

    /// Kernel stack (for syscall handling)
    kernel_stack: Vec<u8>,

    /// Per-process capability table
    pub cap_table: CapabilityTable,

    /// Page table root (CR3)
    pub page_table_root: u64,

    /// Upcall handler address (for async events like IRQs)
    pub upcall_handler: Option<VirtAddr>,

    /// Upcall stack address
    pub upcall_stack: Option<VirtAddr>,

    /// Syscall handler address (for LibOS syscall forwarding)
    /// When set, unknown syscalls are forwarded to this handler instead of returning ENOSYS
    pub syscall_handler: Option<VirtAddr>,

    /// Syscall handler stack address
    pub syscall_handler_stack: Option<VirtAddr>,

    /// Exit code (set when terminated)
    pub exit_code: Option<i32>,

    /// Process name (for debugging)
    pub name: &'static str,
}

impl Process {
    const KERNEL_STACK_SIZE: usize = 4096 * 4; // 16 KiB

    /// Create the kernel "process" (PID 0)
    pub fn kernel() -> Self {
        Self {
            pid: ProcessId::KERNEL,
            state: ProcessState::Running,
            context: Context::new(),
            kernel_stack: Vec::new(), // Kernel doesn't need a separate kernel stack
            cap_table: CapabilityTable::new(0),
            page_table_root: 0,
            upcall_handler: None,
            upcall_stack: None,
            syscall_handler: None,
            syscall_handler_stack: None,
            exit_code: None,
            name: "kernel",
        }
    }

    /// Create a new user process
    pub fn new_user(
        name: &'static str,
        entry_point: VirtAddr,
        user_stack: VirtAddr,
        page_table_root: u64,
    ) -> Self {
        let pid = ProcessId::new();

        // Allocate kernel stack
        let mut kernel_stack = Vec::with_capacity(Self::KERNEL_STACK_SIZE);
        kernel_stack.resize(Self::KERNEL_STACK_SIZE, 0);

        // Set up context for user mode
        let context = Context::new_user(entry_point.as_u64(), user_stack.as_u64());

        Self {
            pid,
            state: ProcessState::Ready,
            context,
            kernel_stack,
            cap_table: CapabilityTable::new(pid.0),
            page_table_root,
            upcall_handler: None,
            upcall_stack: None,
            syscall_handler: None,
            syscall_handler_stack: None,
            exit_code: None,
            name,
        }
    }

    /// Get the top of the kernel stack
    pub fn kernel_stack_top(&self) -> VirtAddr {
        if self.kernel_stack.is_empty() {
            return VirtAddr::zero();
        }
        let ptr = self.kernel_stack.as_ptr() as u64;
        VirtAddr::new(ptr + self.kernel_stack.len() as u64)
    }

    /// Terminate the process
    pub fn terminate(&mut self, exit_code: i32) {
        self.state = ProcessState::Terminated;
        self.exit_code = Some(exit_code);
    }

    /// Check if process is runnable
    pub fn is_runnable(&self) -> bool {
        matches!(self.state, ProcessState::Ready | ProcessState::Running)
    }
}

/// Global process table
static PROCESS_TABLE: Mutex<Option<ProcessTable>> = Mutex::new(None);

/// Process table
pub struct ProcessTable {
    processes: Vec<Process>,
    current: Option<usize>,
}

impl ProcessTable {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            current: None,
        }
    }

    /// Add a process
    pub fn add(&mut self, process: Process) -> ProcessId {
        let pid = process.pid;
        self.processes.push(process);
        pid
    }

    /// Get current process
    pub fn current(&self) -> Option<&Process> {
        self.current.map(|idx| &self.processes[idx])
    }

    /// Get current process mutably
    pub fn current_mut(&mut self) -> Option<&mut Process> {
        self.current.map(|idx| &mut self.processes[idx])
    }

    /// Get process by PID
    pub fn get(&self, pid: ProcessId) -> Option<&Process> {
        self.processes.iter().find(|p| p.pid == pid)
    }

    /// Get process by PID mutably
    pub fn get_mut(&mut self, pid: ProcessId) -> Option<&mut Process> {
        self.processes.iter_mut().find(|p| p.pid == pid)
    }

    /// Set current process
    pub fn set_current(&mut self, pid: ProcessId) -> bool {
        if let Some(idx) = self.processes.iter().position(|p| p.pid == pid) {
            // Mark old as Ready
            if let Some(old_idx) = self.current {
                if self.processes[old_idx].state == ProcessState::Running {
                    self.processes[old_idx].state = ProcessState::Ready;
                }
            }
            // Mark new as Running
            self.processes[idx].state = ProcessState::Running;
            self.current = Some(idx);
            true
        } else {
            false
        }
    }

    /// Get current PID
    pub fn current_pid(&self) -> Option<ProcessId> {
        self.current().map(|p| p.pid)
    }

    /// Remove terminated processes
    pub fn cleanup(&mut self) {
        self.processes.retain(|p| p.state != ProcessState::Terminated);
        self.current = None;
    }

    /// Iterator over all processes
    pub fn iter(&self) -> impl Iterator<Item = &Process> {
        self.processes.iter()
    }

    /// Create a new empty process
    ///
    /// This creates a process with an empty address space.
    /// The parent is responsible for:
    /// - Mapping memory via syscalls
    /// - Loading the binary (user-space ELF loader)
    /// - Setting entry point and stack
    /// - Starting the process
    pub fn create_process(&mut self, name: &'static str) -> Option<ProcessId> {
        // For now, we use the kernel's page table
        // A full implementation would create a new page table
        let page_table_root = 0; // TODO: create new address space

        let pid = ProcessId::new();

        // Allocate kernel stack for the new process
        let mut kernel_stack = Vec::with_capacity(Process::KERNEL_STACK_SIZE);
        kernel_stack.resize(Process::KERNEL_STACK_SIZE, 0);

        let process = Process {
            pid,
            state: ProcessState::Ready,
            context: Context::new(), // Empty context - will be set by parent
            kernel_stack,
            cap_table: CapabilityTable::new(pid.0),
            page_table_root,
            upcall_handler: None,
            upcall_stack: None,
            syscall_handler: None,
            syscall_handler_stack: None,
            exit_code: None,
            name,
        };

        self.processes.push(process);
        Some(pid)
    }
}

impl Default for ProcessTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the process subsystem
pub fn init() {
    let mut table = ProcessTable::new();

    // Add the kernel as PID 0
    table.add(Process::kernel());
    table.set_current(ProcessId::KERNEL);

    *PROCESS_TABLE.lock() = Some(table);
}

/// Access the process table
pub fn with_process_table<F, R>(f: F) -> R
where
    F: FnOnce(&mut ProcessTable) -> R,
{
    let mut guard = PROCESS_TABLE.lock();
    let table = guard.as_mut().expect("Process table not initialized");
    f(table)
}

/// Get the current process ID
pub fn current_pid() -> ProcessId {
    with_process_table(|t| t.current_pid().unwrap_or(ProcessId::KERNEL))
}

/// Spawn a new process
pub fn spawn(
    name: &'static str,
    entry_point: VirtAddr,
    user_stack: VirtAddr,
    page_table_root: u64,
) -> ProcessId {
    let process = Process::new_user(name, entry_point, user_stack, page_table_root);
    with_process_table(|t| t.add(process))
}
