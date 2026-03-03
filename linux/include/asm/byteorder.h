/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _ASM_BYTEORDER_H
#define _ASM_BYTEORDER_H

#include <linux/types.h>
#include <asm/ioctl.h>

/* Little-endian byte order (x86_64 is little-endian) */
#define __LITTLE_ENDIAN 1234
#define __BIG_ENDIAN    4321
#define __BYTE_ORDER    __LITTLE_ENDIAN

/* Byte swapping macros */
#define __swab16(x) ((u16)(                             \
    (((u16)(x) & (u16)0x00ffU) << 8) |                 \
    (((u16)(x) & (u16)0xff00U) >> 8)))

#define __swab32(x) ((u32)(                             \
    (((u32)(x) & (u32)0x000000ffUL) << 24) |           \
    (((u32)(x) & (u32)0x0000ff00UL) <<  8) |           \
    (((u32)(x) & (u32)0x00ff0000UL) >>  8) |           \
    (((u32)(x) & (u32)0xff000000UL) >> 24)))

#define __swab64(x) ((u64)(                             \
    (((u64)(x) & (u64)0x00000000000000ffULL) << 56) |  \
    (((u64)(x) & (u64)0x000000000000ff00ULL) << 40) |  \
    (((u64)(x) & (u64)0x0000000000ff0000ULL) << 24) |  \
    (((u64)(x) & (u64)0x00000000ff000000ULL) <<  8) |  \
    (((u64)(x) & (u64)0x000000ff00000000ULL) >>  8) |  \
    (((u64)(x) & (u64)0x0000ff0000000000ULL) >> 24) |  \
    (((u64)(x) & (u64)0x00ff000000000000ULL) >> 40) |  \
    (((u64)(x) & (u64)0xff00000000000000ULL) >> 56)))

/* CPU to little-endian conversions (no-op on little-endian x86_64) */
#define cpu_to_le16(x) ((__le16)(u16)(x))
#define cpu_to_le32(x) ((__le32)(u32)(x))
#define cpu_to_le64(x) ((__le64)(u64)(x))

#define le16_to_cpu(x) ((u16)(__le16)(x))
#define le32_to_cpu(x) ((u32)(__le32)(x))
#define le64_to_cpu(x) ((u64)(__le64)(x))

/* CPU to big-endian conversions (swap on little-endian x86_64) */
#define cpu_to_be16(x) ((__be16)__swab16(x))
#define cpu_to_be32(x) ((__be32)__swab32(x))
#define cpu_to_be64(x) ((__be64)__swab64(x))

#define be16_to_cpu(x) __swab16((u16)(__be16)(x))
#define be32_to_cpu(x) __swab32((u32)(__be32)(x))
#define be64_to_cpu(x) __swab64((u64)(__be64)(x))

/* Pointer versions */
#define cpu_to_le16p(x) cpu_to_le16(*(x))
#define cpu_to_le32p(x) cpu_to_le32(*(x))
#define cpu_to_le64p(x) cpu_to_le64(*(x))

#define le16_to_cpup(x) le16_to_cpu(*(x))
#define le32_to_cpup(x) le32_to_cpu(*(x))
#define le64_to_cpup(x) le64_to_cpu(*(x))

#define cpu_to_be16p(x) cpu_to_be16(*(x))
#define cpu_to_be32p(x) cpu_to_be32(*(x))
#define cpu_to_be64p(x) cpu_to_be64(*(x))

#define be16_to_cpup(x) be16_to_cpu(*(x))
#define be32_to_cpup(x) be32_to_cpu(*(x))
#define be64_to_cpup(x) be64_to_cpu(*(x))

/* In-place swap versions */
#define cpu_to_le16s(x) do { } while (0)
#define cpu_to_le32s(x) do { } while (0)
#define cpu_to_le64s(x) do { } while (0)

#define le16_to_cpus(x) do { } while (0)
#define le32_to_cpus(x) do { } while (0)
#define le64_to_cpus(x) do { } while (0)

#define cpu_to_be16s(x) do { *(x) = cpu_to_be16(*(x)); } while (0)
#define cpu_to_be32s(x) do { *(x) = cpu_to_be32(*(x)); } while (0)
#define cpu_to_be64s(x) do { *(x) = cpu_to_be64(*(x)); } while (0)

#define be16_to_cpus(x) cpu_to_be16s(x)
#define be32_to_cpus(x) cpu_to_be32s(x)
#define be64_to_cpus(x) cpu_to_be64s(x)

#endif /* _ASM_BYTEORDER_H */
