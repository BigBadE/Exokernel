//! Buffer head implementation
//!
//! The buffer_head is Linux's abstraction for block I/O.

use alloc::collections::BTreeMap;
use core::ffi::{c_int, c_ulong, c_void};
use core::ptr;
use core::sync::atomic::{AtomicU32, Ordering};

use linux_core::{atomic_t, gfp_t, linux_export, list_head, loff_t, sector_t, size_t, time64_t, EINVAL, GFP_KERNEL};
use linux_mm::{kfree, kmalloc, PAGE_SIZE};

use crate::block::block_device;

// Buffer state bits
pub const BH_Uptodate: u32 = 0;
pub const BH_Dirty: u32 = 1;
pub const BH_Lock: u32 = 2;
pub const BH_Req: u32 = 3;
pub const BH_Mapped: u32 = 5;
pub const BH_New: u32 = 6;
pub const BH_Async_Read: u32 = 7;
pub const BH_Async_Write: u32 = 8;
pub const BH_Delay: u32 = 9;
pub const BH_Boundary: u32 = 10;
pub const BH_Write_EIO: u32 = 11;

// ============================================================================
// Forward declarations for VFS types
// ============================================================================

#[repr(C)]
pub struct inode {
    pub _opaque: [u8; 0],
}

#[repr(C)]
pub struct dentry {
    pub _opaque: [u8; 0],
}

#[repr(C)]
pub struct file_system_type {
    pub _opaque: [u8; 0],
}

#[repr(C)]
pub struct super_operations {
    pub _opaque: [u8; 0],
}

#[repr(C)]
pub struct dentry_operations {
    pub _opaque: [u8; 0],
}

#[repr(C)]
pub struct export_operations {
    pub _opaque: [u8; 0],
}

#[repr(C)]
pub struct address_space {
    pub _opaque: [u8; 0],
}

#[repr(C)]
pub struct page {
    pub _opaque: [u8; 0],
}

// ============================================================================
// Super block (minimal for buffer operations)
// ============================================================================

#[repr(C)]
pub struct super_block {
    pub s_blocksize: c_ulong,
    pub s_blocksize_bits: u8,
    pub s_maxbytes: loff_t,
    pub s_flags: c_ulong,
    pub s_magic: c_ulong,
    pub s_fs_info: *mut c_void,
    pub s_bdev: *mut block_device,
    pub s_root: *mut dentry,
    pub s_type: *mut file_system_type,
    pub s_op: *const super_operations,
    pub s_d_op: *const dentry_operations,
    pub s_export_op: *const export_operations,
    pub s_time_gran: u32,
    pub s_time_min: time64_t,
    pub s_time_max: time64_t,
    pub s_id: [i8; 32],
}

// ============================================================================
// Buffer head structure
// ============================================================================

#[repr(C)]
pub struct buffer_head {
    pub b_state: AtomicU32,
    pub b_this_page: *mut buffer_head,
    pub b_page: *mut page,
    pub b_blocknr: sector_t,
    pub b_size: size_t,
    pub b_data: *mut u8,
    pub b_bdev: *mut block_device,
    pub b_end_io: Option<unsafe extern "C" fn(*mut buffer_head, c_int)>,
    pub b_private: *mut c_void,
    pub b_assoc_buffers: list_head,
    pub b_assoc_map: *mut address_space,
    pub b_count: atomic_t,
}

impl buffer_head {
    fn new() -> Self {
        Self {
            b_state: AtomicU32::new(0),
            b_this_page: ptr::null_mut(),
            b_page: ptr::null_mut(),
            b_blocknr: 0,
            b_size: 0,
            b_data: ptr::null_mut(),
            b_bdev: ptr::null_mut(),
            b_end_io: None,
            b_private: ptr::null_mut(),
            b_assoc_buffers: list_head::new(),
            b_assoc_map: ptr::null_mut(),
            b_count: atomic_t::new(0),
        }
    }
}

// ============================================================================
// Buffer state operations
// ============================================================================

#[inline]
fn test_bit(bit: u32, state: &AtomicU32) -> bool {
    state.load(Ordering::Relaxed) & (1 << bit) != 0
}

#[inline]
fn set_bit(bit: u32, state: &AtomicU32) {
    state.fetch_or(1 << bit, Ordering::SeqCst);
}

#[inline]
fn clear_bit(bit: u32, state: &AtomicU32) {
    state.fetch_and(!(1 << bit), Ordering::SeqCst);
}

#[linux_export]
fn buffer_uptodate(bh: &buffer_head) -> c_int {
    test_bit(BH_Uptodate, &bh.b_state) as c_int
}

#[linux_export]
fn buffer_dirty(bh: &buffer_head) -> c_int {
    test_bit(BH_Dirty, &bh.b_state) as c_int
}

#[linux_export]
fn buffer_locked(bh: &buffer_head) -> c_int {
    test_bit(BH_Lock, &bh.b_state) as c_int
}

#[linux_export]
fn buffer_mapped(bh: &buffer_head) -> c_int {
    test_bit(BH_Mapped, &bh.b_state) as c_int
}

#[linux_export]
fn buffer_new(bh: &buffer_head) -> c_int {
    test_bit(BH_New, &bh.b_state) as c_int
}

#[linux_export]
fn buffer_delay(bh: &buffer_head) -> c_int {
    test_bit(BH_Delay, &bh.b_state) as c_int
}

#[linux_export]
fn set_buffer_uptodate(bh: &buffer_head) {
    set_bit(BH_Uptodate, &bh.b_state);
}

#[linux_export]
fn clear_buffer_uptodate(bh: &buffer_head) {
    clear_bit(BH_Uptodate, &bh.b_state);
}

#[linux_export]
fn set_buffer_dirty(bh: &buffer_head) {
    set_bit(BH_Dirty, &bh.b_state);
}

#[linux_export]
fn clear_buffer_dirty(bh: &buffer_head) {
    clear_bit(BH_Dirty, &bh.b_state);
}

#[linux_export]
fn set_buffer_mapped(bh: &buffer_head) {
    set_bit(BH_Mapped, &bh.b_state);
}

#[linux_export]
fn clear_buffer_mapped(bh: &buffer_head) {
    clear_bit(BH_Mapped, &bh.b_state);
}

#[linux_export]
fn set_buffer_new(bh: &buffer_head) {
    set_bit(BH_New, &bh.b_state);
}

#[linux_export]
fn clear_buffer_new(bh: &buffer_head) {
    clear_bit(BH_New, &bh.b_state);
}

#[linux_export]
fn set_buffer_delay(bh: &buffer_head) {
    set_bit(BH_Delay, &bh.b_state);
}

#[linux_export]
fn clear_buffer_delay(bh: &buffer_head) {
    clear_bit(BH_Delay, &bh.b_state);
}

// ============================================================================
// Reference counting
// ============================================================================

#[linux_export]
fn get_bh(bh: &buffer_head) {
    bh.b_count.inc();
}

#[linux_export]
fn put_bh(bh: &buffer_head) {
    bh.b_count.fetch_sub(1);
}

// ============================================================================
// Buffer cache
// ============================================================================

static mut BUFFER_CACHE: Option<BufferCache> = None;

struct BufferCache {
    buffers: BTreeMap<(u32, u64), *mut buffer_head>,
}

impl BufferCache {
    fn new() -> Self {
        Self {
            buffers: BTreeMap::new(),
        }
    }
}

fn get_cache() -> &'static mut BufferCache {
    unsafe {
        let cache_ptr = &raw mut BUFFER_CACHE;
        if (*cache_ptr).is_none() {
            *cache_ptr = Some(BufferCache::new());
        }
        (*cache_ptr).as_mut().unwrap()
    }
}

// ============================================================================
// Block I/O callback to exokernel
// ============================================================================

unsafe extern "C" {
    fn exo_block_read(dev: u32, sector: u64, buffer: *mut c_void, count: u32) -> c_int;
    fn exo_block_write(dev: u32, sector: u64, buffer: *const c_void, count: u32) -> c_int;
}

// ============================================================================
// Buffer head allocation and I/O
// ============================================================================

#[linux_export]
unsafe fn alloc_buffer_head(gfp: gfp_t) -> *mut buffer_head {
    let size = core::mem::size_of::<buffer_head>() + PAGE_SIZE;
    let ptr = kmalloc(size, gfp);
    if ptr.is_null() {
        return ptr::null_mut();
    }

    let bh = ptr as *mut buffer_head;
    ptr::write(bh, buffer_head::new());
    (*bh).b_data = (bh as *mut u8).add(core::mem::size_of::<buffer_head>());
    (*bh).b_count = atomic_t::new(1);
    bh
}

#[linux_export]
unsafe fn free_buffer_head(bh: *mut buffer_head) {
    if !bh.is_null() {
        kfree(bh as *const c_void);
    }
}

#[linux_export]
unsafe fn sb_bread(sb: *mut super_block, block: sector_t) -> *mut buffer_head {
    if sb.is_null() {
        return ptr::null_mut();
    }

    let block_size = (*sb).s_blocksize as usize;
    let bdev = (*sb).s_bdev;
    let dev_id = if !bdev.is_null() { (*bdev).bd_dev } else { 0 };

    let cache = get_cache();
    if let Some(&cached_bh) = cache.buffers.get(&(dev_id, block)) {
        if buffer_uptodate(&*cached_bh) != 0 {
            get_bh(&*cached_bh);
            return cached_bh;
        }
    }

    let bh = alloc_buffer_head(GFP_KERNEL);
    if bh.is_null() {
        return ptr::null_mut();
    }

    (*bh).b_blocknr = block;
    (*bh).b_size = block_size;
    (*bh).b_bdev = bdev;

    let sectors_per_block = block_size / 512;
    let sector = block * sectors_per_block as u64;

    let ret = exo_block_read(
        dev_id,
        sector,
        (*bh).b_data as *mut c_void,
        sectors_per_block as u32,
    );

    if ret < 0 {
        free_buffer_head(bh);
        return ptr::null_mut();
    }

    set_buffer_uptodate(&*bh);
    set_buffer_mapped(&*bh);
    cache.buffers.insert((dev_id, block), bh);
    bh
}

#[linux_export]
unsafe fn sb_bread_unmovable(sb: *mut super_block, block: sector_t) -> *mut buffer_head {
    sb_bread(sb, block)
}

#[linux_export]
unsafe fn sb_getblk(sb: *mut super_block, block: sector_t) -> *mut buffer_head {
    if sb.is_null() {
        return ptr::null_mut();
    }

    let block_size = (*sb).s_blocksize as usize;
    let bdev = (*sb).s_bdev;
    let dev_id = if !bdev.is_null() { (*bdev).bd_dev } else { 0 };

    let cache = get_cache();
    if let Some(&cached_bh) = cache.buffers.get(&(dev_id, block)) {
        get_bh(&*cached_bh);
        return cached_bh;
    }

    let bh = alloc_buffer_head(GFP_KERNEL);
    if bh.is_null() {
        return ptr::null_mut();
    }

    (*bh).b_blocknr = block;
    (*bh).b_size = block_size;
    (*bh).b_bdev = bdev;
    set_buffer_mapped(&*bh);
    cache.buffers.insert((dev_id, block), bh);
    bh
}

#[linux_export]
unsafe fn brelse(bh: *mut buffer_head) {
    if bh.is_null() {
        return;
    }
    put_bh(&*bh);
}

#[linux_export]
unsafe fn bforget(bh: *mut buffer_head) {
    if bh.is_null() {
        return;
    }
    clear_buffer_uptodate(&*bh);
    clear_buffer_dirty(&*bh);
    brelse(bh);
}

#[linux_export]
unsafe fn mark_buffer_dirty(bh: *mut buffer_head) {
    if !bh.is_null() {
        set_buffer_dirty(&*bh);
    }
}

#[linux_export]
unsafe fn sync_dirty_buffer(bh: *mut buffer_head) -> c_int {
    if bh.is_null() {
        return -EINVAL;
    }

    if buffer_dirty(&*bh) == 0 {
        return 0;
    }

    let bdev = (*bh).b_bdev;
    let dev_id = if !bdev.is_null() { (*bdev).bd_dev } else { 0 };
    let block_size = (*bh).b_size;
    let sectors_per_block = block_size / 512;
    let sector = (*bh).b_blocknr * sectors_per_block as u64;

    let ret = exo_block_write(
        dev_id,
        sector,
        (*bh).b_data as *const c_void,
        sectors_per_block as u32,
    );

    if ret < 0 {
        return ret;
    }

    clear_buffer_dirty(&*bh);
    0
}

#[linux_export]
unsafe fn submit_bh(rw: c_int, bh: *mut buffer_head) -> c_int {
    if bh.is_null() {
        return -EINVAL;
    }

    let bdev = (*bh).b_bdev;
    let dev_id = if !bdev.is_null() { (*bdev).bd_dev } else { 0 };
    let block_size = (*bh).b_size;
    let sectors_per_block = block_size / 512;
    let sector = (*bh).b_blocknr * sectors_per_block as u64;

    if rw == 0 {
        let ret = exo_block_read(
            dev_id,
            sector,
            (*bh).b_data as *mut c_void,
            sectors_per_block as u32,
        );
        if ret >= 0 {
            set_buffer_uptodate(&*bh);
        }
        ret
    } else {
        let ret = exo_block_write(
            dev_id,
            sector,
            (*bh).b_data as *const c_void,
            sectors_per_block as u32,
        );
        if ret >= 0 {
            clear_buffer_dirty(&*bh);
        }
        ret
    }
}

// ============================================================================
// Locking
// ============================================================================

#[linux_export]
unsafe fn lock_buffer(bh: *mut buffer_head) {
    if bh.is_null() {
        return;
    }
    while (*bh).b_state.fetch_or(1 << BH_Lock, Ordering::Acquire) & (1 << BH_Lock) != 0 {
        core::hint::spin_loop();
    }
}

#[linux_export]
unsafe fn unlock_buffer(bh: *mut buffer_head) {
    if !bh.is_null() {
        (*bh).b_state.fetch_and(!(1 << BH_Lock), Ordering::Release);
    }
}

#[linux_export]
unsafe fn trylock_buffer(bh: *mut buffer_head) -> c_int {
    if bh.is_null() {
        return 0;
    }
    let old = (*bh).b_state.fetch_or(1 << BH_Lock, Ordering::Acquire);
    if old & (1 << BH_Lock) == 0 { 1 } else { 0 }
}

#[linux_export]
unsafe fn wait_on_buffer(bh: *mut buffer_head) {
    if bh.is_null() {
        return;
    }
    while buffer_locked(&*bh) != 0 {
        core::hint::spin_loop();
    }
}

#[linux_export]
unsafe fn map_bh(bh: *mut buffer_head, sb: *mut super_block, block: sector_t) {
    if bh.is_null() || sb.is_null() {
        return;
    }
    (*bh).b_bdev = (*sb).s_bdev;
    (*bh).b_blocknr = block;
    (*bh).b_size = (*sb).s_blocksize as size_t;
    set_buffer_mapped(&*bh);
}

#[linux_export]
unsafe fn sb_breadahead(sb: *mut super_block, block: sector_t) {
    let bh = sb_bread(sb, block);
    if !bh.is_null() {
        brelse(bh);
    }
}

#[linux_export]
unsafe fn ll_rw_block(rw: c_int, nr: c_int, bhs: *mut *mut buffer_head) {
    if bhs.is_null() {
        return;
    }
    for i in 0..nr {
        let bh = *bhs.add(i as usize);
        if !bh.is_null() {
            submit_bh(rw, bh);
        }
    }
}
