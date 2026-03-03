/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_MAGIC_H
#define _LINUX_MAGIC_H

/* Magic numbers for various filesystem types */
#define MSDOS_SUPER_MAGIC       0x4d44          /* "MD" */
#define EXT2_SUPER_MAGIC        0xEF53
#define EXT3_SUPER_MAGIC        0xEF53
#define EXT4_SUPER_MAGIC        0xEF53
#define BTRFS_SUPER_MAGIC       0x9123683E
#define TMPFS_MAGIC             0x01021994
#define RAMFS_MAGIC             0x858458f6
#define DEVFS_SUPER_MAGIC       0x1373
#define NFS_SUPER_MAGIC         0x6969
#define PROC_SUPER_MAGIC        0x9fa0
#define SYSFS_MAGIC             0x62656572
#define DEBUGFS_MAGIC           0x64626720
#define SECURITYFS_MAGIC        0x73636673
#define TRACEFS_MAGIC           0x74726163
#define CGROUP_SUPER_MAGIC      0x27e0eb
#define CGROUP2_SUPER_MAGIC     0x63677270
#define OVERLAYFS_SUPER_MAGIC   0x794c7630
#define FUSE_SUPER_MAGIC        0x65735546
#define MINIX_SUPER_MAGIC       0x137F
#define MINIX2_SUPER_MAGIC      0x2468
#define MINIX3_SUPER_MAGIC      0x4d5a
#define ISOFS_SUPER_MAGIC       0x9660
#define JFFS2_SUPER_MAGIC       0x72b6
#define XFS_SUPER_MAGIC         0x58465342
#define HPFS_SUPER_MAGIC        0xf995e849
#define NTFS_SB_MAGIC           0x5346544e
#define SQUASHFS_MAGIC          0x73717368
#define VXFS_SUPER_MAGIC        0xa501FCF5
#define XENFS_SUPER_MAGIC       0xabba1974
#define BPF_FS_MAGIC            0xcafe4a11
#define PIPEFS_MAGIC            0x50495045
#define SOCKFS_MAGIC            0x534F434B
#define ANON_INODE_FS_MAGIC     0x09041934

#endif /* _LINUX_MAGIC_H */
