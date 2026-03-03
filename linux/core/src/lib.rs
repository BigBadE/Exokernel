//! Core Linux kernel types and error codes
//!
//! This crate provides the fundamental types used throughout the Linux
//! kernel shim: atomic types, list structures, time types, and error codes.

#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod types;
pub mod errno;
pub mod initcall;

pub use types::*;
pub use errno::*;
pub use initcall::*;

// Re-export the macros
pub use linux_macros::linux_export;
