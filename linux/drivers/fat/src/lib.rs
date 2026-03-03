//! Linux FAT Filesystem Driver
//!
//! This crate provides the FAT filesystem driver compiled from
//! real Linux kernel source code (v6.6).
//!
//! The driver registers itself automatically via the Linux kernel's
//! module_init() mechanism, which places initialization function
//! pointers in ELF sections (.initcall6.init for module_init).
//!
//! No manual initialization is required - the kernel's initcall
//! mechanism will invoke the FAT driver's init functions during boot.

#![no_std]

// Link to the compiled C driver object files.
// The C code uses module_init() which places init function pointers
// in .initcall sections that are processed during kernel startup.
#[link(name = "linux_fat", kind = "static")]
unsafe extern "C" {}
