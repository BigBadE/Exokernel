/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_KERNEL_H
#define _LINUX_KERNEL_H

#include <linux/types.h>
#include <linux/compiler.h>
#include <linux/stddef.h>
#include <linux/stdarg.h>
#include <linux/limits.h>

/* Min/max macros */
#define min(a, b) ((a) < (b) ? (a) : (b))
#define max(a, b) ((a) > (b) ? (a) : (b))
#define min_t(type, a, b) ((type)(a) < (type)(b) ? (type)(a) : (type)(b))
#define max_t(type, a, b) ((type)(a) > (type)(b) ? (type)(a) : (type)(b))
#define clamp(val, lo, hi) min(max(val, lo), hi)
#define clamp_t(type, val, lo, hi) min_t(type, max_t(type, val, lo), hi)

/* Swap macro */
#define swap(a, b) do { typeof(a) __tmp = (a); (a) = (b); (b) = __tmp; } while (0)

/* Array size */
#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))

/* Alignment macros */
#define ALIGN(x, a)             __ALIGN_MASK(x, (typeof(x))(a) - 1)
#define ALIGN_DOWN(x, a)        ALIGN((x) - ((a) - 1), (a))
#define __ALIGN_MASK(x, mask)   (((x) + (mask)) & ~(mask))
#define IS_ALIGNED(x, a)        (((x) & ((typeof(x))(a) - 1)) == 0)

/* Round up/down to power of 2 */
#define round_up(x, y)          ((((x) - 1) | ((y) - 1)) + 1)
#define round_down(x, y)        ((x) & ~((y) - 1))

/* Division helpers */
#define DIV_ROUND_UP(n, d)      (((n) + (d) - 1) / (d))
#define DIV_ROUND_DOWN(n, d)    ((n) / (d))
#define DIV_ROUND_CLOSEST(n, d) (((n) + (d) / 2) / (d))

/* Bit manipulation */
#define BIT(nr)                 (1UL << (nr))
#define BIT_ULL(nr)             (1ULL << (nr))
#define BIT_MASK(nr)            (1UL << ((nr) % BITS_PER_LONG))
#define BIT_WORD(nr)            ((nr) / BITS_PER_LONG)
#define BITS_PER_BYTE           8
#ifdef __LP64__
#define BITS_PER_LONG           64
#else
#define BITS_PER_LONG           32
#endif

/* Upper/lower bits */
#define upper_32_bits(n)        ((u32)(((n) >> 16) >> 16))
#define lower_32_bits(n)        ((u32)(n))

/* Absolute value */
#define abs(x) ({               \
    long __x = (x);             \
    (__x < 0) ? -__x : __x;     \
})

/* Check if power of 2 */
#define is_power_of_2(n)        ((n) != 0 && (((n) & ((n) - 1)) == 0))

/* Stringify */
#define __stringify_1(x)        #x
#define __stringify(x)          __stringify_1(x)

/* Printk stubs (implemented in Rust) */
extern int printk(const char *fmt, ...) __printf(1, 2);

#define KERN_EMERG      "<0>"
#define KERN_ALERT      "<1>"
#define KERN_CRIT       "<2>"
#define KERN_ERR        "<3>"
#define KERN_WARNING    "<4>"
#define KERN_NOTICE     "<5>"
#define KERN_INFO       "<6>"
#define KERN_DEBUG      "<7>"
#define KERN_CONT       ""

#define pr_emerg(fmt, ...)      printk(KERN_EMERG fmt, ##__VA_ARGS__)
#define pr_alert(fmt, ...)      printk(KERN_ALERT fmt, ##__VA_ARGS__)
#define pr_crit(fmt, ...)       printk(KERN_CRIT fmt, ##__VA_ARGS__)
#define pr_err(fmt, ...)        printk(KERN_ERR fmt, ##__VA_ARGS__)
#define pr_warn(fmt, ...)       printk(KERN_WARNING fmt, ##__VA_ARGS__)
#define pr_notice(fmt, ...)     printk(KERN_NOTICE fmt, ##__VA_ARGS__)
#define pr_info(fmt, ...)       printk(KERN_INFO fmt, ##__VA_ARGS__)
#define pr_debug(fmt, ...)      printk(KERN_DEBUG fmt, ##__VA_ARGS__)
#define pr_cont(fmt, ...)       printk(KERN_CONT fmt, ##__VA_ARGS__)

/* WARN/BUG macros */
#define WARN_ON(condition)      ((condition) ? (pr_warn("WARN_ON: %s:%d\n", __FILE__, __LINE__), 1) : 0)
#define WARN_ON_ONCE(condition) WARN_ON(condition)
#define WARN(condition, fmt, ...) ((condition) ? (pr_warn(fmt, ##__VA_ARGS__), 1) : 0)
#define BUG()                   do { pr_err("BUG: %s:%d\n", __FILE__, __LINE__); while(1); } while(0)
#define BUG_ON(condition)       do { if (condition) BUG(); } while(0)

/* Panic */
extern void panic(const char *fmt, ...) __noreturn __printf(1, 2);

/* Hexdump */
enum { DUMP_PREFIX_NONE, DUMP_PREFIX_ADDRESS, DUMP_PREFIX_OFFSET };
static inline void print_hex_dump(const char *level, const char *prefix_str,
                                  int prefix_type, int rowsize, int groupsize,
                                  const void *buf, size_t len, bool ascii) { }

#endif /* _LINUX_KERNEL_H */
