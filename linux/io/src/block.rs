//! Block device operations

use core::ffi::{c_int, c_uint, c_ulong, c_void};
use core::ptr;

use linux_core::{dev_t, fmode_t, gfp_t, linux_export, loff_t, sector_t, EINVAL, GFP_KERNEL};
use linux_mm::{kfree, kmalloc};

use crate::buffer::super_block;

// ============================================================================
// Block device structure
// ============================================================================

#[repr(C)]
pub struct block_device {
    pub bd_dev: dev_t,
    pub bd_inode: *mut c_void,
    pub bd_disk: *mut c_void,
    pub bd_block_size: c_int,
}

// ============================================================================
// Block device operations
// ============================================================================

#[linux_export]
unsafe fn bdev_logical_block_size(bdev: *mut block_device) -> c_uint {
    if bdev.is_null() {
        return 512;
    }
    if (*bdev).bd_block_size > 0 {
        (*bdev).bd_block_size as c_uint
    } else {
        512
    }
}

#[linux_export]
unsafe fn bdev_physical_block_size(bdev: *mut block_device) -> c_uint {
    bdev_logical_block_size(bdev)
}

#[linux_export]
fn bdev_nr_sectors(_bdev: *mut block_device) -> sector_t {
    0xFFFFFFFF
}

#[linux_export]
fn bdev_nr_bytes(bdev: *mut block_device) -> loff_t {
    (bdev_nr_sectors(bdev) * 512) as loff_t
}

#[linux_export]
fn bdev_read_only(_bdev: *mut block_device) -> c_int {
    0
}

#[linux_export]
unsafe fn set_blocksize(bdev: *mut block_device, size: c_int) -> c_int {
    if bdev.is_null() {
        return -EINVAL;
    }

    if size < 512 || size > 4096 || (size & (size - 1)) != 0 {
        return -EINVAL;
    }

    (*bdev).bd_block_size = size;
    0
}

#[linux_export]
unsafe fn sb_min_blocksize(sb: *mut super_block, size: c_int) -> c_int {
    if sb.is_null() {
        return 0;
    }

    let bdev = (*sb).s_bdev;
    let min_size = bdev_logical_block_size(bdev) as c_int;

    if size < min_size {
        if set_blocksize(bdev, min_size) < 0 {
            return 0;
        }
        (*sb).s_blocksize = min_size as c_ulong;
        (*sb).s_blocksize_bits = min_size.trailing_zeros() as u8;
        min_size
    } else {
        if set_blocksize(bdev, size) < 0 {
            return 0;
        }
        (*sb).s_blocksize = size as c_ulong;
        (*sb).s_blocksize_bits = size.trailing_zeros() as u8;
        size
    }
}

#[linux_export]
unsafe fn sb_set_blocksize(sb: *mut super_block, size: c_int) -> c_int {
    if sb.is_null() {
        return 0;
    }

    let bdev = (*sb).s_bdev;
    if set_blocksize(bdev, size) < 0 {
        return 0;
    }
    (*sb).s_blocksize = size as c_ulong;
    (*sb).s_blocksize_bits = size.trailing_zeros() as u8;
    size
}

// ============================================================================
// Block device file operations
// ============================================================================

#[linux_export]
unsafe fn blkdev_get_by_path(
    _path: *const i8,
    _mode: fmode_t,
    _holder: *mut c_void,
) -> *mut block_device {
    let bdev = kmalloc(core::mem::size_of::<block_device>(), GFP_KERNEL) as *mut block_device;
    if bdev.is_null() {
        return ptr::null_mut();
    }

    (*bdev).bd_dev = 0;
    (*bdev).bd_inode = ptr::null_mut();
    (*bdev).bd_disk = ptr::null_mut();
    (*bdev).bd_block_size = 512;

    bdev
}

#[linux_export]
unsafe fn blkdev_put(bdev: *mut block_device, _mode: fmode_t) {
    if !bdev.is_null() {
        kfree(bdev as *const c_void);
    }
}

#[linux_export]
fn sync_blockdev(_bdev: *mut block_device) -> c_int {
    0
}

#[linux_export]
fn sync_blockdev_nowait(_bdev: *mut block_device) -> c_int {
    0
}

#[linux_export]
fn freeze_bdev(_bdev: *mut block_device) -> *mut super_block {
    ptr::null_mut()
}

#[linux_export]
fn thaw_bdev(_bdev: *mut block_device) -> c_int {
    0
}

// ============================================================================
// Request queue (simplified)
// ============================================================================

#[repr(C)]
pub struct request_queue {
    pub _opaque: [u8; 64],
}

#[linux_export]
fn bdev_get_queue(_bdev: *mut block_device) -> *mut request_queue {
    ptr::null_mut()
}

#[linux_export]
fn bdev_max_discard_sectors(_bdev: *mut block_device) -> c_uint {
    0
}

#[linux_export]
fn blk_queue_discard(_q: *mut request_queue) -> c_int {
    0
}

// ============================================================================
// Invalidation
// ============================================================================

#[linux_export]
fn invalidate_bdev(_bdev: *mut block_device) {
}

#[linux_export]
fn invalidate_inodes(_bdev: *mut block_device) -> c_int {
    0
}

#[linux_export]
unsafe fn kill_block_super(sb: *mut super_block) {
    if sb.is_null() {
        return;
    }

    if !(*sb).s_bdev.is_null() {
        sync_blockdev((*sb).s_bdev);
    }
}

// ============================================================================
// FSTRIM / Discard
// ============================================================================

#[repr(C)]
pub struct fstrim_range {
    pub start: u64,
    pub len: u64,
    pub minlen: u64,
}

#[linux_export]
fn blkdev_issue_discard(
    _bdev: *mut block_device,
    _sector: sector_t,
    _nr_sects: sector_t,
    _gfp_mask: gfp_t,
) -> c_int {
    0
}

#[linux_export]
fn blkdev_issue_flush(_bdev: *mut block_device) -> c_int {
    0
}
