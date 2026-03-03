//! User-space ELF loader
//!
//! This module provides ELF loading for spawning child processes.
//! Unlike traditional operating systems where the kernel loads ELF files,
//! in the exokernel model, user-space is responsible for:
//!
//! 1. Reading the ELF file (from disk driver via IPC, or from memory)
//! 2. Parsing ELF headers
//! 3. Allocating memory for the new process
//! 4. Copying/mapping segments into the new process's address space
//! 5. Starting the process at the entry point

use exo_shared::{CapabilityHandle, SysError};
use crate::mem::{self, PAGE_SIZE, align_up};
use crate::syscall;

// =============================================================================
// ELF Constants
// =============================================================================

/// ELF magic number
pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

/// ELF class (64-bit)
pub const ELFCLASS64: u8 = 2;

/// ELF data encoding (little endian)
pub const ELFDATA2LSB: u8 = 1;

/// ELF type: executable
pub const ET_EXEC: u16 = 2;

/// ELF machine: x86-64
pub const EM_X86_64: u16 = 62;

/// Program header type: loadable segment
pub const PT_LOAD: u32 = 1;

/// Segment flags
pub const PF_X: u32 = 1;  // Execute
pub const PF_W: u32 = 2;  // Write
pub const PF_R: u32 = 4;  // Read

// =============================================================================
// ELF Structures
// =============================================================================

/// ELF64 file header
#[derive(Debug, Clone, Copy)]
#[repr(C)]
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

/// ELF64 program header
#[derive(Debug, Clone, Copy)]
#[repr(C)]
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

// =============================================================================
// ELF Parsing
// =============================================================================

/// Parsed ELF information
pub struct ElfInfo<'a> {
    pub header: &'a Elf64Header,
    pub entry_point: u64,
    pub program_headers: &'a [Elf64ProgramHeader],
}

/// Parse an ELF file from a byte slice
pub fn parse_elf(data: &[u8]) -> Result<ElfInfo<'_>, ElfError> {
    if data.len() < core::mem::size_of::<Elf64Header>() {
        return Err(ElfError::TooSmall);
    }

    // Parse header
    let header = unsafe { &*(data.as_ptr() as *const Elf64Header) };

    // Validate magic
    if header.e_ident[0..4] != ELF_MAGIC {
        return Err(ElfError::BadMagic);
    }

    // Validate class (64-bit)
    if header.e_ident[4] != ELFCLASS64 {
        return Err(ElfError::Not64Bit);
    }

    // Validate endianness (little endian)
    if header.e_ident[5] != ELFDATA2LSB {
        return Err(ElfError::BadEndian);
    }

    // Validate type (executable)
    if header.e_type != ET_EXEC {
        return Err(ElfError::NotExecutable);
    }

    // Validate machine (x86-64)
    if header.e_machine != EM_X86_64 {
        return Err(ElfError::WrongArch);
    }

    // Get program headers
    let ph_offset = header.e_phoff as usize;
    let ph_count = header.e_phnum as usize;
    let ph_size = header.e_phentsize as usize;

    if ph_offset + (ph_count * ph_size) > data.len() {
        return Err(ElfError::BadProgramHeaders);
    }

    let program_headers = unsafe {
        core::slice::from_raw_parts(
            data.as_ptr().add(ph_offset) as *const Elf64ProgramHeader,
            ph_count,
        )
    };

    Ok(ElfInfo {
        header,
        entry_point: header.e_entry,
        program_headers,
    })
}

/// ELF parsing errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfError {
    TooSmall,
    BadMagic,
    Not64Bit,
    BadEndian,
    NotExecutable,
    WrongArch,
    BadProgramHeaders,
    LoadFailed,
}

// =============================================================================
// Process Loading
// =============================================================================

/// Load an ELF into a new process
///
/// This function:
/// 1. Creates a new process
/// 2. Allocates memory for each loadable segment
/// 3. Maps memory into the new process
/// 4. Copies segment data
/// 5. Returns the process capability and entry point
///
/// The caller should then:
/// 1. Set up the stack
/// 2. Call process_start to begin execution
pub fn load_elf(
    elf_data: &[u8],
    process_cap: CapabilityHandle,
) -> Result<LoadedElf, ElfError> {
    let elf = parse_elf(elf_data)?;

    let mut loaded_segments = 0;

    // Load each PT_LOAD segment
    for phdr in elf.program_headers {
        if phdr.p_type != PT_LOAD {
            continue;
        }

        // Calculate memory requirements
        let vaddr = phdr.p_vaddr;
        let memsz = phdr.p_memsz;
        let filesz = phdr.p_filesz;
        let offset = phdr.p_offset;

        // Page-align the virtual address
        let page_offset = vaddr & (PAGE_SIZE as u64 - 1);
        let aligned_vaddr = vaddr - page_offset;
        let aligned_memsz = align_up((memsz + page_offset) as usize, PAGE_SIZE);
        let num_pages = aligned_memsz / PAGE_SIZE;

        // Allocate physical memory for this segment
        let mem_cap = mem::alloc_phys(num_pages as u64, 0)
            .map_err(|_| ElfError::LoadFailed)?;

        // Determine protection flags
        let mut flags = mem::flags::USER;
        if phdr.p_flags & PF_R != 0 {
            flags |= mem::flags::READ;
        }
        if phdr.p_flags & PF_W != 0 {
            flags |= mem::flags::WRITE;
        }
        if phdr.p_flags & PF_X != 0 {
            flags |= mem::flags::EXECUTE;
        }

        // Map into the new process's address space
        // TODO: Need a syscall variant to map into another process
        // For now, we map into our own space, copy, then remap
        mem::map(mem_cap, aligned_vaddr, flags)
            .map_err(|_| ElfError::LoadFailed)?;

        // Copy file data to memory
        if filesz > 0 {
            let src = &elf_data[offset as usize..(offset + filesz) as usize];
            let dst = (vaddr) as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src.as_ptr(), dst, filesz as usize);
            }
        }

        // Zero out the BSS (memsz - filesz)
        if memsz > filesz {
            let bss_start = (vaddr + filesz) as *mut u8;
            let bss_size = (memsz - filesz) as usize;
            unsafe {
                core::ptr::write_bytes(bss_start, 0, bss_size);
            }
        }

        loaded_segments += 1;
    }

    if loaded_segments == 0 {
        return Err(ElfError::BadProgramHeaders);
    }

    Ok(LoadedElf {
        entry_point: elf.entry_point,
        process_cap,
    })
}

/// Result of loading an ELF file
pub struct LoadedElf {
    /// Entry point address
    pub entry_point: u64,
    /// Capability to the process
    pub process_cap: CapabilityHandle,
}

// =============================================================================
// High-Level API
// =============================================================================

/// Default stack size for new processes
pub const DEFAULT_STACK_SIZE: usize = 64 * 1024; // 64 KiB

/// Default stack location
pub const DEFAULT_STACK_TOP: u64 = 0x0000_7fff_ffff_0000;

/// Spawn a new process from an ELF binary
///
/// This is the high-level API that:
/// 1. Creates a process
/// 2. Loads the ELF
/// 3. Sets up the stack
/// 4. Starts the process
pub fn spawn(elf_data: &[u8]) -> Result<CapabilityHandle, ElfError> {
    // Create a new process (entry point and stack will be set later)
    let process_cap = syscall::process_create(0, 0, 0)
        .map_err(|_| ElfError::LoadFailed)?;

    // Load the ELF
    let loaded = load_elf(elf_data, process_cap)?;

    // Allocate stack
    let stack_pages = DEFAULT_STACK_SIZE / PAGE_SIZE;
    let stack_cap = mem::alloc_phys(stack_pages as u64, 0)
        .map_err(|_| ElfError::LoadFailed)?;

    // Map stack (grows down, so we map below DEFAULT_STACK_TOP)
    let stack_bottom = DEFAULT_STACK_TOP - DEFAULT_STACK_SIZE as u64;
    mem::map(stack_cap, stack_bottom, mem::flags::RW_USER)
        .map_err(|_| ElfError::LoadFailed)?;

    // Start the process
    syscall::process_start(process_cap, loaded.entry_point, DEFAULT_STACK_TOP)
        .map_err(|_| ElfError::LoadFailed)?;

    Ok(process_cap)
}
