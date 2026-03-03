use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;
use crate::gdt;
use crate::task::context::jump_to_usermode;
use crate::println;

/// User space code will be loaded at this address
pub const USER_CODE_START: u64 = 0x0000_0040_0000;
/// User space stack starts here (grows downward)
pub const USER_STACK_TOP: u64 = 0x0000_0080_0000;
/// User stack size
pub const USER_STACK_SIZE: u64 = 4096 * 4; // 16 KiB

/// Set up user space memory mappings
pub fn setup_user_memory(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), &'static str> {
    // Map user stack pages
    let stack_start = USER_STACK_TOP - USER_STACK_SIZE;
    let stack_start_page = Page::containing_address(VirtAddr::new(stack_start));
    let stack_end_page = Page::containing_address(VirtAddr::new(USER_STACK_TOP - 1));

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE;

    for page in Page::range_inclusive(stack_start_page, stack_end_page) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or("Failed to allocate frame for user stack")?;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .map_err(|_| "Failed to map user stack page")?
                .flush();
        }
    }

    // Map user code page (we'll put a simple test program here)
    let code_page = Page::containing_address(VirtAddr::new(USER_CODE_START));
    let code_frame = frame_allocator
        .allocate_frame()
        .ok_or("Failed to allocate frame for user code")?;

    let code_flags = PageTableFlags::PRESENT
        | PageTableFlags::USER_ACCESSIBLE; // Code is not writable

    unsafe {
        mapper
            .map_to(code_page, code_frame, code_flags, frame_allocator)
            .map_err(|_| "Failed to map user code page")?
            .flush();
    }

    Ok(())
}

/// Write a simple test program to user space memory
///
/// This is a minimal user-space program that just does an infinite loop.
/// In the future, this would be replaced by loading an actual ELF binary.
pub fn load_test_program(mapper: &mut OffsetPageTable) {
    use x86_64::structures::paging::Translate;

    // Get the physical address of the user code page
    let user_code_virt = VirtAddr::new(USER_CODE_START);

    if let Some(phys) = mapper.translate_addr(user_code_virt) {
        // Convert physical to kernel-accessible virtual address
        let kernel_virt = crate::memory::phys_to_virt(phys);
        let code_ptr = kernel_virt.as_mut_ptr::<u8>();

        // Write a simple infinite loop:
        // loop:
        //   jmp loop
        //
        // In x86_64, this is: EB FE (short jump -2)
        unsafe {
            // Add a few NOPs first for alignment
            *code_ptr.add(0) = 0x90; // NOP
            *code_ptr.add(1) = 0x90; // NOP
            *code_ptr.add(2) = 0x90; // NOP
            *code_ptr.add(3) = 0x90; // NOP
            // Infinite loop
            *code_ptr.add(4) = 0xEB; // JMP rel8
            *code_ptr.add(5) = 0xFE; // -2 (jump to self)
        }

        println!("Test program loaded at {:#x}", USER_CODE_START);
    } else {
        println!("Failed to translate user code address");
    }
}

/// Jump to user mode and execute user code
///
/// # Safety
/// User memory must be properly set up before calling this.
pub unsafe fn enter_usermode() -> ! {
    let selectors = gdt::selectors();

    println!("Jumping to user mode...");
    println!("  Code selector: {:#x}", selectors.user_code_selector.0);
    println!("  Data selector: {:#x}", selectors.user_data_selector.0);
    println!("  Entry point: {:#x}", USER_CODE_START);
    println!("  Stack: {:#x}", USER_STACK_TOP);

    unsafe {
        jump_to_usermode(
            selectors.user_code_selector.0 as u64,
            selectors.user_data_selector.0 as u64,
            USER_CODE_START,
            USER_STACK_TOP,
        )
    }
}
