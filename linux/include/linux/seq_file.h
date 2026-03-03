/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_SEQ_FILE_H
#define _LINUX_SEQ_FILE_H

#include <linux/types.h>
#include <linux/fs.h>

/* Seq file structure */
struct seq_file {
    char *buf;
    size_t size;
    size_t from;
    size_t count;
    size_t pad_until;
    loff_t index;
    loff_t read_pos;
    struct mutex lock;
    const struct seq_operations *op;
    int poll_event;
    const struct file *file;
    void *private;
};

/* Seq operations */
struct seq_operations {
    void *(*start)(struct seq_file *m, loff_t *pos);
    void (*stop)(struct seq_file *m, void *v);
    void *(*next)(struct seq_file *m, void *v, loff_t *pos);
    int (*show)(struct seq_file *m, void *v);
};

/* Seq file output functions (implemented in Rust or stubs) */
extern void seq_printf(struct seq_file *m, const char *fmt, ...);
extern void seq_puts(struct seq_file *m, const char *s);
extern void seq_putc(struct seq_file *m, char c);
extern void seq_write(struct seq_file *m, const void *data, size_t len);
extern void seq_escape(struct seq_file *m, const char *s, const char *esc);
extern void seq_hex_dump(struct seq_file *m, const char *prefix_str,
                         int prefix_type, int rowsize, int groupsize,
                         const void *buf, size_t len, bool ascii);

/* Seq file path output */
extern int seq_path(struct seq_file *m, const struct path *path, const char *esc);
extern int seq_dentry(struct seq_file *m, struct dentry *dentry, const char *esc);

/* Seq file helpers */
extern int seq_open(struct file *file, const struct seq_operations *op);
extern int seq_release(struct inode *inode, struct file *file);
extern ssize_t seq_read(struct file *file, char __user *buf, size_t size, loff_t *ppos);
extern loff_t seq_lseek(struct file *file, loff_t offset, int whence);

/* Single open helpers */
extern int single_open(struct file *file, int (*show)(struct seq_file *, void *), void *data);
extern int single_open_size(struct file *file, int (*show)(struct seq_file *, void *), void *data, size_t size);
extern int single_release(struct inode *inode, struct file *file);

/* Show functions */
static inline void seq_show_option(struct seq_file *m, const char *name, const char *value)
{
    seq_putc(m, ',');
    seq_puts(m, name);
    if (value) {
        seq_putc(m, '=');
        seq_puts(m, value);
    }
}

static inline void seq_show_option_n(struct seq_file *m, const char *name, const char *value, size_t length)
{
    seq_putc(m, ',');
    seq_puts(m, name);
    if (value) {
        seq_putc(m, '=');
        seq_write(m, value, length);
    }
}

#endif /* _LINUX_SEQ_FILE_H */
