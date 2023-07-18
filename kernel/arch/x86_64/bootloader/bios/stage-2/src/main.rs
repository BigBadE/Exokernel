#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::ptr;
use common::boot_info::{BootInfo, MemoryInfo, VideoInfo};
use crate::dap::{DAP, DiskRead};
use crate::gdt::GDT;
use crate::memory::detect_memory;
use crate::util::print;
use crate::util::print::{print, printhex, println, printnumb};
use crate::vesa::{enable, get_vbe_info};

mod util;
mod dap;
mod gdt;
mod memory;
mod partitions;
mod vesa;

const PARTITION_TABLE: *const u8 = 0x7DBE as *const u8;
const THIRD_START: u32 = 0x10_000;

const FILE_BUFFER_SIZE: usize = 0x4000;

#[no_mangle]
#[link_section = ".second_stage"]
pub extern "C" fn second_stage(disk_number: u16) -> !{
    println("Entered stage 2");
    unsafe {
        //Enter unreal mode so the kernel is limited to 4 GiB instead of 64 KB
        GDT::enter_unreal();

        let partitions: [PartitionTableEntry; 4] = ptr::read(PARTITION_TABLE as *const [PartitionTableEntry; 4]);

        //Load the file system
        let dap = DAP::new(partitions[2].sectors as u16, THIRD_START, 1);

        dap.load(disk_number);

        let mut buf: [u8; FILE_BUFFER_SIZE] = [0; FILE_BUFFER_SIZE];
        let mut boot_info = get_boot_info(&mut buf);

        //Enter 32 bit mode and jump
        GDT::enter_protected_jump(THIRD_START, &mut boot_info);
    }

    loop {

    }
}

fn get_boot_info(buffer: &mut [u8; FILE_BUFFER_SIZE]) -> BootInfo {
    println("Loading memory!");
    let memory = match detect_memory() {
        Ok(memory) => memory,
        Err(code) => panic!("Failed to map memory.")
    };

    println("Loaded memory!");
    let video = match get_vbe_info(buffer).get_best_mode() {
        Some(value) => value,
        None => panic!("Failed to find a video mode.")
    };

    /*match enable(&video) {
        Ok(_) => {},
        Err(_) => panic!("Failed to enable video mode.")
    }*/

    return BootInfo {
        video,
        memory
    }
}

#[repr(C)]
#[derive(Default, Clone)]
pub struct PartitionTableEntry {
    pub boot_flag: u8,
    pub starting_head: u8,
    pub starting_sector_cylinder: u16,
    pub system_id: u8,
    pub ending_head: u8,
    pub ending_sector_cylinder: u16,
    pub lba: u32,
    pub sectors: u32
}

#[no_mangle]
pub extern "C" fn fail() -> ! {
    panic!("Failed!");
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    use core::fmt::Write;
    let output = info.message().unwrap();
    println(output.as_str().unwrap());
    loop {}
}
