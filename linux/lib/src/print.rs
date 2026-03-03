//! Linux kernel print functions and miscellaneous utilities

use core::ffi::{c_char, c_int, c_long, c_ulong, c_void};
use core::ptr;

use linux_core::{linux_export, size_t};

// ============================================================================
// External print callback
// ============================================================================

unsafe extern "C" {
    fn exo_print(s: *const c_char, len: size_t);
}

// ============================================================================
// Print functions
// ============================================================================

/// Print kernel message (simplified - just prints format string)
#[linux_export]
unsafe fn printk(fmt: *const c_char) -> c_int {
    if fmt.is_null() {
        return 0;
    }

    let mut len = 0;
    let mut p = fmt;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    exo_print(fmt, len);
    len as c_int
}

#[linux_export]
unsafe fn pr_err(fmt: *const c_char) -> c_int {
    printk(fmt)
}

#[linux_export]
unsafe fn pr_warn(fmt: *const c_char) -> c_int {
    printk(fmt)
}

#[linux_export]
unsafe fn pr_info(fmt: *const c_char) -> c_int {
    printk(fmt)
}

#[linux_export]
unsafe fn pr_debug(_fmt: *const c_char) -> c_int {
    0
}

#[linux_export]
fn dump_stack() {
    // No-op in simplified implementation
}

// ============================================================================
// User/kernel space copy
// ============================================================================

#[linux_export]
unsafe fn copy_from_user(
    to: *mut c_void,
    from: *const c_void,
    n: c_ulong,
) -> c_ulong {
    if to.is_null() || from.is_null() {
        return n;
    }
    ptr::copy_nonoverlapping(from as *const u8, to as *mut u8, n as usize);
    0
}

#[linux_export]
unsafe fn copy_to_user(
    to: *mut c_void,
    from: *const c_void,
    n: c_ulong,
) -> c_ulong {
    if to.is_null() || from.is_null() {
        return n;
    }
    ptr::copy_nonoverlapping(from as *const u8, to as *mut u8, n as usize);
    0
}

#[linux_export]
unsafe fn clear_user(to: *mut c_void, n: c_ulong) -> c_ulong {
    if to.is_null() {
        return n;
    }
    ptr::write_bytes(to as *mut u8, 0, n as usize);
    0
}

#[linux_export]
unsafe fn strncpy_from_user(
    dst: *mut c_char,
    src: *const c_char,
    count: c_long,
) -> c_long {
    if dst.is_null() || src.is_null() || count <= 0 {
        return 0;
    }

    let mut i = 0;
    while i < count as usize {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            return i as c_long;
        }
        i += 1;
    }
    count
}

#[linux_export]
unsafe fn strnlen_user(s: *const c_char, n: c_long) -> c_long {
    if s.is_null() {
        return 0;
    }

    let mut len = 0;
    while len < n as usize && *s.add(len) != 0 {
        len += 1;
    }
    (len + 1) as c_long // Include null terminator
}

// ============================================================================
// Math helpers
// ============================================================================

#[linux_export]
fn __udivdi3(a: u64, b: u64) -> u64 {
    a / b
}

#[linux_export]
fn __umoddi3(a: u64, b: u64) -> u64 {
    a % b
}

#[linux_export]
fn __divdi3(a: i64, b: i64) -> i64 {
    a / b
}

#[linux_export]
fn __moddi3(a: i64, b: i64) -> i64 {
    a % b
}

// ============================================================================
// Panic and bug handling
// ============================================================================

#[unsafe(no_mangle)]
pub extern "C" fn panic(fmt: *const c_char) -> ! {
    unsafe {
        if !fmt.is_null() {
            printk(fmt);
        }
    }
    loop {
        core::hint::spin_loop();
    }
}

#[linux_export]
unsafe fn __warn_printk(fmt: *const c_char) {
    if !fmt.is_null() {
        printk(fmt);
    }
}

// ============================================================================
// Hashing
// ============================================================================

#[linux_export]
unsafe fn full_name_hash(_salt: *const c_void, name: *const c_char, len: c_ulong) -> c_ulong {
    if name.is_null() {
        return 0;
    }

    let mut hash: c_ulong = 0;
    for i in 0..len as usize {
        let c = *name.add(i) as u8;
        hash = hash.wrapping_mul(31).wrapping_add(c as c_ulong);
    }
    hash
}

#[linux_export]
unsafe fn hashlen_string(_salt: *const c_void, name: *const c_char) -> u64 {
    if name.is_null() {
        return 0;
    }

    let mut len: c_ulong = 0;
    let mut hash: c_ulong = 0;
    let mut p = name;
    while *p != 0 {
        hash = hash.wrapping_mul(31).wrapping_add(*p as u8 as c_ulong);
        len += 1;
        p = p.add(1);
    }

    // Return hash in upper 32 bits, len in lower 32 bits
    ((hash as u64) << 32) | (len as u64)
}
