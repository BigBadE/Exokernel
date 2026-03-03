/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_CTYPE_H
#define _LINUX_CTYPE_H

/* Character classification (implemented in Rust) */
extern int isalnum(int c);
extern int isalpha(int c);
extern int isdigit(int c);
extern int islower(int c);
extern int isupper(int c);
extern int isxdigit(int c);
extern int isspace(int c);
extern int isprint(int c);
extern int ispunct(int c);
extern int iscntrl(int c);
extern int isgraph(int c);

/* Character conversion (implemented in Rust) */
extern int tolower(int c);
extern int toupper(int c);

/* Inline versions for performance */
static inline int __isalpha(int c)
{
    return ((c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z'));
}

static inline int __isdigit(int c)
{
    return (c >= '0' && c <= '9');
}

static inline int __isalnum(int c)
{
    return __isalpha(c) || __isdigit(c);
}

static inline int __islower(int c)
{
    return (c >= 'a' && c <= 'z');
}

static inline int __isupper(int c)
{
    return (c >= 'A' && c <= 'Z');
}

static inline int __isxdigit(int c)
{
    return __isdigit(c) || (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f');
}

static inline int __isspace(int c)
{
    return (c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v');
}

static inline int __tolower(int c)
{
    return __isupper(c) ? c + 32 : c;
}

static inline int __toupper(int c)
{
    return __islower(c) ? c - 32 : c;
}

#endif /* _LINUX_CTYPE_H */
