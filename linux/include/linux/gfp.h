/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_GFP_H
#define _LINUX_GFP_H

#include <linux/types.h>

/* GFP bitmasks */
#define ___GFP_DMA              0x01u
#define ___GFP_HIGHMEM          0x02u
#define ___GFP_DMA32            0x04u
#define ___GFP_MOVABLE          0x08u
#define ___GFP_RECLAIMABLE      0x10u
#define ___GFP_HIGH             0x20u
#define ___GFP_IO               0x40u
#define ___GFP_FS               0x80u
#define ___GFP_ZERO             0x100u
#define ___GFP_ATOMIC           0x200u
#define ___GFP_DIRECT_RECLAIM   0x400u
#define ___GFP_KSWAPD_RECLAIM   0x800u
#define ___GFP_NOWARN           0x2000u
#define ___GFP_RETRY_MAYFAIL    0x4000u
#define ___GFP_NOFAIL           0x8000u
#define ___GFP_NORETRY          0x10000u
#define ___GFP_HARDWALL         0x100000u
#define ___GFP_COMP             0x4000000u
#define ___GFP_NOMEMALLOC       0x1000000u

/* Type modifiers */
#define __GFP_DMA               (___GFP_DMA)
#define __GFP_HIGHMEM           (___GFP_HIGHMEM)
#define __GFP_DMA32             (___GFP_DMA32)
#define __GFP_MOVABLE           (___GFP_MOVABLE)

/* Action modifiers */
#define __GFP_ZERO              (___GFP_ZERO)
#define __GFP_ATOMIC            (___GFP_ATOMIC)
#define __GFP_HIGH              (___GFP_HIGH)
#define __GFP_IO                (___GFP_IO)
#define __GFP_FS                (___GFP_FS)
#define __GFP_NOWARN            (___GFP_NOWARN)
#define __GFP_RETRY_MAYFAIL     (___GFP_RETRY_MAYFAIL)
#define __GFP_NOFAIL            (___GFP_NOFAIL)
#define __GFP_NORETRY           (___GFP_NORETRY)
#define __GFP_DIRECT_RECLAIM    (___GFP_DIRECT_RECLAIM)
#define __GFP_KSWAPD_RECLAIM    (___GFP_KSWAPD_RECLAIM)
#define __GFP_RECLAIM           (__GFP_DIRECT_RECLAIM | __GFP_KSWAPD_RECLAIM)
#define __GFP_COMP              (___GFP_COMP)
#define __GFP_HARDWALL          (___GFP_HARDWALL)
#define __GFP_RECLAIMABLE       (___GFP_RECLAIMABLE)

/* Common combinations */
#define GFP_ATOMIC              (__GFP_HIGH | __GFP_ATOMIC | __GFP_KSWAPD_RECLAIM)
#define GFP_KERNEL              (__GFP_RECLAIM | __GFP_IO | __GFP_FS)
#define GFP_KERNEL_ACCOUNT      (GFP_KERNEL)
#define GFP_NOWAIT              (__GFP_KSWAPD_RECLAIM)
#define GFP_NOIO                (__GFP_RECLAIM)
#define GFP_NOFS                (__GFP_RECLAIM | __GFP_IO)
#define GFP_USER                (GFP_KERNEL | __GFP_HARDWALL)
#define GFP_DMA                 (__GFP_DMA)
#define GFP_DMA32               (__GFP_DMA32)
#define GFP_HIGHUSER            (GFP_USER | __GFP_HIGHMEM)
#define GFP_HIGHUSER_MOVABLE    (GFP_HIGHUSER | __GFP_MOVABLE)

/* Allocation priority */
static inline bool gfpflags_allow_blocking(const gfp_t gfp_flags)
{
    return !!(gfp_flags & __GFP_DIRECT_RECLAIM);
}

static inline bool gfpflags_normal_context(const gfp_t gfp_flags)
{
    return (gfp_flags & (__GFP_DIRECT_RECLAIM | __GFP_IO | __GFP_FS)) ==
           (__GFP_DIRECT_RECLAIM | __GFP_IO | __GFP_FS);
}

#endif /* _LINUX_GFP_H */
