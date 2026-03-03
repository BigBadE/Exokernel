//! Inode - in-memory representation of a file

use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::sync::atomic::{AtomicU32, AtomicI32, Ordering};

use libos_core::{Mode, Uid, Gid, Offset, Timespec};
use libos_sync::{Spinlock, RwLock};

use crate::traits::{InodeOps, FileOps, InodeAttr, FileType};
use crate::superblock::Superblock;

/// File types (encoded in mode)
pub mod file_type {
    use super::Mode;

    pub const S_IFMT: Mode = 0o170000;   // File type mask
    pub const S_IFSOCK: Mode = 0o140000; // Socket
    pub const S_IFLNK: Mode = 0o120000;  // Symbolic link
    pub const S_IFREG: Mode = 0o100000;  // Regular file
    pub const S_IFBLK: Mode = 0o060000;  // Block device
    pub const S_IFDIR: Mode = 0o040000;  // Directory
    pub const S_IFCHR: Mode = 0o020000;  // Character device
    pub const S_IFIFO: Mode = 0o010000;  // FIFO

    pub fn is_reg(mode: Mode) -> bool { (mode & S_IFMT) == S_IFREG }
    pub fn is_dir(mode: Mode) -> bool { (mode & S_IFMT) == S_IFDIR }
    pub fn is_lnk(mode: Mode) -> bool { (mode & S_IFMT) == S_IFLNK }
    pub fn is_blk(mode: Mode) -> bool { (mode & S_IFMT) == S_IFBLK }
    pub fn is_chr(mode: Mode) -> bool { (mode & S_IFMT) == S_IFCHR }
    pub fn is_fifo(mode: Mode) -> bool { (mode & S_IFMT) == S_IFIFO }
    pub fn is_sock(mode: Mode) -> bool { (mode & S_IFMT) == S_IFSOCK }
}

/// Inode state flags
pub mod state {
    pub const I_NEW: u32 = 1 << 0;
    pub const I_DIRTY: u32 = 1 << 1;
    pub const I_DIRTY_SYNC: u32 = 1 << 2;
    pub const I_DIRTY_DATASYNC: u32 = 1 << 3;
    pub const I_DIRTY_PAGES: u32 = 1 << 4;
    pub const I_FREEING: u32 = 1 << 5;
    pub const I_CLEAR: u32 = 1 << 6;
    pub const I_SYNC: u32 = 1 << 7;
}

/// Inode flags
pub mod flags {
    pub const S_SYNC: u32 = 1 << 0;
    pub const S_IMMUTABLE: u32 = 1 << 1;
    pub const S_APPEND: u32 = 1 << 2;
    pub const S_DEAD: u32 = 1 << 3;
    pub const S_DIRSYNC: u32 = 1 << 4;
}

/// Inode data (mutable fields protected by lock)
struct InodeData {
    /// File mode (type and permissions)
    mode: Mode,
    /// Number of hard links
    nlink: u32,
    /// Owner user ID
    uid: Uid,
    /// Owner group ID
    gid: Gid,
    /// File size in bytes
    size: Offset,
    /// Access time
    atime: Timespec,
    /// Modification time
    mtime: Timespec,
    /// Change time (inode change)
    ctime: Timespec,
    /// Block size for I/O
    blkbits: u8,
    /// Number of 512-byte blocks
    blocks: u64,
    /// Inode version
    version: u64,
    /// Inode flags
    flags: u32,
    /// Generation number
    generation: u32,
}

/// Inode structure
pub struct Inode {
    /// Inode number (immutable)
    ino: u64,
    /// Inode state
    state: AtomicU32,
    /// Reference count
    refcount: AtomicI32,
    /// Write count
    writecount: AtomicI32,
    /// Mutable data
    data: RwLock<InodeData>,
    /// Superblock (weak reference to avoid cycles)
    sb: Weak<Superblock>,
    /// Inode operations
    i_op: Option<Box<dyn InodeOps>>,
    /// Default file operations
    f_op: Option<Box<dyn FileOps>>,
    /// Private filesystem data
    private: Spinlock<Option<Box<dyn core::any::Any + Send + Sync>>>,
}

impl Inode {
    /// Create a new inode
    pub fn new(ino: u64, sb: Weak<Superblock>) -> Self {
        Self {
            ino,
            state: AtomicU32::new(state::I_NEW),
            refcount: AtomicI32::new(1),
            writecount: AtomicI32::new(0),
            data: RwLock::new(InodeData {
                mode: 0,
                nlink: 1,
                uid: 0,
                gid: 0,
                size: 0,
                atime: Timespec::zero(),
                mtime: Timespec::zero(),
                ctime: Timespec::zero(),
                blkbits: 9,
                blocks: 0,
                version: 0,
                flags: 0,
                generation: 0,
            }),
            sb,
            i_op: None,
            f_op: None,
            private: Spinlock::new(None),
        }
    }

    /// Get inode number
    pub fn ino(&self) -> u64 {
        self.ino
    }

    /// Get the superblock
    pub fn superblock(&self) -> Option<Arc<Superblock>> {
        self.sb.upgrade()
    }

    /// Get inode operations
    pub fn inode_ops(&self) -> Option<&dyn InodeOps> {
        self.i_op.as_ref().map(|op| op.as_ref())
    }

    /// Get file operations
    pub fn file_ops(&self) -> Option<&dyn FileOps> {
        self.f_op.as_ref().map(|op| op.as_ref())
    }

    /// Set inode operations
    pub fn set_inode_ops(&mut self, ops: Box<dyn InodeOps>) {
        self.i_op = Some(ops);
    }

    /// Set file operations
    pub fn set_file_ops(&mut self, ops: Box<dyn FileOps>) {
        self.f_op = Some(ops);
    }

    /// Get file mode
    pub fn mode(&self) -> Mode {
        self.data.read().mode
    }

    /// Set file mode
    pub fn set_mode(&self, mode: Mode) {
        self.data.write().mode = mode;
    }

    /// Get link count
    pub fn nlink(&self) -> u32 {
        self.data.read().nlink
    }

    /// Set link count
    pub fn set_nlink(&self, nlink: u32) {
        self.data.write().nlink = nlink;
    }

    /// Increment link count
    pub fn inc_nlink(&self) {
        self.data.write().nlink += 1;
    }

    /// Decrement link count
    pub fn dec_nlink(&self) {
        let mut data = self.data.write();
        if data.nlink > 0 {
            data.nlink -= 1;
        }
    }

    /// Get UID
    pub fn uid(&self) -> Uid {
        self.data.read().uid
    }

    /// Set UID
    pub fn set_uid(&self, uid: Uid) {
        self.data.write().uid = uid;
    }

    /// Get GID
    pub fn gid(&self) -> Gid {
        self.data.read().gid
    }

    /// Set GID
    pub fn set_gid(&self, gid: Gid) {
        self.data.write().gid = gid;
    }

    /// Get file size
    pub fn size(&self) -> Offset {
        self.data.read().size
    }

    /// Set file size
    pub fn set_size(&self, size: Offset) {
        self.data.write().size = size;
    }

    /// Get access time
    pub fn atime(&self) -> Timespec {
        self.data.read().atime
    }

    /// Set access time
    pub fn set_atime(&self, time: Timespec) {
        self.data.write().atime = time;
    }

    /// Get modification time
    pub fn mtime(&self) -> Timespec {
        self.data.read().mtime
    }

    /// Set modification time
    pub fn set_mtime(&self, time: Timespec) {
        self.data.write().mtime = time;
    }

    /// Get change time
    pub fn ctime(&self) -> Timespec {
        self.data.read().ctime
    }

    /// Set change time
    pub fn set_ctime(&self, time: Timespec) {
        self.data.write().ctime = time;
    }

    /// Get block count
    pub fn blocks(&self) -> u64 {
        self.data.read().blocks
    }

    /// Set block count
    pub fn set_blocks(&self, blocks: u64) {
        self.data.write().blocks = blocks;
    }

    /// Check if this is a regular file
    pub fn is_reg(&self) -> bool {
        file_type::is_reg(self.mode())
    }

    /// Check if this is a directory
    pub fn is_dir(&self) -> bool {
        file_type::is_dir(self.mode())
    }

    /// Check if this is a symbolic link
    pub fn is_lnk(&self) -> bool {
        file_type::is_lnk(self.mode())
    }

    /// Get file type
    pub fn file_type(&self) -> FileType {
        FileType::from_mode(self.mode())
    }

    /// Check if inode is dirty
    pub fn is_dirty(&self) -> bool {
        (self.state.load(Ordering::Relaxed) & state::I_DIRTY) != 0
    }

    /// Mark inode dirty
    pub fn mark_dirty(&self) {
        self.state.fetch_or(state::I_DIRTY, Ordering::SeqCst);
    }

    /// Clear dirty flag
    pub fn clear_dirty(&self) {
        self.state.fetch_and(!state::I_DIRTY, Ordering::SeqCst);
    }

    /// Check if inode is new
    pub fn is_new(&self) -> bool {
        (self.state.load(Ordering::Relaxed) & state::I_NEW) != 0
    }

    /// Clear new flag
    pub fn unlock_new(&self) {
        self.state.fetch_and(!state::I_NEW, Ordering::SeqCst);
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

    /// Get attributes
    pub fn attr(&self) -> InodeAttr {
        let data = self.data.read();
        InodeAttr {
            mode: data.mode,
            uid: data.uid,
            gid: data.gid,
            size: data.size,
            atime: data.atime,
            mtime: data.mtime,
            ctime: data.ctime,
            nlink: data.nlink,
            blocks: data.blocks,
        }
    }

    /// Set attributes
    pub fn set_attr(&self, attr: &InodeAttr) {
        let mut data = self.data.write();
        data.mode = attr.mode;
        data.uid = attr.uid;
        data.gid = attr.gid;
        data.size = attr.size;
        data.atime = attr.atime;
        data.mtime = attr.mtime;
        data.ctime = attr.ctime;
        data.nlink = attr.nlink;
        data.blocks = attr.blocks;
    }

    /// Set private data
    pub fn set_private<T: Send + Sync + 'static>(&self, data: T) {
        *self.private.lock() = Some(Box::new(data));
    }

    /// Get private data
    pub fn private<T: 'static>(&self) -> Option<&T> {
        // This is a simplification - real implementation would need unsafe
        None
    }
}

impl Default for Inode {
    fn default() -> Self {
        Self::new(0, Weak::new())
    }
}
