//! LibOS timekeeping
//!
//! Provides time operations and delay functions.

#![no_std]

use libos_core::{Time, TimeNs, Timespec};
use libos_sync::Once;

/// Timer tick frequency (jiffies per second)
pub const HZ: u64 = 100;

/// Time operations trait
///
/// This must be implemented by the platform.
pub trait TimeOps: Send + Sync {
    /// Get current time in nanoseconds since boot
    fn get_ns(&self) -> TimeNs;
}

/// Global time operations
static TIME_OPS: Once<&'static dyn TimeOps> = Once::new();

/// Set the global time operations
///
/// Can only be called once. Subsequent calls are ignored.
pub fn set_time_ops(ops: &'static dyn TimeOps) {
    TIME_OPS.call_once(|| ops);
}

/// Get the global time operations
pub fn get_time_ops() -> Option<&'static dyn TimeOps> {
    TIME_OPS.get().copied()
}

/// Get current time in nanoseconds
pub fn get_ns() -> TimeNs {
    get_time_ops().map(|ops| ops.get_ns()).unwrap_or(0)
}

/// Get current time in seconds
pub fn get_seconds() -> Time {
    (get_ns() / 1_000_000_000) as Time
}

/// Get current timespec
pub fn current_time() -> Timespec {
    let ns = get_ns();
    Timespec {
        sec: (ns / 1_000_000_000) as Time,
        nsec: (ns % 1_000_000_000) as i64,
    }
}

/// Get jiffies (timer ticks since boot)
pub fn jiffies() -> u64 {
    get_ns() / (1_000_000_000 / HZ)
}

/// Convert jiffies to milliseconds
pub fn jiffies_to_msecs(j: u64) -> u64 {
    j * (1000 / HZ)
}

/// Convert milliseconds to jiffies
pub fn msecs_to_jiffies(m: u64) -> u64 {
    m / (1000 / HZ)
}

/// Convert jiffies to timespec
pub fn jiffies_to_timespec(j: u64) -> Timespec {
    let ns = j * (1_000_000_000 / HZ);
    Timespec {
        sec: (ns / 1_000_000_000) as Time,
        nsec: (ns % 1_000_000_000) as i64,
    }
}

/// Convert timespec to jiffies
pub fn timespec_to_jiffies(ts: &Timespec) -> u64 {
    let ns = ts.sec as u64 * 1_000_000_000 + ts.nsec as u64;
    ns / (1_000_000_000 / HZ)
}

/// Delay for a number of nanoseconds (busy wait)
pub fn ndelay(ns: u64) {
    let start = get_ns();
    while get_ns() - start < ns {
        core::hint::spin_loop();
    }
}

/// Delay for a number of microseconds (busy wait)
pub fn udelay(us: u64) {
    ndelay(us * 1000);
}

/// Delay for a number of milliseconds (busy wait)
pub fn mdelay(ms: u64) {
    udelay(ms * 1000);
}

/// Compare timespecs
pub fn timespec_compare(a: &Timespec, b: &Timespec) -> i32 {
    if a.sec < b.sec {
        -1
    } else if a.sec > b.sec {
        1
    } else if a.nsec < b.nsec {
        -1
    } else if a.nsec > b.nsec {
        1
    } else {
        0
    }
}

/// Add two timespecs
pub fn timespec_add(a: &Timespec, b: &Timespec) -> Timespec {
    let mut sec = a.sec + b.sec;
    let mut nsec = a.nsec + b.nsec;

    if nsec >= 1_000_000_000 {
        nsec -= 1_000_000_000;
        sec += 1;
    }

    Timespec { sec, nsec }
}

/// Subtract two timespecs (a - b)
pub fn timespec_sub(a: &Timespec, b: &Timespec) -> Timespec {
    let mut sec = a.sec - b.sec;
    let mut nsec = a.nsec - b.nsec;

    if nsec < 0 {
        nsec += 1_000_000_000;
        sec -= 1;
    }

    Timespec { sec, nsec }
}
