/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_TYPES_H
#define _LINUX_TYPES_H

#include <asm/types.h>

/* Standard fixed-width types */
typedef signed char         s8;
typedef unsigned char       u8;
typedef signed short        s16;
typedef unsigned short      u16;
typedef signed int          s32;
typedef unsigned int        u32;
typedef signed long long    s64;
typedef unsigned long long  u64;

/* Size types */
typedef unsigned long       size_t;
typedef signed long         ssize_t;
typedef signed long         ptrdiff_t;

/* File offset types */
typedef long long           loff_t;

/* Sector/block types */
typedef u64                 sector_t;
typedef u64                 blkcnt_t;

/* Time types */
typedef s64                 time64_t;
typedef s64                 ktime_t;

/* Mode type (file permissions) */
typedef unsigned short      umode_t;

/* User/group IDs */
typedef struct { int val; } kuid_t;
typedef struct { int val; } kgid_t;

/* VFS user/group IDs (wrapper types for idmapped mounts) */
typedef struct { kuid_t val; } vfsuid_t;
typedef struct { kgid_t val; } vfsgid_t;

/* Global root user/group IDs */
#define GLOBAL_ROOT_UID ((kuid_t){ .val = 0 })
#define GLOBAL_ROOT_GID ((kgid_t){ .val = 0 })

/* Invalid VFS UIDs/GIDs */
#define INVALID_VFSUID ((vfsuid_t){ KUIDT_INIT(-1) })
#define INVALID_VFSGID ((vfsgid_t){ KGIDT_INIT(-1) })

/* Init namespace (stub - we use a single namespace) */
struct user_namespace;
extern struct user_namespace init_user_ns;

/* UID/GID helper macros */
#define KUIDT_INIT(value) ((kuid_t){ .val = (value) })
#define KGIDT_INIT(value) ((kgid_t){ .val = (value) })

/* Device number */
typedef u32                 dev_t;

/* Inode number */
typedef unsigned long       ino_t;

/* File mode for open */
typedef unsigned int        fmode_t;

/* GFP flags for memory allocation */
typedef unsigned int        gfp_t;

/* Atomic types */
typedef struct { int counter; } atomic_t;
typedef struct { long counter; } atomic_long_t;
typedef struct { s64 counter; } atomic64_t;

/* Boolean */
#ifndef __cplusplus
typedef _Bool               bool;
#ifndef true
#define true                1
#endif
#ifndef false
#define false               0
#endif
#endif

/* NULL pointer */
#ifndef NULL
#define NULL                ((void *)0)
#endif

/* Pointer-sized integer */
typedef unsigned long       uintptr_t;
typedef signed long         intptr_t;

/* Char types for explicit signedness */
typedef signed char         __s8;
typedef unsigned char       __u8;
typedef signed short        __s16;
typedef unsigned short      __u16;
typedef signed int          __s32;
typedef unsigned int        __u32;
typedef signed long long    __s64;
typedef unsigned long long  __u64;

/* Big/little endian types */
typedef __u16 __le16;
typedef __u32 __le32;
typedef __u64 __le64;
typedef __u16 __be16;
typedef __u32 __be32;
typedef __u64 __be64;

/* Sum type for checksums */
typedef __u32 __wsum;

/* Physical/DMA addresses */
typedef unsigned long       phys_addr_t;
typedef u64                 dma_addr_t;
typedef u64                 resource_size_t;

/* PID type */
typedef int                 pid_t;

/* User ID type */
typedef unsigned int        uid_t;
typedef unsigned int        gid_t;

/* Wide character */
typedef unsigned short      wchar_t;

/* Printf format checking helpers */
typedef long long           __kernel_loff_t;
typedef unsigned int        __kernel_mode_t;
typedef long                __kernel_off_t;
typedef long                __kernel_long_t;

/* UID/GID helper functions (defined after uid_t/gid_t/bool) */
static inline kuid_t make_kuid(struct user_namespace *ns, uid_t uid)
{
    return (kuid_t){ .val = (int)uid };
}

static inline kgid_t make_kgid(struct user_namespace *ns, gid_t gid)
{
    return (kgid_t){ .val = (int)gid };
}

static inline uid_t from_kuid(struct user_namespace *ns, kuid_t kuid)
{
    return (uid_t)kuid.val;
}

static inline gid_t from_kgid(struct user_namespace *ns, kgid_t kgid)
{
    return (gid_t)kgid.val;
}

static inline uid_t from_kuid_munged(struct user_namespace *ns, kuid_t kuid)
{
    return (uid_t)kuid.val;
}

static inline gid_t from_kgid_munged(struct user_namespace *ns, kgid_t kgid)
{
    return (gid_t)kgid.val;
}

static inline bool uid_eq(kuid_t left, kuid_t right)
{
    return left.val == right.val;
}

static inline bool gid_eq(kgid_t left, kgid_t right)
{
    return left.val == right.val;
}

static inline bool uid_valid(kuid_t uid)
{
    return uid.val != -1;
}

static inline bool gid_valid(kgid_t gid)
{
    return gid.val != -1;
}

/* Current user/group/umask helpers (simplified - returns root for now) */
static inline kuid_t current_uid(void)
{
    return GLOBAL_ROOT_UID;
}

static inline kgid_t current_gid(void)
{
    return GLOBAL_ROOT_GID;
}

static inline kuid_t current_fsuid(void)
{
    return GLOBAL_ROOT_UID;
}

static inline kgid_t current_fsgid(void)
{
    return GLOBAL_ROOT_GID;
}

static inline umode_t current_umask(void)
{
    return 0022; /* Default umask */
}

static inline struct user_namespace *current_user_ns(void)
{
    return &init_user_ns;
}

/* VFS UID/GID helper functions */
static inline vfsuid_t make_vfsuid(struct user_namespace *mnt_userns,
                                   struct user_namespace *fs_userns,
                                   kuid_t kuid)
{
    return (vfsuid_t){ .val = kuid };
}

static inline vfsgid_t make_vfsgid(struct user_namespace *mnt_userns,
                                   struct user_namespace *fs_userns,
                                   kgid_t kgid)
{
    return (vfsgid_t){ .val = kgid };
}

static inline kuid_t vfsuid_into_kuid(vfsuid_t vfsuid)
{
    return vfsuid.val;
}

static inline kgid_t vfsgid_into_kgid(vfsgid_t vfsgid)
{
    return vfsgid.val;
}

static inline bool vfsuid_valid(vfsuid_t vfsuid)
{
    return uid_valid(vfsuid.val);
}

static inline bool vfsgid_valid(vfsgid_t vfsgid)
{
    return gid_valid(vfsgid.val);
}

static inline bool vfsuid_eq_kuid(vfsuid_t vfsuid, kuid_t kuid)
{
    return uid_eq(vfsuid.val, kuid);
}

static inline bool vfsgid_eq_kgid(vfsgid_t vfsgid, kgid_t kgid)
{
    return gid_eq(vfsgid.val, kgid);
}

/* from_vfsuid/from_vfsgid - convert vfs*id to kuid/kgid */
struct mnt_idmap;
static inline kuid_t from_vfsuid(struct mnt_idmap *idmap,
                                 struct user_namespace *fs_userns,
                                 vfsuid_t vfsuid)
{
    return vfsuid.val;
}

static inline kgid_t from_vfsgid(struct mnt_idmap *idmap,
                                 struct user_namespace *fs_userns,
                                 vfsgid_t vfsgid)
{
    return vfsgid.val;
}

#endif /* _LINUX_TYPES_H */
