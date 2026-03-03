//! Mount operations
//!
//! Handles mounting and unmounting filesystems.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use core::ptr;

use crate::types::*;
use crate::dentry::dentry;
use crate::superblock::*;
use crate::operations::super_operations;

// ============================================================================
// Mount structure
// ============================================================================

/// VFS mount structure
#[repr(C)]
#[derive(Default)]
pub struct vfsmount_full {
    /// Mount hash
    pub mnt_hash: hlist_node,
    /// Parent mount
    pub mnt_parent: *mut vfsmount_full,
    /// Dentry of mount point
    pub mnt_mountpoint: *mut dentry,
    /// Root dentry of this mount
    pub mnt_root: *mut dentry,
    /// Superblock
    pub mnt_sb: *mut super_block,
    /// Mount flags
    pub mnt_flags: c_int,
    /// Mount ID
    pub mnt_id: c_int,
    /// Group ID
    pub mnt_group_id: c_int,
    /// Expiry mark
    pub mnt_expiry_mark: c_int,
    /// Pin count
    pub mnt_pinned: c_int,
    /// Ghost count
    pub mnt_ghosts: c_int,
    /// ID map
    pub mnt_idmap: *mut mnt_idmap,
}


// ============================================================================
// Mount flags
// ============================================================================

pub const MNT_NOSUID: c_int = 0x01;
pub const MNT_NODEV: c_int = 0x02;
pub const MNT_NOEXEC: c_int = 0x04;
pub const MNT_NOATIME: c_int = 0x08;
pub const MNT_NODIRATIME: c_int = 0x10;
pub const MNT_RELATIME: c_int = 0x20;
pub const MNT_READONLY: c_int = 0x40;
pub const MNT_SHRINKABLE: c_int = 0x100;
pub const MNT_WRITE_HOLD: c_int = 0x200;
pub const MNT_SHARED: c_int = 0x1000;
pub const MNT_UNBINDABLE: c_int = 0x2000;

// ============================================================================
// Mount operations
// ============================================================================

/// Mount a block device filesystem
pub fn mount_bdev_fn(
    fs_type: *mut file_system_type,
    flags: c_int,
    dev_name: *const c_char,
    data: *mut c_void,
    fill_super: Option<unsafe extern "C" fn(*mut super_block, *mut c_void, c_int) -> c_int>,
) -> *mut dentry {
    if fs_type.is_null() {
        return ERR_PTR(-22); // -EINVAL
    }

    // Allocate superblock
    let sb = alloc_super(fs_type, flags);
    if sb.is_null() {
        return ERR_PTR(-12); // -ENOMEM
    }

    // Call fill_super
    if let Some(fill) = fill_super {
        let ret = unsafe { fill(sb, data, flags & SB_SILENT as c_int) };
        if ret != 0 {
            // Would deactivate_locked_super here
            return ERR_PTR(ret as c_long);
        }
    }

    // Return root dentry
    unsafe { (*sb).s_root }
}

/// Mount a pseudo filesystem
pub fn mount_pseudo(
    fs_type: *mut file_system_type,
    name: *const c_char,
    ops: *const super_operations,
    dops: *const crate::operations::dentry_operations,
    magic: c_ulong,
) -> *mut dentry {
    let sb = alloc_super(fs_type, SB_KERNMOUNT as c_int);
    if sb.is_null() {
        return ERR_PTR(-12);
    }

    unsafe {
        (*sb).s_op = ops;
        (*sb).s_magic = magic;
        (*sb).s_d_op = dops;
        (*sb).s_blocksize = 4096;
        (*sb).s_blocksize_bits = 12;
    }

    // Create root inode and dentry
    let root_inode = crate::export::new_inode(sb);
    if root_inode.is_null() {
        return ERR_PTR(-12);
    }

    unsafe {
        (*root_inode).i_mode = S_IFDIR | 0o755;
        (*root_inode).i_uid = 0;
        (*root_inode).i_gid = 0;
    }

    let root_dentry = crate::dcache::d_make_root_fn(root_inode);
    if root_dentry.is_null() {
        return ERR_PTR(-12);
    }

    unsafe {
        (*sb).s_root = root_dentry;
    }

    root_dentry
}

/// Kill a block device superblock
pub fn kill_block_super_fn(sb: *mut super_block) {
    if sb.is_null() {
        return;
    }

    // Sync filesystem
    unsafe {
        if let Some(sync_fn) = (*sb).s_op.as_ref().and_then(|op| (*op).sync_fs) {
            sync_fn(sb, 1);
        }
    }

    // Would invalidate inodes, release block device, etc.
    generic_shutdown_super(sb);
}

/// Kill a litter superblock (no device)
pub fn kill_litter_super(sb: *mut super_block) {
    generic_shutdown_super(sb);
}

/// Generic superblock shutdown
pub fn generic_shutdown_super(sb: *mut super_block) {
    if sb.is_null() {
        return;
    }

    unsafe {
        // Evict all inodes
        // invalidate_inodes(sb, 0);

        // Call put_super
        if let Some(put_super) = (*sb).s_op.as_ref().and_then(|op| (*op).put_super) {
            put_super(sb);
        }
    }
}

// ============================================================================
// Superblock allocation
// ============================================================================

/// Allocate a new superblock
fn alloc_super(fs_type: *mut file_system_type, flags: c_int) -> *mut super_block {
    use alloc::alloc::{alloc_zeroed, Layout};

    let layout = match Layout::from_size_align(core::mem::size_of::<super_block>(), 8) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };

    let ptr = unsafe { alloc_zeroed(layout) as *mut super_block };
    if ptr.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        // Initialize superblock
        (*ptr).s_type = fs_type;
        (*ptr).s_flags = flags as c_ulong;
        (*ptr).s_count = 1;
        (*ptr).s_active = atomic_t::new(1);
        (*ptr).s_blocksize = 4096;
        (*ptr).s_blocksize_bits = 12;
        (*ptr).s_maxbytes = i64::MAX;
        (*ptr).s_time_gran = 1000000000;
        (*ptr).s_time_min = i64::MIN;
        (*ptr).s_time_max = i64::MAX;

        (*ptr).s_list.init();
        (*ptr).s_inodes.init();
        (*ptr).s_dirty.init();
        (*ptr).s_io.init();
        (*ptr).s_more_io.init();
    }

    ptr
}

/// Get a superblock
pub fn sget(
    fs_type: *mut file_system_type,
    test: Option<unsafe extern "C" fn(*mut super_block, *mut c_void) -> c_int>,
    set: Option<unsafe extern "C" fn(*mut super_block, *mut c_void) -> c_int>,
    flags: c_int,
    data: *mut c_void,
) -> *mut super_block {
    // Simplified: always create new superblock
    let sb = alloc_super(fs_type, flags);
    if sb.is_null() {
        return ERR_PTR(-12);
    }

    if let Some(set_fn) = set {
        let err = unsafe { set_fn(sb, data) };
        if err != 0 {
            // Would free superblock
            return ERR_PTR(err as c_long);
        }
    }

    sb
}
