//! Inode implementation
//!
//! An inode represents a file object (file, directory, symlink, etc.)
//! in a filesystem. It contains metadata and pointers to file operations.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use core::ptr;

use crate::types::*;
use crate::superblock::*;
use crate::operations::*;

// ============================================================================
// Inode structure
// ============================================================================

/// Inode - represents a filesystem object
#[repr(C)]
pub struct inode {
    /// File mode (type and permissions)
    pub i_mode: umode_t,
    /// Flags
    pub i_opflags: c_uint,
    /// Owner UID
    pub i_uid: kuid_t,
    /// Owner GID
    pub i_gid: kgid_t,
    /// Inode flags (e.g., S_IMMUTABLE)
    pub i_flags: c_uint,
    /// ACL
    pub i_acl: *mut c_void,
    /// Default ACL
    pub i_default_acl: *mut c_void,
    /// Inode operations
    pub i_op: *const inode_operations,
    /// Superblock
    pub i_sb: *mut super_block,
    /// Address space (for page cache)
    pub i_mapping: *mut address_space,
    /// Security module data
    pub i_security: *mut c_void,
    /// Inode number
    pub i_ino: ino_t,
    /// Link count / device ID union
    pub i_nlink_or_rdev: InodeUnion1,
    /// Device ID (for block/char devices)
    pub i_rdev: dev_t,
    /// File size
    pub i_size: loff_t,
    /// Last access time
    pub i_atime: timespec64,
    /// Last modification time
    pub i_mtime: timespec64,
    /// Creation time
    pub i_ctime: timespec64,
    /// Lock for i_size and i_blocks
    pub i_lock: spinlock_t,
    /// Size in 512-byte blocks
    pub i_bytes: c_uint,
    /// Block count (512-byte units)
    pub i_blocks: blkcnt_t,
    /// Generation number (for NFS)
    pub i_generation: u32,
    /// Writeback list
    pub i_io_list: list_head,
    /// Bdev or cdev info
    pub i_bdev_or_cdev: *mut c_void,
    /// Link count
    pub i_nlink: c_uint,
    /// Block size
    pub i_blkbits: c_uint,
    /// Version
    pub i_version: u64,
    /// Write count
    pub i_writecount: atomic_t,
    /// File operations
    pub i_fop: *const file_operations,
    /// Private data (usually fs-specific)
    pub i_private: *mut c_void,
    /// List of dentries
    pub i_dentry: hlist_head,
    /// RCU head for deferred freeing
    pub i_rcu: rcu_head,
    /// Inode state
    pub i_state: c_ulong,
    /// Mutex for directory operations
    pub i_rwsem: rw_semaphore,
    /// Hash
    pub i_hash: hlist_node,
    /// Dirty list
    pub i_wb_list: list_head,
    /// LRU list
    pub i_lru: list_head,
    /// Superblock list
    pub i_sb_list: list_head,
    /// Private list
    pub i_devices: list_head,
    /// Pipe info (if pipe)
    pub i_pipe: *mut c_void,
    /// Internal data mapping
    pub i_data: address_space,
}

/// Union for link count or device number
#[repr(C)]
#[derive(Clone, Copy)]
pub union InodeUnion1 {
    pub nlink: c_uint,
    pub rdev: dev_t,
}

impl Default for InodeUnion1 {
    fn default() -> Self {
        Self { nlink: 1 }
    }
}

impl inode {
    /// Create a new empty inode
    pub fn new() -> Self {
        Self {
            i_mode: 0,
            i_opflags: 0,
            i_uid: 0,
            i_gid: 0,
            i_flags: 0,
            i_acl: ptr::null_mut(),
            i_default_acl: ptr::null_mut(),
            i_op: ptr::null(),
            i_sb: ptr::null_mut(),
            i_mapping: ptr::null_mut(),
            i_security: ptr::null_mut(),
            i_ino: 0,
            i_nlink_or_rdev: InodeUnion1::default(),
            i_rdev: 0,
            i_size: 0,
            i_atime: timespec64::default(),
            i_mtime: timespec64::default(),
            i_ctime: timespec64::default(),
            i_lock: 0,
            i_bytes: 0,
            i_blocks: 0,
            i_generation: 0,
            i_io_list: list_head::new(),
            i_bdev_or_cdev: ptr::null_mut(),
            i_nlink: 1,
            i_blkbits: 0,
            i_version: 0,
            i_writecount: atomic_t::new(0),
            i_fop: ptr::null(),
            i_private: ptr::null_mut(),
            i_dentry: hlist_head::default(),
            i_rcu: rcu_head::default(),
            i_state: 0,
            i_rwsem: rw_semaphore::default(),
            i_hash: hlist_node::default(),
            i_wb_list: list_head::new(),
            i_lru: list_head::new(),
            i_sb_list: list_head::new(),
            i_devices: list_head::new(),
            i_pipe: ptr::null_mut(),
            i_data: address_space::default(),
        }
    }
}

impl Default for inode {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Address space (page cache)
// ============================================================================

/// Address space - represents a mapping of pages
#[repr(C)]
pub struct address_space {
    /// Host inode
    pub host: *mut inode,
    /// Page tree
    pub i_pages: *mut c_void, // xarray
    /// Invalidation lock
    pub invalidate_lock: rw_semaphore,
    /// GFP mask
    pub gfp_mask: gfp_t,
    /// Write operations
    pub writeback_index: c_ulong,
    /// Address space operations
    pub a_ops: *const address_space_operations,
    /// Flags
    pub flags: c_ulong,
    /// Writers
    pub i_mmap_writable: atomic_t,
    /// Memory mapped files
    pub i_mmap: *mut c_void,
    /// Semaphore
    pub i_mmap_rwsem: rw_semaphore,
    /// Number of pages
    pub nrpages: c_ulong,
    /// Writeback pages
    pub writeback_index2: c_ulong,
    /// Private list
    pub private_list: list_head,
    /// Private data
    pub private_data: *mut c_void,
}

impl Default for address_space {
    fn default() -> Self {
        Self {
            host: ptr::null_mut(),
            i_pages: ptr::null_mut(),
            invalidate_lock: rw_semaphore::default(),
            gfp_mask: 0,
            writeback_index: 0,
            a_ops: ptr::null(),
            flags: 0,
            i_mmap_writable: atomic_t::new(0),
            i_mmap: ptr::null_mut(),
            i_mmap_rwsem: rw_semaphore::default(),
            nrpages: 0,
            writeback_index2: 0,
            private_list: list_head::new(),
            private_data: ptr::null_mut(),
        }
    }
}

// ============================================================================
// Inode state flags
// ============================================================================

pub const I_DIRTY_SYNC: c_ulong = 1 << 0;
pub const I_DIRTY_DATASYNC: c_ulong = 1 << 1;
pub const I_DIRTY_PAGES: c_ulong = 1 << 2;
pub const I_NEW: c_ulong = 1 << 3;
pub const I_WILL_FREE: c_ulong = 1 << 4;
pub const I_FREEING: c_ulong = 1 << 5;
pub const I_CLEAR: c_ulong = 1 << 6;
pub const I_SYNC: c_ulong = 1 << 7;
pub const I_REFERENCED: c_ulong = 1 << 8;
pub const I_LINKABLE: c_ulong = 1 << 9;
pub const I_DIRTY_TIME: c_ulong = 1 << 11;
pub const I_WB_SWITCH: c_ulong = 1 << 12;
pub const I_OVL_INUSE: c_ulong = 1 << 13;
pub const I_CREATING: c_ulong = 1 << 14;

pub const I_DIRTY: c_ulong = I_DIRTY_SYNC | I_DIRTY_DATASYNC | I_DIRTY_PAGES;

// ============================================================================
// Inode flags
// ============================================================================

pub const S_SYNC: c_uint = 1;
pub const S_NOATIME: c_uint = 2;
pub const S_APPEND: c_uint = 4;
pub const S_IMMUTABLE: c_uint = 8;
pub const S_DEAD: c_uint = 16;
pub const S_NOQUOTA: c_uint = 32;
pub const S_DIRSYNC: c_uint = 64;
pub const S_NOCMTIME: c_uint = 128;
pub const S_SWAPFILE: c_uint = 256;
pub const S_PRIVATE: c_uint = 512;
pub const S_IMA: c_uint = 1024;
pub const S_AUTOMOUNT: c_uint = 2048;
pub const S_NOSEC: c_uint = 4096;
pub const S_DAX: c_uint = 8192;
pub const S_ENCRYPTED: c_uint = 16384;
pub const S_CASEFOLD: c_uint = 32768;
pub const S_VERITY: c_uint = 65536;
pub const S_KERNEL_FILE: c_uint = 131072;

// ============================================================================
// Inode helper functions
// ============================================================================

/// Check if inode is a regular file
#[inline]
pub fn i_is_reg(inode: *const inode) -> bool {
    if inode.is_null() {
        return false;
    }
    unsafe { S_ISREG((*inode).i_mode) }
}

/// Check if inode is a directory
#[inline]
pub fn i_is_dir(inode: *const inode) -> bool {
    if inode.is_null() {
        return false;
    }
    unsafe { S_ISDIR((*inode).i_mode) }
}

/// Check if inode is a symlink
#[inline]
pub fn i_is_lnk(inode: *const inode) -> bool {
    if inode.is_null() {
        return false;
    }
    unsafe { S_ISLNK((*inode).i_mode) }
}

/// Get inode size
#[inline]
pub fn i_size_read_fn(inode: *const inode) -> loff_t {
    if inode.is_null() {
        return 0;
    }
    unsafe { (*inode).i_size }
}

/// Set inode size
#[inline]
pub fn i_size_write_fn(inode: *mut inode, size: loff_t) {
    if !inode.is_null() {
        unsafe {
            (*inode).i_size = size;
        }
    }
}
