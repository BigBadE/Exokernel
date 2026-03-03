/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_RATELIMIT_H
#define _LINUX_RATELIMIT_H

#include <linux/types.h>
#include <linux/spinlock.h>

/* Rate limit state */
struct ratelimit_state {
    spinlock_t lock;
    int interval;
    int burst;
    int printed;
    int missed;
    unsigned long begin;
    unsigned long flags;
};

/* Initialize rate limit state */
#define RATELIMIT_STATE_INIT(name, interval_init, burst_init) { \
    .interval = (interval_init), \
    .burst = (burst_init), \
    .printed = 0, \
    .missed = 0, \
    .begin = 0, \
}

/* Define a rate limit state */
#define DEFINE_RATELIMIT_STATE(name, interval_init, burst_init) \
    struct ratelimit_state name = RATELIMIT_STATE_INIT(name, interval_init, burst_init)

/* Default rate limit (10 times per 5 seconds) */
#define DEFAULT_RATELIMIT_INTERVAL (5 * 1000)
#define DEFAULT_RATELIMIT_BURST 10

/* Rate limit checking (always allow for now) */
static inline int __ratelimit(struct ratelimit_state *rs)
{
    return 1;
}

static inline int ratelimit(void)
{
    return 1;
}

/* Initialize rate limit state dynamically */
static inline void ratelimit_state_init(struct ratelimit_state *rs,
                                        int interval, int burst)
{
    rs->interval = interval;
    rs->burst = burst;
    rs->printed = 0;
    rs->missed = 0;
    rs->begin = 0;
}

/* Reset rate limit state */
static inline void ratelimit_state_reset(struct ratelimit_state *rs)
{
    rs->printed = 0;
    rs->missed = 0;
}

/* Set rate limit interval */
static inline void ratelimit_set_flags(struct ratelimit_state *rs,
                                       unsigned long flags)
{
    rs->flags = flags;
}

/* Flags */
#define RATELIMIT_MSG_ON_RELEASE (1 << 0)

#endif /* _LINUX_RATELIMIT_H */
