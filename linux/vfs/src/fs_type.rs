//! Filesystem type registration
//!
//! Handles registration and lookup of filesystem types.

use alloc::vec::Vec;
use core::ffi::{c_char, c_int};
use core::ptr;

use crate::superblock::file_system_type;

// ============================================================================
// Filesystem registry
// ============================================================================

/// Global list of registered filesystems
static mut FILESYSTEMS: Option<FilesystemRegistry> = None;

struct FilesystemRegistry {
    /// Head of filesystem type list
    filesystems: *mut file_system_type,
}

impl FilesystemRegistry {
    fn new() -> Self {
        Self {
            filesystems: ptr::null_mut(),
        }
    }
}

fn get_registry() -> &'static mut FilesystemRegistry {
    unsafe {
        let reg_ptr = &raw mut FILESYSTEMS;
        if (*reg_ptr).is_none() {
            *reg_ptr = Some(FilesystemRegistry::new());
        }
        (*reg_ptr).as_mut().unwrap()
    }
}

// ============================================================================
// Registration functions
// ============================================================================

/// Register a filesystem type
pub fn register_filesystem_fn(fs: *mut file_system_type) -> c_int {
    if fs.is_null() {
        return -22; // -EINVAL
    }

    let registry = get_registry();

    unsafe {
        // Check if already registered
        let mut p = registry.filesystems;
        while !p.is_null() {
            if p == fs {
                return -16; // -EBUSY
            }
            // Check for duplicate name
            if !(*p).name.is_null() && !(*fs).name.is_null() {
                if cmp_fs_names((*p).name, (*fs).name) {
                    return -17; // -EEXIST
                }
            }
            p = (*p).next;
        }

        // Add to list
        (*fs).next = registry.filesystems;
        registry.filesystems = fs;
    }

    0
}

/// Unregister a filesystem type
pub fn unregister_filesystem_fn(fs: *mut file_system_type) -> c_int {
    if fs.is_null() {
        return -22; // -EINVAL
    }

    let registry = get_registry();

    unsafe {
        let mut pp = &mut registry.filesystems as *mut *mut file_system_type;

        while !(*pp).is_null() {
            if *pp == fs {
                *pp = (*fs).next;
                (*fs).next = ptr::null_mut();
                return 0;
            }
            pp = &mut (**pp).next as *mut *mut file_system_type;
        }
    }

    -2 // -ENOENT
}

/// Get a filesystem type by name
pub fn get_fs_type_fn(name: *const c_char) -> *mut file_system_type {
    if name.is_null() {
        return ptr::null_mut();
    }

    let registry = get_registry();

    unsafe {
        let mut p = registry.filesystems;
        while !p.is_null() {
            if !(*p).name.is_null() && cmp_fs_names((*p).name, name) {
                return p;
            }
            p = (*p).next;
        }
    }

    ptr::null_mut()
}

/// List all registered filesystems
pub fn list_registered_filesystems() -> Vec<&'static str> {
    let mut result = Vec::new();
    let registry = get_registry();

    unsafe {
        let mut p = registry.filesystems;
        while !p.is_null() {
            if !(*p).name.is_null() {
                // Convert C string to Rust str
                let mut len = 0;
                let mut q = (*p).name;
                while *q != 0 {
                    len += 1;
                    q = q.add(1);
                }
                if let Ok(s) = core::str::from_utf8(core::slice::from_raw_parts(
                    (*p).name as *const u8,
                    len,
                )) {
                    result.push(s);
                }
            }
            p = (*p).next;
        }
    }

    result
}

// ============================================================================
// Helper functions
// ============================================================================

/// Compare two filesystem names
unsafe fn cmp_fs_names(a: *const c_char, b: *const c_char) -> bool {
    let mut pa = a;
    let mut pb = b;

    while *pa != 0 && *pb != 0 {
        if *pa != *pb {
            return false;
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }

    *pa == 0 && *pb == 0
}
