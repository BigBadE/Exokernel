//! Mount operations

use alloc::sync::Arc;
use alloc::vec::Vec;

use libos_core::{Result, Error};
use libos_sync::{Spinlock, RwLock};

use crate::superblock::Superblock;
use crate::dentry::Dentry;

/// Filesystem type trait
pub trait FileSystemType: Send + Sync {
    /// Get filesystem name
    fn name(&self) -> &str;

    /// Mount the filesystem
    fn mount(&self, source: Option<&str>, flags: u32, data: Option<&str>) -> Result<Arc<Superblock>>;

    /// Kill (unmount) superblock
    fn kill_sb(&self, sb: &Arc<Superblock>);
}

/// Mount point
pub struct Mount {
    /// Superblock
    sb: Arc<Superblock>,
    /// Mount root dentry
    root: Arc<Dentry>,
    /// Mount point dentry (where we're mounted on)
    mountpoint: Option<Arc<Dentry>>,
    /// Parent mount
    parent: Option<Arc<Mount>>,
    /// Filesystem type
    fs_type: Arc<dyn FileSystemType>,
    /// Children mounts
    children: RwLock<Vec<Arc<Mount>>>,
}

impl Mount {
    /// Create a new mount
    pub fn new(
        sb: Arc<Superblock>,
        root: Arc<Dentry>,
        fs_type: Arc<dyn FileSystemType>,
    ) -> Self {
        Self {
            sb,
            root,
            mountpoint: None,
            parent: None,
            fs_type,
            children: RwLock::new(Vec::new()),
        }
    }

    /// Get superblock
    pub fn superblock(&self) -> &Arc<Superblock> {
        &self.sb
    }

    /// Get root dentry
    pub fn root(&self) -> &Arc<Dentry> {
        &self.root
    }

    /// Get mount point
    pub fn mountpoint(&self) -> Option<&Arc<Dentry>> {
        self.mountpoint.as_ref()
    }

    /// Get parent mount
    pub fn parent(&self) -> Option<&Arc<Mount>> {
        self.parent.as_ref()
    }

    /// Get filesystem type
    pub fn fs_type(&self) -> &Arc<dyn FileSystemType> {
        &self.fs_type
    }

    /// Set parent and mountpoint
    pub fn set_parent(&mut self, parent: Arc<Mount>, mountpoint: Arc<Dentry>) {
        self.parent = Some(parent);
        self.mountpoint = Some(mountpoint);
    }

    /// Add child mount
    pub fn add_child(&self, child: Arc<Mount>) {
        self.children.write().push(child);
    }

    /// Remove child mount
    pub fn remove_child(&self, child: &Arc<Mount>) {
        self.children.write().retain(|c| !Arc::ptr_eq(c, child));
    }

    /// Get children
    pub fn children(&self) -> Vec<Arc<Mount>> {
        self.children.read().clone()
    }
}

/// Global filesystem type registry
struct FsRegistry {
    types: Vec<Arc<dyn FileSystemType>>,
}

impl FsRegistry {
    const fn new() -> Self {
        Self { types: Vec::new() }
    }
}

static FS_REGISTRY: Spinlock<Option<FsRegistry>> = Spinlock::new(None);

fn get_registry() -> &'static Spinlock<Option<FsRegistry>> {
    let mut guard = FS_REGISTRY.lock();
    if guard.is_none() {
        *guard = Some(FsRegistry::new());
    }
    drop(guard);
    &FS_REGISTRY
}

/// Initialize the filesystem registry
pub fn init() {
    let _ = get_registry();
}

/// Register a filesystem type
pub fn register_filesystem(fs: Arc<dyn FileSystemType>) -> Result<()> {
    let registry = get_registry();
    let mut guard = registry.lock();
    let reg = guard.as_mut().ok_or(Error::NotSupported)?;

    // Check for duplicate
    for existing in &reg.types {
        if existing.name() == fs.name() {
            return Err(Error::AlreadyExists);
        }
    }

    reg.types.push(fs);
    Ok(())
}

/// Unregister a filesystem type
pub fn unregister_filesystem(name: &str) -> Result<()> {
    let registry = get_registry();
    let mut guard = registry.lock();
    let reg = guard.as_mut().ok_or(Error::NotSupported)?;

    reg.types.retain(|f| f.name() != name);
    Ok(())
}

/// Get filesystem type by name
pub fn get_filesystem(name: &str) -> Option<Arc<dyn FileSystemType>> {
    let registry = get_registry();
    let guard = registry.lock();
    let reg = guard.as_ref()?;

    reg.types.iter().find(|f| f.name() == name).cloned()
}

/// Mount a filesystem
pub fn mount(
    fs_name: &str,
    source: Option<&str>,
    flags: u32,
    data: Option<&str>,
) -> Result<Arc<Mount>> {
    let fs = get_filesystem(fs_name).ok_or(Error::NotFound)?;
    let sb = fs.mount(source, flags, data)?;
    let root = sb.root().ok_or(Error::Io)?;

    Ok(Arc::new(Mount::new(sb, root, fs)))
}

/// Unmount a filesystem
pub fn umount(mount: &Arc<Mount>) -> Result<()> {
    mount.fs_type().kill_sb(mount.superblock());
    Ok(())
}
