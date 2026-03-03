//! VFS trait definitions
//!
//! These traits define the interface that filesystems must implement.

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use libos_core::{Result, Error, Mode, Offset, Size, Uid, Gid, Timespec};

use crate::inode::Inode;
use crate::dentry::Dentry;
use crate::file::File;
use crate::superblock::Superblock;

/// Superblock operations trait
///
/// Implement this trait to provide filesystem-level operations.
pub trait SuperblockOps: Send + Sync {
    /// Allocate a new inode
    fn alloc_inode(&self, sb: &Arc<Superblock>) -> Result<Arc<Inode>>;

    /// Destroy an inode
    fn destroy_inode(&self, _inode: &Arc<Inode>) {
        // Default: do nothing
    }

    /// Read inode from disk
    fn read_inode(&self, _inode: &Arc<Inode>) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Write inode to disk
    fn write_inode(&self, _inode: &Arc<Inode>, _sync: bool) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Evict (delete) inode
    fn evict_inode(&self, _inode: &Arc<Inode>) {
        // Default: do nothing
    }

    /// Put (release) superblock
    fn put_super(&self, _sb: &Arc<Superblock>) {
        // Default: do nothing
    }

    /// Sync filesystem
    fn sync_fs(&self, _sb: &Arc<Superblock>, _wait: bool) -> Result<()> {
        Ok(())
    }

    /// Get filesystem statistics
    fn statfs(&self, _sb: &Arc<Superblock>) -> Result<FsStats> {
        Err(Error::NotSupported)
    }
}

/// Inode operations trait
///
/// Implement this trait to provide inode operations like lookup, create, etc.
pub trait InodeOps: Send + Sync {
    /// Lookup a name in a directory
    fn lookup(&self, dir: &Arc<Inode>, name: &str) -> Result<Arc<Dentry>>;

    /// Create a regular file
    fn create(&self, _dir: &Arc<Inode>, _name: &str, _mode: Mode) -> Result<Arc<Inode>> {
        Err(Error::NotSupported)
    }

    /// Create a hard link
    fn link(&self, _old_dentry: &Arc<Dentry>, _dir: &Arc<Inode>, _name: &str) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Remove a hard link
    fn unlink(&self, _dir: &Arc<Inode>, _dentry: &Arc<Dentry>) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Create a symbolic link
    fn symlink(&self, _dir: &Arc<Inode>, _name: &str, _target: &str) -> Result<Arc<Inode>> {
        Err(Error::NotSupported)
    }

    /// Create a directory
    fn mkdir(&self, _dir: &Arc<Inode>, _name: &str, _mode: Mode) -> Result<Arc<Inode>> {
        Err(Error::NotSupported)
    }

    /// Remove a directory
    fn rmdir(&self, _dir: &Arc<Inode>, _dentry: &Arc<Dentry>) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Create a device node
    fn mknod(&self, _dir: &Arc<Inode>, _name: &str, _mode: Mode, _dev: u32) -> Result<Arc<Inode>> {
        Err(Error::NotSupported)
    }

    /// Rename a file
    fn rename(
        &self,
        _old_dir: &Arc<Inode>,
        _old_dentry: &Arc<Dentry>,
        _new_dir: &Arc<Inode>,
        _new_name: &str,
    ) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Read a symbolic link
    fn readlink(&self, _inode: &Arc<Inode>) -> Result<String> {
        Err(Error::NotSupported)
    }

    /// Get inode attributes
    fn getattr(&self, inode: &Arc<Inode>) -> Result<InodeAttr> {
        Ok(inode.attr())
    }

    /// Set inode attributes
    fn setattr(&self, _inode: &Arc<Inode>, _attr: &InodeAttr) -> Result<()> {
        Err(Error::NotSupported)
    }
}

/// File operations trait
///
/// Implement this trait to provide file operations like read, write, seek, etc.
pub trait FileOps: Send + Sync {
    /// Seek to position
    fn llseek(&self, _file: &File, _offset: Offset, _whence: i32) -> Result<Offset> {
        Err(Error::NotSupported)
    }

    /// Read from file
    fn read(&self, _file: &File, _buf: &mut [u8], _offset: Offset) -> Result<Size> {
        Err(Error::NotSupported)
    }

    /// Write to file
    fn write(&self, _file: &File, _buf: &[u8], _offset: Offset) -> Result<Size> {
        Err(Error::NotSupported)
    }

    /// Read directory entries
    fn readdir(&self, _file: &File) -> Result<Vec<DirEntry>> {
        Err(Error::NotSupported)
    }

    /// Poll for events
    fn poll(&self, _file: &File) -> Result<u32> {
        Ok(0)
    }

    /// Ioctl
    fn ioctl(&self, _file: &File, _cmd: u32, _arg: u64) -> Result<i64> {
        Err(Error::NotSupported)
    }

    /// Open file
    fn open(&self, _inode: &Arc<Inode>, _file: &File) -> Result<()> {
        Ok(())
    }

    /// Flush file (on close)
    fn flush(&self, _file: &File) -> Result<()> {
        Ok(())
    }

    /// Release file (final close)
    fn release(&self, _inode: &Arc<Inode>, _file: &File) -> Result<()> {
        Ok(())
    }

    /// Sync file to disk
    fn fsync(&self, _file: &File, _datasync: bool) -> Result<()> {
        Err(Error::NotSupported)
    }
}

/// Dentry operations trait
pub trait DentryOps: Send + Sync {
    /// Revalidate dentry
    fn revalidate(&self, _dentry: &Arc<Dentry>) -> Result<bool> {
        Ok(true)
    }

    /// Hash name
    fn hash(&self, name: &str) -> u32 {
        let mut hash: u32 = 0;
        for byte in name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }

    /// Compare names
    fn compare(&self, dentry: &Arc<Dentry>, name: &str) -> bool {
        dentry.name() == name
    }

    /// Delete dentry
    fn delete(&self, _dentry: &Arc<Dentry>) -> bool {
        false
    }

    /// Release dentry
    fn release(&self, _dentry: &Arc<Dentry>) {
        // Default: do nothing
    }
}

/// Inode attributes
#[derive(Debug, Clone, Default)]
pub struct InodeAttr {
    pub mode: Mode,
    pub uid: Uid,
    pub gid: Gid,
    pub size: Offset,
    pub atime: Timespec,
    pub mtime: Timespec,
    pub ctime: Timespec,
    pub nlink: u32,
    pub blocks: u64,
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub ino: u64,
    pub file_type: FileType,
}

/// File type for directory entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Unknown,
    Regular,
    Directory,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
    Symlink,
}

impl Default for FileType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl FileType {
    /// Convert from mode bits
    pub fn from_mode(mode: Mode) -> Self {
        let type_bits = mode & 0o170000;
        match type_bits {
            0o100000 => Self::Regular,
            0o040000 => Self::Directory,
            0o020000 => Self::CharDevice,
            0o060000 => Self::BlockDevice,
            0o010000 => Self::Fifo,
            0o140000 => Self::Socket,
            0o120000 => Self::Symlink,
            _ => Self::Unknown,
        }
    }
}

/// Filesystem statistics
#[derive(Debug, Clone, Default)]
pub struct FsStats {
    /// Filesystem type
    pub fs_type: u32,
    /// Optimal block size
    pub block_size: Size,
    /// Total blocks
    pub blocks: u64,
    /// Free blocks
    pub blocks_free: u64,
    /// Available blocks (for unprivileged users)
    pub blocks_available: u64,
    /// Total inodes
    pub files: u64,
    /// Free inodes
    pub files_free: u64,
    /// Maximum filename length
    pub name_max: Size,
}
