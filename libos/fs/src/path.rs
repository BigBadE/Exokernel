//! Path lookup and resolution

use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use libos_core::{Result, Error};

use crate::dentry::Dentry;
use crate::inode::Inode;
use crate::mount::Mount;

/// Path lookup flags
pub mod lookup_flags {
    pub const LOOKUP_FOLLOW: u32 = 1 << 0;      // Follow symlinks
    pub const LOOKUP_DIRECTORY: u32 = 1 << 1;   // Expect directory
    pub const LOOKUP_PARENT: u32 = 1 << 2;      // Lookup parent
    pub const LOOKUP_CREATE: u32 = 1 << 3;      // Creating file
    pub const LOOKUP_EXCL: u32 = 1 << 4;        // Exclusive create
}

/// Path structure for lookup results
#[derive(Clone)]
pub struct Path {
    /// Mount
    pub mnt: Option<Arc<Mount>>,
    /// Dentry
    pub dentry: Arc<Dentry>,
}

impl Path {
    /// Create a new path
    pub fn new(dentry: Arc<Dentry>, mnt: Option<Arc<Mount>>) -> Self {
        Self { mnt, dentry }
    }

    /// Get the inode
    pub fn inode(&self) -> Option<Arc<Inode>> {
        self.dentry.inode()
    }

    /// Check if valid (has inode)
    pub fn is_valid(&self) -> bool {
        self.dentry.is_positive()
    }

    /// Get the dentry
    pub fn dentry(&self) -> &Arc<Dentry> {
        &self.dentry
    }

    /// Get the mount
    pub fn mount(&self) -> Option<&Arc<Mount>> {
        self.mnt.as_ref()
    }
}

/// Nameidata - used during path walk
pub struct Nameidata {
    /// Current path
    pub path: Option<Path>,
    /// Last component name
    pub last: String,
    /// Lookup flags
    pub flags: u32,
    /// Depth (for symlink following)
    pub depth: u32,
}

impl Nameidata {
    pub fn new(flags: u32) -> Self {
        Self {
            path: None,
            last: String::new(),
            flags,
            depth: 0,
        }
    }
}

impl Default for Nameidata {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Parse a path into components
fn parse_path(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .collect()
}

/// Look up a path starting from a root dentry
pub fn lookup_path(
    path: &str,
    flags: u32,
    root: &Arc<Dentry>,
    cwd: &Arc<Dentry>,
) -> Result<Path> {
    if path.is_empty() {
        return Err(Error::NotFound);
    }

    // Determine starting point
    let start = if path.starts_with('/') {
        Arc::clone(root)
    } else {
        Arc::clone(cwd)
    };

    let components = parse_path(path);
    let mut current = start;

    for component in &components {
        // Handle . and ..
        if *component == "." {
            continue;
        } else if *component == ".." {
            if let Some(parent) = current.parent() {
                current = parent;
            }
            continue;
        }

        // Get the inode
        let inode = current.inode().ok_or(Error::NotFound)?;

        // Check if it's a directory
        if !inode.is_dir() {
            return Err(Error::NotDirectory);
        }

        // Try to find in children first (dcache)
        if let Some(child) = current.find_child(component) {
            current = child;
            continue;
        }

        // Need to call filesystem lookup
        let ops = inode.inode_ops().ok_or(Error::NotSupported)?;
        let child = ops.lookup(&inode, component)?;
        current.add_child(Arc::clone(&child));
        current = child;
    }

    // Check for directory requirement
    if (flags & lookup_flags::LOOKUP_DIRECTORY) != 0 {
        if let Some(inode) = current.inode() {
            if !inode.is_dir() {
                return Err(Error::NotDirectory);
            }
        }
    }

    Ok(Path::new(current, None))
}

/// Get the parent path and last component name
pub fn lookup_parent(
    path: &str,
    root: &Arc<Dentry>,
    cwd: &Arc<Dentry>,
) -> Result<(Path, String)> {
    if path.is_empty() {
        return Err(Error::NotFound);
    }

    let components = parse_path(path);
    if components.is_empty() {
        return Err(Error::NotFound);
    }

    // The last component is the name we want
    let last = components.last().unwrap().to_string();

    // Build parent path
    let parent_path = if components.len() == 1 {
        // Parent is either root or cwd
        if path.starts_with('/') {
            "/".to_string()
        } else {
            ".".to_string()
        }
    } else {
        let parent_components: Vec<_> = components[..components.len() - 1].to_vec();
        if path.starts_with('/') {
            format!("/{}", parent_components.join("/"))
        } else {
            parent_components.join("/")
        }
    };

    let parent = lookup_path(&parent_path, lookup_flags::LOOKUP_DIRECTORY, root, cwd)?;
    Ok((parent, last))
}
