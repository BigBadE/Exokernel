//! Linux kernel memory management
//!
//! Provides kmalloc/kfree, slab allocator, and page management.

#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

pub mod alloc_impl;
pub mod slab;
pub mod page;

pub use alloc_impl::*;
pub use slab::*;
pub use page::*;
