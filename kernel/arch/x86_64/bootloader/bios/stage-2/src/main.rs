#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::ptr;
use x86_64_bootloader_bios_stage_2::gdt::GDT;
use crate::dap::DiskRead;
use crate::util::fat::FileSystem;
use crate::util::print::{print, println};

mod partitions;
mod gdt;
mod dap;
mod util;

const PARTITION_TABLE: *const u8 = 0x7DBE as *const u8;
const THIRD_START: u32 = 0x10_000;
const FOURTH_START: u32 = 0x20_000;

const FILE_BUFFER_SIZE: usize = 0x4000;

extern "C" {
    pub static second_end: u16;
}

#[no_mangle]
#[link_section = ".second_stage"]
pub extern "C" fn second_stage(disk_number: u16) {
    println("Entered stage 2");

    unsafe {
        let partitions = ptr::read(PARTITION_TABLE as *const [PartitionTableEntry; 4]);

        //Enter unreal mode so the kernel is limited to 4 GiB instead of 64 KB
        GDT::enter_unreal();

        //Init file system
        let mut fs = FileSystem::parse(
            DiskRead::new(second_end as u32, (partitions[2].lba * 512) as u64, disk_number));
        let mut buf: [u8; FILE_BUFFER_SIZE] = [0; FILE_BUFFER_SIZE];

        //Load third stage and check that it's not longer than its given length
        assert!(load_file(buf, &mut fs, "sys", THIRD_START).unwrap() + THIRD_START < FOURTH_START);

        //Enter 32 bit mode and jump
        //GDT::enter_protected_jump(THIRD_START, 12);
    }
}

fn load_file(mut buf: [u8; FILE_BUFFER_SIZE], fs: &mut FileSystem, file: &str, start: u32) -> Result<u32, ()> {
    let file = fs.find_file_in_root_dir(file, &mut buf).unwrap();

    let mut disk = fs.disk.clone();
    let iterator = fs.file_clusters(&file);
    for cluster in iterator {
        disk.seek(cluster?.start_offset);
    }

    return Ok(0);
}

#[repr(C)]
pub struct PartitionTableEntry {
    pub partition_type: u8,
    pub lba: u32,
    pub sectors: u32,
}

#[no_mangle]
pub extern "C" fn fail() -> ! {
    panic!("Failed!");
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    print(info.message().unwrap().as_str().unwrap());
    loop {}
}
