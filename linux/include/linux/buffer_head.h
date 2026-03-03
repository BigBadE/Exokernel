/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_BUFFER_HEAD_H
#define _LINUX_BUFFER_HEAD_H

#include <linux/types.h>
#include <linux/list.h>
#include <linux/wait.h>
#include <linux/fs.h>
#include <linux/blkdev.h>

/* Forward declarations */
struct page;
struct address_space;

/* Buffer state bits */
enum bh_state_bits {
    BH_Uptodate,
    BH_Dirty,
    BH_Lock,
    BH_Req,
    BH_Mapped,
    BH_New,
    BH_Async_Read,
    BH_Async_Write,
    BH_Delay,
    BH_Boundary,
    BH_Write_EIO,
    BH_Unwritten,
    BH_Quiet,
    BH_Meta,
    BH_Prio,
    BH_Defer_Completion,
    BH_PrivateStart,
};

/* Buffer head structure */
struct buffer_head {
    unsigned long           b_state;
    struct buffer_head      *b_this_page;
    struct page             *b_page;
    sector_t                b_blocknr;
    size_t                  b_size;
    char                    *b_data;
    struct block_device     *b_bdev;
    void                    (*b_end_io)(struct buffer_head *, int);
    void                    *b_private;
    struct list_head        b_assoc_buffers;
    struct address_space    *b_assoc_map;
    atomic_t                b_count;
    spinlock_t              b_uptodate_lock;
};

/* Buffer state test/set/clear macros */
#define BUFFER_FNS(bit, name) \
static inline int buffer_##name(const struct buffer_head *bh) \
{ return (bh->b_state & (1UL << BH_##bit)) != 0; } \
static inline void set_buffer_##name(struct buffer_head *bh) \
{ bh->b_state |= (1UL << BH_##bit); } \
static inline void clear_buffer_##name(struct buffer_head *bh) \
{ bh->b_state &= ~(1UL << BH_##bit); }

BUFFER_FNS(Uptodate, uptodate)
BUFFER_FNS(Dirty, dirty)
BUFFER_FNS(Lock, locked)
BUFFER_FNS(Req, req)
BUFFER_FNS(Mapped, mapped)
BUFFER_FNS(New, new)
BUFFER_FNS(Async_Read, async_read)
BUFFER_FNS(Async_Write, async_write)
BUFFER_FNS(Delay, delay)
BUFFER_FNS(Boundary, boundary)
BUFFER_FNS(Write_EIO, write_io_error)
BUFFER_FNS(Unwritten, unwritten)

/* Buffer operations (implemented in Rust) */
extern struct buffer_head *__bread_gfp(struct block_device *bdev, sector_t block, unsigned size, gfp_t gfp);
extern void brelse(struct buffer_head *bh);
extern void __brelse(struct buffer_head *bh);
extern void bforget(struct buffer_head *bh);
extern void __bforget(struct buffer_head *bh);
extern struct buffer_head *__getblk_gfp(struct block_device *bdev, sector_t block, unsigned size, gfp_t gfp);
extern void __breadahead(struct block_device *bdev, sector_t block, unsigned size);

/* Convenience wrappers */
static inline struct buffer_head *sb_bread(struct super_block *sb, sector_t block)
{
    return __bread_gfp(sb->s_bdev, block, sb->s_blocksize, GFP_KERNEL);
}

static inline struct buffer_head *sb_bread_unmovable(struct super_block *sb, sector_t block)
{
    return __bread_gfp(sb->s_bdev, block, sb->s_blocksize, GFP_KERNEL);
}

static inline struct buffer_head *sb_getblk(struct super_block *sb, sector_t block)
{
    return __getblk_gfp(sb->s_bdev, block, sb->s_blocksize, GFP_KERNEL);
}

static inline void sb_breadahead(struct super_block *sb, sector_t block)
{
    __breadahead(sb->s_bdev, block, sb->s_blocksize);
}

/* Reference counting */
static inline void get_bh(struct buffer_head *bh)
{
    bh->b_count.counter++;
}

static inline void put_bh(struct buffer_head *bh)
{
    bh->b_count.counter--;
}

/* Lock operations */
extern void lock_buffer(struct buffer_head *bh);
extern void unlock_buffer(struct buffer_head *bh);
extern int trylock_buffer(struct buffer_head *bh);

static inline void wait_on_buffer(struct buffer_head *bh)
{
    /* TODO: implement waiting */
}

/* Mark dirty */
extern void mark_buffer_dirty(struct buffer_head *bh);
extern void mark_buffer_dirty_inode(struct buffer_head *bh, struct inode *inode);

/* Sync operations */
extern int sync_dirty_buffer(struct buffer_head *bh);
extern int __sync_dirty_buffer(struct buffer_head *bh, int op_flags);
extern void write_dirty_buffer(struct buffer_head *bh, int op_flags);

/* Map operations */
extern int set_blocksize(struct block_device *bdev, int size);
extern int sb_set_blocksize(struct super_block *sb, int size);
extern int sb_min_blocksize(struct super_block *sb, int size);

/* Block mapping */
extern void map_bh(struct buffer_head *bh, struct super_block *sb, sector_t block);

/* Inline data access */
static inline char *bh_data(struct buffer_head *bh)
{
    return bh->b_data;
}

/* Allocation/deallocation (implemented in Rust) */
extern struct buffer_head *alloc_buffer_head(gfp_t gfp_flags);
extern void free_buffer_head(struct buffer_head *bh);

#endif /* _LINUX_BUFFER_HEAD_H */
