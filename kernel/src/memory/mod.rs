//! Memory management for the Exokernel
//!
//! This module provides:
//! - Physical frame allocation
//! - Page table management
//! - Kernel heap

pub mod frame_allocator;

use bootloader_api::info::MemoryRegions;
use spin::Mutex;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

pub use frame_allocator::BootInfoFrameAllocator;

/// Physical memory offset
static PHYS_MEM_OFFSET: Mutex<Option<VirtAddr>> = Mutex::new(None);

/// Global mapper and allocator
static MEMORY: Mutex<Option<MemoryManager>> = Mutex::new(None);

struct MemoryManager {
    mapper: OffsetPageTable<'static>,
    allocator: BootInfoFrameAllocator,
}

/// Initialize memory management
pub unsafe fn init(
    physical_memory_offset: VirtAddr,
    memory_regions: &'static MemoryRegions,
) -> (OffsetPageTable<'static>, BootInfoFrameAllocator) {
    unsafe {
        *PHYS_MEM_OFFSET.lock() = Some(physical_memory_offset);

        let level_4_table = active_level_4_table(physical_memory_offset);
        let mapper = OffsetPageTable::new(level_4_table, physical_memory_offset);
        let allocator = BootInfoFrameAllocator::new(memory_regions);

        (mapper, allocator)
    }
}

/// Store the mapper and allocator for later use
pub fn store(mapper: OffsetPageTable<'static>, allocator: BootInfoFrameAllocator) {
    *MEMORY.lock() = Some(MemoryManager { mapper, allocator });
}

/// Access mapper and allocator
pub fn with_mapper_and_allocator<F, R>(f: F) -> R
where
    F: FnOnce(&mut OffsetPageTable<'static>, &mut BootInfoFrameAllocator) -> R,
{
    let mut guard = MEMORY.lock();
    let mm = guard.as_mut().expect("Memory not initialized");
    f(&mut mm.mapper, &mut mm.allocator)
}

/// Get active level 4 page table
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

/// Translate physical to virtual address
pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    let offset = PHYS_MEM_OFFSET.lock().expect("Memory not initialized");
    offset + phys.as_u64()
}

/// Allocate a physical frame
pub fn alloc_frame() -> Option<PhysFrame<Size4KiB>> {
    with_mapper_and_allocator(|_, allocator| allocator.allocate_frame())
}

/// Map a page to a frame
pub fn map_page(
    page: Page<Size4KiB>,
    frame: PhysFrame<Size4KiB>,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    with_mapper_and_allocator(|mapper, allocator| {
        unsafe {
            mapper
                .map_to(page, frame, flags, allocator)
                .map_err(|_| "Failed to map page")?
                .flush();
        }
        Ok(())
    })
}
