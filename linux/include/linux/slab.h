/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_SLAB_H
#define _LINUX_SLAB_H

#include <linux/types.h>
#include <linux/gfp.h>

/* Slab cache structure (opaque) */
struct kmem_cache;

/* Memory allocation flags */
#define SLAB_HWCACHE_ALIGN      0x00002000U
#define SLAB_CACHE_DMA          0x00004000U
#define SLAB_PANIC              0x00040000U
#define SLAB_RECLAIM_ACCOUNT    0x00020000U
#define SLAB_MEM_SPREAD         0x00100000U
#define SLAB_ACCOUNT            0x00400000U

/* Basic allocators (implemented in Rust) */
extern void *kmalloc(size_t size, gfp_t flags);
extern void *kzalloc(size_t size, gfp_t flags);
extern void *kcalloc(size_t n, size_t size, gfp_t flags);
extern void *krealloc(const void *p, size_t new_size, gfp_t flags);
extern void kfree(const void *ptr);
extern size_t ksize(const void *ptr);

/* Vmalloc variants */
extern void *vmalloc(size_t size);
extern void *vzalloc(size_t size);
extern void vfree(const void *addr);
extern void *kvmalloc(size_t size, gfp_t flags);
extern void kvfree(const void *addr);

static inline void *kvzalloc(size_t size, gfp_t flags)
{
    return kvmalloc(size, flags | __GFP_ZERO);
}

/* Array allocators */
static inline void *kmalloc_array(size_t n, size_t size, gfp_t flags)
{
    /* TODO: overflow check */
    return kmalloc(n * size, flags);
}

static inline void *kcalloc_node(size_t n, size_t size, gfp_t flags, int node)
{
    return kcalloc(n, size, flags);
}

/* Slab cache operations (implemented in Rust) */
extern struct kmem_cache *kmem_cache_create(const char *name, unsigned int size,
                                            unsigned int align, unsigned int flags,
                                            void (*ctor)(void *));
extern void kmem_cache_destroy(struct kmem_cache *s);
extern void *kmem_cache_alloc(struct kmem_cache *s, gfp_t flags);
extern void kmem_cache_free(struct kmem_cache *s, void *obj);
extern unsigned int kmem_cache_size(struct kmem_cache *s);

/* Convenience macros */
#define KMEM_CACHE(__struct, __flags) \
    kmem_cache_create(#__struct, sizeof(struct __struct), \
                      __alignof__(struct __struct), (__flags), NULL)

/* Memory zeroing */
static inline void *kzalloc_node(size_t size, gfp_t flags, int node)
{
    return kzalloc(size, flags);
}

#endif /* _LINUX_SLAB_H */
