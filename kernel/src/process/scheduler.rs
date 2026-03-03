//! Scheduler - Policy decisions for CPU multiplexing
//!
//! The scheduler implements POLICY (who runs next).
//! The timer interrupt provides MECHANISM (preemption).
//!
//! This separation is key to exokernel design:
//! - Kernel provides scheduling primitives
//! - Policy could even be influenced by user-space in future
//!   (e.g., user-space scheduler hints via upcalls)

use alloc::collections::VecDeque;
use spin::Mutex;

use super::{with_process_table, ProcessId, ProcessState};

/// Ready queue - processes waiting to run
static READY_QUEUE: Mutex<VecDeque<ProcessId>> = Mutex::new(VecDeque::new());

/// Time slice tracking (ticks remaining for current process)
static TICKS_REMAINING: Mutex<u32> = Mutex::new(0);

/// Default time slice in timer ticks
const DEFAULT_TIME_SLICE: u32 = 10;

/// Add a process to the ready queue
pub fn enqueue(pid: ProcessId) {
    READY_QUEUE.lock().push_back(pid);
}

/// Get the next process to run
pub fn dequeue() -> Option<ProcessId> {
    READY_QUEUE.lock().pop_front()
}

/// Schedule the next process
///
/// This is the core scheduling policy. Currently simple round-robin.
/// In a more advanced exokernel, user-space might influence this via hints.
pub fn schedule() -> Option<ProcessId> {
    // Get current process and put it back in queue if still runnable
    let current_pid = with_process_table(|t| {
        if let Some(current) = t.current() {
            if current.is_runnable() && current.pid != ProcessId::KERNEL {
                return Some(current.pid);
            }
        }
        None
    });

    if let Some(pid) = current_pid {
        enqueue(pid);
    }

    // Find next runnable process
    while let Some(next_pid) = dequeue() {
        let runnable = with_process_table(|t| {
            t.get(next_pid).map(|p| p.is_runnable()).unwrap_or(false)
        });

        if runnable {
            with_process_table(|t| t.set_current(next_pid));

            // Reset time slice for the new process
            *TICKS_REMAINING.lock() = DEFAULT_TIME_SLICE;

            return Some(next_pid);
        }
    }

    None
}

/// Start scheduling for a new process
pub fn start_process(pid: ProcessId) {
    enqueue(pid);
    *TICKS_REMAINING.lock() = DEFAULT_TIME_SLICE;
}

/// Yield the current process
pub fn yield_now() {
    schedule();
}

/// Called from timer interrupt - POLICY DECISION POINT
///
/// The timer interrupt (mechanism) calls this function.
/// We decide (policy) whether to preempt the current process.
pub fn timer_tick() {
    let mut ticks = TICKS_REMAINING.lock();

    if *ticks > 0 {
        *ticks -= 1;
        if *ticks == 0 {
            // Time slice expired - preempt
            drop(ticks); // Release lock before schedule()
            schedule();
        }
    }
    // If ticks is 0 and stays 0, we're in a special state (no scheduling)
}

/// Block the current process
pub fn block_current(state: ProcessState) {
    with_process_table(|t| {
        if let Some(current) = t.current_mut() {
            current.state = state;
        }
    });
    schedule();
}

/// Unblock a process
pub fn unblock(pid: ProcessId) {
    let should_enqueue = with_process_table(|t| {
        if let Some(process) = t.get_mut(pid) {
            if matches!(process.state, ProcessState::BlockedRecv | ProcessState::BlockedSend) {
                process.state = ProcessState::Ready;
                return true;
            }
        }
        false
    });

    if should_enqueue {
        enqueue(pid);
    }
}
