//! Dentry cache implementation
//!
//! The dentry cache (dcache) speeds up path lookups by caching
//! directory entries.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::ffi::{c_char, c_int, c_uint, c_ulong};
use core::ptr;

use crate::types::*;
use crate::dentry::*;
use crate::inode::inode;
use crate::superblock::super_block;

// ============================================================================
// Dentry cache
// ============================================================================

/// Global dentry cache
static mut DCACHE: Option<DentryCache> = None;

struct DentryCache {
    /// Hash table for fast lookup
    entries: BTreeMap<u64, *mut dentry>,
    /// Next inode number
    next_ino: u64,
}

impl DentryCache {
    fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            next_ino: 1,
        }
    }
}

fn get_dcache() -> &'static mut DentryCache {
    unsafe {
        let cache_ptr = &raw mut DCACHE;
        if (*cache_ptr).is_none() {
            *cache_ptr = Some(DentryCache::new());
        }
        (*cache_ptr).as_mut().unwrap()
    }
}

// ============================================================================
// Dentry allocation
// ============================================================================

/// Allocate a new dentry
pub fn d_alloc(parent: *mut dentry, name: *const qstr) -> *mut dentry {
    use alloc::alloc::{alloc_zeroed, Layout};

    let layout = match Layout::from_size_align(core::mem::size_of::<dentry>(), 8) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };

    let ptr = unsafe { alloc_zeroed(layout) as *mut dentry };
    if ptr.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        // Initialize the dentry
        (*ptr).d_lockref = lockref { lock: 0, count: 1 };
        (*ptr).d_parent = if parent.is_null() { ptr } else { parent };
        (*ptr).d_lru.init();
        (*ptr).d_child.init();
        (*ptr).d_subdirs.init();

        // Copy name if provided
        if !name.is_null() {
            (*ptr).d_name = *name;
            let name_len = (*name).len as usize;
            if name_len < DNAME_INLINE_LEN && !(*name).name.is_null() {
                ptr::copy_nonoverlapping(
                    (*name).name,
                    (*ptr).d_iname.as_mut_ptr(),
                    name_len,
                );
                (*ptr).d_iname[name_len] = 0;
                (*ptr).d_name.name = (*ptr).d_iname.as_ptr();
            }
        }

        // Get superblock from parent
        if !parent.is_null() {
            (*ptr).d_sb = (*parent).d_sb;
        }
    }

    ptr
}

/// Allocate anonymous dentry
pub fn d_alloc_anon(sb: *mut super_block) -> *mut dentry {
    let d = d_alloc(ptr::null_mut(), ptr::null());
    if !d.is_null() && !sb.is_null() {
        unsafe {
            (*d).d_sb = sb;
        }
    }
    d
}

/// Allocate a root dentry
pub fn d_make_root_fn(root_inode: *mut inode) -> *mut dentry {
    if root_inode.is_null() {
        return ptr::null_mut();
    }

    let sb = unsafe { (*root_inode).i_sb };
    let d = d_alloc_anon(sb);

    if !d.is_null() {
        unsafe {
            (*d).d_inode = root_inode;
            (*d).d_flags |= DCACHE_DIRECTORY_TYPE;
        }
    } else {
        // Would call iput here
    }

    d
}

// ============================================================================
// Dentry operations
// ============================================================================

/// Get a reference to a dentry
pub fn dget(dentry: *mut dentry) -> *mut dentry {
    if dentry.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        (*dentry).d_lockref.count += 1;
    }
    dentry
}

/// Release a dentry reference
pub fn dput_fn(dentry: *mut dentry) {
    if dentry.is_null() {
        return;
    }
    unsafe {
        if (*dentry).d_lockref.count > 0 {
            (*dentry).d_lockref.count -= 1;
        }
        // Would free dentry when count reaches 0
    }
}

/// Instantiate a dentry with an inode
pub fn d_instantiate_fn(dentry: *mut dentry, inode: *mut inode) {
    if dentry.is_null() {
        return;
    }
    unsafe {
        (*dentry).d_inode = inode;

        // Set dentry type based on inode mode
        if !inode.is_null() {
            let mode = (*inode).i_mode;
            (*dentry).d_flags &= !DCACHE_ENTRY_TYPE;
            if S_ISDIR(mode) {
                (*dentry).d_flags |= DCACHE_DIRECTORY_TYPE;
            } else if S_ISLNK(mode) {
                (*dentry).d_flags |= DCACHE_SYMLINK_TYPE;
            } else if S_ISREG(mode) {
                (*dentry).d_flags |= DCACHE_REGULAR_TYPE;
            } else {
                (*dentry).d_flags |= DCACHE_SPECIAL_TYPE;
            }
        }
    }
}

/// Add dentry to hash
pub fn d_add(dentry: *mut dentry, inode: *mut inode) {
    d_instantiate_fn(dentry, inode);
    // Would add to dcache hash table
}

/// Splice a disconnected dentry into the tree
pub fn d_splice_alias_fn(inode: *mut inode, dentry: *mut dentry) -> *mut dentry {
    if inode.is_null() {
        // Return negative dentry
        return dentry;
    }

    // For simplicity, just instantiate
    d_instantiate_fn(dentry, inode);
    dentry
}

/// Find an alias for an inode
pub fn d_find_alias_fn(inode: *mut inode) -> *mut dentry {
    // Simplified: would search d_alias list
    ptr::null_mut()
}

/// Move a dentry
pub fn d_move_fn(dentry: *mut dentry, target: *mut dentry) {
    if dentry.is_null() || target.is_null() {
        return;
    }
    unsafe {
        // Exchange names
        (*dentry).d_name = (*target).d_name;
        (*dentry).d_parent = (*target).d_parent;
    }
}

/// Obtain an alias dentry for an inode
pub fn d_obtain_alias_fn(inode: *mut inode) -> *mut dentry {
    if inode.is_null() {
        return ERR_PTR(-22); // -EINVAL
    }

    // Check for existing alias
    let existing = d_find_alias_fn(inode);
    if !existing.is_null() {
        return existing;
    }

    // Create new anonymous dentry
    let sb = unsafe { (*inode).i_sb };
    let dentry = d_alloc_anon(sb);
    if dentry.is_null() {
        return ERR_PTR(-12); // -ENOMEM
    }

    d_instantiate_fn(dentry, inode);
    dentry
}

// ============================================================================
// Dentry lookup helpers
// ============================================================================

/// Check if dentry really is positive
pub fn d_really_is_positive_fn(dentry: *const dentry) -> bool {
    d_is_positive(dentry)
}

/// Check if dentry really is negative
pub fn d_really_is_negative(dentry: *const dentry) -> bool {
    d_is_negative(dentry)
}

/// Get inode from dentry (C export)
pub fn d_inode_export(dentry: *const dentry) -> *mut inode {
    d_inode_fn(dentry)
}
