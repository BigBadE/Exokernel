//! Linux kernel initcall mechanism
//!
//! This module implements the initcall mechanism used by Linux drivers
//! to register initialization functions. Functions are placed in special
//! ELF sections and called in order during boot.

use core::ffi::c_int;

use linux_core::linux_export;

/// Type for initialization functions
pub type InitcallFn = unsafe extern "C" fn() -> c_int;

/// Initcall levels, matching Linux kernel
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InitcallLevel {
    Pure = 0,      // .initcall0.init
    Core = 1,      // .initcall1.init
    Postcore = 2,  // .initcall2.init
    Arch = 3,      // .initcall3.init
    Subsys = 4,    // .initcall4.init
    Fs = 5,        // .initcall5.init
    Device = 6,    // .initcall6.init (module_init default)
    Late = 7,      // .initcall7.init
}

// External symbols defined by the linker script marking section boundaries
unsafe extern "C" {
    static __initcall0_start: InitcallFn;
    static __initcall0_end: InitcallFn;
    static __initcall1_start: InitcallFn;
    static __initcall1_end: InitcallFn;
    static __initcall2_start: InitcallFn;
    static __initcall2_end: InitcallFn;
    static __initcall3_start: InitcallFn;
    static __initcall3_end: InitcallFn;
    static __initcall4_start: InitcallFn;
    static __initcall4_end: InitcallFn;
    static __initcall5_start: InitcallFn;
    static __initcall5_end: InitcallFn;
    static __initcall6_start: InitcallFn;
    static __initcall6_end: InitcallFn;
    static __initcall7_start: InitcallFn;
    static __initcall7_end: InitcallFn;
}

/// Execute all initcalls at a specific level
///
/// # Safety
/// This function calls arbitrary C functions registered via module_init.
/// The linker symbols must be properly defined.
unsafe fn do_initcall_level(start: *const InitcallFn, end: *const InitcallFn) -> c_int {
    let mut current = start;

    while current < end {
        let func = *current;
        let ret = func();

        if ret != 0 {
            // In Linux, a non-zero return typically means init failed
            // We continue but could log this
            // For now, just continue to next initcall
        }

        current = current.add(1);
    }

    0
}

/// Execute all registered initcalls in order
///
/// This walks through all initcall levels (0-7) and calls each registered
/// initialization function. This is called during kernel startup.
///
/// # Safety
/// This function should only be called once during boot. It calls arbitrary
/// C functions that were registered via module_init and friends.
#[linux_export]
unsafe fn do_initcalls() {
    do_initcall_level(&__initcall0_start, &__initcall0_end);
    do_initcall_level(&__initcall1_start, &__initcall1_end);
    do_initcall_level(&__initcall2_start, &__initcall2_end);
    do_initcall_level(&__initcall3_start, &__initcall3_end);
    do_initcall_level(&__initcall4_start, &__initcall4_end);
    do_initcall_level(&__initcall5_start, &__initcall5_end);
    do_initcall_level(&__initcall6_start, &__initcall6_end);
    do_initcall_level(&__initcall7_start, &__initcall7_end);
}

/// Execute initcalls for a specific level only
///
/// # Safety
/// Same safety requirements as do_initcalls.
#[linux_export]
unsafe fn do_initcall_level_n(level: c_int) {
    match level {
        0 => { do_initcall_level(&__initcall0_start, &__initcall0_end); }
        1 => { do_initcall_level(&__initcall1_start, &__initcall1_end); }
        2 => { do_initcall_level(&__initcall2_start, &__initcall2_end); }
        3 => { do_initcall_level(&__initcall3_start, &__initcall3_end); }
        4 => { do_initcall_level(&__initcall4_start, &__initcall4_end); }
        5 => { do_initcall_level(&__initcall5_start, &__initcall5_end); }
        6 => { do_initcall_level(&__initcall6_start, &__initcall6_end); }
        7 => { do_initcall_level(&__initcall7_start, &__initcall7_end); }
        _ => {}
    }
}
