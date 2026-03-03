/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_BACKING_DEV_DEFS_H
#define _LINUX_BACKING_DEV_DEFS_H

#include <linux/types.h>
#include <linux/list.h>
#include <linux/spinlock.h>

/* Forward declarations */
struct page;
struct device;
struct dentry;

/* Backing device capabilities */
#define BDI_CAP_NO_ACCT_DIRTY   0x00000001
#define BDI_CAP_NO_WRITEBACK    0x00000002
#define BDI_CAP_NO_ACCT_WB      0x00000004
#define BDI_CAP_STABLE_WRITES   0x00000008
#define BDI_CAP_STRICTLIMIT     0x00000010
#define BDI_CAP_CGROUP_WRITEBACK 0x00000020
#define BDI_CAP_WRITEBACK_ACCT  0x00000040

/* Backing device info structure */
struct backing_dev_info {
    u64 id;
    unsigned int capabilities;
    unsigned int min_ratio;
    unsigned int max_ratio;
    unsigned int max_prop_frac;
    atomic_long_t tot_write_bandwidth;
    struct list_head bdi_list;
    unsigned long ra_pages;
    unsigned long io_pages;
    char *name;
    struct device *dev;
    struct device *owner;
    struct dentry *debug_dir;
};

/* Writeback state */
enum wb_state {
    WB_registered,
    WB_writeback_running,
    WB_has_dirty_io,
    WB_start_all,
};

/* Default backing device info */
extern struct backing_dev_info noop_backing_dev_info;

/* Read-ahead defaults */
#define VM_READAHEAD_PAGES  (128 * 1024 / PAGE_SIZE)

#endif /* _LINUX_BACKING_DEV_DEFS_H */
