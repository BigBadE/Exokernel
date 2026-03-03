/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_STDDEF_H
#define _LINUX_STDDEF_H

#undef NULL
#define NULL ((void *)0)

#ifndef offsetof
#define offsetof(TYPE, MEMBER) __builtin_offsetof(TYPE, MEMBER)
#endif

/* Struct field type checker */
#define sizeof_field(TYPE, MEMBER) sizeof((((TYPE *)0)->MEMBER))
#define offsetofend(TYPE, MEMBER) (offsetof(TYPE, MEMBER) + sizeof_field(TYPE, MEMBER))

/* For flexible array members */
enum { FLEX_ARRAY };

#endif /* _LINUX_STDDEF_H */
