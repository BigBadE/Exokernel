/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_FS_H
#define _LINUX_FS_H

#include <linux/types.h>
#include <linux/list.h>
#include <linux/spinlock.h>
#include <linux/mutex.h>
#include <linux/rwsem.h>
#include <linux/dcache.h>
#include <linux/stat.h>
#include <linux/time.h>
#include <linux/wait.h>
#include <linux/errno.h>
#include <linux/kernel.h>
#include <linux/slab.h>
#include <linux/sched.h>
#include <linux/module.h>

/* Forward declarations */
struct file;
struct inode;
struct dentry;
struct super_block;
struct file_system_type;
struct vm_area_struct;

/* Simplified address_space structure for inode embedding */
struct address_space {
    struct inode            *host;
    void                    *i_pages;
    atomic_t                i_mmap_writable;
    gfp_t                   gfp_mask;
    unsigned long           nrpages;
    unsigned long           writeback_index;
    const void              *a_ops;
    unsigned long           flags;
    spinlock_t              private_lock;
    struct list_head        private_list;
    void                    *private_data;
};
struct kiocb;
struct iov_iter;
struct dir_context;
struct poll_table_struct;
struct kstat;
struct block_device;
struct fiemap_extent_info;
struct file_lock;
struct file_lease;
struct seq_file;
struct page;
struct writeback_control;
struct readahead_control;
struct swap_info_struct;
struct mnt_idmap;
struct delayed_call;
struct posix_acl;

/* File attribute structure for setattr */
struct iattr {
    unsigned int    ia_valid;
    umode_t         ia_mode;
    kuid_t          ia_uid;
    kgid_t          ia_gid;
    vfsuid_t        ia_vfsuid;
    vfsgid_t        ia_vfsgid;
    loff_t          ia_size;
    struct timespec64 ia_atime;
    struct timespec64 ia_mtime;
    struct timespec64 ia_ctime;
    struct file     *ia_file;
};
struct shrink_control;
struct fs_context;
struct fs_parameter_spec;
struct io_uring_cmd;
struct io_comp_batch;
struct pipe_inode_info;
struct fileattr;

/* Path structure */
struct path {
    struct vfsmount *mnt;
    struct dentry *dentry;
};

/* UUID type */
typedef struct {
    __u8 b[16];
} uuid_t;

/* User-space marker (no-op in kernel builds) */
#ifndef __user
#define __user
#endif

/* Types for poll and file operations */
typedef unsigned int __poll_t;
typedef void *fl_owner_t;

/* Freeze holder */
enum freeze_holder { FREEZE_HOLDER_KERNEL, FREEZE_HOLDER_USERSPACE };

/* Directory entry types */
#define DT_UNKNOWN      0
#define DT_FIFO         1
#define DT_CHR          2
#define DT_DIR          4
#define DT_BLK          6
#define DT_REG          8
#define DT_LNK          10
#define DT_SOCK         12
#define DT_WHT          14

/* File mode flags */
#define FMODE_READ              0x1
#define FMODE_WRITE             0x2
#define FMODE_LSEEK             0x4
#define FMODE_PREAD             0x8
#define FMODE_PWRITE            0x10
#define FMODE_EXEC              0x20
#define FMODE_NDELAY            0x40
#define FMODE_EXCL              0x80
#define FMODE_NOCMTIME          0x800
#define FMODE_RANDOM            0x1000
#define FMODE_UNSIGNED_OFFSET   0x2000

/* Inode flags */
#define S_SYNC          1
#define S_NOATIME       2
#define S_APPEND        4
#define S_IMMUTABLE     8
#define S_DEAD          16
#define S_NOQUOTA       32
#define S_DIRSYNC       64
#define S_NOCMTIME      128
#define S_SWAPFILE      256
#define S_PRIVATE       512
#define S_IMA           1024
#define S_AUTOMOUNT     2048
#define S_NOSEC         4096
#define S_DAX           8192
#define S_ENCRYPTED     16384
#define S_CASEFOLD      32768
#define S_VERITY        65536

/* Inode dirty flags (for __mark_inode_dirty) */
#define I_DIRTY_SYNC        (1 << 0)
#define I_DIRTY_DATASYNC    (1 << 1)
#define I_DIRTY_PAGES       (1 << 2)
#define I_NEW               (1 << 3)
#define I_WILL_FREE         (1 << 4)
#define I_FREEING           (1 << 5)
#define I_CLEAR             (1 << 6)
#define I_SYNC              (1 << 7)
#define I_DIRTY_TIME        (1 << 8)
#define I_DIRTY_TIME_EXPIRED (1 << 9)
#define I_REFERENCED        (1 << 10)
#define I_LINKABLE          (1 << 11)
#define I_WB_SWITCH         (1 << 12)
#define I_OVL_INUSE         (1 << 13)
#define I_CREATING          (1 << 14)
#define I_DONTCACHE         (1 << 15)
#define I_SYNC_QUEUED       (1 << 16)
#define I_PINNING_FSCACHE_WB (1 << 17)
#define I_LRU_ISOLATING     (1 << 18)

#define I_DIRTY             (I_DIRTY_SYNC | I_DIRTY_DATASYNC | I_DIRTY_PAGES)

/* Superblock flags */
#define SB_RDONLY       1
#define SB_NOSUID       2
#define SB_NODEV        4
#define SB_NOEXEC       8
#define SB_SYNCHRONOUS  16
#define SB_MANDLOCK     64
#define SB_DIRSYNC      128
#define SB_NOATIME      1024
#define SB_NODIRATIME   2048
#define SB_SILENT       32768
#define SB_POSIXACL     (1<<16)
#define SB_INLINECRYPT  (1<<17)
#define SB_KERNMOUNT    (1<<22)
#define SB_I_VERSION    (1<<23)
#define SB_LAZYTIME     (1<<25)
#define SB_ACTIVE       (1<<30)

/* Filesystem type flags (for file_system_type.fs_flags) */
#define FS_REQUIRES_DEV         1
#define FS_BINARY_MOUNTDATA     2
#define FS_HAS_SUBTYPE          4
#define FS_USERNS_MOUNT         8
#define FS_DISALLOW_NOTIFY_PERM 16
#define FS_ALLOW_IDMAP          32
#define FS_RENAME_DOES_D_MOVE   32768

/* Lookup flags */
#define LOOKUP_FOLLOW           0x0001
#define LOOKUP_DIRECTORY        0x0002
#define LOOKUP_AUTOMOUNT        0x0004
#define LOOKUP_EMPTY            0x4000
#define LOOKUP_DOWN             0x8000
#define LOOKUP_MOUNTPOINT       0x0080
#define LOOKUP_RCU              0x40
#define LOOKUP_OPEN             0x0100
#define LOOKUP_CREATE           0x0200
#define LOOKUP_EXCL             0x0400
#define LOOKUP_RENAME_TARGET    0x0800
#define LOOKUP_JUMPED           0x1000
#define LOOKUP_ROOT             0x2000
#define LOOKUP_NO_XDEV          0x100000

/* Rename flags */
#define RENAME_NOREPLACE        (1 << 0)
#define RENAME_EXCHANGE         (1 << 1)
#define RENAME_WHITEOUT         (1 << 2)

/* File open flags */
#define O_ACCMODE       00000003
#define O_RDONLY        00000000
#define O_WRONLY        00000001
#define O_RDWR          00000002
#define O_CREAT         00000100
#define O_EXCL          00000200
#define O_NOCTTY        00000400
#define O_TRUNC         00001000
#define O_APPEND        00002000
#define O_NONBLOCK      00004000
#define O_DSYNC         00010000
#define O_DIRECT        00040000
#define O_LARGEFILE     00100000
#define O_DIRECTORY     00200000
#define O_NOFOLLOW      00400000
#define O_NOATIME       01000000
#define O_CLOEXEC       02000000
#define O_SYNC          04010000
#define O_PATH          010000000
#define O_TMPFILE       020200000

/* Seek origins */
#define SEEK_SET        0
#define SEEK_CUR        1
#define SEEK_END        2
#define SEEK_DATA       3
#define SEEK_HOLE       4

/* Attribute change flags (for notify_change/setattr) */
#define ATTR_MODE       (1 << 0)
#define ATTR_UID        (1 << 1)
#define ATTR_GID        (1 << 2)
#define ATTR_SIZE       (1 << 3)
#define ATTR_ATIME      (1 << 4)
#define ATTR_MTIME      (1 << 5)
#define ATTR_CTIME      (1 << 6)
#define ATTR_ATIME_SET  (1 << 7)
#define ATTR_MTIME_SET  (1 << 8)
#define ATTR_FORCE      (1 << 9)
#define ATTR_KILL_SUID  (1 << 11)
#define ATTR_KILL_SGID  (1 << 12)
#define ATTR_FILE       (1 << 13)
#define ATTR_KILL_PRIV  (1 << 14)
#define ATTR_OPEN       (1 << 15)
#define ATTR_TIMES_SET  (1 << 16)

/* Timestamp flags for generic_update_time */
#define S_ATIME         1
#define S_MTIME         2
#define S_CTIME         4
#define S_VERSION       8

/* I/O direction */
#define READ            0
#define WRITE           1

/* Permission check flags */
#define MAY_EXEC        0x00000001
#define MAY_WRITE       0x00000002
#define MAY_READ        0x00000004
#define MAY_APPEND      0x00000008
#define MAY_ACCESS      0x00000010
#define MAY_OPEN        0x00000020
#define MAY_CHDIR       0x00000040
#define MAY_NOT_BLOCK   0x00000080

/* File lock types */
#define F_RDLCK         0
#define F_WRLCK         1
#define F_UNLCK         2

/* The inode structure */
struct inode {
    umode_t                 i_mode;
    unsigned short          i_opflags;
    kuid_t                  i_uid;
    kgid_t                  i_gid;
    unsigned int            i_flags;
    const struct inode_operations *i_op;
    struct super_block      *i_sb;
    struct address_space    *i_mapping;
    unsigned long           i_ino;
    unsigned int            i_nlink;
    dev_t                   i_rdev;
    loff_t                  i_size;
    struct timespec64       i_atime;
    struct timespec64       i_mtime;
    struct timespec64       __i_ctime;
    spinlock_t              i_lock;
    unsigned short          i_bytes;
    u8                      i_blkbits;
    blkcnt_t                i_blocks;
    unsigned long           i_state;
    struct rw_semaphore     i_rwsem;
    unsigned long           dirtied_when;
    unsigned long           dirtied_time_when;
    struct hlist_node       i_hash;
    struct list_head        i_io_list;
    struct list_head        i_lru;
    struct list_head        i_sb_list;
    struct list_head        i_wb_list;
    atomic_t                i_count;
    atomic_t                i_writecount;
    union {
        const struct file_operations *i_fop;
        void (*free_inode)(struct inode *);
    };
    void                    *i_private;
    u64                     i_version;
    atomic64_t              i_sequence;
    u32                     i_generation;
    struct address_space    i_data;
};

/* Directory entry context */
struct dir_context {
    int (*actor)(struct dir_context *, const char *, int,
                 loff_t, u64, unsigned);
    loff_t pos;
};

/* Old-style filldir callback type */
typedef int (*filldir_t)(struct dir_context *, const char *, int, loff_t, u64, unsigned);

/* Generic read for directories */
static inline ssize_t generic_read_dir(struct file *filp, char __user *buf, size_t siz, loff_t *ppos)
{
    return -EISDIR;
}

/* Page size (if not already defined) */
#ifndef PAGE_SIZE
#define PAGE_SIZE       4096
#define PAGE_SHIFT      12
#endif

/* Maximum number of buffers per page (for directory operations) */
#define MAX_BUF_PER_PAGE (PAGE_SIZE / 512)

/* The file structure */
struct file {
    struct path             f_path;
    struct inode            *f_inode;
    const struct file_operations *f_op;
    spinlock_t              f_lock;
    atomic_long_t           f_count;
    unsigned int            f_flags;
    fmode_t                 f_mode;
    struct mutex            f_pos_lock;
    loff_t                  f_pos;
    u64                     f_version;
    void                    *private_data;
    struct address_space    *f_mapping;
};

/* The superblock structure */
struct super_block {
    struct list_head        s_list;
    dev_t                   s_dev;
    unsigned char           s_blocksize_bits;
    unsigned long           s_blocksize;
    loff_t                  s_maxbytes;
    struct file_system_type *s_type;
    const struct super_operations *s_op;
    const struct dquot_operations *dq_op;
    const struct quotactl_ops *s_qcop;
    const struct export_operations *s_export_op;
    unsigned long           s_flags;
    unsigned long           s_iflags;
    unsigned long           s_magic;
    struct dentry           *s_root;
    struct rw_semaphore     s_umount;
    int                     s_count;
    atomic_t                s_active;
    struct list_head        s_inodes;
    struct list_head        s_inodes_wb;
    spinlock_t              s_inode_list_lock;
    struct block_device     *s_bdev;
    struct backing_dev_info *s_bdi;
    struct hlist_node       s_instances;
    unsigned int            s_max_links;
    char                    s_id[32];
    uuid_t                  s_uuid;
    void                    *s_fs_info;
    fmode_t                 s_mode;
    struct mutex            s_vfs_rename_mutex;
    const char              *s_subtype;
    const struct dentry_operations *s_d_op;
    time64_t                s_time_min;
    time64_t                s_time_max;
    u32                     s_time_gran;
    struct list_head        s_mounts;
};

/* File system type */
struct file_system_type {
    const char              *name;
    int                     fs_flags;
    int (*init_fs_context)(struct fs_context *);
    const struct fs_parameter_spec *parameters;
    struct dentry *(*mount)(struct file_system_type *, int, const char *, void *);
    void (*kill_sb)(struct super_block *);
    struct module           *owner;
    struct file_system_type *next;
    struct hlist_head       fs_supers;
};

/* Operations structures */
struct inode_operations {
    struct dentry *(*lookup)(struct inode *, struct dentry *, unsigned int);
    const char *(*get_link)(struct dentry *, struct inode *, struct delayed_call *);
    int (*permission)(struct mnt_idmap *, struct inode *, int);
    struct posix_acl *(*get_inode_acl)(struct inode *, int, bool);
    int (*readlink)(struct dentry *, char __user *, int);
    int (*create)(struct mnt_idmap *, struct inode *, struct dentry *, umode_t, bool);
    int (*link)(struct dentry *, struct inode *, struct dentry *);
    int (*unlink)(struct inode *, struct dentry *);
    int (*symlink)(struct mnt_idmap *, struct inode *, struct dentry *, const char *);
    int (*mkdir)(struct mnt_idmap *, struct inode *, struct dentry *, umode_t);
    int (*rmdir)(struct inode *, struct dentry *);
    int (*mknod)(struct mnt_idmap *, struct inode *, struct dentry *, umode_t, dev_t);
    int (*rename)(struct mnt_idmap *, struct inode *, struct dentry *, struct inode *, struct dentry *, unsigned int);
    int (*setattr)(struct mnt_idmap *, struct dentry *, struct iattr *);
    int (*getattr)(struct mnt_idmap *, const struct path *, struct kstat *, u32, unsigned int);
    ssize_t (*listxattr)(struct dentry *, char *, size_t);
    int (*fiemap)(struct inode *, struct fiemap_extent_info *, u64 start, u64 len);
    int (*update_time)(struct inode *, int);
    int (*atomic_open)(struct inode *, struct dentry *, struct file *, unsigned open_flag, umode_t create_mode);
    int (*tmpfile)(struct mnt_idmap *, struct inode *, struct file *, umode_t);
    struct posix_acl *(*get_acl)(struct mnt_idmap *, struct dentry *, int);
    int (*set_acl)(struct mnt_idmap *, struct dentry *, struct posix_acl *, int);
    int (*fileattr_set)(struct mnt_idmap *, struct dentry *, struct fileattr *);
    int (*fileattr_get)(struct dentry *, struct fileattr *);
};

struct file_operations {
    struct module *owner;
    loff_t (*llseek)(struct file *, loff_t, int);
    ssize_t (*read)(struct file *, char __user *, size_t, loff_t *);
    ssize_t (*write)(struct file *, const char __user *, size_t, loff_t *);
    ssize_t (*read_iter)(struct kiocb *, struct iov_iter *);
    ssize_t (*write_iter)(struct kiocb *, struct iov_iter *);
    int (*iopoll)(struct kiocb *, struct io_comp_batch *, unsigned int);
    int (*iterate_shared)(struct file *, struct dir_context *);
    __poll_t (*poll)(struct file *, struct poll_table_struct *);
    long (*unlocked_ioctl)(struct file *, unsigned int, unsigned long);
    long (*compat_ioctl)(struct file *, unsigned int, unsigned long);
    int (*mmap)(struct file *, struct vm_area_struct *);
    int (*open)(struct inode *, struct file *);
    int (*flush)(struct file *, fl_owner_t id);
    int (*release)(struct inode *, struct file *);
    int (*fsync)(struct file *, loff_t, loff_t, int datasync);
    int (*fasync)(int, struct file *, int);
    int (*lock)(struct file *, int, struct file_lock *);
    unsigned long (*get_unmapped_area)(struct file *, unsigned long, unsigned long, unsigned long, unsigned long);
    int (*check_flags)(int);
    int (*flock)(struct file *, int, struct file_lock *);
    ssize_t (*splice_write)(struct pipe_inode_info *, struct file *, loff_t *, size_t, unsigned int);
    ssize_t (*splice_read)(struct file *, loff_t *, struct pipe_inode_info *, size_t, unsigned int);
    void (*splice_eof)(struct file *);
    int (*setlease)(struct file *, int, struct file_lease **, void **);
    long (*fallocate)(struct file *, int mode, loff_t offset, loff_t len);
    void (*show_fdinfo)(struct seq_file *, struct file *);
    ssize_t (*copy_file_range)(struct file *, loff_t, struct file *, loff_t, size_t, unsigned int);
    loff_t (*remap_file_range)(struct file *, loff_t, struct file *, loff_t, loff_t, unsigned int);
    int (*fadvise)(struct file *, loff_t, loff_t, int);
    int (*uring_cmd)(struct io_uring_cmd *, unsigned int);
    int (*uring_cmd_iopoll)(struct io_uring_cmd *, struct io_comp_batch *, unsigned int);
};

struct super_operations {
    struct inode *(*alloc_inode)(struct super_block *sb);
    void (*destroy_inode)(struct inode *);
    void (*free_inode)(struct inode *);
    void (*dirty_inode)(struct inode *, int flags);
    int (*write_inode)(struct inode *, struct writeback_control *wbc);
    int (*drop_inode)(struct inode *);
    void (*evict_inode)(struct inode *);
    void (*put_super)(struct super_block *);
    int (*sync_fs)(struct super_block *sb, int wait);
    int (*freeze_super)(struct super_block *, enum freeze_holder who);
    int (*freeze_fs)(struct super_block *);
    int (*thaw_super)(struct super_block *, enum freeze_holder who);
    int (*unfreeze_fs)(struct super_block *);
    int (*statfs)(struct dentry *, struct kstatfs *);
    int (*remount_fs)(struct super_block *, int *, char *);
    void (*umount_begin)(struct super_block *);
    int (*show_options)(struct seq_file *, struct dentry *);
    int (*show_devname)(struct seq_file *, struct dentry *);
    int (*show_path)(struct seq_file *, struct dentry *);
    int (*show_stats)(struct seq_file *, struct dentry *);
    ssize_t (*quota_read)(struct super_block *, int, char *, size_t, loff_t);
    ssize_t (*quota_write)(struct super_block *, int, const char *, size_t, loff_t);
    struct dquot **(*get_dquots)(struct inode *);
    long (*nr_cached_objects)(struct super_block *, struct shrink_control *);
    long (*free_cached_objects)(struct super_block *, struct shrink_control *);
    void (*shutdown)(struct super_block *);
};

/* File system registration (implemented in Rust) */
extern int register_filesystem(struct file_system_type *);
extern int unregister_filesystem(struct file_system_type *);

/* Inode operations (implemented in Rust) */
extern struct inode *new_inode(struct super_block *sb);
extern void iget_failed(struct inode *inode);
extern struct inode *iget_locked(struct super_block *sb, unsigned long ino);
extern void unlock_new_inode(struct inode *inode);
extern void iput(struct inode *inode);
extern void ihold(struct inode *inode);
extern void clear_inode(struct inode *inode);
extern void clear_nlink(struct inode *inode);
extern void set_nlink(struct inode *inode, unsigned int nlink);
extern void inc_nlink(struct inode *inode);
extern void drop_nlink(struct inode *inode);
extern void mark_inode_dirty(struct inode *inode);
extern void mark_inode_dirty_sync(struct inode *inode);
extern void inode_init_once(struct inode *inode);
extern void inode_init_owner(struct mnt_idmap *, struct inode *inode, const struct inode *dir, umode_t mode);

/* Superblock helpers */
extern void kill_block_super(struct super_block *sb);
extern void kill_anon_super(struct super_block *sb);
extern void kill_litter_super(struct super_block *sb);
extern struct dentry *mount_bdev(struct file_system_type *fs_type, int flags,
                                 const char *dev_name, void *data,
                                 int (*fill_super)(struct super_block *, void *, int));
extern struct super_block *sget(struct file_system_type *type,
                                int (*test)(struct super_block *, void *),
                                int (*set)(struct super_block *, void *),
                                int flags, void *data);
extern void deactivate_locked_super(struct super_block *sb);

/* File helpers */
extern loff_t generic_file_llseek(struct file *file, loff_t offset, int whence);
extern ssize_t generic_file_read_iter(struct kiocb *iocb, struct iov_iter *iter);
extern ssize_t generic_file_write_iter(struct kiocb *iocb, struct iov_iter *from);
extern int generic_file_mmap(struct file *file, struct vm_area_struct *vma);
extern int generic_file_fsync(struct file *file, loff_t start, loff_t end, int datasync);

/* Directory iteration */
static inline bool dir_emit(struct dir_context *ctx, const char *name, int namelen,
                            u64 ino, unsigned type)
{
    return ctx->actor(ctx, name, namelen, ctx->pos, ino, type) == 0;
}

static inline bool dir_emit_dot(struct file *file, struct dir_context *ctx)
{
    return ctx->actor(ctx, ".", 1, ctx->pos, file->f_path.dentry->d_inode->i_ino, DT_DIR) == 0;
}

static inline bool dir_emit_dotdot(struct file *file, struct dir_context *ctx)
{
    return ctx->actor(ctx, "..", 2, ctx->pos,
                      file->f_path.dentry->d_parent->d_inode->i_ino, DT_DIR) == 0;
}

static inline bool dir_emit_dots(struct file *file, struct dir_context *ctx)
{
    if (ctx->pos == 0) {
        if (!dir_emit_dot(file, ctx))
            return false;
        ctx->pos = 1;
    }
    if (ctx->pos == 1) {
        if (!dir_emit_dotdot(file, ctx))
            return false;
        ctx->pos = 2;
    }
    return true;
}

/* Timestamp helpers */
static inline struct timespec64 inode_get_ctime(const struct inode *inode)
{
    return inode->__i_ctime;
}

static inline struct timespec64 inode_set_ctime(struct inode *inode, time64_t sec, long nsec)
{
    inode->__i_ctime.tv_sec = sec;
    inode->__i_ctime.tv_nsec = nsec;
    return inode->__i_ctime;
}

static inline struct timespec64 inode_set_ctime_current(struct inode *inode)
{
    /* TODO: get current time */
    return inode_set_ctime(inode, 0, 0);
}

static inline struct timespec64 inode_set_ctime_to_ts(struct inode *inode,
                                                       struct timespec64 ts)
{
    inode->__i_ctime = ts;
    return ts;
}

/* Generic simple_xxx functions */
extern int simple_statfs(struct dentry *, struct kstatfs *);
extern int simple_empty(struct dentry *);
extern int simple_link(struct dentry *, struct inode *, struct dentry *);
extern int simple_unlink(struct inode *, struct dentry *);
extern int simple_rmdir(struct inode *, struct dentry *);

/* Splice operations */
extern ssize_t filemap_splice_read(struct file *in, loff_t *ppos,
                                   struct pipe_inode_info *pipe,
                                   size_t len, unsigned int flags);
extern ssize_t iter_file_splice_write(struct pipe_inode_info *pipe,
                                      struct file *out, loff_t *ppos,
                                      size_t len, unsigned int flags);
extern ssize_t generic_file_splice_read(struct file *, loff_t *,
                                        struct pipe_inode_info *, size_t, unsigned int);
extern ssize_t generic_splice_sendpage(struct pipe_inode_info *,
                                       struct file *, loff_t *, size_t, unsigned int);

/* Compat ioctl handler */
extern long compat_ptr_ioctl(struct file *file, unsigned int cmd, unsigned long arg);

/* Setattr helper */
extern int setattr_prepare(struct mnt_idmap *, struct dentry *, struct iattr *);
extern void setattr_copy(struct mnt_idmap *, struct inode *, const struct iattr *);
extern int notify_change(struct mnt_idmap *, struct dentry *, struct iattr *, struct inode **);

/* File inode helper */
static inline struct inode *file_inode(const struct file *f)
{
    return f->f_inode;
}

/* Inline for IS_IMMUTABLE etc. */
static inline int IS_IMMUTABLE(const struct inode *inode)
{
    return inode->i_flags & S_IMMUTABLE;
}

static inline int IS_APPEND(const struct inode *inode)
{
    return inode->i_flags & S_APPEND;
}

static inline int IS_SYNC(const struct inode *inode)
{
    return (inode->i_flags & S_SYNC) || (inode->i_sb->s_flags & SB_SYNCHRONOUS);
}

static inline int IS_DIRSYNC(const struct inode *inode)
{
    return (inode->i_flags & S_DIRSYNC) || (inode->i_sb->s_flags & SB_DIRSYNC);
}

static inline int IS_NOQUOTA(const struct inode *inode)
{
    return inode->i_flags & S_NOQUOTA;
}

static inline int IS_DEADDIR(const struct inode *inode)
{
    return inode->i_flags & S_DEAD;
}

/* Inode user namespace helper */
static inline struct user_namespace *i_user_ns(const struct inode *inode)
{
    /* For simplicity, return init namespace */
    return &init_user_ns;
}

/* VFS UID/GID helpers for inodes */
static inline vfsuid_t i_uid_into_vfsuid(struct mnt_idmap *idmap, const struct inode *inode)
{
    return (vfsuid_t){ .val = inode->i_uid };
}

static inline vfsgid_t i_gid_into_vfsgid(struct mnt_idmap *idmap, const struct inode *inode)
{
    return (vfsgid_t){ .val = inode->i_gid };
}

/* File mnt_idmap helper */
static inline struct mnt_idmap *file_mnt_idmap(const struct file *file)
{
    return (struct mnt_idmap *)0; /* No idmapping for now */
}

/* noop_mnt_idmap - identity mapping */
extern struct mnt_idmap *noop_mnt_idmap;

#endif /* _LINUX_FS_H */
