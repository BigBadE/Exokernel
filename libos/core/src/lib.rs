//! Core libOS types and traits
//!
//! This crate provides fundamental types and traits used throughout the libOS.
//! It has no dependencies and can be used by all other libOS crates.

#![no_std]

pub mod error;
pub mod types;

pub use error::*;
pub use types::*;
