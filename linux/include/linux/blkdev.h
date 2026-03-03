/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_BLKDEV_H
#define _LINUX_BLKDEV_H

#include <linux/types.h>
#include <linux/gfp.h>
#include <linux/list.h>
#include <linux/spinlock.h>
#include <asm/ioctl.h>

/* Forward declarations */
struct super_block;
struct request_queue;
struct bio;
struct bio_vec;
struct gendisk;
struct blk_zone;
struct hd_geometry;

/* Callback for report_zones */
typedef int (*report_zones_cb)(struct blk_zone *zone, unsigned int idx, void *data);

/* Block device structure */
struct block_device {
    dev_t                   bd_dev;
    struct inode            *bd_inode;
    struct super_block      *bd_super;
    void                    *bd_holder;
    unsigned                bd_holders;
    const struct block_device_operations *bd_disk;
    struct request_queue    *bd_queue;
    void                    *bd_private;
    sector_t                bd_nr_sectors;
    sector_t                bd_start_sect;
    unsigned                bd_read_only;
    u8                      bd_partno;
    spinlock_t              bd_size_lock;
};

/* Block device operations */
struct block_device_operations {
    void (*submit_bio)(struct bio *bio);
    int (*open)(struct gendisk *disk, fmode_t mode);
    void (*release)(struct gendisk *disk);
    int (*ioctl)(struct block_device *bdev, fmode_t mode, unsigned int cmd, unsigned long arg);
    int (*compat_ioctl)(struct block_device *bdev, fmode_t mode, unsigned int cmd, unsigned long arg);
    unsigned int (*check_events)(struct gendisk *disk, unsigned int clearing);
    void (*unlock_native_capacity)(struct gendisk *disk);
    int (*getgeo)(struct block_device *, struct hd_geometry *);
    int (*set_read_only)(struct block_device *bdev, bool ro);
    void (*free_disk)(struct gendisk *disk);
    void (*swap_slot_free_notify)(struct block_device *, unsigned long);
    int (*report_zones)(struct gendisk *, sector_t sector, unsigned int nr_zones, report_zones_cb cb, void *data);
    char *(*devnode)(struct gendisk *disk, umode_t *mode);
    int (*alternative_gpt_sector)(struct gendisk *disk, sector_t *sector);
    struct module *owner;
    const struct pr_ops *pr_ops;
};

/* Generic disk structure */
struct gendisk {
    int major;
    int first_minor;
    int minors;
    char disk_name[32];
    struct block_device *part0;
    const struct block_device_operations *fops;
    struct request_queue *queue;
    void *private_data;
    sector_t capacity;
    int flags;
};

/* Request queue (opaque for now) */
struct request_queue {
    void *queuedata;
};

/* Block plug for batching I/O */
struct blk_plug {
    struct list_head mq_list;
    struct list_head cb_list;
    unsigned short rq_count;
    bool multiple_queues;
    bool nowait;
};

/* Plug/unplug functions */
static inline void blk_start_plug(struct blk_plug *plug)
{
    plug->rq_count = 0;
    plug->multiple_queues = false;
    plug->nowait = false;
}

static inline void blk_finish_plug(struct blk_plug *plug)
{
    /* No-op for now */
}

/* Geometry info */
struct hd_geometry {
    unsigned char heads;
    unsigned char sectors;
    unsigned short cylinders;
    unsigned long start;
};

/* Block device operations (implemented in Rust) */
extern sector_t bdev_nr_sectors(struct block_device *bdev);
extern int bdev_read_only(struct block_device *bdev);
extern struct block_device *blkdev_get_by_path(const char *path, fmode_t mode, void *holder);
extern void blkdev_put(struct block_device *bdev, fmode_t mode);
extern int sync_blockdev(struct block_device *bdev);
extern int sync_blockdev_nowait(struct block_device *bdev);
extern struct super_block *freeze_bdev(struct block_device *bdev);
extern int thaw_bdev(struct block_device *bdev);

/* Inline helpers */
static inline sector_t get_capacity(struct gendisk *disk)
{
    return disk->capacity;
}

static inline void set_capacity(struct gendisk *disk, sector_t capacity)
{
    disk->capacity = capacity;
}

static inline sector_t bdev_get_queue(struct block_device *bdev)
{
    return (sector_t)bdev->bd_queue;
}

static inline int bdev_is_partition(struct block_device *bdev)
{
    return bdev->bd_partno != 0;
}

static inline struct block_device *bdev_whole(struct block_device *bdev)
{
    /* For simplicity, return self if not a partition */
    return bdev;
}

/* Sector size helpers */
static inline unsigned int bdev_logical_block_size(struct block_device *bdev)
{
    return 512; /* Default to 512 bytes */
}

static inline unsigned int bdev_physical_block_size(struct block_device *bdev)
{
    return 512;
}

/* Discard support */
extern unsigned int bdev_max_discard_sectors(struct block_device *bdev);
extern int blk_queue_discard(struct request_queue *q);
extern int blkdev_issue_discard(struct block_device *bdev, sector_t sector,
                                sector_t nr_sects, gfp_t gfp_mask);
extern int blkdev_issue_flush(struct block_device *bdev);

/* Invalidation */
extern void invalidate_bdev(struct block_device *bdev);
extern int invalidate_inodes(struct block_device *bdev);

/* Max sectors */
static inline unsigned int queue_max_sectors(struct request_queue *q)
{
    return 256; /* 128KB default */
}

/* Alignment */
static inline unsigned int queue_logical_block_size(struct request_queue *q)
{
    return 512;
}

/* FS trim range structure */
struct fstrim_range {
    u64 start;
    u64 len;
    u64 minlen;
};

/* FITRIM ioctl */
#define FITRIM          _IOWR('X', 121, struct fstrim_range)

/* File ioctls */
#define FIFREEZE        _IOWR('X', 119, int)
#define FITHAW          _IOWR('X', 120, int)
#define FIDEDUPERANGE   _IOWR('X', 122, struct file_dedupe_range)
#define FICLONE         _IOW('X', 125, int)
#define FICLONERANGE    _IOW('X', 126, struct file_clone_range)

#endif /* _LINUX_BLKDEV_H */
