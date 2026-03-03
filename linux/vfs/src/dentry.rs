//! Dentry (directory entry) implementation
//!
//! A dentry represents a name-to-inode mapping in the filesystem.
//! Dentries are cached for fast path lookup.

use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use core::ptr;

use crate::types::*;
use crate::inode::inode;
use crate::superblock::super_block;
use crate::operations::dentry_operations;

// ============================================================================
// Dentry structure
// ============================================================================

/// Dentry - represents a directory entry (name -> inode mapping)
#[repr(C)]
pub struct dentry {
    /// Dentry flags
    pub d_flags: c_uint,
    /// Per-dentry seqlock
    pub d_seq: seqlock_t,
    /// Hash list
    pub d_hash: hlist_bl_node,
    /// Parent dentry
    pub d_parent: *mut dentry,
    /// Name of this entry
    pub d_name: qstr,
    /// Associated inode
    pub d_inode: *mut inode,
    /// Short name storage
    pub d_iname: [u8; DNAME_INLINE_LEN],
    /// Reference count and lock
    pub d_lockref: lockref,
    /// Dentry operations
    pub d_op: *const dentry_operations,
    /// Superblock
    pub d_sb: *mut super_block,
    /// LRU list
    pub d_lru: list_head,
    /// Child list
    pub d_child: list_head,
    /// Subdirectories
    pub d_subdirs: list_head,
    /// Alias list (multiple dentries for same inode)
    pub d_u: dentry_union,
    /// Time of last access
    pub d_time: c_ulong,
    /// Filesystem-specific data
    pub d_fsdata: *mut c_void,
}

/// Inline name length
pub const DNAME_INLINE_LEN: usize = 32;

/// Dentry union for different list usage
#[repr(C)]
#[derive(Clone, Copy)]
pub union dentry_union {
    /// Alias list head
    pub d_alias: hlist_node,
    /// LRU list head
    pub d_in_lookup_hash: hlist_bl_node,
    /// RCU head for deferred freeing
    pub d_rcu: rcu_head,
}

impl Default for dentry_union {
    fn default() -> Self {
        Self {
            d_alias: hlist_node::default(),
        }
    }
}

impl dentry {
    /// Create a new empty dentry
    pub fn new() -> Self {
        Self {
            d_flags: 0,
            d_seq: seqlock_t::default(),
            d_hash: hlist_bl_node::default(),
            d_parent: ptr::null_mut(),
            d_name: qstr::default(),
            d_inode: ptr::null_mut(),
            d_iname: [0; DNAME_INLINE_LEN],
            d_lockref: lockref::default(),
            d_op: ptr::null(),
            d_sb: ptr::null_mut(),
            d_lru: list_head::new(),
            d_child: list_head::new(),
            d_subdirs: list_head::new(),
            d_u: dentry_union::default(),
            d_time: 0,
            d_fsdata: ptr::null_mut(),
        }
    }
}

impl Default for dentry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Dentry flags
// ============================================================================

/// Dentry is currently being looked up
pub const DCACHE_OP_HASH: c_uint = 0x0001;
/// Dentry has custom compare
pub const DCACHE_OP_COMPARE: c_uint = 0x0002;
/// Dentry has revalidate
pub const DCACHE_OP_REVALIDATE: c_uint = 0x0004;
/// Dentry has delete
pub const DCACHE_OP_DELETE: c_uint = 0x0008;
/// Dentry has prune
pub const DCACHE_OP_PRUNE: c_uint = 0x0010;

/// Dentry is negative (no inode)
pub const DCACHE_NEGATIVE: c_uint = 0x0020;
/// Dentry is disconnected
pub const DCACHE_DISCONNECTED: c_uint = 0x0040;
/// Dentry is referenced
pub const DCACHE_REFERENCED: c_uint = 0x0080;
/// Dentry cannot be mounted on
pub const DCACHE_MOUNTED: c_uint = 0x0100;
/// Dentry requires d_op->d_automount
pub const DCACHE_NEED_AUTOMOUNT: c_uint = 0x0200;
/// Dentry is being freed
pub const DCACHE_GENOCIDE: c_uint = 0x0400;
/// Dentry should shrink
pub const DCACHE_SHRINK_LIST: c_uint = 0x0800;
/// Dentry has weak reference
pub const DCACHE_OP_WEAK_REVALIDATE: c_uint = 0x1000;

/// Dentry cannot be unlocked
pub const DCACHE_NFSFS_RENAMED: c_uint = 0x2000;
/// Dentry is case-insensitive
pub const DCACHE_FALLTHRU: c_uint = 0x4000;
/// Dentry is encrypted
pub const DCACHE_ENCRYPTED_NAME: c_uint = 0x8000;

/// Root dentry
pub const DCACHE_ENTRY_TYPE: c_uint = 0x00070000;
pub const DCACHE_MISS_TYPE: c_uint = 0x00000000;
pub const DCACHE_WHITEOUT_TYPE: c_uint = 0x00010000;
pub const DCACHE_DIRECTORY_TYPE: c_uint = 0x00020000;
pub const DCACHE_AUTODIR_TYPE: c_uint = 0x00030000;
pub const DCACHE_REGULAR_TYPE: c_uint = 0x00040000;
pub const DCACHE_SPECIAL_TYPE: c_uint = 0x00050000;
pub const DCACHE_SYMLINK_TYPE: c_uint = 0x00060000;

// ============================================================================
// Dentry helper functions
// ============================================================================

/// Check if dentry is positive (has inode)
#[inline]
pub fn d_is_positive(dentry: *const dentry) -> bool {
    if dentry.is_null() {
        return false;
    }
    unsafe { !(*dentry).d_inode.is_null() }
}

/// Check if dentry is negative (no inode)
#[inline]
pub fn d_is_negative(dentry: *const dentry) -> bool {
    if dentry.is_null() {
        return true;
    }
    unsafe { (*dentry).d_inode.is_null() }
}

/// Check if dentry is a directory
#[inline]
pub fn d_is_dir(dentry: *const dentry) -> bool {
    if dentry.is_null() {
        return false;
    }
    unsafe { ((*dentry).d_flags & DCACHE_ENTRY_TYPE) == DCACHE_DIRECTORY_TYPE }
}

/// Check if dentry is a regular file
#[inline]
pub fn d_is_reg(dentry: *const dentry) -> bool {
    if dentry.is_null() {
        return false;
    }
    unsafe { ((*dentry).d_flags & DCACHE_ENTRY_TYPE) == DCACHE_REGULAR_TYPE }
}

/// Check if dentry is a symlink
#[inline]
pub fn d_is_symlink(dentry: *const dentry) -> bool {
    if dentry.is_null() {
        return false;
    }
    unsafe { ((*dentry).d_flags & DCACHE_ENTRY_TYPE) == DCACHE_SYMLINK_TYPE }
}

/// Get inode from dentry
#[inline]
pub fn d_inode_fn(dentry: *const dentry) -> *mut inode {
    if dentry.is_null() {
        return ptr::null_mut();
    }
    unsafe { (*dentry).d_inode }
}

/// Get inode from dentry (RCU-safe)
#[inline]
pub fn d_inode_rcu(dentry: *const dentry) -> *mut inode {
    // For now, same as d_inode_fn - would need atomic read in real impl
    d_inode_fn(dentry)
}

/// Get backing inode (for overlayfs)
#[inline]
pub fn d_backing_inode(dentry: *const dentry) -> *mut inode {
    // For now, same as d_inode_fn - overlayfs would override
    d_inode_fn(dentry)
}

/// Get parent dentry
#[inline]
pub fn dget_parent(dentry: *mut dentry) -> *mut dentry {
    if dentry.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        let parent = (*dentry).d_parent;
        // Would increment refcount here
        parent
    }
}
