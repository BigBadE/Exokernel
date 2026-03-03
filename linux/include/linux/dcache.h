/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_DCACHE_H
#define _LINUX_DCACHE_H

#include <linux/types.h>
#include <linux/list.h>
#include <linux/spinlock.h>
#include <linux/rwsem.h>

/* Forward declarations */
struct inode;
struct super_block;
struct dentry_operations;
struct path;
struct vfsmount;

/* RCU head for deferred freeing */
struct rcu_head {
    struct rcu_head *next;
    void (*func)(struct rcu_head *head);
};

/* Quick string for dentry names */
struct qstr {
    u32 hash;
    u32 len;
    const unsigned char *name;
};

#define QSTR_INIT(n, l) { .name = (n), .len = (l) }

/* Dentry structure */
struct dentry {
    unsigned int            d_flags;
    spinlock_t              d_lock;
    struct inode            *d_inode;
    struct dentry           *d_parent;
    struct qstr             d_name;
    unsigned char           d_iname[32];
    struct list_head        d_child;
    struct list_head        d_subdirs;
    struct hlist_node       d_hash;
    const struct dentry_operations *d_op;
    struct super_block      *d_sb;
    unsigned long           d_time;
    void                    *d_fsdata;
    struct hlist_head       d_children;
    struct list_head        d_lru;
    union {
        struct hlist_node   d_alias;
        struct rcu_head     d_rcu;
    };
    atomic_t                d_count;
};

/* Dentry operations */
struct dentry_operations {
    int (*d_revalidate)(struct dentry *, unsigned int);
    int (*d_weak_revalidate)(struct dentry *, unsigned int);
    int (*d_hash)(const struct dentry *, struct qstr *);
    int (*d_compare)(const struct dentry *, unsigned int, const char *, const struct qstr *);
    int (*d_delete)(const struct dentry *);
    int (*d_init)(struct dentry *);
    void (*d_release)(struct dentry *);
    void (*d_prune)(struct dentry *);
    void (*d_iput)(struct dentry *, struct inode *);
    char *(*d_dname)(struct dentry *, char *, int);
    struct vfsmount *(*d_automount)(struct path *);
    int (*d_manage)(const struct path *, bool);
    struct dentry *(*d_real)(struct dentry *, const struct inode *);
};

/* Dentry flags */
#define DCACHE_OP_HASH          0x00000001
#define DCACHE_OP_COMPARE       0x00000002
#define DCACHE_OP_REVALIDATE    0x00000004
#define DCACHE_OP_DELETE        0x00000008
#define DCACHE_OP_PRUNE         0x00000010
#define DCACHE_DISCONNECTED     0x00000020
#define DCACHE_REFERENCED       0x00000040
#define DCACHE_RCUACCESS        0x00000080
#define DCACHE_CANT_MOUNT       0x00000100
#define DCACHE_GENOCIDE         0x00000200
#define DCACHE_SHRINK_LIST      0x00000400
#define DCACHE_OP_WEAK_REVALIDATE 0x00000800
#define DCACHE_DIRECTORY_TYPE   0x00200000
#define DCACHE_AUTODIR_TYPE     0x00400000
#define DCACHE_REGULAR_TYPE     0x01000000
#define DCACHE_SPECIAL_TYPE     0x02000000
#define DCACHE_SYMLINK_TYPE     0x04000000
#define DCACHE_FILE_TYPE        (DCACHE_REGULAR_TYPE | DCACHE_SPECIAL_TYPE)
#define DCACHE_ENTRY_TYPE       (DCACHE_DIRECTORY_TYPE | DCACHE_AUTODIR_TYPE | DCACHE_REGULAR_TYPE | DCACHE_SPECIAL_TYPE | DCACHE_SYMLINK_TYPE)

/* Dentry operations (implemented in Rust) */
extern struct dentry *d_alloc(struct dentry *parent, const struct qstr *name);
extern struct dentry *d_alloc_anon(struct super_block *sb);
extern struct dentry *d_make_root(struct inode *root_inode);
extern void d_instantiate(struct dentry *dentry, struct inode *inode);
extern struct dentry *d_instantiate_unique(struct dentry *entry, struct inode *inode);
extern void d_add(struct dentry *dentry, struct inode *inode);
extern void d_drop(struct dentry *dentry);
extern void d_delete(struct dentry *dentry);
extern void dput(struct dentry *dentry);
extern struct dentry *dget(struct dentry *dentry);
extern int d_invalidate(struct dentry *dentry);
extern void d_prune_aliases(struct inode *inode);
extern void d_rehash(struct dentry *dentry);
extern void d_move(struct dentry *dentry, struct dentry *target);

/* Dentry name helpers */
extern int d_unlinked(const struct dentry *dentry);
extern int d_is_positive(const struct dentry *dentry);
extern int d_is_negative(const struct dentry *dentry);
extern struct inode *d_inode(const struct dentry *dentry);

/* Name comparison */
extern int d_name_equal(const struct dentry *d, const struct qstr *name);
extern void d_lookup_done(struct dentry *dentry);
extern struct dentry *d_lookup(const struct dentry *parent, const struct qstr *name);

/* Path helpers */
extern char *dentry_path_raw(const struct dentry *dentry, char *buf, int buflen);
extern char *dentry_path(const struct dentry *dentry, char *buf, int buflen);

/* Inline helpers */
static inline int d_really_is_positive(const struct dentry *dentry)
{
    return dentry->d_inode != NULL;
}

static inline int d_really_is_negative(const struct dentry *dentry)
{
    return dentry->d_inode == NULL;
}

static inline struct inode *d_inode_rcu(const struct dentry *dentry)
{
    return dentry->d_inode;
}

static inline struct inode *d_backing_inode(const struct dentry *dentry)
{
    return dentry->d_inode;
}

static inline void d_set_d_op(struct dentry *dentry, const struct dentry_operations *op)
{
    dentry->d_op = op;
    if (op && op->d_hash)
        dentry->d_flags |= DCACHE_OP_HASH;
    if (op && op->d_compare)
        dentry->d_flags |= DCACHE_OP_COMPARE;
    if (op && op->d_revalidate)
        dentry->d_flags |= DCACHE_OP_REVALIDATE;
    if (op && op->d_delete)
        dentry->d_flags |= DCACHE_OP_DELETE;
}

static inline unsigned d_count(const struct dentry *dentry)
{
    return dentry->d_count.counter;
}

/* Common dentry operations */
extern int simple_dentry_operations_delete(const struct dentry *dentry);

#endif /* _LINUX_DCACHE_H */
