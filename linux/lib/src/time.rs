//! Linux kernel time functions

use core::ffi::{c_int, c_long, c_uint, c_ulong, c_void};

use linux_core::{linux_export, time64_t, timespec64};

// ============================================================================
// External time callback
// ============================================================================

unsafe extern "C" {
    fn exo_get_time_ns() -> c_ulong;
}

// ============================================================================
// Current time
// ============================================================================

#[linux_export]
unsafe fn get_seconds() -> c_ulong {
    exo_get_time_ns() / 1_000_000_000
}

#[linux_export]
unsafe fn current_time(_inode: *mut c_void) -> timespec64 {
    let ns = exo_get_time_ns();
    timespec64 {
        tv_sec: (ns / 1_000_000_000) as time64_t,
        tv_nsec: (ns % 1_000_000_000) as c_long,
    }
}

#[linux_export]
unsafe fn ktime_get() -> i64 {
    exo_get_time_ns() as i64
}

#[linux_export]
unsafe fn ktime_get_real() -> i64 {
    exo_get_time_ns() as i64
}

fn get_current_time_ts64(ts: &mut timespec64) {
    let ns = unsafe { exo_get_time_ns() };
    ts.tv_sec = (ns / 1_000_000_000) as time64_t;
    ts.tv_nsec = (ns % 1_000_000_000) as c_long;
}

#[linux_export]
fn ktime_get_coarse_real_ts64(ts: &mut timespec64) {
    get_current_time_ts64(ts)
}

#[linux_export]
fn ktime_get_real_ts64(ts: &mut timespec64) {
    get_current_time_ts64(ts)
}

// ============================================================================
// Time conversion
// ============================================================================

#[linux_export]
fn timespec64_to_ktime(ts: timespec64) -> i64 {
    ts.tv_sec as i64 * 1_000_000_000 + ts.tv_nsec as i64
}

#[linux_export]
fn ktime_to_timespec64(kt: i64) -> timespec64 {
    let secs = kt / 1_000_000_000;
    let nsecs = kt % 1_000_000_000;
    timespec64 {
        tv_sec: secs as time64_t,
        tv_nsec: nsecs as c_long,
    }
}

#[linux_export]
fn timespec64_compare(lhs: &timespec64, rhs: &timespec64) -> c_int {
    if lhs.tv_sec < rhs.tv_sec {
        return -1;
    }
    if lhs.tv_sec > rhs.tv_sec {
        return 1;
    }
    if lhs.tv_nsec < rhs.tv_nsec {
        return -1;
    }
    if lhs.tv_nsec > rhs.tv_nsec {
        return 1;
    }
    0
}

#[linux_export]
fn timespec64_trunc(t: timespec64, gran: c_uint) -> timespec64 {
    if gran == 0 || gran == 1 {
        return t;
    }

    let gran = gran as i64;
    let nsec = t.tv_nsec as i64;
    let remainder = nsec % gran;

    timespec64 {
        tv_sec: t.tv_sec,
        tv_nsec: (nsec - remainder) as c_long,
    }
}

#[linux_export]
fn timespec64_add(lhs: timespec64, rhs: timespec64) -> timespec64 {
    let mut result = timespec64 {
        tv_sec: lhs.tv_sec + rhs.tv_sec,
        tv_nsec: lhs.tv_nsec + rhs.tv_nsec,
    };

    if result.tv_nsec >= 1_000_000_000 {
        result.tv_sec += 1;
        result.tv_nsec -= 1_000_000_000;
    }

    result
}

#[linux_export]
fn timespec64_sub(lhs: timespec64, rhs: timespec64) -> timespec64 {
    let mut result = timespec64 {
        tv_sec: lhs.tv_sec - rhs.tv_sec,
        tv_nsec: lhs.tv_nsec - rhs.tv_nsec,
    };

    if result.tv_nsec < 0 {
        result.tv_sec -= 1;
        result.tv_nsec += 1_000_000_000;
    }

    result
}

// ============================================================================
// Jiffies
// ============================================================================

pub const HZ: c_ulong = 1000;

#[linux_export]
unsafe fn jiffies_get() -> c_ulong {
    exo_get_time_ns() / (1_000_000_000 / HZ)
}

#[unsafe(no_mangle)]
pub static mut jiffies: c_ulong = 0;

#[linux_export]
fn jiffies_to_msecs(j: c_ulong) -> c_uint {
    (j * 1000 / HZ) as c_uint
}

#[linux_export]
fn msecs_to_jiffies(m: c_uint) -> c_ulong {
    (m as c_ulong) * HZ / 1000
}

#[linux_export]
unsafe fn jiffies_to_timespec64(j: c_ulong, value: *mut timespec64) {
    if value.is_null() {
        return;
    }
    let nsec = j as i64 * (1_000_000_000 / HZ as i64);
    (*value).tv_sec = (nsec / 1_000_000_000) as time64_t;
    (*value).tv_nsec = (nsec % 1_000_000_000) as c_long;
}

#[linux_export]
fn timespec64_to_jiffies(value: &timespec64) -> c_ulong {
    let nsec = value.tv_sec as i64 * 1_000_000_000 + value.tv_nsec as i64;
    (nsec / (1_000_000_000 / HZ as i64)) as c_ulong
}

// ============================================================================
// Delays
// ============================================================================

#[linux_export]
fn msleep(_msecs: c_uint) {
}

#[linux_export]
fn usleep_range(_min: c_ulong, _max: c_ulong) {
}

#[linux_export]
fn udelay(_usecs: c_ulong) {
}

#[linux_export]
fn ndelay(_nsecs: c_ulong) {
}
