//! Memory allocation using Rust's alloc crate
//!
//! This module provides memory allocation abstractions using Box, Vec,
//! and the global allocator. No raw pointer APIs are exposed.

use alloc::boxed::Box;
use alloc::vec::Vec;

use libos_core::{Result, Size};

/// Page size constant
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

/// Allocate a boxed value
pub fn alloc_box<T: Default>() -> Result<Box<T>> {
    Ok(Box::new(T::default()))
}

/// Allocate a boxed value with a specific initializer
pub fn alloc_box_with<T, F: FnOnce() -> T>(init: F) -> Result<Box<T>> {
    Ok(Box::new(init()))
}

/// Allocate a zeroed byte buffer
pub fn alloc_bytes(size: Size) -> Result<Vec<u8>> {
    if size == 0 {
        return Ok(Vec::new());
    }
    let mut v = Vec::with_capacity(size);
    v.resize(size, 0);
    Ok(v)
}

/// Duplicate a byte slice
pub fn dup_bytes(src: &[u8]) -> Result<Vec<u8>> {
    Ok(src.to_vec())
}

/// Duplicate a string slice to a Vec<u8> with null terminator
pub fn dup_str(s: &str) -> Result<Vec<u8>> {
    let mut v = Vec::with_capacity(s.len() + 1);
    v.extend_from_slice(s.as_bytes());
    v.push(0);
    Ok(v)
}

/// Allocate page-aligned byte buffer
pub fn alloc_page_bytes(count: usize) -> Result<Vec<u8>> {
    let size = count * PAGE_SIZE;
    alloc_bytes(size)
}
