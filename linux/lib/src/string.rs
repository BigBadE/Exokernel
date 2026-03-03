//! Linux kernel string and memory operations

use core::ffi::{c_char, c_int, c_long, c_ulong, c_void};
use core::ptr;

use linux_core::{linux_export, size_t};

// ============================================================================
// Memory operations
// ============================================================================

#[linux_export]
unsafe fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void {
    if dest.is_null() || src.is_null() || n == 0 {
        return dest;
    }
    ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, n);
    dest
}

#[linux_export]
unsafe fn memmove(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void {
    if dest.is_null() || src.is_null() || n == 0 {
        return dest;
    }
    ptr::copy(src as *const u8, dest as *mut u8, n);
    dest
}

#[linux_export]
unsafe fn memset(dest: *mut c_void, c: c_int, n: size_t) -> *mut c_void {
    if dest.is_null() || n == 0 {
        return dest;
    }
    ptr::write_bytes(dest as *mut u8, c as u8, n);
    dest
}

#[linux_export]
unsafe fn memcmp(s1: *const c_void, s2: *const c_void, n: size_t) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }
    let p1 = s1 as *const u8;
    let p2 = s2 as *const u8;
    for i in 0..n {
        let a = *p1.add(i);
        let b = *p2.add(i);
        if a != b {
            return (a as c_int) - (b as c_int);
        }
    }
    0
}

#[linux_export]
unsafe fn memchr(s: *const c_void, c: c_int, n: size_t) -> *mut c_void {
    if s.is_null() {
        return ptr::null_mut();
    }
    let p = s as *const u8;
    let c = c as u8;
    for i in 0..n {
        if *p.add(i) == c {
            return p.add(i) as *mut c_void;
        }
    }
    ptr::null_mut()
}

// ============================================================================
// String operations
// ============================================================================

#[linux_export]
unsafe fn strlen(s: *const c_char) -> size_t {
    if s.is_null() {
        return 0;
    }
    let mut len = 0;
    let mut p = s;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

#[linux_export]
unsafe fn strnlen(s: *const c_char, maxlen: size_t) -> size_t {
    if s.is_null() {
        return 0;
    }
    let mut len = 0;
    let mut p = s;
    while len < maxlen && *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

#[linux_export]
unsafe fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    if dest.is_null() || src.is_null() {
        return dest;
    }
    let mut d = dest;
    let mut s = src;
    while *s != 0 {
        *d = *s;
        d = d.add(1);
        s = s.add(1);
    }
    *d = 0;
    dest
}

#[linux_export]
unsafe fn strncpy(dest: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char {
    if dest.is_null() || src.is_null() {
        return dest;
    }
    let mut i = 0;
    while i < n {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    while i < n {
        *dest.add(i) = 0;
        i += 1;
    }
    dest
}

#[linux_export]
unsafe fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    if dest.is_null() || src.is_null() {
        return dest;
    }
    let len = strlen(dest);
    strcpy(dest.add(len), src);
    dest
}

#[linux_export]
unsafe fn strncat(dest: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char {
    if dest.is_null() || src.is_null() {
        return dest;
    }
    let dest_len = strlen(dest);
    let mut i = 0;
    while i < n {
        let c = *src.add(i);
        if c == 0 {
            break;
        }
        *dest.add(dest_len + i) = c;
        i += 1;
    }
    *dest.add(dest_len + i) = 0;
    dest
}

#[linux_export]
unsafe fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }
    let mut p1 = s1;
    let mut p2 = s2;
    while *p1 != 0 && *p1 == *p2 {
        p1 = p1.add(1);
        p2 = p2.add(1);
    }
    (*p1 as u8 as c_int) - (*p2 as u8 as c_int)
}

#[linux_export]
unsafe fn strncmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
    if s1.is_null() || s2.is_null() || n == 0 {
        return 0;
    }
    let mut i = 0;
    while i < n {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 != c2 {
            return (c1 as u8 as c_int) - (c2 as u8 as c_int);
        }
        if c1 == 0 {
            break;
        }
        i += 1;
    }
    0
}

#[inline]
fn to_lower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' { c + 32 } else { c }
}

#[inline]
fn to_upper(c: u8) -> u8 {
    if c >= b'a' && c <= b'z' { c - 32 } else { c }
}

#[linux_export]
unsafe fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }
    let mut p1 = s1;
    let mut p2 = s2;
    while *p1 != 0 {
        let c1 = to_lower(*p1 as u8);
        let c2 = to_lower(*p2 as u8);
        if c1 != c2 {
            return (c1 as c_int) - (c2 as c_int);
        }
        if *p2 == 0 {
            break;
        }
        p1 = p1.add(1);
        p2 = p2.add(1);
    }
    (to_lower(*p1 as u8) as c_int) - (to_lower(*p2 as u8) as c_int)
}

#[linux_export]
unsafe fn strncasecmp(s1: *const c_char, s2: *const c_char, n: size_t) -> c_int {
    if s1.is_null() || s2.is_null() || n == 0 {
        return 0;
    }
    let mut i = 0;
    while i < n {
        let c1 = to_lower(*s1.add(i) as u8);
        let c2 = to_lower(*s2.add(i) as u8);
        if c1 != c2 {
            return (c1 as c_int) - (c2 as c_int);
        }
        if c1 == 0 {
            break;
        }
        i += 1;
    }
    0
}

#[linux_export]
unsafe fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    let c = c as c_char;
    let mut p = s;
    while *p != 0 {
        if *p == c {
            return p as *mut c_char;
        }
        p = p.add(1);
    }
    if c == 0 {
        return p as *mut c_char;
    }
    ptr::null_mut()
}

#[linux_export]
unsafe fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    let c = c as c_char;
    let mut result: *mut c_char = ptr::null_mut();
    let mut p = s;
    while *p != 0 {
        if *p == c {
            result = p as *mut c_char;
        }
        p = p.add(1);
    }
    if c == 0 {
        return p as *mut c_char;
    }
    result
}

#[linux_export]
unsafe fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    if haystack.is_null() || needle.is_null() {
        return ptr::null_mut();
    }
    if *needle == 0 {
        return haystack as *mut c_char;
    }
    let needle_len = strlen(needle);
    let mut p = haystack;
    while *p != 0 {
        if strncmp(p, needle, needle_len) == 0 {
            return p as *mut c_char;
        }
        p = p.add(1);
    }
    ptr::null_mut()
}

// ============================================================================
// Character classification
// ============================================================================

#[linux_export]
fn tolower(c: c_int) -> c_int {
    to_lower(c as u8) as c_int
}

#[linux_export]
fn toupper(c: c_int) -> c_int {
    to_upper(c as u8) as c_int
}

#[linux_export]
fn isalpha(c: c_int) -> c_int {
    let c = c as u8;
    ((c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z')) as c_int
}

#[linux_export]
fn isdigit(c: c_int) -> c_int {
    let c = c as u8;
    (c >= b'0' && c <= b'9') as c_int
}

#[linux_export]
fn isalnum(c: c_int) -> c_int {
    let c = c as u8;
    ((c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z') || (c >= b'0' && c <= b'9')) as c_int
}

#[linux_export]
fn isspace(c: c_int) -> c_int {
    let c = c as u8;
    (c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0B || c == 0x0C) as c_int
}

#[linux_export]
fn isxdigit(c: c_int) -> c_int {
    let c = c as u8;
    ((c >= b'0' && c <= b'9') || (c >= b'A' && c <= b'F') || (c >= b'a' && c <= b'f')) as c_int
}

#[linux_export]
fn isupper(c: c_int) -> c_int {
    let c = c as u8;
    (c >= b'A' && c <= b'Z') as c_int
}

#[linux_export]
fn islower(c: c_int) -> c_int {
    let c = c as u8;
    (c >= b'a' && c <= b'z') as c_int
}

// ============================================================================
// Number parsing
// ============================================================================

#[linux_export]
unsafe fn simple_strtoul(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_ulong {
    if s.is_null() {
        return 0;
    }

    let mut p = s;
    let mut result: c_ulong = 0;
    let base = if base == 0 {
        if *p == b'0' as c_char {
            p = p.add(1);
            if *p == b'x' as c_char || *p == b'X' as c_char {
                p = p.add(1);
                16
            } else {
                8
            }
        } else {
            10
        }
    } else {
        base as c_ulong
    };

    while *p != 0 {
        let c = *p as u8;
        let digit = if c >= b'0' && c <= b'9' {
            (c - b'0') as c_ulong
        } else if c >= b'a' && c <= b'z' {
            (c - b'a' + 10) as c_ulong
        } else if c >= b'A' && c <= b'Z' {
            (c - b'A' + 10) as c_ulong
        } else {
            break;
        };

        if digit >= base {
            break;
        }

        result = result * base + digit;
        p = p.add(1);
    }

    if !endptr.is_null() {
        *endptr = p as *mut c_char;
    }

    result
}

#[linux_export]
unsafe fn simple_strtol(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_long {
    if s.is_null() {
        return 0;
    }

    let mut p = s;
    let negative = if *p == b'-' as c_char {
        p = p.add(1);
        true
    } else {
        if *p == b'+' as c_char {
            p = p.add(1);
        }
        false
    };

    let result = simple_strtoul(p, endptr, base) as c_long;
    if negative { -result } else { result }
}

#[linux_export]
unsafe fn simple_strtoull(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> u64 {
    simple_strtoul(s, endptr, base) as u64
}

#[linux_export]
unsafe fn simple_strtoll(
    s: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> i64 {
    simple_strtol(s, endptr, base) as i64
}
