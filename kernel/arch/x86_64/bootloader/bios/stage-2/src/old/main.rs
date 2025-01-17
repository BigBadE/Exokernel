#![no_std]
#![no_main]

use crate::gdt::GDT;
use crate::memory::detect_memory;
use crate::util::print::{print, print_hex_buf, print_numb, println};
use crate::vesa::get_vbe_info;
use common::boot_info::BootInfo;
use core::ptr;
use crate::disk::{AlignedArrayBuffer, Read};
use crate::disk::dap::DAP;
use crate::disk::disk::DiskReader;
use crate::disk::fat::{Bpb, FileSystem};

mod disk;
mod util;
mod gdt;
mod memory;
mod vesa;

const PARTITION_TABLE: *const u8 = 0x7DBE as *const u8;
const THIRD_START: u32 = 0x10_000;

const FILE_BUFFER_SIZE: usize = 0x4000;

static mut DISK_BUFFER: AlignedArrayBuffer<FILE_BUFFER_SIZE> = AlignedArrayBuffer {
    buffer: [0; FILE_BUFFER_SIZE],
};

#[no_mangle]
#[link_section = ".second_stage"]
pub extern "C" fn second_stage(disk_number: u16, partition_table: *const u8) -> !{
    // Enter unreal mode before doing anything else.
    unsafe {
        GDT::enter_unreal();
    }

    let partitions = unsafe { ptr::read(partition_table as * const [PartitionTableEntry; 4]) };

    // load fat partition
    let mut disk = DiskReader {
        disk_number,
        base: partitions[1].lba as u64 * 512,
        offset: 0,
    };

    let mut fs = FileSystem::parse(disk.clone());

    let disk_buffer = unsafe { &mut DISK_BUFFER };

    if fs.find_file_in_root_dir(".check", disk_buffer).is_none() {
        println("Failed!");
    } else {
        println("Passed!");
    }

    println("Entered stage 2");

    loop {

    }
    unsafe {
        //Enter unreal mode so the kernel is limited to 4 GiB instead of 64 KB
        GDT::enter_unreal();
        let partitions: [PartitionTableEntry; 4] = ptr::read(partition_table as *const [PartitionTableEntry; 4]);

        //Load the file system
        let mut file_system = FileSystem::parse(DiskReader {
            disk_number,
            base: partitions[1].lba as u64 * 512,
            offset: 0,
        });

        if file_system.find_file_in_root_dir(".check", &mut DISK_BUFFER).is_none() {
            println("Failed to load file system!");
        } else {
            println("Found it!");
        }

        let _ = file_system.find_file_in_root_dir(".check", &mut DISK_BUFFER).unwrap();
        if !DISK_BUFFER.buffer.starts_with(b"PASS") {
            panic!("File failed to load!");
        }

        let mut buf: [u8; FILE_BUFFER_SIZE] = [0; FILE_BUFFER_SIZE];
        let mut boot_info = get_boot_info(&mut buf);

        //Enter 32 bit mode and jump
        GDT::enter_protected_jump(THIRD_START, &mut boot_info);
    }
}

fn get_boot_info(buffer: &mut [u8; FILE_BUFFER_SIZE]) -> BootInfo {
    println("Loading memory!");
    let memory = match detect_memory() {
        Ok(memory) => memory,
        Err(_code) => panic!("Failed to map memory.")
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

    BootInfo {
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
    let output = info.message();
    println(output.as_str().unwrap());
    loop {}
}
