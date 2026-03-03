//! File structure implementation
//!
//! A file represents an open file handle. It contains the current
//! position, flags, and pointers to operations.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use core::ptr;

use crate::types::*;
use crate::inode::{inode, address_space};
use crate::dentry::dentry;
use crate::operations::file_operations;

// ============================================================================
// File structure
// ============================================================================

/// File - represents an open file
#[repr(C)]
pub struct file {
    /// File position union
    pub f_u: file_union,
    /// Path to file
    pub f_path: path,
    /// Inode
    pub f_inode: *mut inode,
    /// File operations
    pub f_op: *const file_operations,
    /// Spinlock for f_ep, f_flags
    pub f_lock: spinlock_t,
    /// File mode
    pub f_mode: fmode_t,
    /// Reference count
    pub f_count: atomic_long_t,
    /// File flags (O_RDONLY, etc.)
    pub f_flags: c_uint,
    /// File position
    pub f_pos: loff_t,
    /// Position lock
    pub f_pos_lock: mutex,
    /// Owner
    pub f_owner: fown_struct,
    /// Credentials
    pub f_cred: *const cred,
    /// Read-ahead state
    pub f_ra: file_ra_state,
    /// Version
    pub f_version: u64,
    /// Security module data
    pub f_security: *mut c_void,
    /// Private data (tty, etc.)
    pub f_private: *mut c_void,
    /// Epoll links
    pub f_ep: *mut c_void,
    /// Address space
    pub f_mapping: *mut address_space,
    /// Error code
    pub f_wb_err: c_int,
    /// Spin count
    pub f_sb_err: c_int,
}

/// File union for list management
#[repr(C)]
#[derive(Clone, Copy)]
pub union file_union {
    /// LRU list
    pub fu_llist: list_head,
    /// RCU head
    pub fu_rcuhead: rcu_head,
}

impl Default for file_union {
    fn default() -> Self {
        Self {
            fu_llist: list_head::new(),
        }
    }
}

impl file {
    /// Create a new empty file
    pub fn new() -> Self {
        Self {
            f_u: file_union::default(),
            f_path: path::default(),
            f_inode: ptr::null_mut(),
            f_op: ptr::null(),
            f_lock: 0,
            f_mode: 0,
            f_count: atomic_long_t::new(1),
            f_flags: 0,
            f_pos: 0,
            f_pos_lock: mutex::default(),
            f_owner: fown_struct::default(),
            f_cred: ptr::null(),
            f_ra: file_ra_state::default(),
            f_version: 0,
            f_security: ptr::null_mut(),
            f_private: ptr::null_mut(),
            f_ep: ptr::null_mut(),
            f_mapping: ptr::null_mut(),
            f_wb_err: 0,
            f_sb_err: 0,
        }
    }
}

impl Default for file {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Supporting structures
// ============================================================================

/// File owner structure
#[repr(C)]
#[derive(Debug, Default)]
pub struct fown_struct {
    pub lock: rwlock_t,
    pub pid: *mut c_void, // pid struct
    pub pid_type: c_int,
    pub uid: kuid_t,
    pub euid: kuid_t,
    pub signum: c_int,
}

/// Read-ahead state
#[repr(C)]
#[derive(Debug, Default)]
pub struct file_ra_state {
    pub start: c_ulong,
    pub size: c_uint,
    pub async_size: c_uint,
    pub ra_pages: c_uint,
    pub mmap_miss: c_uint,
    pub prev_pos: loff_t,
}

// ============================================================================
// File mode flags
// ============================================================================

pub const FMODE_READ: fmode_t = 0x1;
pub const FMODE_WRITE: fmode_t = 0x2;
pub const FMODE_LSEEK: fmode_t = 0x4;
pub const FMODE_PREAD: fmode_t = 0x8;
pub const FMODE_PWRITE: fmode_t = 0x10;
pub const FMODE_EXEC: fmode_t = 0x20;
pub const FMODE_NDELAY: fmode_t = 0x40;
pub const FMODE_EXCL: fmode_t = 0x80;
pub const FMODE_WRITE_IOCTL: fmode_t = 0x100;
pub const FMODE_32BITHASH: fmode_t = 0x200;
pub const FMODE_64BITHASH: fmode_t = 0x400;
pub const FMODE_NOCMTIME: fmode_t = 0x800;
pub const FMODE_RANDOM: fmode_t = 0x1000;
pub const FMODE_UNSIGNED_OFFSET: fmode_t = 0x2000;
pub const FMODE_PATH: fmode_t = 0x4000;
pub const FMODE_ATOMIC_POS: fmode_t = 0x8000;
pub const FMODE_WRITER: fmode_t = 0x10000;
pub const FMODE_CAN_READ: fmode_t = 0x20000;
pub const FMODE_CAN_WRITE: fmode_t = 0x40000;
pub const FMODE_OPENED: fmode_t = 0x80000;
pub const FMODE_CREATED: fmode_t = 0x100000;
pub const FMODE_STREAM: fmode_t = 0x200000;
pub const FMODE_NONOTIFY: fmode_t = 0x4000000;
pub const FMODE_NOWAIT: fmode_t = 0x8000000;
pub const FMODE_NEED_UNMOUNT: fmode_t = 0x10000000;
pub const FMODE_NOACCOUNT: fmode_t = 0x20000000;
pub const FMODE_BUF_RASYNC: fmode_t = 0x40000000;

// ============================================================================
// Open flags (O_*)
// ============================================================================

pub const O_ACCMODE: c_uint = 0o003;
pub const O_RDONLY: c_uint = 0o0;
pub const O_WRONLY: c_uint = 0o1;
pub const O_RDWR: c_uint = 0o2;
pub const O_CREAT: c_uint = 0o100;
pub const O_EXCL: c_uint = 0o200;
pub const O_NOCTTY: c_uint = 0o400;
pub const O_TRUNC: c_uint = 0o1000;
pub const O_APPEND: c_uint = 0o2000;
pub const O_NONBLOCK: c_uint = 0o4000;
pub const O_DSYNC: c_uint = 0o10000;
pub const O_SYNC: c_uint = 0o4010000;
pub const O_DIRECTORY: c_uint = 0o200000;
pub const O_NOFOLLOW: c_uint = 0o400000;
pub const O_CLOEXEC: c_uint = 0o2000000;
pub const O_DIRECT: c_uint = 0o40000;
pub const O_LARGEFILE: c_uint = 0o100000;
pub const O_NOATIME: c_uint = 0o1000000;
pub const O_PATH: c_uint = 0o10000000;
pub const O_TMPFILE: c_uint = 0o20200000;

// ============================================================================
// File helper functions
// ============================================================================

/// Get inode from file
#[inline]
pub fn file_inode_fn(f: *const file) -> *mut inode {
    if f.is_null() {
        return ptr::null_mut();
    }
    unsafe { (*f).f_inode }
}

/// Get dentry from file
#[inline]
pub fn file_dentry(f: *const file) -> *mut dentry {
    if f.is_null() {
        return ptr::null_mut();
    }
    unsafe { (*f).f_path.dentry }
}

/// Check if file is readable
#[inline]
pub fn file_readable(f: *const file) -> bool {
    if f.is_null() {
        return false;
    }
    unsafe { ((*f).f_mode & FMODE_READ) != 0 }
}

/// Check if file is writable
#[inline]
pub fn file_writeable(f: *const file) -> bool {
    if f.is_null() {
        return false;
    }
    unsafe { ((*f).f_mode & FMODE_WRITE) != 0 }
}

// ============================================================================
// I/O context structures
// ============================================================================

/// Kernel I/O control block
#[repr(C)]
#[derive(Default)]
pub struct kiocb {
    pub ki_filp: *mut file,
    pub ki_pos: loff_t,
    pub ki_complete: Option<unsafe extern "C" fn(*mut kiocb, c_long)>,
    pub ki_flags: c_int,
    pub ki_ioprio: u16,
    pub ki_waitq: *mut c_void,
}


/// I/O vector iterator
#[repr(C)]
#[derive(Default)]
pub struct iov_iter {
    pub iter_type: u8,
    pub copy_mc: bool,
    pub nofault: bool,
    pub data_source: bool,
    pub iov_offset: usize,
    pub count: usize,
    /// Union of different iterator types
    pub iov: *const iovec,
    pub nr_segs: c_ulong,
}


/// I/O vector
#[repr(C)]
#[derive(Default)]
pub struct iovec {
    pub iov_base: *mut c_void,
    pub iov_len: usize,
}


/// Writeback control
#[repr(C)]
#[derive(Default)]
pub struct writeback_control {
    pub nr_to_write: c_long,
    pub pages_skipped: c_long,
    pub range_start: loff_t,
    pub range_end: loff_t,
    pub sync_mode: c_int,
    pub tagged_writepages: c_uint,
    pub for_kupdate: c_uint,
    pub for_background: c_uint,
    pub for_reclaim: c_uint,
    pub range_cyclic: c_uint,
    pub for_sync: c_uint,
    pub no_cgroup_owner: c_uint,
    pub punt_to_cgroup: c_uint,
    pub wb: *mut c_void,
    pub inode: *mut inode,
    pub wb_id: c_int,
    pub wb_lcand_id: c_int,
    pub wb_tcand_id: c_int,
    pub wb_bytes: usize,
    pub wb_lcand_bytes: usize,
    pub wb_tcand_bytes: usize,
}


/// Dir context for readdir
#[repr(C)]
#[derive(Default)]
pub struct dir_context {
    pub actor: Option<unsafe extern "C" fn(*mut dir_context, *const c_char, c_int, loff_t, u64, c_uint) -> bool>,
    pub pos: loff_t,
}

