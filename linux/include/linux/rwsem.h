/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_RWSEM_H
#define _LINUX_RWSEM_H

#include <linux/types.h>
#include <linux/list.h>
#include <linux/spinlock.h>

/* Reader/writer semaphore */
struct rw_semaphore {
    atomic_long_t count;
    spinlock_t wait_lock;
    struct list_head wait_list;
};

/* Static initializer */
#define __RWSEM_INITIALIZER(name) { \
    .count = { 0 }, \
    .wait_lock = __SPIN_LOCK_UNLOCKED, \
    .wait_list = LIST_HEAD_INIT((name).wait_list), \
}

#define DECLARE_RWSEM(name) \
    struct rw_semaphore name = __RWSEM_INITIALIZER(name)

/* RW semaphore operations (implemented in Rust) */
extern void init_rwsem(struct rw_semaphore *sem);
extern void down_read(struct rw_semaphore *sem);
extern void up_read(struct rw_semaphore *sem);
extern void down_write(struct rw_semaphore *sem);
extern void up_write(struct rw_semaphore *sem);
extern int down_read_trylock(struct rw_semaphore *sem);
extern int down_write_trylock(struct rw_semaphore *sem);
extern void downgrade_write(struct rw_semaphore *sem);

/* Interruptible/killable variants */
extern int down_read_interruptible(struct rw_semaphore *sem);
extern int down_read_killable(struct rw_semaphore *sem);
extern int down_write_killable(struct rw_semaphore *sem);

/* Nested variants (debug annotation only) */
#define down_read_nested(sem, subclass)     down_read(sem)
#define down_write_nested(sem, subclass)    down_write(sem)

/* Assert helpers */
#define rwsem_assert_held(sem)              do { } while(0)
#define rwsem_assert_held_write(sem)        do { } while(0)

static inline void __init_rwsem(struct rw_semaphore *sem, const char *name, struct lock_class_key *key)
{
    init_rwsem(sem);
}

#endif /* _LINUX_RWSEM_H */
