//! Slab allocator implementation

use alloc::boxed::Box;
use core::ffi::{c_char, c_uint, c_void};
use core::ptr;

use linux_core::{gfp_t, linux_export, size_t, GFP_ZERO};
use crate::alloc_impl::kmalloc;

// ============================================================================
// Slab cache structure
// ============================================================================

/// Slab cache (simplified implementation)
#[repr(C)]
pub struct kmem_cache {
    pub name: *const c_char,
    pub size: size_t,
    pub align: size_t,
    pub flags: c_uint,
    pub ctor: Option<unsafe extern "C" fn(*mut c_void)>,
}

impl kmem_cache {
    pub fn new(
        name: *const c_char,
        size: size_t,
        align: size_t,
        flags: c_uint,
        ctor: Option<unsafe extern "C" fn(*mut c_void)>,
    ) -> Self {
        Self {
            name,
            size,
            align: if align == 0 { 8 } else { align },
            flags,
            ctor,
        }
    }
}

// ============================================================================
// Slab cache operations
// ============================================================================

/// Create a slab cache
#[linux_export]
fn kmem_cache_create(
    name: *const c_char,
    size: size_t,
    align: size_t,
    flags: c_uint,
    ctor: Option<unsafe extern "C" fn(*mut c_void)>,
) -> *mut kmem_cache {
    let cache = Box::new(kmem_cache::new(name, size, align, flags, ctor));
    Box::into_raw(cache)
}

/// Destroy a slab cache
#[linux_export]
unsafe fn kmem_cache_destroy(cache: *mut kmem_cache) {
    if cache.is_null() {
        return;
    }
    let _ = Box::from_raw(cache);
}

/// Allocate from slab cache
#[linux_export]
unsafe fn kmem_cache_alloc(cache: *mut kmem_cache, flags: gfp_t) -> *mut c_void {
    if cache.is_null() {
        return ptr::null_mut();
    }

    let size = (*cache).size;
    let ptr = kmalloc(size, flags);

    if !ptr.is_null() {
        if let Some(ctor) = (*cache).ctor {
            ctor(ptr);
        }
    }

    ptr
}

/// Allocate zeroed from slab cache
#[linux_export]
unsafe fn kmem_cache_zalloc(cache: *mut kmem_cache, flags: gfp_t) -> *mut c_void {
    kmem_cache_alloc(cache, flags | GFP_ZERO)
}

/// Free to slab cache
#[linux_export]
fn kmem_cache_free(_cache: *mut kmem_cache, _ptr: *mut c_void) {
    // Simplified: just leak the memory
    // Real implementation would return to slab
}
