/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_EXPORTFS_H
#define _LINUX_EXPORTFS_H

#include <linux/types.h>

/* Forward declarations */
struct super_block;
struct dentry;
struct inode;

/* File handle types */
enum fid_type {
    FILEID_ROOT = 0,
    FILEID_INO32_GEN = 1,
    FILEID_INO32_GEN_PARENT = 2,
    FILEID_BTRFS_WITHOUT_PARENT = 0x4d,
    FILEID_BTRFS_WITH_PARENT = 0x4e,
    FILEID_BTRFS_WITH_PARENT_ROOT = 0x4f,
    FILEID_UDF_WITHOUT_PARENT = 0x51,
    FILEID_UDF_WITH_PARENT = 0x52,
    FILEID_NILFS_WITHOUT_PARENT = 0x61,
    FILEID_NILFS_WITH_PARENT = 0x62,
    FILEID_FAT_WITHOUT_PARENT = 0x71,
    FILEID_FAT_WITH_PARENT = 0x72,
    FILEID_LUSTRE = 0x97,
    FILEID_KERNFS = 0xfe,
    FILEID_INVALID = 0xff,
};

/* File ID structure */
struct fid {
    union {
        struct {
            u32 ino;
            u32 gen;
            u32 parent_ino;
            u32 parent_gen;
        } i32;
        struct {
            u64 ino;
            u64 gen;
        } i64;
        __u32 raw[0];
    };
};

/* Export operations */
struct export_operations {
    int (*encode_fh)(struct inode *inode, __u32 *fh, int *max_len,
                     struct inode *parent);
    struct dentry *(*fh_to_dentry)(struct super_block *sb, struct fid *fid,
                                   int fh_len, int fh_type);
    struct dentry *(*fh_to_parent)(struct super_block *sb, struct fid *fid,
                                   int fh_len, int fh_type);
    int (*get_name)(struct dentry *parent, char *name, struct dentry *child);
    struct dentry *(*get_parent)(struct dentry *child);
    int (*commit_metadata)(struct inode *inode);
    int (*get_uuid)(struct super_block *sb, u8 *buf, u32 *len, u64 *offset);
    int (*map_blocks)(struct inode *inode, loff_t offset, u64 len,
                      struct iomap *iomap, bool write, u32 *device_generation);
    int (*commit_blocks)(struct inode *inode, struct iomap *iomaps, int nr_iomaps,
                         struct iattr *iattr);
    unsigned long flags;
};

/* Flags */
#define EXPORT_OP_NOWCC             (0x1)
#define EXPORT_OP_NOSUBTREECHK      (0x2)
#define EXPORT_OP_CLOSE_BEFORE_UNLINK (0x4)
#define EXPORT_OP_REMOTE_FS         (0x8)
#define EXPORT_OP_NOATOMIC_ATTR     (0x10)

/* Generic implementations */
extern int generic_encode_ino32_fh(struct inode *inode, __u32 *fh, int *max_len,
                                   struct inode *parent);
extern struct dentry *generic_fh_to_dentry(struct super_block *sb, struct fid *fid,
                                           int fh_len, int fh_type,
                                           struct inode *(*get_inode)(struct super_block *, u64, u32));
extern struct dentry *generic_fh_to_parent(struct super_block *sb, struct fid *fid,
                                           int fh_len, int fh_type,
                                           struct inode *(*get_inode)(struct super_block *, u64, u32));

/* Iomap for blocks (simplified) */
struct iomap {
    u64 addr;
    loff_t offset;
    u64 length;
    u16 type;
    u16 flags;
    struct block_device *bdev;
    struct dax_device *dax_dev;
    void *inline_data;
};

#endif /* _LINUX_EXPORTFS_H */
