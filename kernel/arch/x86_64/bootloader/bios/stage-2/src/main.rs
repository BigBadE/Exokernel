#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::{ptr, slice};
use crate::dap::DiskRead;
use crate::gdt::GDT;
use crate::util::fat::FileSystem;
use crate::util::print::{print, printhex, printhexbuf, println, printnumb};

mod partitions;
mod gdt;
mod dap;
mod util;

const PARTITION_TABLE: *const u8 = 0x7DBE as *const u8;
const THIRD_START: u32 = 0x10_000;
const FOURTH_START: u32 = 0x20_000;

const FILE_BUFFER_SIZE: usize = 0x4000;

#[no_mangle]
#[link_section = ".second_stage"]
pub extern "C" fn second_stage(disk_number: u16) -> !{
    println("Entered stage 2");

    unsafe {
        //Enter unreal mode so the kernel is limited to 4 GiB instead of 64 KB
        GDT::enter_unreal();

        let partitions: [PartitionTableEntry; 4] = ptr::read(PARTITION_TABLE as *const [PartitionTableEntry; 4]);
        //Init file system
        let mut fs = FileSystem::parse(
            DiskRead::new(partitions[1].lba as u64 * 512, disk_number));
        let mut buf: [u8; FILE_BUFFER_SIZE] = [0; FILE_BUFFER_SIZE];

        //Load third stage and check that it's not longer than its given length
        assert!(load_file(buf, &mut fs, "sys", THIRD_START).unwrap() + THIRD_START < FOURTH_START);
        //Enter 32 bit mode and jump
        GDT::enter_protected_jump(THIRD_START, 12);
    }
}

fn load_file(mut buf: [u8; FILE_BUFFER_SIZE], fs: &mut FileSystem, file: &str, start: u32) -> Result<u32, ()> {
    let file = fs.find_file_in_root_dir(file, &mut buf).unwrap();

    let mut offset = start;
    let mut disk = fs.disk.clone();
    let mut iterator = fs.file_clusters(&file);
    while let Some(cluster) = iterator.next() {
        println("1");
        let cluster = cluster?;
        println("2");
        disk.seek(cluster.start_offset);
        println("3");
        let mut cluster_len = cluster.len_bytes;
        while cluster_len > 0 {
            println("4");
            //We can only read from the disk into >14kB so we have to copy it in
            let read = disk.read(&mut buf).unwrap().max(cluster_len as usize);
            println("5");
            unsafe {
                ptr::copy_nonoverlapping(ptr::addr_of!(buf) as *const u8, offset as *mut u8, read);
            }
            cluster_len -= read as u32;
            offset += read as u32;
            println("6");
        }
    }

    return Ok(offset-start);
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
    print(info.message().unwrap().as_str().unwrap());
    loop {}
}
