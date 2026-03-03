/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_MODULE_H
#define _LINUX_MODULE_H

#include <linux/init.h>
#include <linux/compiler.h>

/*
 * Module metadata macros - these are no-ops for built-in drivers
 * but kept for compatibility with Linux driver source.
 */
#define MODULE_LICENSE(x)
#define MODULE_AUTHOR(x)
#define MODULE_DESCRIPTION(x)
#define MODULE_VERSION(x)
#define MODULE_ALIAS(x)
#define MODULE_ALIAS_FS(x)
#define MODULE_DEVICE_TABLE(type, table)
#define MODULE_FIRMWARE(x)
#define MODULE_INFO(tag, info)
#define MODULE_SOFTDEP(x)
#define MODULE_IMPORT_NS(x)

/* Module parameters - stubs for now */
#define module_param(name, type, perm)
#define module_param_named(name, var, type, perm)
#define module_param_string(name, str, len, perm)
#define module_param_array(name, type, nump, perm)
#define MODULE_PARM_DESC(name, desc)

/* This module marker - NULL for built-in */
#define THIS_MODULE ((struct module *)0)

/* Module state (for built-in, always "live") */
enum module_state {
    MODULE_STATE_LIVE,
    MODULE_STATE_COMING,
    MODULE_STATE_GOING,
    MODULE_STATE_UNFORMED,
};

/* Forward declaration */
struct module;

/* Module reference counting - no-ops for built-in */
static inline int try_module_get(struct module *mod) { return 1; }
static inline void module_put(struct module *mod) { }
static inline void __module_get(struct module *mod) { }
static inline int module_is_live(struct module *mod) { return 1; }

/* Symbol export - place in export section for potential future use */
#define EXPORT_SYMBOL(sym)
#define EXPORT_SYMBOL_GPL(sym)
#define EXPORT_SYMBOL_NS(sym, ns)
#define EXPORT_SYMBOL_NS_GPL(sym, ns)

#endif /* _LINUX_MODULE_H */
