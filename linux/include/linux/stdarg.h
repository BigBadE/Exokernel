/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_STDARG_H
#define _LINUX_STDARG_H

/* Use compiler builtins for va_list */
typedef __builtin_va_list va_list;

#define va_start(ap, param) __builtin_va_start(ap, param)
#define va_end(ap)          __builtin_va_end(ap)
#define va_arg(ap, type)    __builtin_va_arg(ap, type)
#define va_copy(dest, src)  __builtin_va_copy(dest, src)

/* Struct for passing va_format to %pV printk */
struct va_format {
    const char *fmt;
    va_list *va;
};

#endif /* _LINUX_STDARG_H */
