/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_PAGEMAP_H
#define _LINUX_PAGEMAP_H

#include <linux/types.h>
#include <linux/fs.h>
#include <linux/list.h>
#include <linux/gfp.h>
#include <linux/mm.h>

/* Page flags */
#define PAGE_SIZE       4096
#define PAGE_SHIFT      12
#define PAGE_MASK       (~(PAGE_SIZE - 1))

/* Forward declarations */
struct buffer_head;

/* Block callback type - must be defined before use */
typedef int (get_block_t)(struct inode *inode, sector_t iblock, struct buffer_head *bh, int create);

/* Radix tree root (simplified) */
struct radix_tree_root {
    void *rnode;
};

/* Migrate mode */
enum migrate_mode {
    MIGRATE_ASYNC,
    MIGRATE_SYNC_LIGHT,
    MIGRATE_SYNC,
};

/* Folio/page abstraction */
struct page {
    unsigned long flags;
    atomic_t _refcount;
    atomic_t _mapcount;
    struct list_head lru;
    struct address_space *mapping;
    unsigned long index;
    unsigned long private;
    void *virtual;
};

struct folio {
    struct page page;
};

/* Address space is defined in fs.h */

/* Address space operations */
struct address_space_operations {
    int (*writepage)(struct page *page, struct writeback_control *wbc);
    int (*read_folio)(struct file *, struct folio *);
    int (*writepages)(struct address_space *, struct writeback_control *);
    bool (*dirty_folio)(struct address_space *, struct folio *);
    void (*readahead)(struct readahead_control *);
    int (*write_begin)(struct file *, struct address_space *mapping,
                       loff_t pos, unsigned len, struct page **pagep, void **fsdata);
    int (*write_end)(struct file *, struct address_space *mapping,
                     loff_t pos, unsigned len, unsigned copied,
                     struct page *page, void *fsdata);
    sector_t (*bmap)(struct address_space *, sector_t);
    void (*invalidate_folio)(struct folio *, size_t offset, size_t len);
    bool (*release_folio)(struct folio *, gfp_t);
    void (*free_folio)(struct folio *);
    ssize_t (*direct_IO)(struct kiocb *, struct iov_iter *iter);
    int (*migrate_folio)(struct address_space *, struct folio *dst, struct folio *src, enum migrate_mode);
    int (*launder_folio)(struct folio *);
    bool (*is_partially_uptodate)(struct folio *, size_t from, size_t count);
    void (*is_dirty_writeback)(struct folio *, bool *, bool *);
    int (*error_remove_page)(struct address_space *, struct page *);
    int (*swap_activate)(struct swap_info_struct *, struct file *, sector_t *);
    void (*swap_deactivate)(struct file *);
    int (*swap_rw)(struct kiocb *, struct iov_iter *);
};

/* Writeback sync modes */
enum writeback_sync_modes {
    WB_SYNC_NONE,       /* Don't wait for any writeback */
    WB_SYNC_ALL,        /* Wait on every mapping */
};

/* Writeback control */
struct writeback_control {
    long nr_to_write;
    long pages_skipped;
    loff_t range_start;
    loff_t range_end;
    unsigned for_kupdate:1;
    unsigned for_background:1;
    unsigned tagged_writepages:1;
    unsigned for_reclaim:1;
    unsigned range_cyclic:1;
    unsigned for_sync:1;
    unsigned unpinned_fscache_wb:1;
    unsigned no_cgroup_owner:1;
    enum writeback_sync_modes sync_mode;
};

/* Readahead control */
struct readahead_control {
    struct file *file;
    struct address_space *mapping;
    pgoff_t _index;
    unsigned int _nr_pages;
    unsigned int _batch_count;
};

/* Page cache operations (implemented in Rust) */
extern struct page *find_get_page(struct address_space *mapping, pgoff_t offset);
extern struct page *find_lock_page(struct address_space *mapping, pgoff_t offset);
extern struct page *find_or_create_page(struct address_space *mapping, pgoff_t index, gfp_t gfp_mask);
extern struct page *grab_cache_page_write_begin(struct address_space *mapping, pgoff_t index, unsigned flags);
extern void unlock_page(struct page *page);
extern void put_page(struct page *page);
extern void get_page(struct page *page);
extern int add_to_page_cache_lru(struct page *page, struct address_space *mapping, pgoff_t index, gfp_t gfp);
extern void delete_from_page_cache(struct page *page);

/* Page locking */
extern void lock_page(struct page *page);
extern int trylock_page(struct page *page);
extern void wait_on_page_locked(struct page *page);

/* Page state */
static inline int PageUptodate(struct page *page)
{
    return (page->flags & (1UL << 0)) != 0;
}

static inline void SetPageUptodate(struct page *page)
{
    page->flags |= (1UL << 0);
}

static inline void ClearPageUptodate(struct page *page)
{
    page->flags &= ~(1UL << 0);
}

static inline int PageDirty(struct page *page)
{
    return (page->flags & (1UL << 1)) != 0;
}

static inline void SetPageDirty(struct page *page)
{
    page->flags |= (1UL << 1);
}

static inline int PageLocked(struct page *page)
{
    return (page->flags & (1UL << 2)) != 0;
}

static inline int PageError(struct page *page)
{
    return (page->flags & (1UL << 3)) != 0;
}

static inline void SetPageError(struct page *page)
{
    page->flags |= (1UL << 3);
}

/* Folio helpers */
static inline struct page *folio_page(struct folio *folio, size_t n)
{
    return &folio->page + n;
}

static inline struct folio *page_folio(struct page *page)
{
    return (struct folio *)page;
}

static inline void *folio_address(struct folio *folio)
{
    return folio->page.virtual;
}

/* Block mapping helpers */
extern int block_read_full_folio(struct folio *, get_block_t *);
extern int block_write_full_page(struct page *, get_block_t *, struct writeback_control *);
extern int block_write_begin(struct address_space *, loff_t, unsigned, struct page **, get_block_t *);
extern int block_write_end(struct file *, struct address_space *, loff_t, unsigned, unsigned, struct page *, void *);
extern int generic_write_end(struct file *, struct address_space *, loff_t, unsigned, unsigned, struct page *, void *);
extern sector_t generic_block_bmap(struct address_space *, sector_t, get_block_t *);

/* Folio helpers for block devices */
extern bool block_dirty_folio(struct address_space *mapping, struct folio *folio);
extern void block_invalidate_folio(struct folio *folio, size_t offset, size_t length);
extern int buffer_migrate_folio(struct address_space *mapping, struct folio *dst, struct folio *src, enum migrate_mode mode);
extern int buffer_migrate_folio_norefs(struct address_space *mapping, struct folio *dst, struct folio *src, enum migrate_mode mode);

/* Truncation */
extern void truncate_inode_pages(struct address_space *mapping, loff_t lstart);
extern void truncate_inode_pages_final(struct address_space *mapping);
extern void truncate_inode_pages_range(struct address_space *mapping, loff_t lstart, loff_t lend);
extern void truncate_pagecache(struct inode *inode, loff_t newsize);
extern void truncate_setsize(struct inode *inode, loff_t newsize);

/* Invalidation */
extern int invalidate_inode_pages2(struct address_space *mapping);
extern int invalidate_inode_pages2_range(struct address_space *mapping, pgoff_t start, pgoff_t end);

/* Readahead */
extern void page_cache_readahead_unbounded(struct address_space *mapping,
                                           struct file *file, pgoff_t index,
                                           unsigned long nr_to_read,
                                           unsigned long lookahead_size);

/* Filemap functions */
extern ssize_t generic_file_buffered_read(struct kiocb *iocb, struct iov_iter *to, ssize_t already_read);
extern ssize_t filemap_read(struct kiocb *iocb, struct iov_iter *to, ssize_t already_read);
extern int filemap_fault(struct vm_fault *vmf);
extern vm_fault_t filemap_map_pages(struct vm_fault *vmf, pgoff_t start_pgoff, pgoff_t end_pgoff);

#endif /* _LINUX_PAGEMAP_H */
