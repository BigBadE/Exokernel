//! Linux kmalloc/kfree wrappers
//!
//! These wrap the libos memory allocator with Linux-compatible signatures.

use core::ffi::{c_ulong, c_void};

use linux_core::{gfp_t, linux_export, size_t, GFP_ZERO};
use libos_core::AllocFlags;
use libos_mm::allocator;

// Re-export page constants
pub use libos_mm::allocator::{PAGE_SIZE, PAGE_SHIFT};

/// Convert GFP flags to libos AllocFlags
fn gfp_to_alloc_flags(gfp: gfp_t) -> AllocFlags {
    let mut flags = AllocFlags::NONE;
    if (gfp & GFP_ZERO) != 0 {
        flags = flags | AllocFlags::ZERO;
    }
    flags
}

/// Allocate kernel memory
#[linux_export]
unsafe fn kmalloc(size: size_t, flags: gfp_t) -> *mut c_void {
    allocator::allocate(size, gfp_to_alloc_flags(flags)) as *mut c_void
}

/// Allocate zeroed kernel memory
#[linux_export]
unsafe fn kzalloc(size: size_t, flags: gfp_t) -> *mut c_void {
    allocator::allocate_zeroed(size, gfp_to_alloc_flags(flags)) as *mut c_void
}

/// Allocate array
#[linux_export]
unsafe fn kmalloc_array(n: size_t, size: size_t, flags: gfp_t) -> *mut c_void {
    allocator::allocate_array(n, size, gfp_to_alloc_flags(flags)) as *mut c_void
}

/// Allocate zeroed array
#[linux_export]
unsafe fn kcalloc(n: size_t, size: size_t, flags: gfp_t) -> *mut c_void {
    allocator::allocate_array(n, size, gfp_to_alloc_flags(flags) | AllocFlags::ZERO) as *mut c_void
}

/// Free kernel memory
#[linux_export]
fn kfree(ptr: *const c_void) {
    allocator::free(ptr as *const u8);
}

/// Reallocate kernel memory
#[linux_export]
unsafe fn krealloc(ptr: *const c_void, new_size: size_t, flags: gfp_t) -> *mut c_void {
    allocator::reallocate(ptr as *const u8, new_size, gfp_to_alloc_flags(flags)) as *mut c_void
}

/// Get allocation size (simplified - returns 0)
#[linux_export]
fn ksize(_ptr: *const c_void) -> size_t {
    // Would need allocation tracking to implement properly
    0
}

/// Duplicate memory
#[linux_export]
unsafe fn kmemdup(src: *const c_void, len: size_t, flags: gfp_t) -> *mut c_void {
    allocator::memdup(src as *const u8, len, gfp_to_alloc_flags(flags)) as *mut c_void
}

/// Duplicate string
#[linux_export]
unsafe fn kstrdup(s: *const u8, flags: gfp_t) -> *mut u8 {
    allocator::strdup(s, gfp_to_alloc_flags(flags))
}

/// Free virtual memory (same as kfree for us)
#[linux_export]
fn kvfree(ptr: *const c_void) {
    allocator::vfree(ptr as *const u8);
}

/// Allocate virtual memory
#[linux_export]
unsafe fn vmalloc(size: c_ulong) -> *mut c_void {
    allocator::vmalloc(size as size_t) as *mut c_void
}

/// Free virtual memory
#[linux_export]
fn vfree(ptr: *const c_void) {
    allocator::vfree(ptr as *const u8);
}
