/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_TIME_H
#define _LINUX_TIME_H

#include <linux/types.h>

/* Timespec with 64-bit seconds */
struct timespec64 {
    time64_t    tv_sec;
    long        tv_nsec;
};

/* Old 32-bit timespec (for compatibility) */
struct timespec {
    long        tv_sec;
    long        tv_nsec;
};

/* Time value (microseconds) */
struct timeval {
    long        tv_sec;
    long        tv_usec;
};

/* Time zone */
struct timezone {
    int         tz_minuteswest;
    int         tz_dsttime;
};

/* System timezone (declared as extern, defined in Rust) */
extern struct timezone sys_tz;

/* Broken-down time structure */
struct tm {
    int tm_sec;     /* seconds after the minute - [0, 60] */
    int tm_min;     /* minutes after the hour - [0, 59] */
    int tm_hour;    /* hours since midnight - [0, 23] */
    int tm_mday;    /* day of the month - [1, 31] */
    int tm_mon;     /* months since January - [0, 11] */
    int tm_year;    /* years since 1900 */
    int tm_wday;    /* days since Sunday - [0, 6] */
    int tm_yday;    /* days since January 1 - [0, 365] */
    int tm_isdst;   /* daylight saving time flag */
};

/* Time conversion functions */
extern void time64_to_tm(time64_t totalsecs, int offset, struct tm *result);

/* Nanoseconds per second etc */
#define NSEC_PER_SEC        1000000000L
#define NSEC_PER_MSEC       1000000L
#define NSEC_PER_USEC       1000L
#define USEC_PER_SEC        1000000L
#define USEC_PER_MSEC       1000L
#define MSEC_PER_SEC        1000L

/* Time comparison */
static inline int timespec64_compare(const struct timespec64 *lhs, const struct timespec64 *rhs)
{
    if (lhs->tv_sec < rhs->tv_sec)
        return -1;
    if (lhs->tv_sec > rhs->tv_sec)
        return 1;
    return lhs->tv_nsec - rhs->tv_nsec;
}

static inline bool timespec64_equal(const struct timespec64 *a, const struct timespec64 *b)
{
    return (a->tv_sec == b->tv_sec) && (a->tv_nsec == b->tv_nsec);
}

/* Current time functions (implemented in Rust) */
extern struct timespec64 current_time(struct inode *inode);
extern void ktime_get_real_ts64(struct timespec64 *ts);
extern void ktime_get_coarse_real_ts64(struct timespec64 *ts);
extern void ktime_get_ts64(struct timespec64 *ts);

/* Timespec conversion */
static inline struct timespec64 timespec_to_timespec64(const struct timespec ts)
{
    struct timespec64 ret;
    ret.tv_sec = ts.tv_sec;
    ret.tv_nsec = ts.tv_nsec;
    return ret;
}

static inline struct timespec timespec64_to_timespec(const struct timespec64 ts64)
{
    struct timespec ret;
    ret.tv_sec = ts64.tv_sec;
    ret.tv_nsec = ts64.tv_nsec;
    return ret;
}

/* Timespec arithmetic */
static inline struct timespec64 timespec64_add(struct timespec64 lhs, struct timespec64 rhs)
{
    struct timespec64 ts_delta;
    ts_delta.tv_sec = lhs.tv_sec + rhs.tv_sec;
    ts_delta.tv_nsec = lhs.tv_nsec + rhs.tv_nsec;
    while (ts_delta.tv_nsec >= NSEC_PER_SEC) {
        ts_delta.tv_nsec -= NSEC_PER_SEC;
        ts_delta.tv_sec++;
    }
    return ts_delta;
}

static inline struct timespec64 timespec64_sub(struct timespec64 lhs, struct timespec64 rhs)
{
    struct timespec64 ts_delta;
    ts_delta.tv_sec = lhs.tv_sec - rhs.tv_sec;
    ts_delta.tv_nsec = lhs.tv_nsec - rhs.tv_nsec;
    while (ts_delta.tv_nsec < 0) {
        ts_delta.tv_nsec += NSEC_PER_SEC;
        ts_delta.tv_sec--;
    }
    return ts_delta;
}

/* Truncate time to granularity */
extern struct timespec64 timestamp_truncate(struct timespec64 t, struct inode *inode);

/* ktime functions */
static inline ktime_t ktime_get(void)
{
    return 0; /* TODO: implement */
}

static inline ktime_t ktime_get_real(void)
{
    return 0; /* TODO: implement */
}

static inline s64 ktime_to_ms(ktime_t kt)
{
    return kt / NSEC_PER_MSEC;
}

static inline s64 ktime_to_us(ktime_t kt)
{
    return kt / NSEC_PER_USEC;
}

static inline s64 ktime_to_ns(ktime_t kt)
{
    return kt;
}

#endif /* _LINUX_TIME_H */
