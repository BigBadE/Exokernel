#![no_std]
#![no_main]

use crate::disk::disk::DiskReader;
use crate::disk::fat::FileSystem;
use crate::disk::{AlignedArrayBuffer, Read};
use crate::gdt::GDT;
use crate::memory::detect_memory;
use crate::util::print::{print_char, println};
use crate::vesa::get_vbe_info;
use common::boot_info::BootInfo;
use core::ptr;
use crate::disk::file::{load_file, read_file, test_file};

mod disk;
mod gdt;
mod memory;
mod util;
mod vesa;

const PARTITION_TABLE: *const u8 = 0x7DBE as *const u8;
const THIRD_START: u32 = 0x10_000;

const FILE_BUFFER_SIZE: usize = 0x4000;

static mut DISK_BUFFER: AlignedArrayBuffer<FILE_BUFFER_SIZE> = AlignedArrayBuffer {
    buffer: [0; FILE_BUFFER_SIZE],
};

// This has to be a separate function or the partition table is loaded incorrectly
// I have zero clue why
#[no_mangle]
#[link_section = ".second_stage"]
pub extern "C" fn second_stage(disk_number: u16, partition_table: *const u8) -> ! {
    println("Entered stage 2");
    unsafe {
        // Enter unreal mode before doing anything else.
        GDT::enter_unreal();
        let partitions: [PartitionTableEntry; 4] =
            ptr::read(partition_table as *const [PartitionTableEntry; 4]);

        //Load the file system
        let mut file_system = FileSystem::parse(DiskReader {
            disk_number,
            base: partitions[1].lba as u64 * 512,
            offset: 0,
        });

        let mut temp = [0; 512];
        let mut disk = file_system.disk.clone();
        //load_file(".check", temp.as_mut_ptr(), &mut file_system, &mut disk, &mut DISK_BUFFER);
        /*read_file(&file_system
            .find_file_in_root_dir(".check", &mut DISK_BUFFER)
            .unwrap(), ".check", file_system, &mut DISK_BUFFER, temp.as_mut_ptr());*/
        test_file(&file_system
            .find_file_in_root_dir(".check", &mut DISK_BUFFER)
            .unwrap(), file_system);
        if !temp.starts_with(b"PASS") {
            panic!("File failed to load!");
        }
        println("File system loaded!");
        loop {

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
        Err(_code) => panic!("Failed to map memory."),
    };

    println("Loaded memory!");
    let video = match get_vbe_info(buffer).get_best_mode() {
        Some(value) => value,
        None => panic!("Failed to find a video mode."),
    };

    /*match enable(&video) {
        Ok(_) => {},
        Err(_) => panic!("Failed to enable video mode.")
    }*/

    BootInfo { video, memory }
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
    pub sectors: u32,
}

#[no_mangle]
pub extern "C" fn fail() -> ! {
    panic!("Failed to load from disc!");
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    let output = info.message();
    println(output.as_str().unwrap());
    loop {}
}
