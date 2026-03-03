/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_INIT_H
#define _LINUX_INIT_H

#include <linux/compiler.h>

/* Type definitions for initcalls */
typedef int (*initcall_t)(void);
typedef void (*exitcall_t)(void);

/* These macros are used to mark functions/data for initialization */
#define __init      __section(".init.text")
#define __initdata  __section(".init.data")
#define __initconst __section(".init.rodata")
#define __exit      __section(".exit.text")
#define __exitdata  __section(".exit.data")

/* Used for init functions that can be discarded after init */
#define __init_or_module __init
#define __initdata_or_module __initdata

/*
 * Initcall levels - functions are called in this order during boot.
 * Each level has its own ELF section.
 */
#define __define_initcall(fn, id) \
    static initcall_t __initcall_##fn##id __used \
    __attribute__((__section__(".initcall" #id ".init"))) = fn;

#define pure_initcall(fn)       __define_initcall(fn, 0)
#define core_initcall(fn)       __define_initcall(fn, 1)
#define postcore_initcall(fn)   __define_initcall(fn, 2)
#define arch_initcall(fn)       __define_initcall(fn, 3)
#define subsys_initcall(fn)     __define_initcall(fn, 4)
#define fs_initcall(fn)         __define_initcall(fn, 5)
#define device_initcall(fn)     __define_initcall(fn, 6)
#define late_initcall(fn)       __define_initcall(fn, 7)

/* module_init defaults to device_initcall for built-in drivers */
#define __initcall(fn) device_initcall(fn)

/*
 * module_init - driver initialization entry point
 * @fn: function to be run at kernel boot time or module insertion
 *
 * For built-in drivers, this places a pointer to the init function
 * in the .initcall6.init section.
 */
#define module_init(fn) __initcall(fn)

/*
 * module_exit - driver exit entry point
 * @fn: function to be run when driver is removed
 *
 * For built-in drivers, this is typically a no-op since built-in
 * drivers cannot be unloaded.
 */
#define module_exit(fn) static exitcall_t __exitcall_##fn __used __exitdata = fn;

/* Entry points for initcall execution (implemented in Rust) */
extern void do_initcalls(void);

#endif /* _LINUX_INIT_H */
