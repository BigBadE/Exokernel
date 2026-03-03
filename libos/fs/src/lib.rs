//! LibOS Virtual Filesystem
//!
//! Provides VFS abstractions using Rust traits instead of C function pointers.
//! All types are idiomatic Rust with no raw pointers.

#![no_std]

extern crate alloc;

pub mod traits;
pub mod inode;
pub mod dentry;
pub mod file;
pub mod superblock;
pub mod mount;
pub mod path;

pub use traits::*;
pub use inode::*;
pub use dentry::*;
pub use file::*;
pub use superblock::*;
pub use mount::*;
pub use path::*;
