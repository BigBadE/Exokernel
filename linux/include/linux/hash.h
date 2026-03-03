/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_HASH_H
#define _LINUX_HASH_H

#include <linux/types.h>

/* Golden ratio prime numbers for hashing */
#define GOLDEN_RATIO_32 0x61C88647
#define GOLDEN_RATIO_64 0x61C8864680B583EBull

#ifdef CONFIG_64BIT
#define GOLDEN_RATIO_PRIME GOLDEN_RATIO_64
#else
#define GOLDEN_RATIO_PRIME GOLDEN_RATIO_32
#endif

/* Hash a 64-bit value down to specified bits */
static inline u32 hash_64(u64 val, unsigned int bits)
{
    return (u32)(val * GOLDEN_RATIO_64) >> (64 - bits);
}

/* Hash a 32-bit value down to specified bits */
static inline u32 hash_32(u32 val, unsigned int bits)
{
    return (val * GOLDEN_RATIO_32) >> (32 - bits);
}

/* Hash a pointer down to specified bits */
static inline u32 hash_ptr(const void *ptr, unsigned int bits)
{
#ifdef CONFIG_64BIT
    return hash_64((unsigned long)ptr, bits);
#else
    return hash_32((unsigned long)ptr, bits);
#endif
}

/* Hash a long value */
static inline unsigned long hash_long(unsigned long val, unsigned int bits)
{
#ifdef CONFIG_64BIT
    return hash_64(val, bits);
#else
    return hash_32(val, bits);
#endif
}

/* Full hash without bit reduction */
static inline u32 __hash_32(u32 val)
{
    return val * GOLDEN_RATIO_32;
}

/* String hash init */
#define HASH_INIT 0

/* Partial string hash */
static inline unsigned long partial_name_hash(unsigned long c, unsigned long prevhash)
{
    return (prevhash + (c << 4) + (c >> 4)) * 11;
}

/* End string hash */
static inline unsigned long end_name_hash(unsigned long hash)
{
    return hash;
}

/* Full string hash */
static inline unsigned int full_name_hash(const void *salt, const char *name, unsigned int len)
{
    unsigned long hash = (unsigned long)salt;
    while (len--)
        hash = partial_name_hash(*name++, hash);
    return end_name_hash(hash);
}

#endif /* _LINUX_HASH_H */
