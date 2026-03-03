use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

/// A frame allocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a new frame allocator from the bootloader memory map.
    ///
    /// # Safety
    /// The caller must guarantee that the passed memory map is valid.
    /// All frames marked as USABLE must be unused.
    pub unsafe fn new(memory_regions: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_regions,
            next: 0,
        }
    }

    /// Returns an iterator over usable frames.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        // Get usable regions from memory map
        let regions = self.memory_regions.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);

        // Map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.start..r.end);

        // Transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        // Create PhysFrame types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Statistics about memory usage
pub struct MemoryStats {
    pub total_frames: usize,
    pub used_frames: usize,
    pub free_frames: usize,
}

impl BootInfoFrameAllocator {
    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        let total_frames = self.usable_frames().count();
        MemoryStats {
            total_frames,
            used_frames: self.next,
            free_frames: total_frames.saturating_sub(self.next),
        }
    }
}
