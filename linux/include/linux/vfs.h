/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_VFS_H
#define _LINUX_VFS_H

/*
 * This header just includes fs.h for compatibility.
 * In the real kernel, vfs.h doesn't exist - drivers include fs.h directly.
 * But some code uses <linux/vfs.h> so we provide this redirect.
 */

#include <linux/fs.h>

#endif /* _LINUX_VFS_H */
