//! Memory syscall implementations

use exo_shared::{
    CapabilityHandle, ResourceDescriptor, ResourceType, Rights, SysError,
};
use x86_64::structures::paging::{PageTableFlags, Page, PhysFrame, Size4KiB, FrameAllocator, Mapper};
use x86_64::{PhysAddr, VirtAddr};

use crate::caps;
use crate::memory::with_mapper_and_allocator;
use crate::process::current_pid;

/// Allocate physical memory frames
/// Args: frame_count, flags
/// Returns: capability handle to physical frames
pub fn sys_mem_alloc_phys(frame_count: u64, _flags: u64) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();

    if frame_count == 0 || frame_count > 1024 {
        return Err(SysError::InvalidArgument);
    }

    // Allocate frames
    let base_frame = with_mapper_and_allocator(|_, allocator| {
        // Get the first frame to use as base
        allocator.allocate_frame().map(|f: PhysFrame<Size4KiB>| f.start_address().as_u64() / 4096)
    }).ok_or(SysError::OutOfMemory)?;

    // Allocate remaining frames (they should be contiguous from our simple allocator)
    for _ in 1..frame_count {
        with_mapper_and_allocator(|_, allocator| {
            allocator.allocate_frame()
        }).ok_or(SysError::OutOfMemory)?;
    }

    // Create capability for the frames
    let resource = ResourceDescriptor::physical_memory(base_frame, frame_count);
    let rights = Rights::READ | Rights::WRITE | Rights::MAP | Rights::GRANT | Rights::DELEGATE;

    let handle = caps::create_root_cap(resource, rights, pid)?;
    Ok(handle.as_raw() as i64)
}

/// Map physical frames to virtual address
/// Args: phys_cap, virt_addr, flags
pub fn sys_mem_map(phys_cap: u64, virt_addr: u64, flags: u64) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();
    let cap_handle = CapabilityHandle::from_raw(phys_cap);

    // Validate capability
    let cap = caps::validate(cap_handle, pid, Rights::MAP)?;

    // Check resource type
    if cap.resource.resource_type != ResourceType::PhysicalMemory {
        return Err(SysError::InvalidArgument);
    }

    // Validate virtual address is in user space
    if virt_addr >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    // Ensure page alignment
    if virt_addr & 0xFFF != 0 {
        return Err(SysError::AlignmentError);
    }

    // Build page table flags
    let mut pt_flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

    if flags & 0x01 != 0 {
        pt_flags |= PageTableFlags::WRITABLE;
    }
    if flags & 0x04 != 0 {
        pt_flags |= PageTableFlags::NO_CACHE;
    }

    // Map each frame
    let base_frame = cap.resource.base;
    let frame_count = cap.resource.size;

    with_mapper_and_allocator(|mapper, allocator| {
        for i in 0..frame_count {
            let phys = PhysAddr::new((base_frame + i) * 4096);
            let virt = VirtAddr::new(virt_addr + i * 4096);

            let frame = PhysFrame::<Size4KiB>::containing_address(phys);
            let page = Page::<Size4KiB>::containing_address(virt);

            unsafe {
                mapper.map_to(page, frame, pt_flags, allocator)
                    .map_err(|_| SysError::OutOfMemory)?
                    .flush();
            }
        }
        Ok::<(), SysError>(())
    })?;

    Ok(0)
}

/// Unmap a virtual address range
/// Args: virt_addr, size
pub fn sys_mem_unmap(virt_addr: u64, size: u64) -> Result<i64, SysError> {
    // Validate address
    if virt_addr >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    if virt_addr & 0xFFF != 0 || size & 0xFFF != 0 {
        return Err(SysError::AlignmentError);
    }

    let page_count = size / 4096;

    with_mapper_and_allocator(|mapper, _| {
        for i in 0..page_count {
            let virt = VirtAddr::new(virt_addr + i * 4096);
            let page = Page::<Size4KiB>::containing_address(virt);

            if let Ok((_, flush)) = Mapper::<Size4KiB>::unmap(mapper, page) {
                flush.flush();
            }
        }
    });

    Ok(0)
}

/// Change protection on a virtual address range
/// Args: virt_addr, size, new_flags
pub fn sys_mem_protect(virt_addr: u64, size: u64, flags: u64) -> Result<i64, SysError> {
    // Validate address
    if virt_addr >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    if virt_addr & 0xFFF != 0 || size & 0xFFF != 0 {
        return Err(SysError::AlignmentError);
    }

    // Build new flags
    let mut pt_flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

    if flags & 0x01 != 0 {
        pt_flags |= PageTableFlags::WRITABLE;
    }
    if flags & 0x04 != 0 {
        pt_flags |= PageTableFlags::NO_EXECUTE;
    }

    let page_count = size / 4096;

    with_mapper_and_allocator(|mapper, allocator| {
        for i in 0..page_count {
            let virt = VirtAddr::new(virt_addr + i * 4096);
            let page = Page::<Size4KiB>::containing_address(virt);

            // Get current mapping
            use x86_64::structures::paging::Translate;
            if let Some(phys) = mapper.translate_addr(virt) {
                // Unmap and remap with new flags
                if let Ok((_, flush)) = Mapper::<Size4KiB>::unmap(mapper, page) {
                    flush.flush();
                }

                let frame = PhysFrame::<Size4KiB>::containing_address(phys);
                unsafe {
                    let _ = Mapper::<Size4KiB>::map_to(mapper, page, frame, pt_flags, allocator);
                }
            }
        }
    });

    Ok(0)
}

/// Query physical address for virtual address
/// Args: virt_addr
/// Returns: physical address
pub fn sys_mem_query(virt_addr: u64) -> Result<i64, SysError> {
    use x86_64::structures::paging::Translate;

    if virt_addr >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    let virt = VirtAddr::new(virt_addr);

    let phys = with_mapper_and_allocator(|mapper, _| {
        mapper.translate_addr(virt)
    });

    match phys {
        Some(addr) => Ok(addr.as_u64() as i64),
        None => Err(SysError::NotFound),
    }
}
