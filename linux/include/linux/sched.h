/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_SCHED_H
#define _LINUX_SCHED_H

#include <linux/types.h>

/* HZ (timer ticks per second) - typical value for x86_64 */
#ifndef HZ
#define HZ 100
#endif

/* Jiffies type */
extern unsigned long volatile jiffies;

/* Task states */
#define TASK_RUNNING            0x00000000
#define TASK_INTERRUPTIBLE      0x00000001
#define TASK_UNINTERRUPTIBLE    0x00000002
#define TASK_STOPPED            0x00000004
#define TASK_TRACED             0x00000008
#define EXIT_DEAD               0x00000010
#define EXIT_ZOMBIE             0x00000020
#define TASK_PARKED             0x00000040
#define TASK_DEAD               0x00000080
#define TASK_WAKEKILL           0x00000100
#define TASK_WAKING             0x00000200
#define TASK_NOLOAD             0x00000400
#define TASK_NEW                0x00000800
#define TASK_RTLOCK_WAIT        0x00001000
#define TASK_FREEZABLE          0x00002000
#define TASK_KILLABLE           (TASK_WAKEKILL | TASK_UNINTERRUPTIBLE)
#define TASK_IDLE               (TASK_UNINTERRUPTIBLE | TASK_NOLOAD)

/* Current task pointer (stubbed) */
struct task_struct;
extern struct task_struct *current;

/* Schedule/yield */
extern void schedule(void);
extern void yield(void);
extern int cond_resched(void);
extern void set_current_state(long state);
extern void __set_current_state(long state);

/* Wait queue (simplified) */
static inline void msleep(unsigned int msecs)
{
    /* TODO: implement actual sleeping */
}

static inline void schedule_timeout(long timeout)
{
    /* TODO: implement actual timeout */
}

static inline void schedule_timeout_uninterruptible(long timeout)
{
    /* TODO: implement actual timeout */
}

/* Signal handling */
static inline int signal_pending(struct task_struct *p)
{
    return 0;
}

static inline int fatal_signal_pending(struct task_struct *p)
{
    return 0;
}

/* Capability checking - declaration in capability.h, implementation elsewhere */
/* Note: capable() is declared in capability.h as 'bool capable(int cap)' */

#define CAP_SYS_ADMIN       21
#define CAP_SYS_RAWIO       17
#define CAP_FOWNER          3
#define CAP_FSETID          4
#define CAP_DAC_OVERRIDE    1
#define CAP_DAC_READ_SEARCH 2
#define CAP_MKNOD           27
#define CAP_SETUID          7
#define CAP_SETGID          6

#endif /* _LINUX_SCHED_H */
