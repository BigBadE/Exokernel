//! String operations using Rust slices
//!
//! All functions use idiomatic Rust types (&str, &[u8]) instead of raw pointers.

use alloc::string::String;
use alloc::vec::Vec;

/// Get string length (bytes)
pub fn strlen(s: &str) -> usize {
    s.len()
}

/// Get byte slice length with limit
pub fn strnlen(s: &[u8], max: usize) -> usize {
    s.iter().take(max).position(|&b| b == 0).unwrap_or(max.min(s.len()))
}

/// Compare strings
pub fn strcmp(s1: &str, s2: &str) -> i32 {
    match s1.cmp(s2) {
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Equal => 0,
        core::cmp::Ordering::Greater => 1,
    }
}

/// Compare byte slices with limit
pub fn strncmp(s1: &[u8], s2: &[u8], n: usize) -> i32 {
    let s1 = &s1[..n.min(s1.len())];
    let s2 = &s2[..n.min(s2.len())];

    for (a, b) in s1.iter().zip(s2.iter()) {
        if *a == 0 && *b == 0 {
            return 0;
        }
        if *a != *b {
            return (*a as i32) - (*b as i32);
        }
        if *a == 0 {
            return 0;
        }
    }

    if s1.len() < n && s1.len() < s2.len() {
        -(s2[s1.len()] as i32)
    } else if s2.len() < n && s2.len() < s1.len() {
        s1[s2.len()] as i32
    } else {
        0
    }
}

/// Case-insensitive string compare
pub fn strcasecmp(s1: &str, s2: &str) -> i32 {
    let s1_lower = s1.to_lowercase();
    let s2_lower = s2.to_lowercase();
    strcmp(&s1_lower, &s2_lower)
}

/// Find character in string
pub fn strchr(s: &str, c: char) -> Option<usize> {
    s.find(c)
}

/// Find last character in string
pub fn strrchr(s: &str, c: char) -> Option<usize> {
    s.rfind(c)
}

/// Find substring in string
pub fn strstr(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)
}

/// Check if string starts with prefix
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Check if string ends with suffix
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

/// Split a string by delimiter
pub fn split<'a>(s: &'a str, delim: char) -> impl Iterator<Item = &'a str> {
    s.split(delim)
}

/// Trim whitespace from both ends
pub fn trim(s: &str) -> &str {
    s.trim()
}

/// Convert bytes to string (lossy)
pub fn from_utf8_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Convert C-style null-terminated bytes to string
pub fn from_cstr(bytes: &[u8]) -> Option<&str> {
    let len = bytes.iter().position(|&b| b == 0)?;
    core::str::from_utf8(&bytes[..len]).ok()
}

/// Convert string to null-terminated bytes
pub fn to_cstr(s: &str) -> Vec<u8> {
    let mut v = Vec::with_capacity(s.len() + 1);
    v.extend_from_slice(s.as_bytes());
    v.push(0);
    v
}

// =============================================================================
// Memory operations (using slices)
// =============================================================================

/// Copy memory (non-overlapping)
pub fn memcpy(dst: &mut [u8], src: &[u8]) -> usize {
    let len = dst.len().min(src.len());
    dst[..len].copy_from_slice(&src[..len]);
    len
}

/// Move memory (handles overlap)
pub fn memmove(dst: &mut [u8], src: &[u8]) -> usize {
    let len = dst.len().min(src.len());
    // copy_from_slice handles overlapping correctly when used with proper slicing
    dst[..len].copy_from_slice(&src[..len]);
    len
}

/// Set memory to a value
pub fn memset(dst: &mut [u8], val: u8) {
    dst.fill(val);
}

/// Compare memory
pub fn memcmp(s1: &[u8], s2: &[u8]) -> i32 {
    let len = s1.len().min(s2.len());
    for i in 0..len {
        if s1[i] != s2[i] {
            return (s1[i] as i32) - (s2[i] as i32);
        }
    }
    (s1.len() as i32) - (s2.len() as i32)
}

/// Find byte in memory
pub fn memchr(s: &[u8], c: u8) -> Option<usize> {
    s.iter().position(|&b| b == c)
}

/// Find byte in memory (reverse)
pub fn memrchr(s: &[u8], c: u8) -> Option<usize> {
    s.iter().rposition(|&b| b == c)
}

/// Check if slices are equal
pub fn memeq(s1: &[u8], s2: &[u8]) -> bool {
    s1 == s2
}

/// Zero out memory
pub fn memzero(dst: &mut [u8]) {
    dst.fill(0);
}
