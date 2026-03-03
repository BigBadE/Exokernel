//! VFS operation structures
//!
//! These structures define the function pointers that filesystem
//! drivers implement to provide their functionality.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use core::ptr;

use crate::types::*;
use crate::inode::{inode, address_space};
use crate::dentry::dentry;
use crate::file::{file, kiocb, iov_iter, writeback_control, dir_context};
use crate::superblock::super_block;

// ============================================================================
// Superblock operations
// ============================================================================

/// Superblock operations
#[repr(C)]
#[derive(Default)]
pub struct super_operations {
    /// Allocate inode
    pub alloc_inode: Option<unsafe extern "C" fn(*mut super_block) -> *mut inode>,
    /// Destroy inode
    pub destroy_inode: Option<unsafe extern "C" fn(*mut inode)>,
    /// Free inode
    pub free_inode: Option<unsafe extern "C" fn(*mut inode)>,
    /// Dirty inode
    pub dirty_inode: Option<unsafe extern "C" fn(*mut inode, c_int)>,
    /// Write inode
    pub write_inode: Option<unsafe extern "C" fn(*mut inode, *mut c_void) -> c_int>,
    /// Drop inode
    pub drop_inode: Option<unsafe extern "C" fn(*mut inode) -> c_int>,
    /// Evict inode
    pub evict_inode: Option<unsafe extern "C" fn(*mut inode)>,
    /// Put superblock
    pub put_super: Option<unsafe extern "C" fn(*mut super_block)>,
    /// Sync filesystem
    pub sync_fs: Option<unsafe extern "C" fn(*mut super_block, c_int) -> c_int>,
    /// Freeze superblock
    pub freeze_super: Option<unsafe extern "C" fn(*mut super_block, c_int) -> c_int>,
    /// Freeze filesystem
    pub freeze_fs: Option<unsafe extern "C" fn(*mut super_block) -> c_int>,
    /// Thaw superblock
    pub thaw_super: Option<unsafe extern "C" fn(*mut super_block, c_int) -> c_int>,
    /// Unfreeze filesystem
    pub unfreeze_fs: Option<unsafe extern "C" fn(*mut super_block) -> c_int>,
    /// Get filesystem statistics
    pub statfs: Option<unsafe extern "C" fn(*mut dentry, *mut kstatfs) -> c_int>,
    /// Remount filesystem
    pub remount_fs: Option<unsafe extern "C" fn(*mut super_block, *mut c_int, *mut c_char) -> c_int>,
    /// Unmount begin
    pub umount_begin: Option<unsafe extern "C" fn(*mut super_block)>,
    /// Show options
    pub show_options: Option<unsafe extern "C" fn(*mut c_void, *mut dentry) -> c_int>,
    /// Show devname
    pub show_devname: Option<unsafe extern "C" fn(*mut c_void, *mut dentry) -> c_int>,
    /// Show path
    pub show_path: Option<unsafe extern "C" fn(*mut c_void, *mut dentry) -> c_int>,
    /// Show stats
    pub show_stats: Option<unsafe extern "C" fn(*mut c_void, *mut dentry) -> c_int>,
    /// Quota read
    pub quota_read: Option<unsafe extern "C" fn(*mut super_block, c_int, *mut c_char, usize, loff_t) -> isize>,
    /// Quota write
    pub quota_write: Option<unsafe extern "C" fn(*mut super_block, c_int, *const c_char, usize, loff_t) -> isize>,
    /// Get dquots
    pub get_dquots: Option<unsafe extern "C" fn(*mut inode) -> *mut *mut c_void>,
    /// Nr cached objects
    pub nr_cached_objects: Option<unsafe extern "C" fn(*mut super_block, *mut c_void) -> c_long>,
    /// Free cached objects
    pub free_cached_objects: Option<unsafe extern "C" fn(*mut super_block, *mut c_void, c_long)>,
}

/// Filesystem statistics
#[repr(C)]
#[derive(Debug, Default)]
pub struct kstatfs {
    pub f_type: c_long,
    pub f_bsize: c_long,
    pub f_blocks: u64,
    pub f_bfree: u64,
    pub f_bavail: u64,
    pub f_files: u64,
    pub f_ffree: u64,
    pub f_fsid: [c_int; 2],
    pub f_namelen: c_long,
    pub f_frsize: c_long,
    pub f_flags: c_long,
    pub f_spare: [c_long; 4],
}

// ============================================================================
// Inode operations
// ============================================================================

/// Inode operations
#[repr(C)]
#[derive(Default)]
pub struct inode_operations {
    /// Lookup file in directory
    pub lookup: Option<unsafe extern "C" fn(*mut inode, *mut dentry, c_uint) -> *mut dentry>,
    /// Get link target
    pub get_link: Option<unsafe extern "C" fn(*mut dentry, *mut inode, *mut c_void) -> *const c_char>,
    /// Check permission
    pub permission: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut inode, c_int) -> c_int>,
    /// Get ACL
    pub get_inode_acl: Option<unsafe extern "C" fn(*mut inode, c_int, bool) -> *mut c_void>,
    /// Readlink
    pub readlink: Option<unsafe extern "C" fn(*mut dentry, *mut c_char, c_int) -> c_int>,
    /// Create file
    pub create: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut inode, *mut dentry, umode_t, bool) -> c_int>,
    /// Link
    pub link: Option<unsafe extern "C" fn(*mut dentry, *mut inode, *mut dentry) -> c_int>,
    /// Unlink
    pub unlink: Option<unsafe extern "C" fn(*mut inode, *mut dentry) -> c_int>,
    /// Symlink
    pub symlink: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut inode, *mut dentry, *const c_char) -> c_int>,
    /// Mkdir
    pub mkdir: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut inode, *mut dentry, umode_t) -> c_int>,
    /// Rmdir
    pub rmdir: Option<unsafe extern "C" fn(*mut inode, *mut dentry) -> c_int>,
    /// Mknod
    pub mknod: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut inode, *mut dentry, umode_t, dev_t) -> c_int>,
    /// Rename
    pub rename: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut inode, *mut dentry, *mut inode, *mut dentry, c_uint) -> c_int>,
    /// Setattr
    pub setattr: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut dentry, *mut iattr) -> c_int>,
    /// Getattr
    pub getattr: Option<unsafe extern "C" fn(*mut mnt_idmap, *const path, *mut kstat, u32, c_uint) -> c_int>,
    /// Listxattr
    pub listxattr: Option<unsafe extern "C" fn(*mut dentry, *mut c_char, usize) -> isize>,
    /// Fiemap
    pub fiemap: Option<unsafe extern "C" fn(*mut inode, *mut c_void, u64, u64) -> c_int>,
    /// Update time
    pub update_time: Option<unsafe extern "C" fn(*mut inode, c_int) -> c_int>,
    /// Atomic open
    pub atomic_open: Option<unsafe extern "C" fn(*mut inode, *mut dentry, *mut file, c_uint, umode_t) -> c_int>,
    /// Tmpfile
    pub tmpfile: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut inode, *mut file, umode_t) -> c_int>,
    /// Set ACL
    pub set_acl: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut dentry, *mut c_void, c_int) -> c_int>,
    /// Fileattr get
    pub fileattr_get: Option<unsafe extern "C" fn(*mut dentry, *mut c_void) -> c_int>,
    /// Fileattr set
    pub fileattr_set: Option<unsafe extern "C" fn(*mut mnt_idmap, *mut dentry, *mut c_void) -> c_int>,
    /// Get offset ctx
    pub get_offset_ctx: Option<unsafe extern "C" fn(*mut inode) -> *mut c_void>,
}

/// Inode attributes for setattr
#[repr(C)]
#[derive(Debug, Default)]
pub struct iattr {
    pub ia_valid: c_uint,
    pub ia_mode: umode_t,
    pub ia_uid: kuid_t,
    pub ia_gid: kgid_t,
    pub ia_size: loff_t,
    pub ia_atime: timespec64,
    pub ia_mtime: timespec64,
    pub ia_ctime: timespec64,
    pub ia_file: *mut file,
}

/// File statistics
#[repr(C)]
#[derive(Debug, Default)]
pub struct kstat {
    pub result_mask: u32,
    pub mode: umode_t,
    pub nlink: c_uint,
    pub blksize: u32,
    pub attributes: u64,
    pub attributes_mask: u64,
    pub ino: u64,
    pub dev: dev_t,
    pub rdev: dev_t,
    pub uid: kuid_t,
    pub gid: kgid_t,
    pub size: loff_t,
    pub atime: timespec64,
    pub mtime: timespec64,
    pub ctime: timespec64,
    pub btime: timespec64,
    pub blocks: u64,
    pub mnt_id: u64,
    pub dio_mem_align: u32,
    pub dio_offset_align: u32,
    pub change_cookie: u64,
    pub subvol: u64,
}

// ============================================================================
// File operations
// ============================================================================

/// File operations
#[repr(C)]
#[derive(Default)]
pub struct file_operations {
    /// Module owner
    pub owner: *mut c_void,
    /// Seek
    pub llseek: Option<unsafe extern "C" fn(*mut file, loff_t, c_int) -> loff_t>,
    /// Read
    pub read: Option<unsafe extern "C" fn(*mut file, *mut c_char, usize, *mut loff_t) -> isize>,
    /// Write
    pub write: Option<unsafe extern "C" fn(*mut file, *const c_char, usize, *mut loff_t) -> isize>,
    /// Read iter
    pub read_iter: Option<unsafe extern "C" fn(*mut kiocb, *mut iov_iter) -> isize>,
    /// Write iter
    pub write_iter: Option<unsafe extern "C" fn(*mut kiocb, *mut iov_iter) -> isize>,
    /// iopoll
    pub iopoll: Option<unsafe extern "C" fn(*mut kiocb, *mut c_void, c_uint) -> c_int>,
    /// Iterate shared
    pub iterate_shared: Option<unsafe extern "C" fn(*mut file, *mut dir_context) -> c_int>,
    /// Poll
    pub poll: Option<unsafe extern "C" fn(*mut file, *mut c_void) -> c_uint>,
    /// Unlocked ioctl
    pub unlocked_ioctl: Option<unsafe extern "C" fn(*mut file, c_uint, c_ulong) -> c_long>,
    /// Compat ioctl
    pub compat_ioctl: Option<unsafe extern "C" fn(*mut file, c_uint, c_ulong) -> c_long>,
    /// Mmap
    pub mmap: Option<unsafe extern "C" fn(*mut file, *mut c_void) -> c_int>,
    /// Open
    pub open: Option<unsafe extern "C" fn(*mut inode, *mut file) -> c_int>,
    /// Flush
    pub flush: Option<unsafe extern "C" fn(*mut file, *mut c_void) -> c_int>,
    /// Release
    pub release: Option<unsafe extern "C" fn(*mut inode, *mut file) -> c_int>,
    /// Fsync
    pub fsync: Option<unsafe extern "C" fn(*mut file, loff_t, loff_t, c_int) -> c_int>,
    /// Fasync
    pub fasync: Option<unsafe extern "C" fn(c_int, *mut file, c_int) -> c_int>,
    /// Lock
    pub lock: Option<unsafe extern "C" fn(*mut file, c_int, *mut c_void) -> c_int>,
    /// Get unmapped area
    pub get_unmapped_area: Option<unsafe extern "C" fn(*mut file, c_ulong, c_ulong, c_ulong, c_ulong) -> c_ulong>,
    /// Check flags
    pub check_flags: Option<unsafe extern "C" fn(c_int) -> c_int>,
    /// Flock
    pub flock: Option<unsafe extern "C" fn(*mut file, c_int, *mut c_void) -> c_int>,
    /// Splice write
    pub splice_write: Option<unsafe extern "C" fn(*mut c_void, *mut file, *mut loff_t, usize, c_uint) -> isize>,
    /// Splice read
    pub splice_read: Option<unsafe extern "C" fn(*mut file, *mut loff_t, *mut c_void, usize, c_uint) -> isize>,
    /// Splice eof
    pub splice_eof: Option<unsafe extern "C" fn(*mut file)>,
    /// Setlease
    pub setlease: Option<unsafe extern "C" fn(*mut file, c_int, *mut *mut c_void, *mut *mut c_void) -> c_int>,
    /// Fallocate
    pub fallocate: Option<unsafe extern "C" fn(*mut file, c_int, loff_t, loff_t) -> c_long>,
    /// Show fdinfo
    pub show_fdinfo: Option<unsafe extern "C" fn(*mut c_void, *mut file)>,
    /// Copy file range
    pub copy_file_range: Option<unsafe extern "C" fn(*mut file, loff_t, *mut file, loff_t, usize, c_uint) -> isize>,
    /// Remap file range
    pub remap_file_range: Option<unsafe extern "C" fn(*mut file, loff_t, *mut file, loff_t, loff_t, c_uint) -> loff_t>,
    /// Fadvise
    pub fadvise: Option<unsafe extern "C" fn(*mut file, loff_t, loff_t, c_int) -> c_int>,
    /// Uring cmd
    pub uring_cmd: Option<unsafe extern "C" fn(*mut c_void, c_uint) -> c_int>,
    /// Uring cmd iopoll
    pub uring_cmd_iopoll: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, c_uint) -> c_int>,
}

// ============================================================================
// Dentry operations
// ============================================================================

/// Dentry operations
#[repr(C)]
#[derive(Default)]
pub struct dentry_operations {
    /// Revalidate dentry
    pub d_revalidate: Option<unsafe extern "C" fn(*mut dentry, c_uint) -> c_int>,
    /// Weak revalidate
    pub d_weak_revalidate: Option<unsafe extern "C" fn(*mut dentry, c_uint) -> c_int>,
    /// Hash
    pub d_hash: Option<unsafe extern "C" fn(*const dentry, *mut qstr) -> c_int>,
    /// Compare
    pub d_compare: Option<unsafe extern "C" fn(*const dentry, c_uint, *const c_char, *const qstr) -> c_int>,
    /// Delete
    pub d_delete: Option<unsafe extern "C" fn(*const dentry) -> c_int>,
    /// Init
    pub d_init: Option<unsafe extern "C" fn(*mut dentry) -> c_int>,
    /// Release
    pub d_release: Option<unsafe extern "C" fn(*mut dentry)>,
    /// Prune
    pub d_prune: Option<unsafe extern "C" fn(*mut dentry)>,
    /// Iput
    pub d_iput: Option<unsafe extern "C" fn(*mut dentry, *mut inode)>,
    /// Dname
    pub d_dname: Option<unsafe extern "C" fn(*mut dentry, *mut c_char, c_int) -> *mut c_char>,
    /// Automount
    pub d_automount: Option<unsafe extern "C" fn(*mut path) -> *mut c_void>,
    /// Manage
    pub d_manage: Option<unsafe extern "C" fn(*const path, bool) -> c_int>,
    /// Real
    pub d_real: Option<unsafe extern "C" fn(*mut dentry, *mut inode) -> *mut dentry>,
}

// ============================================================================
// Address space operations
// ============================================================================

/// Address space operations (for page cache)
#[repr(C)]
#[derive(Default)]
pub struct address_space_operations {
    /// Write a page
    pub writepage: Option<unsafe extern "C" fn(*mut c_void, *mut writeback_control) -> c_int>,
    /// Read a folio
    pub read_folio: Option<unsafe extern "C" fn(*mut file, *mut c_void) -> c_int>,
    /// Write pages
    pub writepages: Option<unsafe extern "C" fn(*mut address_space, *mut writeback_control) -> c_int>,
    /// Dirty folio
    pub dirty_folio: Option<unsafe extern "C" fn(*mut address_space, *mut c_void) -> bool>,
    /// Readahead
    pub readahead: Option<unsafe extern "C" fn(*mut c_void)>,
    /// Write begin
    pub write_begin: Option<unsafe extern "C" fn(*mut file, *mut address_space, loff_t, c_uint, *mut *mut c_void, *mut *mut c_void) -> c_int>,
    /// Write end
    pub write_end: Option<unsafe extern "C" fn(*mut file, *mut address_space, loff_t, c_uint, c_uint, *mut c_void, *mut c_void) -> c_int>,
    /// Bmap
    pub bmap: Option<unsafe extern "C" fn(*mut address_space, sector_t) -> sector_t>,
    /// Invalidate folio
    pub invalidate_folio: Option<unsafe extern "C" fn(*mut c_void, usize, usize)>,
    /// Release folio
    pub release_folio: Option<unsafe extern "C" fn(*mut c_void, gfp_t) -> bool>,
    /// Free folio
    pub free_folio: Option<unsafe extern "C" fn(*mut c_void)>,
    /// Direct IO
    pub direct_IO: Option<unsafe extern "C" fn(*mut kiocb, *mut iov_iter) -> isize>,
    /// Migrate folio
    pub migrate_folio: Option<unsafe extern "C" fn(*mut address_space, *mut c_void, *mut c_void, c_int) -> c_int>,
    /// Launder folio
    pub launder_folio: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    /// Is partially uptodate
    pub is_partially_uptodate: Option<unsafe extern "C" fn(*mut c_void, usize, usize) -> bool>,
    /// Is dirty writeback
    pub is_dirty_writeback: Option<unsafe extern "C" fn(*mut c_void, *mut bool, *mut bool)>,
    /// Error remove folio
    pub error_remove_folio: Option<unsafe extern "C" fn(*mut address_space, *mut c_void) -> c_int>,
    /// Swap activate
    pub swap_activate: Option<unsafe extern "C" fn(*mut c_void, *mut file, *mut sector_t) -> c_int>,
    /// Swap deactivate
    pub swap_deactivate: Option<unsafe extern "C" fn(*mut file)>,
    /// Swap rw
    pub swap_rw: Option<unsafe extern "C" fn(*mut kiocb, *mut iov_iter) -> c_int>,
}

// ============================================================================
// Export operations (for NFS)
// ============================================================================

/// Export operations
#[repr(C)]
#[derive(Default)]
pub struct export_operations {
    /// Encode file handle
    pub encode_fh: Option<unsafe extern "C" fn(*mut inode, *mut u32, *mut c_int, *mut inode) -> c_int>,
    /// File handle to dentry
    pub fh_to_dentry: Option<unsafe extern "C" fn(*mut super_block, *mut c_void, c_int, c_int) -> *mut dentry>,
    /// File handle to parent
    pub fh_to_parent: Option<unsafe extern "C" fn(*mut super_block, *mut c_void, c_int, c_int) -> *mut dentry>,
    /// Get parent
    pub get_parent: Option<unsafe extern "C" fn(*mut dentry) -> *mut dentry>,
    /// Get name
    pub get_name: Option<unsafe extern "C" fn(*mut dentry, *mut c_char, *mut dentry) -> c_int>,
    /// Commit metadata
    pub commit_metadata: Option<unsafe extern "C" fn(*mut inode) -> c_int>,
    /// Flags
    pub flags: c_ulong,
}
