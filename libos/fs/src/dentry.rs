//! Dentry - directory entry cache

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicI32, Ordering};

use libos_sync::{Spinlock, RwLock};

use crate::inode::Inode;
use crate::superblock::Superblock;
use crate::traits::DentryOps;

/// Maximum name length
pub const NAME_MAX: usize = 255;

/// Dentry flags
pub mod flags {
    pub const DCACHE_MOUNTED: u32 = 1 << 0;
    pub const DCACHE_DISCONNECTED: u32 = 1 << 1;
    pub const DCACHE_REFERENCED: u32 = 1 << 2;
    pub const DCACHE_NEGATIVE: u32 = 1 << 3;
}

/// Dentry data (mutable fields)
struct DentryData {
    /// Associated inode (may be None for negative dentry)
    inode: Option<Arc<Inode>>,
    /// Children
    children: Vec<Arc<Dentry>>,
}

/// Directory entry
pub struct Dentry {
    /// Reference count
    refcount: AtomicI32,
    /// Flags
    flags: AtomicU32,
    /// Name
    name: String,
    /// Name hash
    hash: u32,
    /// Mutable data
    data: RwLock<DentryData>,
    /// Parent dentry (weak reference to avoid cycles)
    parent: Weak<Dentry>,
    /// Superblock (weak reference)
    sb: Weak<Superblock>,
    /// Dentry operations
    d_op: Option<Box<dyn DentryOps>>,
    /// Private filesystem data
    private: Spinlock<Option<Box<dyn core::any::Any + Send + Sync>>>,
}

impl Dentry {
    /// Create a new dentry
    pub fn new(name: &str, parent: Weak<Dentry>, sb: Weak<Superblock>) -> Self {
        let hash = compute_hash(name);
        Self {
            refcount: AtomicI32::new(1),
            flags: AtomicU32::new(0),
            name: String::from(name),
            hash,
            data: RwLock::new(DentryData {
                inode: None,
                children: Vec::new(),
            }),
            parent,
            sb,
            d_op: None,
            private: Spinlock::new(None),
        }
    }

    /// Create a root dentry
    pub fn new_root(sb: Weak<Superblock>) -> Arc<Self> {
        Arc::new_cyclic(|weak| {
            Self {
                refcount: AtomicI32::new(1),
                flags: AtomicU32::new(0),
                name: String::from("/"),
                hash: compute_hash("/"),
                data: RwLock::new(DentryData {
                    inode: None,
                    children: Vec::new(),
                }),
                parent: weak.clone(),
                sb,
                d_op: None,
                private: Spinlock::new(None),
            }
        })
    }

    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get name hash
    pub fn hash(&self) -> u32 {
        self.hash
    }

    /// Get parent
    pub fn parent(&self) -> Option<Arc<Dentry>> {
        self.parent.upgrade()
    }

    /// Get superblock
    pub fn superblock(&self) -> Option<Arc<Superblock>> {
        self.sb.upgrade()
    }

    /// Get inode
    pub fn inode(&self) -> Option<Arc<Inode>> {
        self.data.read().inode.clone()
    }

    /// Set inode
    pub fn set_inode(&self, inode: Arc<Inode>) {
        let mut data = self.data.write();
        data.inode = Some(inode);
        self.flags.fetch_and(!flags::DCACHE_NEGATIVE, Ordering::SeqCst);
    }

    /// Clear inode (make negative)
    pub fn clear_inode(&self) {
        let mut data = self.data.write();
        data.inode = None;
        self.flags.fetch_or(flags::DCACHE_NEGATIVE, Ordering::SeqCst);
    }

    /// Check if dentry is positive (has inode)
    pub fn is_positive(&self) -> bool {
        self.data.read().inode.is_some()
    }

    /// Check if dentry is negative (no inode)
    pub fn is_negative(&self) -> bool {
        self.data.read().inode.is_none()
    }

    /// Check if dentry is root
    pub fn is_root(&self) -> bool {
        // Root if has no parent or parent is itself
        self.name == "/"
    }

    /// Add child
    pub fn add_child(&self, child: Arc<Dentry>) {
        self.data.write().children.push(child);
    }

    /// Remove child by name
    pub fn remove_child(&self, name: &str) {
        self.data.write().children.retain(|c| c.name() != name);
    }

    /// Find child by name
    pub fn find_child(&self, name: &str) -> Option<Arc<Dentry>> {
        let data = self.data.read();
        data.children.iter().find(|c| c.name() == name).cloned()
    }

    /// Get all children
    pub fn children(&self) -> Vec<Arc<Dentry>> {
        self.data.read().children.clone()
    }

    /// Check if mounted
    pub fn is_mounted(&self) -> bool {
        (self.flags.load(Ordering::Relaxed) & flags::DCACHE_MOUNTED) != 0
    }

    /// Set mounted flag
    pub fn set_mounted(&self) {
        self.flags.fetch_or(flags::DCACHE_MOUNTED, Ordering::SeqCst);
    }

    /// Clear mounted flag
    pub fn clear_mounted(&self) {
        self.flags.fetch_and(!flags::DCACHE_MOUNTED, Ordering::SeqCst);
    }

    /// Get reference count
    pub fn refcount(&self) -> i32 {
        self.refcount.load(Ordering::Relaxed)
    }

    /// Increment reference count
    pub fn get(&self) {
        self.refcount.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement reference count
    pub fn put(&self) {
        self.refcount.fetch_sub(1, Ordering::SeqCst);
    }

    /// Set dentry operations
    pub fn set_ops(&mut self, ops: Box<dyn DentryOps>) {
        self.d_op = Some(ops);
    }

    /// Get dentry operations
    pub fn ops(&self) -> Option<&dyn DentryOps> {
        self.d_op.as_ref().map(|op| op.as_ref())
    }
}

/// Compute name hash
fn compute_hash(name: &str) -> u32 {
    let mut hash: u32 = 0;
    for byte in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    hash
}

/// Create a child dentry
pub fn create_child(parent: &Arc<Dentry>, name: &str, inode: Option<Arc<Inode>>) -> Arc<Dentry> {
    let dentry = Arc::new(Dentry::new(
        name,
        Arc::downgrade(parent),
        parent.sb.clone(),
    ));

    if let Some(inode) = inode {
        dentry.set_inode(inode);
    }

    parent.add_child(Arc::clone(&dentry));
    dentry
}
