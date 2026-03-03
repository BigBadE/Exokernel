//! Linux Virtual Filesystem Layer
//!
//! This crate provides the VFS implementation that:
//! - Manages filesystem registration (register_filesystem)
//! - Handles mount/unmount operations
//! - Implements inode and dentry caches
//! - Routes file operations to drivers via function pointers
//!
//! The VFS sits between user code and the Linux filesystem drivers,
//! calling into drivers when needed and providing the generic
//! infrastructure they expect (new_inode, d_instantiate, etc.)

#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

extern crate alloc;

pub mod types;
pub mod inode;
pub mod dentry;
pub mod file;
pub mod superblock;
pub mod mount;
pub mod fs_type;
pub mod dcache;
pub mod operations;
pub mod export;

// Re-export main types
pub use types::*;
pub use inode::*;
pub use dentry::*;
pub use file::*;
pub use superblock::*;
pub use mount::*;
pub use fs_type::*;

use alloc::string::String;
use alloc::vec::Vec;

// ============================================================================
// High-level Rust API
// ============================================================================

/// A mounted Linux filesystem
pub struct LinuxFs {
    root: *mut dentry::dentry,
    sb: *mut superblock::super_block,
}

impl LinuxFs {
    /// Mount a filesystem
    pub fn mount(fstype: &str, device: &str, flags: MountFlags) -> FsResult<Self> {
        Err(FsError::NotSupported)
    }

    /// Read a file's contents
    pub fn read_file(&self, _path: &str) -> FsResult<Vec<u8>> {
        Err(FsError::NotSupported)
    }

    /// List directory contents
    pub fn read_dir(&self, _path: &str) -> FsResult<Vec<DirEntry>> {
        Err(FsError::NotSupported)
    }

    /// Get file metadata
    pub fn stat(&self, _path: &str) -> FsResult<FileStat> {
        Err(FsError::NotSupported)
    }
}

impl Drop for LinuxFs {
    fn drop(&mut self) {
        // Unmount filesystem
    }
}

// ============================================================================
// Mount flags and error types
// ============================================================================

/// Mount flags
#[derive(Debug, Clone, Copy)]
pub struct MountFlags(pub u32);

impl MountFlags {
    pub const RDONLY: Self = Self(1);
    pub const NOSUID: Self = Self(2);
    pub const NODEV: Self = Self(4);
    pub const NOEXEC: Self = Self(8);
    pub const SYNCHRONOUS: Self = Self(16);
    pub const MANDLOCK: Self = Self(64);
    pub const DIRSYNC: Self = Self(128);
    pub const NOATIME: Self = Self(1024);
    pub const NODIRATIME: Self = Self(2048);
}

impl core::ops::BitOr for MountFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Result type for filesystem operations
pub type FsResult<T> = Result<T, FsError>;

/// Filesystem error
#[derive(Debug)]
pub enum FsError {
    NoSuchFile,
    PermissionDenied,
    IoError,
    InvalidArgument,
    NoMemory,
    NotADirectory,
    IsADirectory,
    NoSpace,
    ReadOnly,
    NotSupported,
    Other(i32),
}

/// Directory entry (high-level)
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub inode: u64,
    pub file_type: FileType,
}

/// File type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    Symlink,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
    Unknown,
}

/// File statistics (high-level)
#[derive(Debug, Clone)]
pub struct FileStat {
    pub inode: u64,
    pub mode: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub blksize: u32,
    pub blocks: u64,
}

/// Available filesystem types
pub fn list_filesystems() -> Vec<&'static str> {
    fs_type::list_registered_filesystems()
}
