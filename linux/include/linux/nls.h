/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_NLS_H
#define _LINUX_NLS_H

#include <linux/types.h>

/* Unicode character type (avoid conflict with types.h) */
#ifndef _WCHAR_T_DEFINED
#define _WCHAR_T_DEFINED
#endif

/* Maximum character size for any charset */
#define NLS_MAX_CHARSET_SIZE    6

/* UTF16 endianness flags */
#define UTF16_HOST_ENDIAN       0
#define UTF16_LITTLE_ENDIAN     1
#define UTF16_BIG_ENDIAN        2

/* NLS table structure */
struct nls_table {
    const char *charset;
    const char *alias;
    int (*uni2char)(wchar_t uni, unsigned char *out, int boundlen);
    int (*char2uni)(const unsigned char *rawstring, int boundlen, wchar_t *uni);
    const unsigned char *charset2lower;
    const unsigned char *charset2upper;
    struct module *owner;
    struct nls_table *next;
};

/* NLS operations (implemented in Rust) */
extern struct nls_table *load_nls(const char *charset);
extern struct nls_table *load_nls_default(void);
extern void unload_nls(struct nls_table *nls);
extern struct nls_table *get_default_nls(void);

/* Character conversion helpers */
extern int nls_uni16s_to_nls(struct nls_table *nls, const wchar_t *uni,
                              int unilen, unsigned char *out, int outlen);
extern int nls_nls_to_uni16s(struct nls_table *nls, const unsigned char *in,
                              int inlen, wchar_t *uni, int unilen);

/* Case conversion */
static inline unsigned char nls_tolower(struct nls_table *t, unsigned char c)
{
    if (t && t->charset2lower)
        return t->charset2lower[c];
    return (c >= 'A' && c <= 'Z') ? c + 32 : c;
}

static inline unsigned char nls_toupper(struct nls_table *t, unsigned char c)
{
    if (t && t->charset2upper)
        return t->charset2upper[c];
    return (c >= 'a' && c <= 'z') ? c - 32 : c;
}

/* String length in Unicode chars */
extern int nls_strlen(struct nls_table *nls, const unsigned char *s);

/* UTF-8 helpers */
extern int utf8_mbtowc(wchar_t *p, const unsigned char *s, int n);
extern int utf8_mbstowcs(wchar_t *pwcs, const unsigned char *s, int n);
extern int utf8_wctomb(unsigned char *s, wchar_t wc, int maxlen);
extern int utf8_wcstombs(unsigned char *s, const wchar_t *pwcs, int maxlen);

/* Unicode character properties */
static inline int nls_is_unicode_control(wchar_t c)
{
    return c < 0x20 || (c >= 0x7f && c < 0xa0);
}

#endif /* _LINUX_NLS_H */
