/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_BACKING_DEV_H
#define _LINUX_BACKING_DEV_H

#include <linux/backing-dev-defs.h>

/* Backing device info operations */
extern int bdi_init(struct backing_dev_info *bdi);
extern void bdi_destroy(struct backing_dev_info *bdi);
extern int bdi_register(struct backing_dev_info *bdi, const char *fmt, ...);
extern void bdi_unregister(struct backing_dev_info *bdi);
extern void bdi_put(struct backing_dev_info *bdi);

/* Get the backing_dev_info for a block device */
static inline struct backing_dev_info *bdev_get_bdi(struct block_device *bdev)
{
    return NULL; /* TODO: implement */
}

/* Writeback control */
extern void wbc_attach_and_unlock_inode(struct writeback_control *wbc, struct inode *inode);
extern void wbc_detach_inode(struct writeback_control *wbc);

/* Congestion */
static inline bool inode_cgwb_enabled(struct inode *inode)
{
    return false;
}

static inline bool bdi_cap_account_dirty(struct backing_dev_info *bdi)
{
    return true;
}

#endif /* _LINUX_BACKING_DEV_H */
