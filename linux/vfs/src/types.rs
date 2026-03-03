//! Core VFS types
//!
//! Re-exports common types from linux-core and adds VFS-specific types.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use core::ptr;

// Re-export all core types
pub use linux_core::{
    size_t, ssize_t, loff_t, sector_t, blkcnt_t, time64_t, ktime_t,
    umode_t, kuid_t, kgid_t, dev_t, fmode_t, gfp_t, ino_t,
    atomic_t, atomic_long_t, atomic64_t,
    timespec64, list_head, hlist_head, hlist_node, hlist_bl_head, hlist_bl_node,
    rcu_head, lockref, seqlock_t, qstr, wait_queue_head_t, completion,
    S_IFMT, S_IFSOCK, S_IFLNK, S_IFREG, S_IFBLK, S_IFDIR, S_IFCHR, S_IFIFO,
    S_ISUID, S_ISGID, S_ISVTX,
    S_IRWXU, S_IRUSR, S_IWUSR, S_IXUSR,
    S_IRWXG, S_IRGRP, S_IWGRP, S_IXGRP,
    S_IRWXO, S_IROTH, S_IWOTH, S_IXOTH,
    S_ISREG, S_ISDIR, S_ISCHR, S_ISBLK, S_ISFIFO, S_ISLNK, S_ISSOCK,
    IS_ERR, IS_ERR_OR_NULL, PTR_ERR, ERR_PTR, ERR_CAST,
    GFP_KERNEL, GFP_ATOMIC, GFP_ZERO,
};

// Sync types as simple values for struct layout
pub type spinlock_t = u32;
pub type rwlock_t = u32;

// ============================================================================
// VFS-specific types
// ============================================================================

/// RW semaphore (VFS version with correct layout)
#[repr(C)]
#[derive(Debug, Default)]
pub struct rw_semaphore {
    pub count: atomic_long_t,
    pub wait_lock: spinlock_t,
    pub wait_list: list_head,
}

/// Mutex (VFS version with correct layout)
#[repr(C)]
#[derive(Debug)]
pub struct mutex {
    pub owner: atomic_long_t,
    pub wait_lock: spinlock_t,
    pub wait_list: list_head,
}

impl Default for mutex {
    fn default() -> Self {
        Self {
            owner: atomic_long_t::new(0),
            wait_lock: 0,
            wait_list: list_head::new(),
        }
    }
}

/// Path structure
#[repr(C)]
#[derive(Debug)]
pub struct path {
    pub mnt: *mut vfsmount,
    pub dentry: *mut super::dentry::dentry,
}

impl Default for path {
    fn default() -> Self {
        Self {
            mnt: ptr::null_mut(),
            dentry: ptr::null_mut(),
        }
    }
}

/// VFS mount (forward declaration)
#[repr(C)]
pub struct vfsmount {
    pub _opaque: [u8; 0],
}

/// Credentials
#[repr(C)]
#[derive(Debug, Default)]
pub struct cred {
    pub uid: kuid_t,
    pub gid: kgid_t,
    pub suid: kuid_t,
    pub sgid: kgid_t,
    pub euid: kuid_t,
    pub egid: kgid_t,
    pub fsuid: kuid_t,
    pub fsgid: kgid_t,
}

/// User namespace
#[repr(C)]
pub struct user_namespace {
    pub _opaque: [u8; 0],
}

/// Mount ID map
#[repr(C)]
pub struct mnt_idmap {
    pub _opaque: [u8; 0],
}
