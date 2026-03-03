/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_COMPILER_H
#define _LINUX_COMPILER_H

/* Compiler attributes */
#define __section(s)        __attribute__((__section__(s)))
#define __used              __attribute__((__used__))
#define __maybe_unused      __attribute__((__unused__))
#define __always_unused     __attribute__((__unused__))
#define __packed            __attribute__((__packed__))
#define __aligned(x)        __attribute__((__aligned__(x)))
#define __noreturn          __attribute__((__noreturn__))
#define __weak              __attribute__((__weak__))
#define __alias(s)          __attribute__((__alias__(s)))
#define __cold              __attribute__((__cold__))
#define __pure              __attribute__((__pure__))
#define __const             __attribute__((__const__))
#define __flatten           __attribute__((__flatten__))
#define __noinline          __attribute__((__noinline__))
#define __always_inline     inline __attribute__((__always_inline__))
#define __deprecated        __attribute__((__deprecated__))
#define __malloc            __attribute__((__malloc__))
#define __must_check        __attribute__((__warn_unused_result__))
#define __noclone           __attribute__((__noclone__))
#define __assume_aligned(a) __attribute__((__assume_aligned__(a)))
#define __printf(a, b)      __attribute__((__format__(printf, a, b)))
#define __scanf(a, b)       __attribute__((__format__(scanf, a, b)))

/* Prevent the compiler from merging or refetching accesses */
#define READ_ONCE(x)        (*(const volatile typeof(x) *)&(x))
#define WRITE_ONCE(x, val)  (*(volatile typeof(x) *)&(x) = (val))

/* Likely/unlikely hints for branch prediction */
#define likely(x)           __builtin_expect(!!(x), 1)
#define unlikely(x)         __builtin_expect(!!(x), 0)

/* Memory barriers */
#define barrier()           __asm__ __volatile__("" ::: "memory")
#define mb()                __asm__ __volatile__("mfence" ::: "memory")
#define rmb()               __asm__ __volatile__("lfence" ::: "memory")
#define wmb()               __asm__ __volatile__("sfence" ::: "memory")
#define smp_mb()            mb()
#define smp_rmb()           barrier()
#define smp_wmb()           barrier()

/* Compiler memory barrier */
#define __compiler_membar() barrier()

/* Unreachable code marker */
#define unreachable()       __builtin_unreachable()

/* Static assertion */
#define static_assert       _Static_assert
#define BUILD_BUG_ON(e)     static_assert(!(e), "BUILD_BUG_ON: " #e)

/* Container of macro */
#define container_of(ptr, type, member) ({ \
    const typeof(((type *)0)->member) *__mptr = (ptr); \
    (type *)((char *)__mptr - offsetof(type, member)); \
})

/* Sizeof field in struct */
#define sizeof_field(TYPE, MEMBER) sizeof((((TYPE *)0)->MEMBER))

/* Offset of field in struct */
#ifndef offsetof
#define offsetof(TYPE, MEMBER) __builtin_offsetof(TYPE, MEMBER)
#endif

/* Check at compile time that something is of a particular type */
#define typecheck(type, x) ({ type __dummy; typeof(x) __dummy2; (void)(&__dummy == &__dummy2); 1; })

#endif /* _LINUX_COMPILER_H */
