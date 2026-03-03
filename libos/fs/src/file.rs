//! File - open file handle

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicI32, AtomicI64, Ordering};

use libos_core::{Offset, Size, Result, Error};
use libos_sync::Spinlock;

use crate::dentry::Dentry;
use crate::inode::Inode;
use crate::traits::FileOps;

/// File open flags
pub mod open_flags {
    pub const O_RDONLY: u32 = 0;
    pub const O_WRONLY: u32 = 1;
    pub const O_RDWR: u32 = 2;
    pub const O_ACCMODE: u32 = 3;
    pub const O_CREAT: u32 = 0o100;
    pub const O_EXCL: u32 = 0o200;
    pub const O_TRUNC: u32 = 0o1000;
    pub const O_APPEND: u32 = 0o2000;
    pub const O_NONBLOCK: u32 = 0o4000;
    pub const O_DIRECTORY: u32 = 0o200000;
    pub const O_CLOEXEC: u32 = 0o2000000;
}

/// File mode flags
pub mod file_mode {
    pub const FMODE_READ: u32 = 1 << 0;
    pub const FMODE_WRITE: u32 = 1 << 1;
    pub const FMODE_EXEC: u32 = 1 << 2;
    pub const FMODE_LSEEK: u32 = 1 << 3;
    pub const FMODE_PREAD: u32 = 1 << 4;
    pub const FMODE_PWRITE: u32 = 1 << 5;
}

/// Seek whence values
pub mod seek {
    pub const SEEK_SET: i32 = 0;
    pub const SEEK_CUR: i32 = 1;
    pub const SEEK_END: i32 = 2;
}

/// Open file structure
pub struct File {
    /// File position
    pos: AtomicI64,
    /// Open flags
    flags: u32,
    /// File mode
    mode: u32,
    /// Reference count
    refcount: AtomicI32,
    /// Associated dentry
    dentry: Arc<Dentry>,
    /// File operations
    f_op: Option<Box<dyn FileOps>>,
    /// Private data
    private: Spinlock<Option<Box<dyn core::any::Any + Send + Sync>>>,
}

impl File {
    /// Create a new file
    pub fn new(dentry: Arc<Dentry>, flags: u32) -> Self {
        let mode = flags_to_mode(flags);
        Self {
            pos: AtomicI64::new(0),
            flags,
            mode,
            refcount: AtomicI32::new(1),
            dentry,
            f_op: None,
            private: Spinlock::new(None),
        }
    }

    /// Get the dentry
    pub fn dentry(&self) -> &Arc<Dentry> {
        &self.dentry
    }

    /// Get the inode
    pub fn inode(&self) -> Option<Arc<Inode>> {
        self.dentry.inode()
    }

    /// Get open flags
    pub fn flags(&self) -> u32 {
        self.flags
    }

    /// Get file mode
    pub fn mode(&self) -> u32 {
        self.mode
    }

    /// Check if file is readable
    pub fn is_readable(&self) -> bool {
        (self.mode & file_mode::FMODE_READ) != 0
    }

    /// Check if file is writable
    pub fn is_writable(&self) -> bool {
        (self.mode & file_mode::FMODE_WRITE) != 0
    }

    /// Check if file is seekable
    pub fn is_seekable(&self) -> bool {
        (self.mode & file_mode::FMODE_LSEEK) != 0
    }

    /// Get current position
    pub fn position(&self) -> Offset {
        self.pos.load(Ordering::Relaxed)
    }

    /// Set position
    pub fn set_position(&self, pos: Offset) {
        self.pos.store(pos, Ordering::Relaxed);
    }

    /// Seek to new position
    pub fn seek(&self, offset: Offset, whence: i32) -> Result<Offset> {
        let size = self.inode().map(|i| i.size()).unwrap_or(0);
        let current = self.position();

        let new_pos = match whence {
            seek::SEEK_SET => offset,
            seek::SEEK_CUR => current + offset,
            seek::SEEK_END => size + offset,
            _ => return Err(Error::InvalidArgument),
        };

        if new_pos < 0 {
            return Err(Error::InvalidSeek);
        }

        self.set_position(new_pos);
        Ok(new_pos)
    }

    /// Read from file
    pub fn read(&self, buf: &mut [u8]) -> Result<Size> {
        if !self.is_readable() {
            return Err(Error::AccessDenied);
        }

        let ops = self.f_op.as_ref().ok_or(Error::NotSupported)?;
        let pos = self.position();
        let bytes = ops.read(self, buf, pos)?;
        self.set_position(pos + bytes as Offset);
        Ok(bytes)
    }

    /// Write to file
    pub fn write(&self, buf: &[u8]) -> Result<Size> {
        if !self.is_writable() {
            return Err(Error::AccessDenied);
        }

        let ops = self.f_op.as_ref().ok_or(Error::NotSupported)?;
        let pos = self.position();
        let bytes = ops.write(self, buf, pos)?;
        self.set_position(pos + bytes as Offset);
        Ok(bytes)
    }

    /// Sync file to disk
    pub fn sync(&self) -> Result<()> {
        let ops = self.f_op.as_ref().ok_or(Error::NotSupported)?;
        ops.fsync(self, false)
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

    /// Set file operations
    pub fn set_ops(&mut self, ops: Box<dyn FileOps>) {
        self.f_op = Some(ops);
    }

    /// Get file operations
    pub fn ops(&self) -> Option<&dyn FileOps> {
        self.f_op.as_ref().map(|op| op.as_ref())
    }

    /// Set private data
    pub fn set_private<T: Send + Sync + 'static>(&self, data: T) {
        *self.private.lock() = Some(Box::new(data));
    }
}

/// Convert open flags to file mode
pub fn flags_to_mode(flags: u32) -> u32 {
    let mut mode = 0;

    match flags & open_flags::O_ACCMODE {
        open_flags::O_RDONLY => mode |= file_mode::FMODE_READ,
        open_flags::O_WRONLY => mode |= file_mode::FMODE_WRITE,
        open_flags::O_RDWR => mode |= file_mode::FMODE_READ | file_mode::FMODE_WRITE,
        _ => {}
    }

    // Most files are seekable
    mode |= file_mode::FMODE_LSEEK | file_mode::FMODE_PREAD | file_mode::FMODE_PWRITE;

    mode
}
