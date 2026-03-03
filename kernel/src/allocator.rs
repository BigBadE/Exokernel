use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

/// Heap start address (in the higher half of the address space)
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// Heap size: 1 MiB initially
pub const HEAP_SIZE: usize = 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the kernel heap.
///
/// # Safety
/// This function must only be called once.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), HeapError> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Map all heap pages
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(HeapError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .map_err(|_| HeapError::MappingFailed)?
                .flush();
        }
    }

    // Initialize the allocator
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}

#[derive(Debug)]
pub enum HeapError {
    FrameAllocationFailed,
    MappingFailed,
}

/// Get heap usage statistics
pub fn heap_stats() -> (usize, usize) {
    let allocator = ALLOCATOR.lock();
    let used = allocator.used();
    let free = allocator.free();
    (used, free)
}
