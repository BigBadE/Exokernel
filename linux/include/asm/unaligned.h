/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _ASM_UNALIGNED_H
#define _ASM_UNALIGNED_H

#include <linux/types.h>

/*
 * Unaligned memory access helpers.
 * On x86, unaligned access is supported by hardware, but we
 * still provide these for portability.
 */

/* Little-endian get functions */
static inline u16 get_unaligned_le16(const void *p)
{
    const u8 *ptr = (const u8 *)p;
    return ptr[0] | (ptr[1] << 8);
}

static inline u32 get_unaligned_le32(const void *p)
{
    const u8 *ptr = (const u8 *)p;
    return ptr[0] | (ptr[1] << 8) | (ptr[2] << 16) | (ptr[3] << 24);
}

static inline u64 get_unaligned_le64(const void *p)
{
    const u8 *ptr = (const u8 *)p;
    return (u64)get_unaligned_le32(ptr) |
           ((u64)get_unaligned_le32(ptr + 4) << 32);
}

/* Big-endian get functions */
static inline u16 get_unaligned_be16(const void *p)
{
    const u8 *ptr = (const u8 *)p;
    return (ptr[0] << 8) | ptr[1];
}

static inline u32 get_unaligned_be32(const void *p)
{
    const u8 *ptr = (const u8 *)p;
    return (ptr[0] << 24) | (ptr[1] << 16) | (ptr[2] << 8) | ptr[3];
}

static inline u64 get_unaligned_be64(const void *p)
{
    const u8 *ptr = (const u8 *)p;
    return ((u64)get_unaligned_be32(ptr) << 32) |
           (u64)get_unaligned_be32(ptr + 4);
}

/* Little-endian put functions */
static inline void put_unaligned_le16(u16 val, void *p)
{
    u8 *ptr = (u8 *)p;
    ptr[0] = val;
    ptr[1] = val >> 8;
}

static inline void put_unaligned_le32(u32 val, void *p)
{
    u8 *ptr = (u8 *)p;
    ptr[0] = val;
    ptr[1] = val >> 8;
    ptr[2] = val >> 16;
    ptr[3] = val >> 24;
}

static inline void put_unaligned_le64(u64 val, void *p)
{
    put_unaligned_le32(val, p);
    put_unaligned_le32(val >> 32, (u8 *)p + 4);
}

/* Big-endian put functions */
static inline void put_unaligned_be16(u16 val, void *p)
{
    u8 *ptr = (u8 *)p;
    ptr[0] = val >> 8;
    ptr[1] = val;
}

static inline void put_unaligned_be32(u32 val, void *p)
{
    u8 *ptr = (u8 *)p;
    ptr[0] = val >> 24;
    ptr[1] = val >> 16;
    ptr[2] = val >> 8;
    ptr[3] = val;
}

static inline void put_unaligned_be64(u64 val, void *p)
{
    put_unaligned_be32(val >> 32, p);
    put_unaligned_be32(val, (u8 *)p + 4);
}

/* Generic (native endian) versions - on x86 just cast */
static inline u16 get_unaligned(const u16 *p)
{
    return *p;
}

static inline u32 get_unaligned32(const u32 *p)
{
    return *p;
}

static inline u64 get_unaligned64(const u64 *p)
{
    return *p;
}

static inline void put_unaligned(u16 val, u16 *p)
{
    *p = val;
}

static inline void put_unaligned32(u32 val, u32 *p)
{
    *p = val;
}

static inline void put_unaligned64(u64 val, u64 *p)
{
    *p = val;
}

#endif /* _ASM_UNALIGNED_H */
