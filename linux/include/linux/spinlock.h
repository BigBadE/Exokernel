/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_SPINLOCK_H
#define _LINUX_SPINLOCK_H

#include <linux/types.h>
#include <linux/compiler.h>

/* Spinlock type */
typedef struct {
    unsigned int lock;
} spinlock_t;

typedef struct {
    unsigned int lock;
} rwlock_t;

/* Static initializers */
#define __SPIN_LOCK_UNLOCKED    { .lock = 0 }
#define DEFINE_SPINLOCK(x)      spinlock_t x = __SPIN_LOCK_UNLOCKED
#define __RW_LOCK_UNLOCKED      { .lock = 0 }
#define DEFINE_RWLOCK(x)        rwlock_t x = __RW_LOCK_UNLOCKED

/* Spinlock operations (implemented in Rust) */
extern void spin_lock_init(spinlock_t *lock);
extern void spin_lock(spinlock_t *lock);
extern void spin_unlock(spinlock_t *lock);
extern int spin_trylock(spinlock_t *lock);
extern int spin_is_locked(spinlock_t *lock);

/* IRQ-saving variants */
extern void spin_lock_irq(spinlock_t *lock);
extern void spin_unlock_irq(spinlock_t *lock);
extern void spin_lock_irqsave(spinlock_t *lock, unsigned long flags);
extern void spin_unlock_irqrestore(spinlock_t *lock, unsigned long flags);

/* BH variants */
extern void spin_lock_bh(spinlock_t *lock);
extern void spin_unlock_bh(spinlock_t *lock);

/* Read-write lock operations */
extern void rwlock_init(rwlock_t *lock);
extern void read_lock(rwlock_t *lock);
extern void read_unlock(rwlock_t *lock);
extern void write_lock(rwlock_t *lock);
extern void write_unlock(rwlock_t *lock);

/* Assert macros (debug only) */
#define lockdep_assert_held(l)          do { } while(0)
#define lockdep_assert_held_write(l)    do { } while(0)
#define lockdep_assert_held_read(l)     do { } while(0)
#define lock_acquire(l, s, t, r, c, n, i)   do { } while(0)
#define lock_release(l, i)              do { } while(0)

/* Local IRQ control */
#define local_irq_save(flags)           do { (void)(flags); } while(0)
#define local_irq_restore(flags)        do { (void)(flags); } while(0)
#define local_irq_enable()              do { } while(0)
#define local_irq_disable()             do { } while(0)

#endif /* _LINUX_SPINLOCK_H */
