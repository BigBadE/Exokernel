use core::arch::asm;
use core::slice;
use common::boot_info::{MemoryInfo, MemoryRegion};
use crate::FILE_BUFFER_SIZE;
use crate::util::print::{print, printhex, println, printnumb};

pub static MEMORY: [MemoryRegion; 100] = [MemoryRegion::default(); 100];

pub fn detect_memory() -> Result<MemoryInfo, u32> {
    const SMAP: u32 = 0x534D4150;

    let mut i = 0;

    let mut offset = 0;
    loop {
        let ret: u32;
        let buf_written_len: usize;
        unsafe {
            asm!(
            "push ebx",
            "mov ebx, edx",
            "mov edx, 0x534D4150",
            "int 0x15",
            "mov edx, ebx",
            "pop ebx",
            inout("eax") 0xe820 => ret,
            out("edx") offset,
            inout("ecx") 24 => buf_written_len,
            in("di") &MEMORY[i]
            )
        };
        if ret != SMAP {
            print("Failed: ");
            printhex(ret);
            println("");
            return Err(ret);
        }

        if buf_written_len != 0 {
            i += 1;
        }

        if offset == 0 {
            break;
        }
    }

    return Ok(MemoryInfo {
        memory: &MEMORY[..i-1]
    })
}