/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_IVERSION_H
#define _LINUX_IVERSION_H

#include <linux/types.h>
#include <linux/fs.h>

/*
 * Inode version/generation number tracking.
 * Used for NFS and other protocols that need change detection.
 */

#define I_VERSION_QUERIED_SHIFT     63
#define I_VERSION_QUERIED           (1ULL << I_VERSION_QUERIED_SHIFT)
#define I_VERSION_INCREMENT         (1ULL << 1)

/* Get raw version value */
static inline u64 inode_peek_iversion_raw(const struct inode *inode)
{
    return inode->i_version;
}

/* Set raw version value */
static inline void inode_set_iversion_raw(struct inode *inode, u64 val)
{
    inode->i_version = val;
}

/* Initialize version to 1 */
static inline void inode_set_iversion(struct inode *inode, u64 val)
{
    inode->i_version = (val << 1);
}

/* Increment version number */
static inline bool inode_maybe_inc_iversion(struct inode *inode, bool force)
{
    u64 cur = inode->i_version;

    if (!force && !(cur & I_VERSION_QUERIED))
        return false;

    inode->i_version = (cur & ~I_VERSION_QUERIED) + I_VERSION_INCREMENT;
    return true;
}

static inline void inode_inc_iversion(struct inode *inode)
{
    inode_maybe_inc_iversion(inode, true);
}

/* Query version - marks it as queried */
static inline u64 inode_query_iversion(struct inode *inode)
{
    u64 cur = inode->i_version;
    if (!(cur & I_VERSION_QUERIED))
        inode->i_version = cur | I_VERSION_QUERIED;
    return cur >> 1;
}

/* Peek without marking as queried */
static inline u64 inode_peek_iversion(const struct inode *inode)
{
    return inode->i_version >> 1;
}

/* Check if version needs increment */
static inline bool inode_iversion_need_inc(struct inode *inode)
{
    return (inode->i_version & I_VERSION_QUERIED);
}

/* Compare versions */
static inline s64 inode_cmp_iversion_raw(const struct inode *inode, u64 old)
{
    return (s64)(inode->i_version & ~I_VERSION_QUERIED) - (s64)old;
}

static inline s64 inode_cmp_iversion(const struct inode *inode, u64 old)
{
    return inode_cmp_iversion_raw(inode, old << 1);
}

/* Check if different from old value */
static inline bool inode_iversion_newer_raw(const struct inode *inode, u64 old)
{
    return inode_cmp_iversion_raw(inode, old) > 0;
}

#endif /* _LINUX_IVERSION_H */
