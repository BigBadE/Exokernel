#![no_std]
#![no_main]

use crate::{
    protected_mode::{
        copy_to_protected_mode, enter_unreal_mode,
    },
};
use core::{fmt::Write as _, ptr, slice};
use crate::disk::{AlignedArrayBuffer, Read, Seek, SeekFrom};
use crate::disk::disk::DiskReader;
use crate::disk::fat::FileSystem;
use crate::gdt::GDT;
use crate::print::println;

mod gdt;
mod dap;
mod disk;
mod protected_mode;
mod print;

/// We use this partition type to store the second bootloader stage;
const BOOTLOADER_SECOND_STAGE_PARTITION_TYPE: u8 = 0x20;

// 1MiB (typically 14MiB accessible here)
const STAGE_3_DST: *mut u8 = 0x0010_0000 as *mut u8;
// must match the start address in bios/stage-4/stage-4-link.ld
const STAGE_4_DST: *mut u8 = 0x0013_0000 as *mut u8;
// 16MiB
const KERNEL_DST: *mut u8 = 0x0100_0000 as *mut u8;

static mut DISK_BUFFER: AlignedArrayBuffer<0x4000> = AlignedArrayBuffer {
    buffer: [0; 0x4000],
};

#[no_mangle]
#[link_section = ".second_stage"]
pub extern "C" fn second_stage(disk_number: u16, partition_table_start: *const u8) -> ! {
    start(disk_number, partition_table_start)
}

fn start(disk_number: u16, partition_table_start: *const u8) -> ! {
    // Enter unreal mode before doing anything else.
    unsafe {
        GDT::enter_unreal();
    }

    let partitions = unsafe { ptr::read(partition_table_start as * const [PartitionTableEntry; 4]) };

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

    //let stage_3_len = load_file("boot-stage-3", STAGE_3_DST, &mut fs, &mut disk, disk_buffer);
    //writeln!(screen::Writer, "stage 3 loaded at {STAGE_3_DST:#p}").unwrap();
    loop {}
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    let output = info.message();
    println(output.as_str().unwrap());
    loop {}
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PartitionTableEntry {
    pub bootable: u32,
    pub partition_type: u32,
    pub lba: u32,
    pub sectors: u32,
}

fn try_load_file(
    file_name: &str,
    dst: *mut u8,
    fs: &mut FileSystem<DiskReader>,
    disk: &mut DiskReader,
    disk_buffer: &mut AlignedArrayBuffer<16384>,
) -> Option<u64> {
    let disk_buffer_size = disk_buffer.buffer.len();
    let file = fs.find_file_in_root_dir(file_name, disk_buffer)?;

    let file_size = file.file_size().into();

    let mut total_offset = 0;
    for cluster in fs.file_clusters(&file) {
        let cluster = cluster.unwrap();
        let cluster_start = cluster.start_offset;
        let cluster_end = cluster_start + u64::from(cluster.len_bytes);

        let mut offset = 0;
        loop {
            let range_start = cluster_start + offset;
            if range_start >= cluster_end {
                break;
            }
            let range_end = u64::min(
                range_start + u64::try_from(disk_buffer_size).unwrap(),
                cluster_end,
            );
            let len = range_end - range_start;

            disk.seek(SeekFrom::Start(range_start));
            disk.read_exact_into(disk_buffer_size, disk_buffer);

            let slice = &disk_buffer.buffer[..usize::try_from(len).unwrap()];
            unsafe { copy_to_protected_mode(dst.wrapping_add(total_offset), slice) };
            let written =
                unsafe { protected_mode::read_from_protected_mode(dst.wrapping_add(total_offset)) };
            assert_eq!(slice[0], written);

            offset += len;
            total_offset += usize::try_from(len).unwrap();
        }
    }
    Some(file_size)
}

fn load_file(
    file_name: &str,
    dst: *mut u8,
    fs: &mut FileSystem<DiskReader>,
    disk: &mut DiskReader,
    disk_buffer: &mut AlignedArrayBuffer<16384>,
) -> u64 {
    try_load_file(file_name, dst, fs, disk, disk_buffer).expect("file not found")
}

/// Taken from https://github.com/rust-lang/rust/blob/e100ec5bc7cd768ec17d75448b29c9ab4a39272b/library/core/src/slice/mod.rs#L1673-L1677
///
/// TODO replace with `split_array` feature in stdlib as soon as it's stabilized,
/// see https://github.com/rust-lang/rust/issues/90091
fn split_array_ref<const N: usize, T>(slice: &[T]) -> (&[T; N], &[T]) {
    if N > slice.len() {
        fail(b'S');
    }
    let (a, b) = slice.split_at(N);
    // SAFETY: a points to [T; N]? Yes it's [T] of length N (checked by split_at)
    unsafe { (&*(a.as_ptr() as *const [T; N]), b) }
}

#[cold]
#[inline(never)]
#[no_mangle]
pub extern "C" fn fail(code: u8) -> ! {
    panic!("fail: {}", code as char);
}
