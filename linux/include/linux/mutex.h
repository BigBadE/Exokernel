/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_MUTEX_H
#define _LINUX_MUTEX_H

#include <linux/types.h>
#include <linux/list.h>
#include <linux/spinlock.h>

/* Mutex structure */
struct mutex {
    atomic_long_t owner;
    spinlock_t wait_lock;
    struct list_head wait_list;
};

/* Static initializer */
#define __MUTEX_INITIALIZER(name) { \
    .owner = { 0 }, \
    .wait_lock = __SPIN_LOCK_UNLOCKED, \
    .wait_list = LIST_HEAD_INIT((name).wait_list), \
}

#define DEFINE_MUTEX(mutexname) \
    struct mutex mutexname = __MUTEX_INITIALIZER(mutexname)

/* Mutex operations (implemented in Rust) */
extern void mutex_init(struct mutex *lock);
extern void mutex_destroy(struct mutex *lock);
extern void mutex_lock(struct mutex *lock);
extern void mutex_unlock(struct mutex *lock);
extern int mutex_trylock(struct mutex *lock);
extern int mutex_is_locked(struct mutex *lock);

/* Interruptible variant */
extern int mutex_lock_interruptible(struct mutex *lock);
extern int mutex_lock_killable(struct mutex *lock);

/* Nested lock annotation (debug only) */
#define mutex_lock_nested(lock, subclass)   mutex_lock(lock)

/* Inline wrappers */
static inline void __mutex_init(struct mutex *lock, const char *name, struct lock_class_key *key)
{
    mutex_init(lock);
}

#endif /* _LINUX_MUTEX_H */
