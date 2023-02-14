#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::ptr;
use common::boot_info::{BootInfo, VideoInfo};
use crate::dap::DiskRead;
use crate::gdt::GDT;
use crate::util::fat::FileSystem;
use crate::util::print;
use crate::util::print::{print, printhex, println, printnumb};
use crate::vesa::{enable, get_vbe_info};

mod partitions;
mod gdt;
mod dap;
mod util;
mod vesa;

const PARTITION_TABLE: *const u8 = 0x7DBE as *const u8;
const THIRD_START: u32 = 0x10_000;
const FOURTH_START: u32 = 0x30_000;

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
        assert!(load_file(&mut buf, &mut fs, "sys", THIRD_START).unwrap() + THIRD_START < FOURTH_START);
        let mut boot_info = get_boot_info(&mut buf);
        //Enter 32 bit mode and jump
        //GDT::enter_protected_jump(THIRD_START, &mut boot_info);
    }

    loop {

    }
}

fn get_boot_info(buffer: &mut [u8; FILE_BUFFER_SIZE]) -> BootInfo {
    let video = match get_vbe_info(buffer).get_best_mode() {
        Some(value) => value,
        None => panic!("Failed to find a video mode.")
    };

    match enable(&video) {
        Ok(_) => {},
        Err(_) => panic!("Failed to enable video mode.")
    }

    return BootInfo {
        video
    }
}

fn load_file(buf: &mut [u8; FILE_BUFFER_SIZE], fs: &mut FileSystem, file: &str, start: u32) -> Result<u32, ()> {
    let file = fs.find_file_in_root_dir(file, buf).unwrap();
    let mut offset = start;
    let mut disk = fs.disk.clone();
    let mut iterator = fs.file_clusters(&file);
    for cluster in iterator {
        let cluster = cluster?;
        disk.seek(cluster.start_offset);
        let mut cluster_len = 0;
        while cluster_len < cluster.len_bytes as u64 {
            let range_end = u64::min(
                cluster.start_offset + cluster_len + FILE_BUFFER_SIZE as u64,
                cluster.start_offset + cluster.len_bytes as u64
            );
            let reading = FILE_BUFFER_SIZE.min((range_end-cluster_len) as usize);

            //We can only read from the disk into >14kB so we have to copy it in
            disk.read_len(reading, buf);

            unsafe {
                ptr::copy_nonoverlapping(ptr::addr_of!(buf) as *const u8, (offset+reading as u32) as *mut u8, reading);
            }
            cluster_len += reading as u64;
            offset += reading as u32;
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
    use core::fmt::Write;
    let output = info.message().unwrap();
    println(output.as_str().unwrap());
    loop {}
}
