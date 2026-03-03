//! Superblock - filesystem instance

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

use libos_core::{Size, Offset, DevId};
use libos_sync::{Spinlock, RwLock};

use crate::dentry::Dentry;
use crate::inode::Inode;
use crate::traits::{SuperblockOps, DentryOps};

/// Superblock flags
pub mod sb_flags {
    pub const SB_RDONLY: u32 = 1 << 0;
    pub const SB_NOSUID: u32 = 1 << 1;
    pub const SB_NODEV: u32 = 1 << 2;
    pub const SB_NOEXEC: u32 = 1 << 3;
    pub const SB_SYNCHRONOUS: u32 = 1 << 4;
    pub const SB_MANDLOCK: u32 = 1 << 5;
    pub const SB_DIRSYNC: u32 = 1 << 6;
    pub const SB_NOATIME: u32 = 1 << 7;
    pub const SB_NODIRATIME: u32 = 1 << 8;
    pub const SB_POSIXACL: u32 = 1 << 9;
}

/// Superblock data (mutable fields)
struct SuperblockData {
    /// Root dentry
    root: Option<Arc<Dentry>>,
    /// List of inodes
    inodes: Vec<Weak<Inode>>,
    /// List of dirty inodes
    dirty_inodes: Vec<Weak<Inode>>,
}

/// Superblock - represents a mounted filesystem
pub struct Superblock {
    /// Block size
    block_size: Size,
    /// Block size bits
    block_bits: u8,
    /// Maximum file size
    max_bytes: Offset,
    /// Mount flags
    flags: u32,
    /// Filesystem magic number
    magic: u32,
    /// Device ID (if block device)
    dev_id: Option<DevId>,
    /// Filesystem type name
    fs_name: String,
    /// Time granularity in nanoseconds
    time_gran: u32,
    /// Mutable data
    data: RwLock<SuperblockData>,
    /// Superblock operations
    s_op: Option<Box<dyn SuperblockOps>>,
    /// Default dentry operations
    d_op: Option<Box<dyn DentryOps>>,
    /// Private filesystem data
    private: Spinlock<Option<Box<dyn core::any::Any + Send + Sync>>>,
}

impl Superblock {
    /// Create a new superblock
    pub fn new(fs_name: &str) -> Self {
        Self {
            block_size: 4096,
            block_bits: 12,
            max_bytes: i64::MAX,
            flags: 0,
            magic: 0,
            dev_id: None,
            fs_name: String::from(fs_name),
            time_gran: 1,
            data: RwLock::new(SuperblockData {
                root: None,
                inodes: Vec::new(),
                dirty_inodes: Vec::new(),
            }),
            s_op: None,
            d_op: None,
            private: Spinlock::new(None),
        }
    }

    /// Get filesystem name
    pub fn fs_name(&self) -> &str {
        &self.fs_name
    }

    /// Get block size
    pub fn block_size(&self) -> Size {
        self.block_size
    }

    /// Set block size
    pub fn set_block_size(&mut self, size: Size) {
        self.block_size = size;
        self.block_bits = size.trailing_zeros() as u8;
    }

    /// Get block size bits
    pub fn block_bits(&self) -> u8 {
        self.block_bits
    }

    /// Get maximum file size
    pub fn max_bytes(&self) -> Offset {
        self.max_bytes
    }

    /// Set maximum file size
    pub fn set_max_bytes(&mut self, max: Offset) {
        self.max_bytes = max;
    }

    /// Get mount flags
    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Set mount flags
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }

    /// Check if read-only
    pub fn is_rdonly(&self) -> bool {
        (self.flags & sb_flags::SB_RDONLY) != 0
    }

    /// Get magic number
    pub fn magic(&self) -> u32 {
        self.magic
    }

    /// Set magic number
    pub fn set_magic(&mut self, magic: u32) {
        self.magic = magic;
    }

    /// Get device ID
    pub fn dev_id(&self) -> Option<DevId> {
        self.dev_id
    }

    /// Set device ID
    pub fn set_dev_id(&mut self, dev_id: DevId) {
        self.dev_id = Some(dev_id);
    }

    /// Get time granularity
    pub fn time_gran(&self) -> u32 {
        self.time_gran
    }

    /// Set time granularity
    pub fn set_time_gran(&mut self, gran: u32) {
        self.time_gran = gran;
    }

    /// Get root dentry
    pub fn root(&self) -> Option<Arc<Dentry>> {
        self.data.read().root.clone()
    }

    /// Set root dentry
    pub fn set_root(&self, root: Arc<Dentry>) {
        self.data.write().root = Some(root);
    }

    /// Set superblock operations
    pub fn set_ops(&mut self, ops: Box<dyn SuperblockOps>) {
        self.s_op = Some(ops);
    }

    /// Get superblock operations
    pub fn ops(&self) -> Option<&dyn SuperblockOps> {
        self.s_op.as_ref().map(|op| op.as_ref())
    }

    /// Set default dentry operations
    pub fn set_dentry_ops(&mut self, ops: Box<dyn DentryOps>) {
        self.d_op = Some(ops);
    }

    /// Get default dentry operations
    pub fn dentry_ops(&self) -> Option<&dyn DentryOps> {
        self.d_op.as_ref().map(|op| op.as_ref())
    }

    /// Add inode to superblock
    pub fn add_inode(&self, inode: &Arc<Inode>) {
        self.data.write().inodes.push(Arc::downgrade(inode));
    }

    /// Remove dead inode references
    pub fn prune_inodes(&self) {
        self.data.write().inodes.retain(|w| w.strong_count() > 0);
    }

    /// Mark inode as dirty
    pub fn mark_inode_dirty(&self, inode: &Arc<Inode>) {
        let mut data = self.data.write();
        // Check if already in dirty list
        if !data.dirty_inodes.iter().any(|w| {
            w.upgrade().map_or(false, |i| Arc::ptr_eq(&i, inode))
        }) {
            data.dirty_inodes.push(Arc::downgrade(inode));
        }
    }

    /// Get dirty inodes
    pub fn dirty_inodes(&self) -> Vec<Arc<Inode>> {
        self.data.read().dirty_inodes
            .iter()
            .filter_map(|w| w.upgrade())
            .collect()
    }

    /// Clear dirty inode list
    pub fn clear_dirty_inodes(&self) {
        self.data.write().dirty_inodes.clear();
    }

    /// Set private data
    pub fn set_private<T: Send + Sync + 'static>(&self, data: T) {
        *self.private.lock() = Some(Box::new(data));
    }
}

impl Default for Superblock {
    fn default() -> Self {
        Self::new("unknown")
    }
}
