//! Memory management
//!
//! This module provides:
//! - Raw syscall wrappers for memory operations
//! - A simple bump allocator for user-space heap
//! - Memory mapping utilities

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use exo_shared::{CapabilityHandle, SysError, SyscallNumber};
use crate::{syscall2, syscall3};

/// Page size (4 KiB)
pub const PAGE_SIZE: usize = 4096;

/// Memory mapping flags
pub mod flags {
    pub const READ: u64 = 1 << 0;
    pub const WRITE: u64 = 1 << 1;
    pub const EXECUTE: u64 = 1 << 2;
    pub const USER: u64 = 1 << 3;

    pub const RW: u64 = READ | WRITE;
    pub const RWX: u64 = READ | WRITE | EXECUTE;
    pub const RW_USER: u64 = READ | WRITE | USER;
}

// =============================================================================
// Raw Syscall Wrappers
// =============================================================================

/// Allocate physical memory frames
/// Returns a capability for the allocated frames
pub fn alloc_phys(num_frames: u64, flags: u64) -> Result<CapabilityHandle, SysError> {
    let ret = unsafe {
        syscall2(SyscallNumber::MemAllocPhys as u64, num_frames, flags)
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(CapabilityHandle::from_raw(ret as u64))
    }
}

/// Map memory capability to virtual address
pub fn map(
    mem_cap: CapabilityHandle,
    virt_addr: u64,
    flags: u64,
) -> Result<(), SysError> {
    let ret = unsafe {
        syscall3(
            SyscallNumber::MemMap as u64,
            mem_cap.as_raw(),
            virt_addr,
            flags,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Unmap memory
pub fn unmap(virt_addr: u64, num_pages: u64) -> Result<(), SysError> {
    let ret = unsafe {
        syscall2(
            SyscallNumber::MemUnmap as u64,
            virt_addr,
            num_pages,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Change memory protection
pub fn protect(virt_addr: u64, num_pages: u64, flags: u64) -> Result<(), SysError> {
    let ret = unsafe {
        syscall3(
            SyscallNumber::MemProtect as u64,
            virt_addr,
            num_pages,
            flags,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Query physical address for a virtual address
pub fn query(virt_addr: u64) -> Result<u64, SysError> {
    let ret = unsafe {
        syscall2(SyscallNumber::MemQuery as u64, virt_addr, 0)
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(ret as u64)
    }
}

// =============================================================================
// User-Space Heap Allocator
// =============================================================================

/// Simple bump allocator for user-space heap
///
/// This allocator requests pages from the kernel as needed and
/// allocates memory linearly. It does not support freeing individual
/// allocations (the entire heap is freed when the process exits).
pub struct BumpAllocator {
    /// Start of the heap region
    heap_start: AtomicU64,
    /// Current allocation pointer
    heap_ptr: AtomicU64,
    /// End of currently mapped heap
    heap_end: AtomicU64,
    /// Total bytes allocated
    allocated: AtomicUsize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: AtomicU64::new(0),
            heap_ptr: AtomicU64::new(0),
            heap_end: AtomicU64::new(0),
            allocated: AtomicUsize::new(0),
        }
    }

    /// Initialize the allocator with a heap region
    ///
    /// `heap_start` should be a page-aligned virtual address in user space
    pub fn init(&self, heap_start: u64) {
        self.heap_start.store(heap_start, Ordering::SeqCst);
        self.heap_ptr.store(heap_start, Ordering::SeqCst);
        self.heap_end.store(heap_start, Ordering::SeqCst);
    }

    /// Allocate memory
    ///
    /// Returns a pointer to the allocated memory, or an error if allocation fails.
    pub fn alloc(&self, size: usize, align: usize) -> Result<*mut u8, SysError> {
        loop {
            let current = self.heap_ptr.load(Ordering::SeqCst);
            let aligned = align_up(current as usize, align) as u64;
            let new_end = aligned + size as u64;

            // Check if we need to map more memory
            let heap_end = self.heap_end.load(Ordering::SeqCst);
            if new_end > heap_end {
                // Calculate how many pages we need
                let needed = new_end - heap_end;
                let pages = align_up(needed as usize, PAGE_SIZE) / PAGE_SIZE;

                // Allocate physical frames
                let cap = alloc_phys(pages as u64, 0)?;

                // Map them at the current heap end
                map(cap, heap_end, flags::RW_USER)?;

                // Update heap end
                let new_heap_end = heap_end + (pages * PAGE_SIZE) as u64;
                self.heap_end.store(new_heap_end, Ordering::SeqCst);
            }

            // Try to claim this allocation
            if self.heap_ptr.compare_exchange(
                current,
                new_end,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ).is_ok() {
                self.allocated.fetch_add(size, Ordering::Relaxed);
                return Ok(aligned as *mut u8);
            }
            // Another thread beat us, retry
        }
    }

    /// Get allocation statistics
    pub fn stats(&self) -> (usize, usize) {
        let allocated = self.allocated.load(Ordering::Relaxed);
        let heap_size = (self.heap_end.load(Ordering::Relaxed)
            - self.heap_start.load(Ordering::Relaxed)) as usize;
        (allocated, heap_size)
    }
}

/// Global heap allocator
static HEAP: BumpAllocator = BumpAllocator::new();

/// Initialize the global heap
pub fn init_heap(heap_start: u64) {
    HEAP.init(heap_start);
}

/// Allocate from the global heap
pub fn heap_alloc(size: usize, align: usize) -> Result<*mut u8, SysError> {
    HEAP.alloc(size, align)
}

/// Get heap statistics (allocated, total_mapped)
pub fn heap_stats() -> (usize, usize) {
    HEAP.stats()
}

// =============================================================================
// Global Allocator for alloc crate
// =============================================================================

use core::alloc::{GlobalAlloc, Layout};

/// Global allocator wrapper for use with the `alloc` crate
pub struct ExoAllocator;

unsafe impl GlobalAlloc for ExoAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match HEAP.alloc(layout.size(), layout.align()) {
            Ok(ptr) => ptr,
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't support deallocation
        // Memory is reclaimed when the process exits
    }
}

/// Export the global allocator
/// Users should add `#[global_allocator] static ALLOCATOR: ExoAllocator = ExoAllocator;`
/// in their crate root, or use the `exo_allocator!()` macro.
#[macro_export]
macro_rules! exo_allocator {
    () => {
        #[global_allocator]
        static ALLOCATOR: $crate::mem::ExoAllocator = $crate::mem::ExoAllocator;
    };
}

// =============================================================================
// Utilities
// =============================================================================

/// Align a value up to the given alignment
#[inline]
pub const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

/// Align a value down to the given alignment
#[inline]
pub const fn align_down(val: usize, align: usize) -> usize {
    val & !(align - 1)
}

/// Check if a value is aligned
#[inline]
pub const fn is_aligned(val: usize, align: usize) -> bool {
    val & (align - 1) == 0
}
