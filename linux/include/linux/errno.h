/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_ERRNO_H
#define _LINUX_ERRNO_H

#include <asm/errno.h>

/* Kernel-specific errno extensions */
#define ERESTARTSYS     512
#define ERESTARTNOINTR  513
#define ERESTARTNOHAND  514
#define ENOIOCTLCMD     515
#define ERESTART_RESTARTBLOCK 516
#define EPROBE_DEFER    517
#define EOPENSTALE      518
#define ENOPARAM        519

/* Error pointer handling */
#define MAX_ERRNO       4095
#define IS_ERR_VALUE(x) unlikely((unsigned long)(void *)(x) >= (unsigned long)-MAX_ERRNO)

static inline void *ERR_PTR(long error)
{
    return (void *)error;
}

static inline long PTR_ERR(const void *ptr)
{
    return (long)ptr;
}

static inline bool IS_ERR(const void *ptr)
{
    return IS_ERR_VALUE((unsigned long)ptr);
}

static inline bool IS_ERR_OR_NULL(const void *ptr)
{
    return unlikely(!ptr) || IS_ERR_VALUE((unsigned long)ptr);
}

static inline void *ERR_CAST(const void *ptr)
{
    return (void *)ptr;
}

static inline int PTR_ERR_OR_ZERO(const void *ptr)
{
    if (IS_ERR(ptr))
        return PTR_ERR(ptr);
    return 0;
}

#endif /* _LINUX_ERRNO_H */
