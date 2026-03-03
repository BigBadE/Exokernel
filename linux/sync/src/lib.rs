//! Linux kernel synchronization primitives
//!
//! Provides spinlock, mutex, rwsem, semaphore, and RCU implementations.

#![no_std]
#![allow(non_camel_case_types)]

pub mod spinlock;
pub mod mutex;
pub mod rwsem;
pub mod rcu;

pub use spinlock::*;
pub use mutex::*;
pub use rwsem::*;
pub use rcu::*;
