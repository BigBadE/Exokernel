/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_PARSER_H
#define _LINUX_PARSER_H

#include <linux/types.h>

/* Maximum number of arguments for match_token */
#define MAX_OPT_ARGS    3

/* Substring type for parser */
typedef struct {
    char *from;
    char *to;
} substring_t;

/* Match token structure */
struct match_token {
    int token;
    const char *pattern;
};

/* Match table type (array of match_token terminated by {0, NULL}) */
typedef struct match_token match_table_t[];

/* Parser helper functions */
extern int match_token(char *s, const struct match_token *table, substring_t args[]);
extern int match_int(substring_t *s, int *result);
extern int match_uint(substring_t *s, unsigned int *result);
extern int match_u64(substring_t *s, u64 *result);
extern int match_octal(substring_t *s, int *result);
extern int match_hex(substring_t *s, int *result);
extern bool match_wildcard(const char *pattern, const char *str);
extern size_t match_strlcpy(char *dest, const substring_t *src, size_t size);
extern char *match_strdup(const substring_t *s);

/* Inline helpers */
static inline int match_string(substring_t *s, const char *str)
{
    size_t len = s->to - s->from;
    if (strlen(str) != len)
        return 0;
    return memcmp(s->from, str, len) == 0;
}

#endif /* _LINUX_PARSER_H */
