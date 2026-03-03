/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_WAIT_H
#define _LINUX_WAIT_H

#include <linux/types.h>
#include <linux/list.h>
#include <linux/spinlock.h>

/* Wait queue entry flags */
#define WQ_FLAG_EXCLUSIVE       0x01
#define WQ_FLAG_WOKEN           0x02
#define WQ_FLAG_BOOKMARK        0x04
#define WQ_FLAG_CUSTOM          0x08
#define WQ_FLAG_DONE            0x10
#define WQ_FLAG_PRIORITY        0x20

/* Wait queue head */
struct wait_queue_head {
    spinlock_t          lock;
    struct list_head    head;
};
typedef struct wait_queue_head wait_queue_head_t;

/* Wait queue entry */
struct wait_queue_entry {
    unsigned int        flags;
    void               *private;
    int (*func)(struct wait_queue_entry *, unsigned mode, int flags, void *key);
    struct list_head    entry;
};
typedef struct wait_queue_entry wait_queue_entry_t;

/* Static initializer */
#define __WAIT_QUEUE_HEAD_INITIALIZER(name) { \
    .lock = __SPIN_LOCK_UNLOCKED, \
    .head = LIST_HEAD_INIT((name).head), \
}

#define DECLARE_WAIT_QUEUE_HEAD(name) \
    wait_queue_head_t name = __WAIT_QUEUE_HEAD_INITIALIZER(name)

/* Wait queue operations */
static inline void init_waitqueue_head(wait_queue_head_t *wq_head)
{
    spin_lock_init(&wq_head->lock);
    INIT_LIST_HEAD(&wq_head->head);
}

static inline void init_waitqueue_entry(wait_queue_entry_t *wq_entry, void *p)
{
    wq_entry->flags = 0;
    wq_entry->private = p;
    wq_entry->func = NULL;
}

static inline int waitqueue_active(wait_queue_head_t *wq_head)
{
    return !list_empty(&wq_head->head);
}

/* Wake functions (implemented in Rust) */
extern void wake_up(wait_queue_head_t *wq_head);
extern void wake_up_all(wait_queue_head_t *wq_head);
extern void wake_up_interruptible(wait_queue_head_t *wq_head);
extern void wake_up_interruptible_all(wait_queue_head_t *wq_head);

/* Wait macros - these would need scheduler support */
#define wait_event(wq_head, condition)  do { (void)(condition); } while(0)
#define wait_event_interruptible(wq_head, condition) ({ (void)(condition); 0; })
#define wait_event_timeout(wq_head, condition, timeout) ({ (void)(condition); (timeout); })
#define wait_event_interruptible_timeout(wq_head, condition, timeout) ({ (void)(condition); (timeout); })
#define wait_event_killable(wq_head, condition) ({ (void)(condition); 0; })

/* Add/remove from wait queue (implemented in Rust) */
extern void add_wait_queue(wait_queue_head_t *wq_head, wait_queue_entry_t *wq_entry);
extern void add_wait_queue_exclusive(wait_queue_head_t *wq_head, wait_queue_entry_t *wq_entry);
extern void remove_wait_queue(wait_queue_head_t *wq_head, wait_queue_entry_t *wq_entry);

/* Completion structure */
struct completion {
    unsigned int done;
    wait_queue_head_t wait;
};

#define COMPLETION_INITIALIZER(work) { \
    .done = 0, \
    .wait = __WAIT_QUEUE_HEAD_INITIALIZER((work).wait), \
}

#define DECLARE_COMPLETION(work) \
    struct completion work = COMPLETION_INITIALIZER(work)

static inline void init_completion(struct completion *x)
{
    x->done = 0;
    init_waitqueue_head(&x->wait);
}

static inline void reinit_completion(struct completion *x)
{
    x->done = 0;
}

/* Completion operations (implemented in Rust) */
extern void complete(struct completion *x);
extern void complete_all(struct completion *x);
extern void wait_for_completion(struct completion *x);
extern int wait_for_completion_interruptible(struct completion *x);
extern unsigned long wait_for_completion_timeout(struct completion *x, unsigned long timeout);
extern int wait_for_completion_killable(struct completion *x);
extern bool try_wait_for_completion(struct completion *x);
extern bool completion_done(struct completion *x);

#endif /* _LINUX_WAIT_H */
