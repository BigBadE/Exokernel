/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_COMPAT_H
#define _LINUX_COMPAT_H

#include <linux/types.h>

/* User-space marker (no-op in kernel builds) */
#ifndef __user
#define __user
#endif

/* 32-bit compatibility types for 64-bit systems */
typedef s32 compat_int_t;
typedef u32 compat_uint_t;
typedef s32 compat_long_t;
typedef u32 compat_ulong_t;
typedef s64 compat_s64;
typedef u64 compat_u64;
typedef u32 compat_size_t;
typedef s32 compat_ssize_t;
typedef s32 compat_off_t;
typedef s64 compat_loff_t;
typedef u32 compat_uptr_t;

/* Check if we're in compat mode */
static inline bool in_compat_syscall(void)
{
    return false; /* 64-bit kernel, not in compat mode */
}

/* Convert compat pointer */
static inline void __user *compat_ptr(compat_uptr_t uptr)
{
    return (void __user *)(unsigned long)uptr;
}

static inline compat_uptr_t ptr_to_compat(void __user *uptr)
{
    return (compat_uptr_t)(unsigned long)uptr;
}

#endif /* _LINUX_COMPAT_H */
