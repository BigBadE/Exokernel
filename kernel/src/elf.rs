//! Minimal ELF loader for init bootstrap ONLY
//!
//! This loader exists solely to load the first user-space process (init/capability manager).
//! All subsequent ELF loading is done by user-space libOS using:
//! - `mem_alloc_phys` / `mem_map` for memory
//! - `process_create` for new address spaces
//!
//! The kernel intentionally keeps this minimal - just enough to bootstrap.
//! User-space processes load their own children via syscalls.

use alloc::vec::Vec;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

use crate::memory;

/// ELF magic number
const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// ELF class (64-bit)
const ELFCLASS64: u8 = 2;

/// ELF data encoding (little endian)
const ELFDATA2LSB: u8 = 1;

/// ELF type: executable
const ET_EXEC: u16 = 2;

/// ELF type: shared object (also used for PIE executables)
const ET_DYN: u16 = 3;

/// ELF machine type: x86_64
const EM_X86_64: u16 = 62;

/// Program header type: loadable segment
const PT_LOAD: u32 = 1;

/// Program header type: dynamic linking info
const PT_DYNAMIC: u32 = 2;

/// Program header flags
const PF_X: u32 = 1; // Execute
const PF_W: u32 = 2; // Write
const PF_R: u32 = 4; // Read

/// Dynamic entry types
const DT_NULL: u64 = 0;
const DT_RELA: u64 = 7;      // Address of relocation table
const DT_RELASZ: u64 = 8;    // Size of relocation table
const DT_RELAENT: u64 = 9;   // Size of relocation entry

/// Relocation types
const R_X86_64_RELATIVE: u32 = 8;

/// ELF64 Dynamic entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Dyn {
    pub d_tag: u64,
    pub d_val: u64,
}

/// ELF64 Relocation entry with addend
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Rela {
    pub r_offset: u64,
    pub r_info: u64,
    pub r_addend: i64,
}

/// ELF64 Header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

/// ELF64 Program Header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

/// Loaded ELF information
pub struct LoadedElf {
    pub entry_point: u64,
    pub stack_top: u64,
}

/// Error types for ELF loading
#[derive(Debug)]
pub enum ElfError {
    InvalidMagic,
    InvalidClass,
    InvalidEndianness,
    InvalidType,
    InvalidMachine,
    InvalidProgramHeader,
    MappingFailed,
    FrameAllocationFailed,
}

/// Parse and validate an ELF header
pub fn parse_header(data: &[u8]) -> Result<Elf64Header, ElfError> {
    if data.len() < core::mem::size_of::<Elf64Header>() {
        return Err(ElfError::InvalidMagic);
    }

    // Use read_unaligned to handle potentially unaligned data
    let header = unsafe {
        core::ptr::read_unaligned(data.as_ptr() as *const Elf64Header)
    };

    // Validate magic number
    if header.e_ident[0..4] != ELF_MAGIC {
        return Err(ElfError::InvalidMagic);
    }

    // Validate class (64-bit)
    if header.e_ident[4] != ELFCLASS64 {
        return Err(ElfError::InvalidClass);
    }

    // Validate endianness (little endian)
    if header.e_ident[5] != ELFDATA2LSB {
        return Err(ElfError::InvalidEndianness);
    }

    // Validate type (executable or PIE)
    if header.e_type != ET_EXEC && header.e_type != ET_DYN {
        return Err(ElfError::InvalidType);
    }

    // Validate machine (x86_64)
    if header.e_machine != EM_X86_64 {
        return Err(ElfError::InvalidMachine);
    }

    Ok(header)
}

/// Get program headers from ELF data
pub fn get_program_headers(data: &[u8], header: &Elf64Header) -> Result<Vec<Elf64ProgramHeader>, ElfError> {
    let phoff = header.e_phoff as usize;
    let phnum = header.e_phnum as usize;
    let phentsize = header.e_phentsize as usize;

    if phoff + phnum * phentsize > data.len() {
        return Err(ElfError::InvalidProgramHeader);
    }

    // Read program headers with proper alignment handling
    let mut phdrs = Vec::with_capacity(phnum);
    for i in 0..phnum {
        let offset = phoff + i * phentsize;
        let phdr = unsafe {
            core::ptr::read_unaligned(data.as_ptr().add(offset) as *const Elf64ProgramHeader)
        };
        phdrs.push(phdr);
    }

    Ok(phdrs)
}

/// Load an ELF executable into memory
pub fn load_elf(
    data: &[u8],
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<LoadedElf, ElfError> {
    let header = parse_header(data)?;
    let phdrs = get_program_headers(data, &header)?;

    // Load each program segment
    for phdr in phdrs.iter() {
        let p_type = phdr.p_type;
        if p_type != PT_LOAD {
            continue;
        }
        load_segment(data, phdr, mapper, frame_allocator)?;
    }

    // Process relocations for PIE executables (ET_DYN)
    if header.e_type == ET_DYN {
        process_relocations(data, &phdrs, mapper)?;
    }

    // Set up user stack
    let stack_top = setup_user_stack(mapper, frame_allocator)?;

    Ok(LoadedElf {
        entry_point: header.e_entry,
        stack_top,
    })
}

/// Process relocations for PIE executables
fn process_relocations(
    data: &[u8],
    phdrs: &[Elf64ProgramHeader],
    mapper: &OffsetPageTable,
) -> Result<(), ElfError> {
    // Find PT_DYNAMIC segment
    let dynamic_phdr = phdrs.iter().find(|p| p.p_type == PT_DYNAMIC);
    let dynamic_phdr = match dynamic_phdr {
        Some(p) => p,
        None => return Ok(()), // No dynamic segment, nothing to do
    };

    let dyn_offset = dynamic_phdr.p_offset as usize;
    let dyn_size = dynamic_phdr.p_filesz as usize;

    // Parse dynamic entries to find RELA table
    let mut rela_addr: Option<u64> = None;
    let mut rela_size: Option<u64> = None;
    let mut rela_ent: Option<u64> = None;

    let entry_size = core::mem::size_of::<Elf64Dyn>();
    let num_entries = dyn_size / entry_size;

    for i in 0..num_entries {
        let offset = dyn_offset + i * entry_size;
        if offset + entry_size > data.len() {
            break;
        }

        let dyn_entry = unsafe {
            core::ptr::read_unaligned(data.as_ptr().add(offset) as *const Elf64Dyn)
        };

        match dyn_entry.d_tag {
            DT_NULL => break,
            DT_RELA => rela_addr = Some(dyn_entry.d_val),
            DT_RELASZ => rela_size = Some(dyn_entry.d_val),
            DT_RELAENT => rela_ent = Some(dyn_entry.d_val),
            _ => {}
        }
    }

    // Process relocations if we found the RELA table
    if let (Some(rela_vaddr), Some(size), Some(ent_size)) = (rela_addr, rela_size, rela_ent) {
        // Convert virtual address to file offset by finding which segment contains it
        let rela_file_offset = vaddr_to_file_offset(rela_vaddr, phdrs)
            .ok_or(ElfError::InvalidProgramHeader)?;

        let num_relas = size / ent_size;

        for i in 0..num_relas {
            let offset = rela_file_offset as usize + (i * ent_size) as usize;
            if offset + core::mem::size_of::<Elf64Rela>() > data.len() {
                break;
            }

            let rela = unsafe {
                core::ptr::read_unaligned(data.as_ptr().add(offset) as *const Elf64Rela)
            };
            let r_type = (rela.r_info & 0xFFFFFFFF) as u32;

            if r_type == R_X86_64_RELATIVE {
                // R_X86_64_RELATIVE: *offset = base + addend
                // For our case, base is 0 (we load at the addresses specified in the ELF)
                let target_vaddr = rela.r_offset;
                let value = rela.r_addend as u64; // base (0) + addend

                // Write the relocated value to memory
                let page = Page::<Size4KiB>::containing_address(VirtAddr::new(target_vaddr));
                if let Ok(frame) = mapper.translate_page(page) {
                    let page_offset = (target_vaddr % 4096) as usize;
                    let phys_addr = frame.start_address() + page_offset as u64;
                    let kernel_vaddr = memory::phys_to_virt(phys_addr);
                    let ptr = kernel_vaddr.as_mut_ptr::<u64>();
                    unsafe {
                        *ptr = value;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Convert a virtual address to file offset using program headers
fn vaddr_to_file_offset(vaddr: u64, phdrs: &[Elf64ProgramHeader]) -> Option<u64> {
    for phdr in phdrs {
        let p_type = phdr.p_type;
        if p_type != PT_LOAD {
            continue;
        }

        let seg_vaddr = phdr.p_vaddr;
        let seg_size = phdr.p_filesz;
        let seg_offset = phdr.p_offset;

        if vaddr >= seg_vaddr && vaddr < seg_vaddr + seg_size {
            return Some(seg_offset + (vaddr - seg_vaddr));
        }
    }
    None
}

/// Load a single program segment
fn load_segment(
    data: &[u8],
    phdr: &Elf64ProgramHeader,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), ElfError> {
    let vaddr_start = phdr.p_vaddr;
    let vaddr_end = vaddr_start + phdr.p_memsz;
    let file_offset = phdr.p_offset as usize;
    let file_size = phdr.p_filesz as usize;

    // Calculate page range
    let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(vaddr_start));
    let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(vaddr_end.saturating_sub(1)));

    // Build page flags
    let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
    if phdr.p_flags & PF_W != 0 {
        flags |= PageTableFlags::WRITABLE;
    }
    // Note: x86_64 doesn't have a "no execute" bit that needs to be cleared for executable pages
    // The NX bit is opt-in, so we don't need to do anything special for executable segments

    // Map each page in the segment
    for page in Page::range_inclusive(start_page, end_page) {
        let page_vaddr = page.start_address().as_u64();

        // Get or create the physical frame for this page
        let frame = if let Ok(existing_frame) = mapper.translate_page(page) {
            // Page already mapped (can happen with overlapping segments)
            existing_frame
        } else {
            // Allocate and map a new frame
            let new_frame = frame_allocator
                .allocate_frame()
                .ok_or(ElfError::FrameAllocationFailed)?;

            unsafe {
                mapper
                    .map_to(page, new_frame, flags, frame_allocator)
                    .map_err(|_| ElfError::MappingFailed)?
                    .flush();
            }

            // Zero the page
            let page_kernel_vaddr = memory::phys_to_virt(new_frame.start_address());
            let page_ptr = page_kernel_vaddr.as_mut_ptr::<u8>();
            unsafe {
                core::ptr::write_bytes(page_ptr, 0, 4096);
            }

            new_frame
        };

        // Copy data from ELF file if this page contains file data
        let page_start = page_vaddr;
        let page_end = page_start + 4096;

        // Calculate overlap with file data
        let file_data_start = vaddr_start;
        let file_data_end = vaddr_start + file_size as u64;

        if page_start < file_data_end && page_end > file_data_start {
            // There's overlap - calculate what to copy
            let copy_start = core::cmp::max(page_start, file_data_start);
            let copy_end = core::cmp::min(page_end, file_data_end);

            let src_offset = file_offset + (copy_start - vaddr_start) as usize;
            let dst_offset = (copy_start - page_start) as usize;
            let copy_len = (copy_end - copy_start) as usize;

            if src_offset + copy_len <= data.len() {
                let page_kernel_vaddr = memory::phys_to_virt(frame.start_address());
                let page_ptr = page_kernel_vaddr.as_mut_ptr::<u8>();
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        data.as_ptr().add(src_offset),
                        page_ptr.add(dst_offset),
                        copy_len,
                    );
                }
            }
        }
    }

    Ok(())
}

/// Set up a user stack
fn setup_user_stack(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<u64, ElfError> {
    // User stack at high address (below kernel space)
    const USER_STACK_TOP: u64 = 0x0000_7fff_ffff_0000;
    const USER_STACK_SIZE: u64 = 4096 * 16; // 64 KiB

    let stack_bottom = USER_STACK_TOP - USER_STACK_SIZE;
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE;

    let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(stack_bottom));
    let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(USER_STACK_TOP - 1));

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(ElfError::FrameAllocationFailed)?;

        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .map_err(|_| ElfError::MappingFailed)?
                .flush();
        }

        // Zero the stack page
        let page_phys = frame.start_address();
        let page_kernel_vaddr = memory::phys_to_virt(page_phys);
        let page_ptr = page_kernel_vaddr.as_mut_ptr::<u8>();

        unsafe {
            core::ptr::write_bytes(page_ptr, 0, 4096);
        }
    }

    Ok(USER_STACK_TOP)
}

/// Create a minimal test ELF that makes exokernel syscalls
///
/// This creates a simple init process that:
/// 1. Prints a message using SYS_DEBUG_PRINT (0xF0)
/// 2. Gets its PID using SYS_PROCESS_GET_PID (0x22)
/// 3. Prints exit message
/// 4. Exits using SYS_PROCESS_EXIT (0x23)
pub fn create_test_elf() -> Vec<u8> {
    let mut elf = Vec::new();

    // Code layout (using exokernel syscalls, not Linux):
    // 0x00-0x06: mov rax, 0xF0 (7 bytes) - SYS_DEBUG_PRINT
    // 0x07-0x0D: lea rdi, [rip+offset1] (7 bytes)
    // 0x0E-0x14: mov rsi, 29 (7 bytes) - message length
    // 0x15-0x16: syscall (2 bytes)
    // 0x17-0x1D: mov rax, 0x22 (7 bytes) - SYS_PROCESS_GET_PID
    // 0x1E-0x1F: syscall (2 bytes)
    // 0x20-0x26: mov rax, 0xF0 (7 bytes) - SYS_DEBUG_PRINT
    // 0x27-0x2D: lea rdi, [rip+offset2] (7 bytes)
    // 0x2E-0x34: mov rsi, 21 (7 bytes) - message length
    // 0x35-0x36: syscall (2 bytes)
    // 0x37-0x3D: mov rax, 0x23 (7 bytes) - SYS_PROCESS_EXIT
    // 0x3E-0x40: xor rdi, rdi (3 bytes) - exit code 0
    // 0x41-0x42: syscall (2 bytes)
    // 0x43-0x44: jmp $ (2 bytes) - infinite loop (safety)
    // 0x45: Message 1 "Init: Capability manager started\n" (34 bytes? let's use 29)
    // Message 2 "Init: Exiting cleanly\n" (22 bytes? let's use 21)
    //
    // For lea at 0x07: RIP after = 0x0E, msg1 at 0x45, offset = 0x45 - 0x0E = 0x37
    // For lea at 0x27: RIP after = 0x2E, msg2 at 0x45+29=0x62, offset = 0x62 - 0x2E = 0x34

    let code: &[u8] = &[
        // === Print "Init: Capability manager started!\n" ===
        // mov rax, 0xF0 (SYS_DEBUG_PRINT)
        0x48, 0xc7, 0xc0, 0xF0, 0x00, 0x00, 0x00,
        // lea rdi, [rip+0x37]
        0x48, 0x8d, 0x3d, 0x37, 0x00, 0x00, 0x00,
        // mov rsi, 34 (message length)
        0x48, 0xc7, 0xc6, 0x22, 0x00, 0x00, 0x00,
        // syscall
        0x0f, 0x05,

        // === Get PID ===
        // mov rax, 0x22 (SYS_PROCESS_GET_PID)
        0x48, 0xc7, 0xc0, 0x22, 0x00, 0x00, 0x00,
        // syscall
        0x0f, 0x05,

        // === Print "Init: Exiting cleanly!\n" ===
        // mov rax, 0xF0 (SYS_DEBUG_PRINT)
        0x48, 0xc7, 0xc0, 0xF0, 0x00, 0x00, 0x00,
        // lea rdi, [rip+0x34]
        0x48, 0x8d, 0x3d, 0x34, 0x00, 0x00, 0x00,
        // mov rsi, 23 (message length)
        0x48, 0xc7, 0xc6, 0x17, 0x00, 0x00, 0x00,
        // syscall
        0x0f, 0x05,

        // === Exit ===
        // mov rax, 0x23 (SYS_PROCESS_EXIT)
        0x48, 0xc7, 0xc0, 0x23, 0x00, 0x00, 0x00,
        // xor rdi, rdi (exit code 0)
        0x48, 0x31, 0xff,
        // syscall
        0x0f, 0x05,

        // Infinite loop (shouldn't reach here)
        0xeb, 0xfe,

        // Message 1: "Init: Capability manager started!\n" (34 bytes)
        b'I', b'n', b'i', b't', b':', b' ', b'C', b'a',
        b'p', b'a', b'b', b'i', b'l', b'i', b't', b'y',
        b' ', b'm', b'a', b'n', b'a', b'g', b'e', b'r',
        b' ', b's', b't', b'a', b'r', b't', b'e', b'd',
        b'!', b'\n',

        // Message 2: "Init: Exiting cleanly!\n" (23 bytes)
        b'I', b'n', b'i', b't', b':', b' ', b'E', b'x',
        b'i', b't', b'i', b'n', b'g', b' ', b'c', b'l',
        b'e', b'a', b'n', b'l', b'y', b'!', b'\n',
    ];

    // ELF Header
    let entry_point: u64 = 0x400000; // Where our code will be loaded
    let phoff: u64 = 64; // Program header offset (right after ELF header)
    let phnum: u16 = 1;

    // ELF Header (64 bytes)
    elf.extend_from_slice(&ELF_MAGIC);           // e_ident[0..4]: magic
    elf.push(ELFCLASS64);                         // e_ident[4]: class
    elf.push(ELFDATA2LSB);                        // e_ident[5]: data
    elf.push(1);                                  // e_ident[6]: version
    elf.push(0);                                  // e_ident[7]: OS/ABI
    elf.extend_from_slice(&[0u8; 8]);            // e_ident[8..16]: padding
    elf.extend_from_slice(&ET_EXEC.to_le_bytes()); // e_type
    elf.extend_from_slice(&EM_X86_64.to_le_bytes()); // e_machine
    elf.extend_from_slice(&1u32.to_le_bytes());  // e_version
    elf.extend_from_slice(&entry_point.to_le_bytes()); // e_entry
    elf.extend_from_slice(&phoff.to_le_bytes()); // e_phoff
    elf.extend_from_slice(&0u64.to_le_bytes());  // e_shoff
    elf.extend_from_slice(&0u32.to_le_bytes());  // e_flags
    elf.extend_from_slice(&64u16.to_le_bytes()); // e_ehsize
    elf.extend_from_slice(&56u16.to_le_bytes()); // e_phentsize
    elf.extend_from_slice(&phnum.to_le_bytes()); // e_phnum
    elf.extend_from_slice(&64u16.to_le_bytes()); // e_shentsize
    elf.extend_from_slice(&0u16.to_le_bytes());  // e_shnum
    elf.extend_from_slice(&0u16.to_le_bytes());  // e_shstrndx

    // Program Header (56 bytes)
    let code_offset: u64 = 64 + 56; // After ELF header and program header
    let code_size = code.len() as u64;

    elf.extend_from_slice(&PT_LOAD.to_le_bytes());    // p_type
    elf.extend_from_slice(&(PF_R | PF_X).to_le_bytes()); // p_flags
    elf.extend_from_slice(&code_offset.to_le_bytes()); // p_offset
    elf.extend_from_slice(&entry_point.to_le_bytes()); // p_vaddr
    elf.extend_from_slice(&entry_point.to_le_bytes()); // p_paddr
    elf.extend_from_slice(&code_size.to_le_bytes());  // p_filesz
    elf.extend_from_slice(&code_size.to_le_bytes());  // p_memsz
    elf.extend_from_slice(&0x1000u64.to_le_bytes());  // p_align

    // Code segment
    elf.extend_from_slice(code);

    elf
}
