/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _ASM_GENERIC_INT_LL64_H
#define _ASM_GENERIC_INT_LL64_H

/*
 * Integer type definitions for architectures where:
 * - char is 8 bits
 * - short is 16 bits
 * - int is 32 bits
 * - long long is 64 bits
 * - long is word size (32 or 64 bits)
 */

#ifndef __ASSEMBLY__

typedef signed char         __s8;
typedef unsigned char       __u8;

typedef signed short        __s16;
typedef unsigned short      __u16;

typedef signed int          __s32;
typedef unsigned int        __u32;

typedef signed long long    __s64;
typedef unsigned long long  __u64;

#endif /* __ASSEMBLY__ */

#endif /* _ASM_GENERIC_INT_LL64_H */
