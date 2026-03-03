//! Exokernel Shared Types
//!
//! This crate contains types shared between the kernel and userspace.
//! It defines the capability system, syscall numbers, and error codes.

#![no_std]

pub mod caps;
pub mod error;
pub mod syscall;
pub mod ipc;

pub use caps::*;
pub use error::*;
pub use syscall::{Syscall, MemFlags, ProcessFlags, IpcMessage};

/// Alias for Syscall enum as SyscallNumber for clarity
pub type SyscallNumber = Syscall;
