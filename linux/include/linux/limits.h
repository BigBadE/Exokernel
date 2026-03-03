/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_LIMITS_H
#define _LINUX_LIMITS_H

#define UCHAR_MAX       255
#define SCHAR_MAX       127
#define SCHAR_MIN       (-128)
#define CHAR_MAX        SCHAR_MAX
#define CHAR_MIN        SCHAR_MIN

#define USHRT_MAX       65535
#define SHRT_MAX        32767
#define SHRT_MIN        (-32768)

#define UINT_MAX        (~0U)
#define INT_MAX         ((int)(~0U >> 1))
#define INT_MIN         (-INT_MAX - 1)

#define ULONG_MAX       (~0UL)
#define LONG_MAX        ((long)(~0UL >> 1))
#define LONG_MIN        (-LONG_MAX - 1)

#define ULLONG_MAX      (~0ULL)
#define LLONG_MAX       ((long long)(~0ULL >> 1))
#define LLONG_MIN       (-LLONG_MAX - 1)

#define SIZE_MAX        ULONG_MAX
#define SSIZE_MAX       LONG_MAX

#define U8_MAX          ((u8)255)
#define S8_MAX          ((s8)127)
#define S8_MIN          ((s8)(-128))
#define U16_MAX         ((u16)65535)
#define S16_MAX         ((s16)32767)
#define S16_MIN         ((s16)(-32768))
#define U32_MAX         ((u32)~0U)
#define S32_MAX         ((s32)(U32_MAX >> 1))
#define S32_MIN         ((s32)(-S32_MAX - 1))
#define U64_MAX         ((u64)~0ULL)
#define S64_MAX         ((s64)(U64_MAX >> 1))
#define S64_MIN         ((s64)(-S64_MAX - 1))

/* Path limits */
#define PATH_MAX        4096
#define NAME_MAX        255
#define LINK_MAX        127

/* Other limits */
#define NGROUPS_MAX     65536
#define ARG_MAX         131072
#define PIPE_BUF        4096
#define XATTR_NAME_MAX  255
#define XATTR_SIZE_MAX  65536
#define XATTR_LIST_MAX  65536

#endif /* _LINUX_LIMITS_H */
