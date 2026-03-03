use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::Mutex;
use super::{Context, Process, ProcessId, ProcessState};
use super::context::switch_context;

/// Global scheduler instance
pub static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

/// Simple round-robin scheduler
pub struct Scheduler {
    /// All processes in the system
    processes: Vec<Process>,

    /// Queue of ready process indices
    ready_queue: VecDeque<usize>,

    /// Index of the currently running process
    current: Option<usize>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Scheduler {
            processes: Vec::new(),
            ready_queue: VecDeque::new(),
            current: None,
        }
    }

    /// Add a process to the scheduler
    pub fn add_process(&mut self, process: Process) -> ProcessId {
        let pid = process.pid;
        let idx = self.processes.len();
        self.processes.push(process);
        self.ready_queue.push_back(idx);
        pid
    }

    /// Get the current process
    pub fn current_process(&self) -> Option<&Process> {
        self.current.map(|idx| &self.processes[idx])
    }

    /// Get the current process mutably
    pub fn current_process_mut(&mut self) -> Option<&mut Process> {
        self.current.map(|idx| &mut self.processes[idx])
    }

    /// Get a process by PID
    pub fn get_process(&self, pid: ProcessId) -> Option<&Process> {
        self.processes.iter().find(|p| p.pid == pid)
    }

    /// Get a process by PID mutably
    pub fn get_process_mut(&mut self, pid: ProcessId) -> Option<&mut Process> {
        self.processes.iter_mut().find(|p| p.pid == pid)
    }

    /// Schedule the next process to run.
    /// Returns true if a context switch occurred.
    pub fn schedule(&mut self) -> bool {
        // Put current process back in ready queue if it's still runnable
        if let Some(current_idx) = self.current {
            let current = &mut self.processes[current_idx];
            if current.state == ProcessState::Running {
                current.state = ProcessState::Ready;
                self.ready_queue.push_back(current_idx);
            }
        }

        // Get next ready process
        let next_idx = match self.ready_queue.pop_front() {
            Some(idx) => idx,
            None => return false, // No ready processes
        };

        // If it's the same process, no switch needed
        if self.current == Some(next_idx) {
            self.processes[next_idx].state = ProcessState::Running;
            return false;
        }

        let old_idx = self.current;
        self.current = Some(next_idx);
        self.processes[next_idx].state = ProcessState::Running;

        // Perform context switch if there was a previous process
        if let Some(old) = old_idx {
            unsafe {
                let old_ctx = &mut self.processes[old].context as *mut Context;
                let new_ctx = &self.processes[next_idx].context as *const Context;
                switch_context(old_ctx, new_ctx);
            }
        }

        true
    }

    /// Yield the current process and schedule another
    pub fn yield_current(&mut self) {
        self.schedule();
    }

    /// Block the current process
    pub fn block_current(&mut self) {
        if let Some(current_idx) = self.current {
            self.processes[current_idx].state = ProcessState::Blocked;
            self.schedule();
        }
    }

    /// Unblock a process by PID
    pub fn unblock(&mut self, pid: ProcessId) {
        if let Some(idx) = self.processes.iter().position(|p| p.pid == pid) {
            if self.processes[idx].state == ProcessState::Blocked {
                self.processes[idx].state = ProcessState::Ready;
                self.ready_queue.push_back(idx);
            }
        }
    }

    /// Terminate the current process
    pub fn terminate_current(&mut self) {
        if let Some(current_idx) = self.current {
            self.processes[current_idx].state = ProcessState::Terminated;
            self.current = None;
            self.schedule();
        }
    }

    /// Get number of processes
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    /// Get number of ready processes
    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the global scheduler
pub fn init() {
    *SCHEDULER.lock() = Some(Scheduler::new());
}

/// Add a process to the global scheduler
pub fn spawn(process: Process) -> ProcessId {
    SCHEDULER
        .lock()
        .as_mut()
        .expect("Scheduler not initialized")
        .add_process(process)
}

/// Yield the current process
pub fn yield_now() {
    SCHEDULER
        .lock()
        .as_mut()
        .expect("Scheduler not initialized")
        .yield_current();
}

/// Run the scheduler (call from timer interrupt)
pub fn tick() {
    if let Some(ref mut scheduler) = *SCHEDULER.lock() {
        scheduler.schedule();
    }
}
