//! Linux kernel library functions
//!
//! Provides string operations, time functions, NLS support, and utility functions.

#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

pub mod string;
pub mod time;
pub mod nls;
pub mod print;

pub use string::*;
pub use time::*;
pub use nls::*;
pub use print::*;
