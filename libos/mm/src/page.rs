//! Page management using Rust types
//!
//! Provides page abstractions with automatic memory management.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicI32, Ordering};

use libos_core::Result;
use crate::allocator::PAGE_SIZE;

/// Page flags
pub mod flags {
    pub const LOCKED: u32 = 1 << 0;
    pub const DIRTY: u32 = 1 << 1;
    pub const LRU: u32 = 1 << 2;
    pub const ACTIVE: u32 = 1 << 3;
    pub const UPTODATE: u32 = 1 << 11;
}

/// A memory page with associated data
pub struct Page {
    /// Page flags
    flags: AtomicU32,
    /// Reference count
    refcount: AtomicI32,
    /// Page index (for file-backed pages)
    index: u64,
    /// The actual page data
    data: Vec<u8>,
}

impl Page {
    /// Create a new zeroed page
    pub fn new() -> Self {
        Self {
            flags: AtomicU32::new(0),
            refcount: AtomicI32::new(1),
            index: 0,
            data: alloc::vec![0u8; PAGE_SIZE],
        }
    }

    /// Create a new page with specific size
    pub fn with_size(size: usize) -> Self {
        Self {
            flags: AtomicU32::new(0),
            refcount: AtomicI32::new(1),
            index: 0,
            data: alloc::vec![0u8; size],
        }
    }

    /// Get the page data as a slice
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the page data as a mutable slice
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get the page size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get the page index
    pub fn index(&self) -> u64 {
        self.index
    }

    /// Set the page index
    pub fn set_index(&mut self, index: u64) {
        self.index = index;
    }

    /// Check if the page is locked
    pub fn is_locked(&self) -> bool {
        (self.flags.load(Ordering::Relaxed) & flags::LOCKED) != 0
    }

    /// Check if the page is dirty
    pub fn is_dirty(&self) -> bool {
        (self.flags.load(Ordering::Relaxed) & flags::DIRTY) != 0
    }

    /// Check if the page is up to date
    pub fn is_uptodate(&self) -> bool {
        (self.flags.load(Ordering::Relaxed) & flags::UPTODATE) != 0
    }

    /// Lock the page
    pub fn lock(&self) {
        while self.flags.fetch_or(flags::LOCKED, Ordering::Acquire) & flags::LOCKED != 0 {
            core::hint::spin_loop();
        }
    }

    /// Unlock the page
    pub fn unlock(&self) {
        self.flags.fetch_and(!flags::LOCKED, Ordering::Release);
    }

    /// Try to lock the page
    pub fn try_lock(&self) -> bool {
        (self.flags.fetch_or(flags::LOCKED, Ordering::Acquire) & flags::LOCKED) == 0
    }

    /// Mark the page dirty
    pub fn set_dirty(&self) {
        self.flags.fetch_or(flags::DIRTY, Ordering::SeqCst);
    }

    /// Clear the dirty flag
    pub fn clear_dirty(&self) {
        self.flags.fetch_and(!flags::DIRTY, Ordering::SeqCst);
    }

    /// Mark the page up to date
    pub fn set_uptodate(&self) {
        self.flags.fetch_or(flags::UPTODATE, Ordering::SeqCst);
    }

    /// Clear the up to date flag
    pub fn clear_uptodate(&self) {
        self.flags.fetch_and(!flags::UPTODATE, Ordering::SeqCst);
    }

    /// Get the reference count
    pub fn refcount(&self) -> i32 {
        self.refcount.load(Ordering::Relaxed)
    }

    /// Increment reference count
    pub fn get(&self) {
        self.refcount.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement reference count, returns true if this was the last reference
    pub fn put(&self) -> bool {
        self.refcount.fetch_sub(1, Ordering::SeqCst) == 1
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

/// Allocate a new page
pub fn alloc_page() -> Result<Box<Page>> {
    Ok(Box::new(Page::new()))
}

/// Allocate multiple contiguous pages
pub fn alloc_pages(order: u32) -> Result<Vec<Box<Page>>> {
    let count = 1usize << order;
    let mut pages = Vec::with_capacity(count);
    for _ in 0..count {
        pages.push(Box::new(Page::new()));
    }
    Ok(pages)
}

/// A shared reference-counted page
pub type SharedPage = Arc<Page>;

/// Allocate a shared page
pub fn alloc_shared_page() -> Result<SharedPage> {
    Ok(Arc::new(Page::new()))
}
