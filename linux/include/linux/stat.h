/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_STAT_H
#define _LINUX_STAT_H

#include <linux/types.h>
#include <linux/time.h>

/* Filesystem ID */
typedef struct { int val[2]; } __kernel_fsid_t;

/* Helper to convert u64 to fsid */
static inline __kernel_fsid_t u64_to_fsid(u64 v)
{
    __kernel_fsid_t fsid;
    fsid.val[0] = (u32)v;
    fsid.val[1] = (u32)(v >> 32);
    return fsid;
}

/* Forward declaration */
struct mnt_idmap;

/* File type masks */
#define S_IFMT      00170000
#define S_IFSOCK    0140000
#define S_IFLNK     0120000
#define S_IFREG     0100000
#define S_IFBLK     0060000
#define S_IFDIR     0040000
#define S_IFCHR     0020000
#define S_IFIFO     0010000

/* Set-user-ID, set-group-ID, and sticky bits */
#define S_ISUID     0004000
#define S_ISGID     0002000
#define S_ISVTX     0001000

/* Permission bits */
#define S_IRWXU     00700
#define S_IRUSR     00400
#define S_IWUSR     00200
#define S_IXUSR     00100

#define S_IRWXG     00070
#define S_IRGRP     00040
#define S_IWGRP     00020
#define S_IXGRP     00010

#define S_IRWXO     00007
#define S_IROTH     00004
#define S_IWOTH     00002
#define S_IXOTH     00001

/* Combined permission macros */
#define S_IRWXUGO   (S_IRWXU|S_IRWXG|S_IRWXO)
#define S_IALLUGO   (S_ISUID|S_ISGID|S_ISVTX|S_IRWXUGO)
#define S_IRUGO     (S_IRUSR|S_IRGRP|S_IROTH)
#define S_IWUGO     (S_IWUSR|S_IWGRP|S_IWOTH)
#define S_IXUGO     (S_IXUSR|S_IXGRP|S_IXOTH)

/* Type test macros */
#define S_ISLNK(m)      (((m) & S_IFMT) == S_IFLNK)
#define S_ISREG(m)      (((m) & S_IFMT) == S_IFREG)
#define S_ISDIR(m)      (((m) & S_IFMT) == S_IFDIR)
#define S_ISCHR(m)      (((m) & S_IFMT) == S_IFCHR)
#define S_ISBLK(m)      (((m) & S_IFMT) == S_IFBLK)
#define S_ISFIFO(m)     (((m) & S_IFMT) == S_IFIFO)
#define S_ISSOCK(m)     (((m) & S_IFMT) == S_IFSOCK)

/* Generic stat structure */
struct kstat {
    u32             result_mask;
    umode_t         mode;
    unsigned int    nlink;
    kuid_t          uid;
    kgid_t          gid;
    dev_t           rdev;
    loff_t          size;
    struct timespec64 atime;
    struct timespec64 mtime;
    struct timespec64 ctime;
    struct timespec64 btime;
    u64             ino;
    dev_t           dev;
    u64             blocks;
    u32             blksize;
    u64             attributes;
    u64             attributes_mask;
    u32             subvol;
    u32             change_cookie;
};

/* Extended stat request mask bits */
#define STATX_TYPE              0x00000001U
#define STATX_MODE              0x00000002U
#define STATX_NLINK             0x00000004U
#define STATX_UID               0x00000008U
#define STATX_GID               0x00000010U
#define STATX_ATIME             0x00000020U
#define STATX_MTIME             0x00000040U
#define STATX_CTIME             0x00000080U
#define STATX_INO               0x00000100U
#define STATX_SIZE              0x00000200U
#define STATX_BLOCKS            0x00000400U
#define STATX_BASIC_STATS       0x000007ffU
#define STATX_BTIME             0x00000800U
#define STATX_MNT_ID            0x00001000U

/* Attributes bits */
#define STATX_ATTR_COMPRESSED   0x00000004U
#define STATX_ATTR_IMMUTABLE    0x00000010U
#define STATX_ATTR_APPEND       0x00000020U
#define STATX_ATTR_NODUMP       0x00000040U
#define STATX_ATTR_ENCRYPTED    0x00000800U
#define STATX_ATTR_AUTOMOUNT    0x00001000U
#define STATX_ATTR_VERITY       0x00100000U
#define STATX_ATTR_DAX          0x00200000U

/* Statfs structure */
struct kstatfs {
    long            f_type;
    long            f_bsize;
    u64             f_blocks;
    u64             f_bfree;
    u64             f_bavail;
    u64             f_files;
    u64             f_ffree;
    __kernel_fsid_t f_fsid;
    long            f_namelen;
    long            f_frsize;
    long            f_flags;
    long            f_spare[4];
};

/* Generic stat operations */
extern void generic_fillattr(struct mnt_idmap *, u32, struct inode *, struct kstat *);

#endif /* _LINUX_STAT_H */
