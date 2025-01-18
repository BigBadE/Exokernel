use crate::disk::disk::DiskReader;
use crate::disk::fat::{File, FileSystem};
use crate::disk::{AlignedArrayBuffer, Read, Seek, SeekFrom};
use core::ptr::slice_from_raw_parts_mut;
use crate::util::print::println;

pub unsafe fn test_file(file: &File, mut file_system: FileSystem<DiskReader>) {
    for cluster in file_system.file_clusters(&file) {
        return;
    }
}

pub unsafe fn read_file<const SIZE: usize>(file: &File, file_name: &str, mut file_system: FileSystem<DiskReader>,
                                           buffer: &mut AlignedArrayBuffer<SIZE>, target_address: *mut u8) {
    let mut disk = file_system.disk.clone();
    let file = file_system
        .find_file_in_root_dir(file_name, buffer)
        .expect("file not found");
    for cluster in file_system.file_clusters(&file) {
        return;
        /*
        let cluster = cluster.unwrap();

        disk.seek(SeekFrom::Start(cluster.start_offset));
        let remaining = cluster.len_bytes as usize;
        while remaining > 0 {
            let reading = remaining.min(buffer.buffer.len());
            disk.read_exact_into(reading, buffer);
            println("Got it!");
            (&mut *slice_from_raw_parts_mut(target_address, reading))
                .copy_from_slice(&buffer.buffer[..reading]);
        }*/
    }
}

pub fn load_file(
    file_name: &str,
    dst: *mut u8,
    fs: &mut FileSystem<DiskReader>,
    disk: &mut DiskReader,
    disk_buffer: &mut AlignedArrayBuffer<16384>,
) -> u64 {
    let disk_buffer_size = disk_buffer.buffer.len();
    let file = fs
        .find_file_in_root_dir(file_name, disk_buffer)
        .expect("file not found");
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
            unsafe {
                (&mut *slice_from_raw_parts_mut(dst.wrapping_add(total_offset), usize::try_from(len).unwrap()))
                    .copy_from_slice(slice);
            }
            /*
            unsafe { copy_to_protected_mode(dst.wrapping_add(total_offset), slice) };
            let written =
                unsafe { protected_mode::read_from_protected_mode(dst.wrapping_add(total_offset)) };
            assert_eq!(slice[0], written);*/

            offset += len;
            total_offset += usize::try_from(len).unwrap();
        }
    }
    file_size
}