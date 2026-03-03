/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_CAPABILITY_H
#define _LINUX_CAPABILITY_H

#include <linux/types.h>

/* Forward declarations to avoid circular dependency with fs.h */
struct file;
struct inode;
struct mnt_idmap;
struct user_namespace;

/* Capability numbers */
#define CAP_CHOWN               0
#define CAP_DAC_OVERRIDE        1
#define CAP_DAC_READ_SEARCH     2
#define CAP_FOWNER              3
#define CAP_FSETID              4
#define CAP_KILL                5
#define CAP_SETGID              6
#define CAP_SETUID              7
#define CAP_SETPCAP             8
#define CAP_LINUX_IMMUTABLE     9
#define CAP_NET_BIND_SERVICE    10
#define CAP_NET_BROADCAST       11
#define CAP_NET_ADMIN           12
#define CAP_NET_RAW             13
#define CAP_IPC_LOCK            14
#define CAP_IPC_OWNER           15
#define CAP_SYS_MODULE          16
#define CAP_SYS_RAWIO           17
#define CAP_SYS_CHROOT          18
#define CAP_SYS_PTRACE          19
#define CAP_SYS_PACCT           20
#define CAP_SYS_ADMIN           21
#define CAP_SYS_BOOT            22
#define CAP_SYS_NICE            23
#define CAP_SYS_RESOURCE        24
#define CAP_SYS_TIME            25
#define CAP_SYS_TTY_CONFIG      26
#define CAP_MKNOD               27
#define CAP_LEASE               28
#define CAP_AUDIT_WRITE         29
#define CAP_AUDIT_CONTROL       30
#define CAP_SETFCAP             31
#define CAP_MAC_OVERRIDE        32
#define CAP_MAC_ADMIN           33
#define CAP_SYSLOG              34
#define CAP_WAKE_ALARM          35
#define CAP_BLOCK_SUSPEND       36
#define CAP_AUDIT_READ          37
#define CAP_PERFMON             38
#define CAP_BPF                 39
#define CAP_CHECKPOINT_RESTORE  40

#define CAP_LAST_CAP            CAP_CHECKPOINT_RESTORE

/* Capability checking (implemented in Rust) */
extern bool capable(int cap);
extern bool ns_capable(struct user_namespace *ns, int cap);
extern bool file_ns_capable(const struct file *file, struct user_namespace *ns, int cap);

/* Stubs that allow everything for now */
static inline bool capable_wrt_inode_uidgid(struct mnt_idmap *idmap, const struct inode *inode, int cap)
{
    return true;
}

static inline bool inode_capable(const struct inode *inode, int cap)
{
    return true;
}

static inline bool inode_owner_or_capable(struct mnt_idmap *idmap, const struct inode *inode)
{
    return true;
}

#endif /* _LINUX_CAPABILITY_H */
