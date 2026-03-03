/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_STRING_H
#define _LINUX_STRING_H

#include <linux/types.h>
#include <linux/compiler.h>
#include <linux/stddef.h>

/* Memory operations (implemented in Rust) */
extern void *memcpy(void *dest, const void *src, size_t n);
extern void *memmove(void *dest, const void *src, size_t n);
extern void *memset(void *s, int c, size_t n);
extern int memcmp(const void *s1, const void *s2, size_t n);
extern void *memchr(const void *s, int c, size_t n);

/* String operations (implemented in Rust) */
extern size_t strlen(const char *s);
extern size_t strnlen(const char *s, size_t maxlen);
extern char *strcpy(char *dest, const char *src);
extern char *strncpy(char *dest, const char *src, size_t n);
extern char *strcat(char *dest, const char *src);
extern char *strncat(char *dest, const char *src, size_t n);
extern int strcmp(const char *s1, const char *s2);
extern int strncmp(const char *s1, const char *s2, size_t n);
extern int strcasecmp(const char *s1, const char *s2);
extern int strncasecmp(const char *s1, const char *s2, size_t n);
extern char *strchr(const char *s, int c);
extern char *strrchr(const char *s, int c);
extern char *strstr(const char *haystack, const char *needle);

/* Kernel-specific string functions */
extern char *kstrdup(const char *s, gfp_t gfp);
extern char *kstrndup(const char *s, size_t max, gfp_t gfp);
extern char *kmemdup(const void *src, size_t len, gfp_t gfp);

/* Safe string functions */
extern size_t strlcpy(char *dest, const char *src, size_t size);
extern size_t strlcat(char *dest, const char *src, size_t size);

/* String to number conversions */
extern unsigned long simple_strtoul(const char *cp, char **endp, unsigned int base);
extern long simple_strtol(const char *cp, char **endp, unsigned int base);
extern unsigned long long simple_strtoull(const char *cp, char **endp, unsigned int base);
extern long long simple_strtoll(const char *cp, char **endp, unsigned int base);

/* Kernel string/number conversions */
extern int kstrtouint(const char *s, unsigned int base, unsigned int *res);
extern int kstrtoint(const char *s, unsigned int base, int *res);
extern int kstrtoul(const char *s, unsigned int base, unsigned long *res);
extern int kstrtol(const char *s, unsigned int base, long *res);

/* Hex string operations */
extern int hex_to_bin(unsigned char ch);
extern int hex2bin(unsigned char *dst, const char *src, size_t count);
extern char *bin2hex(char *dst, const void *src, size_t count);

/* Match helper */
static inline bool strstarts(const char *str, const char *prefix)
{
    return strncmp(str, prefix, strlen(prefix)) == 0;
}

/* Memscan */
static inline void *memscan(void *addr, int c, size_t size)
{
    unsigned char *p = addr;
    while (size) {
        if (*p == (unsigned char)c)
            return (void *)p;
        p++;
        size--;
    }
    return (void *)p;
}

/* Zero memory securely */
static inline void memzero_explicit(void *s, size_t count)
{
    memset(s, 0, count);
    barrier();
}

#endif /* _LINUX_STRING_H */
