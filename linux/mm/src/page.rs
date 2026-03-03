//! Page allocation and management

use core::ffi::{c_int, c_ulong, c_void};
use core::ptr;

use linux_core::{atomic_t, gfp_t, linux_export, GFP_KERNEL};
use crate::alloc_impl::{kfree, kzalloc, PAGE_SIZE};

// ============================================================================
// Page flags
// ============================================================================

pub const PG_locked: c_ulong = 0;
pub const PG_uptodate: c_ulong = 1;
pub const PG_dirty: c_ulong = 2;
pub const PG_writeback: c_ulong = 3;

// ============================================================================
// Page structure
// ============================================================================

/// A memory page
#[repr(C)]
pub struct page {
    pub flags: c_ulong,
    pub mapping: *mut c_void,
    pub index: c_ulong,
    pub private: c_ulong,
    pub _refcount: atomic_t,
    pub data: *mut u8,
}

impl page {
    pub fn new() -> *mut page {
        let p = kzalloc(core::mem::size_of::<page>() + PAGE_SIZE, GFP_KERNEL) as *mut page;
        if !p.is_null() {
            unsafe {
                (*p).data = (p as *mut u8).add(core::mem::size_of::<page>());
                (*p)._refcount = atomic_t::new(1);
            }
        }
        p
    }
}

// ============================================================================
// Page operations
// ============================================================================

/// Get page data pointer
#[linux_export]
unsafe fn page_address(p: *mut page) -> *mut c_void {
    if p.is_null() {
        return ptr::null_mut();
    }
    (*p).data as *mut c_void
}

/// Map page to kernel address
#[linux_export]
unsafe fn kmap(p: *mut page) -> *mut c_void {
    page_address(p)
}

/// Unmap page
#[linux_export]
fn kunmap(_p: *mut page) {
    // No-op in simplified implementation
}

/// Map page atomically
#[linux_export]
unsafe fn kmap_atomic(p: *mut page) -> *mut c_void {
    page_address(p)
}

/// Unmap atomic mapping
#[linux_export]
fn kunmap_atomic(_addr: *mut c_void) {
    // No-op
}

/// Map page locally
#[linux_export]
unsafe fn kmap_local_page(p: *mut page) -> *mut c_void {
    page_address(p)
}

/// Unmap local mapping
#[linux_export]
fn kunmap_local(_addr: *mut c_void) {
    // No-op
}

/// Get page reference
#[linux_export]
fn get_page(p: &page) {
    p._refcount.inc();
}

/// Put page reference
#[linux_export]
unsafe fn put_page(p: *mut page) {
    if p.is_null() {
        return;
    }
    let page_ref = &*p;
    if page_ref._refcount.dec_and_test() {
        kfree(p as *const c_void);
    }
}

// ============================================================================
// Page flag operations
// ============================================================================

/// Check if page is locked
#[linux_export]
fn PageLocked(p: &page) -> c_int {
    ((p.flags & (1 << PG_locked)) != 0) as c_int
}

/// Check if page is uptodate
#[linux_export]
fn PageUptodate(p: &page) -> c_int {
    ((p.flags & (1 << PG_uptodate)) != 0) as c_int
}

/// Set page uptodate
#[linux_export]
fn SetPageUptodate(p: &mut page) {
    p.flags |= 1 << PG_uptodate;
}

/// Clear page uptodate
#[linux_export]
fn ClearPageUptodate(p: &mut page) {
    p.flags &= !(1 << PG_uptodate);
}

/// Lock a page
#[linux_export]
fn lock_page(p: &mut page) {
    while (p.flags & (1 << PG_locked)) != 0 {
        core::hint::spin_loop();
    }
    p.flags |= 1 << PG_locked;
}

/// Unlock a page
#[linux_export]
fn unlock_page(p: &mut page) {
    p.flags &= !(1 << PG_locked);
}

/// Try to lock a page
#[linux_export]
fn trylock_page(p: &mut page) -> c_int {
    if (p.flags & (1 << PG_locked)) == 0 {
        p.flags |= 1 << PG_locked;
        1
    } else {
        0
    }
}

/// Wait for page to be unlocked
#[linux_export]
fn wait_on_page_locked(p: &page) {
    while (p.flags & (1 << PG_locked)) != 0 {
        core::hint::spin_loop();
    }
}

// ============================================================================
// Page allocation
// ============================================================================

/// Allocate pages (2^order pages)
#[linux_export]
fn alloc_pages(_gfp: gfp_t, order: c_int) -> *mut page {
    let count = 1usize << order;
    for _ in 0..count {
        let p = page::new();
        if p.is_null() {
            return ptr::null_mut();
        }
        // For simplicity, just return the first page
        // Real implementation would link them together
        return p;
    }
    ptr::null_mut()
}

/// Allocate single page
#[linux_export]
fn alloc_page(_gfp: gfp_t) -> *mut page {
    page::new()
}

/// Free pages
#[linux_export]
unsafe fn __free_pages(p: *mut page, _order: c_int) {
    if !p.is_null() {
        put_page(p);
    }
}

/// Free single page
#[linux_export]
fn free_page(_addr: c_ulong) {
    // Simplified - would need to convert address to page
}

/// Get free pages and return address
#[linux_export]
unsafe fn __get_free_pages(gfp: gfp_t, order: c_int) -> c_ulong {
    let p = alloc_pages(gfp, order);
    if p.is_null() {
        return 0;
    }
    page_address(p) as c_ulong
}

/// Get zeroed page
#[linux_export]
unsafe fn get_zeroed_page(gfp: gfp_t) -> c_ulong {
    let p = alloc_page(gfp);
    if p.is_null() {
        return 0;
    }
    let addr = page_address(p);
    if !addr.is_null() {
        ptr::write_bytes(addr as *mut u8, 0, PAGE_SIZE);
    }
    addr as c_ulong
}
