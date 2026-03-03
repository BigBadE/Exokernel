/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_UIO_H
#define _LINUX_UIO_H

#include <linux/types.h>

/* I/O vector for scatter-gather */
struct iovec {
    void __user *iov_base;
    size_t iov_len;
};

struct kvec {
    void *iov_base;
    size_t iov_len;
};

/* Iterator types */
enum iter_type {
    ITER_IOVEC,
    ITER_KVEC,
    ITER_BVEC,
    ITER_XARRAY,
    ITER_DISCARD,
    ITER_UBUF,
};

/* Bio vector */
struct bio_vec {
    struct page *bv_page;
    unsigned int bv_len;
    unsigned int bv_offset;
};

/* I/O iterator */
struct iov_iter {
    u8 iter_type;
    bool copy_mc;
    bool nofault;
    bool data_source;
    size_t iov_offset;
    union {
        size_t count;
        size_t __ubuf_iovec_count;
    };
    union {
        const struct iovec *__iov;
        const struct kvec *kvec;
        const struct bio_vec *bvec;
        struct xarray *xarray;
        void __user *ubuf;
    };
    union {
        unsigned long nr_segs;
        loff_t xarray_start;
    };
};

/* Iterator operations */
extern size_t copy_to_iter(const void *addr, size_t bytes, struct iov_iter *i);
extern size_t copy_from_iter(void *addr, size_t bytes, struct iov_iter *i);
extern size_t copy_from_iter_nocache(void *addr, size_t bytes, struct iov_iter *i);
extern size_t iov_iter_zero(size_t bytes, struct iov_iter *i);
extern unsigned long iov_iter_alignment(const struct iov_iter *i);
extern unsigned long iov_iter_gap_alignment(const struct iov_iter *i);

/* Iterator setup */
extern void iov_iter_init(struct iov_iter *i, unsigned int direction,
                          const struct iovec *iov, unsigned long nr_segs,
                          size_t count);
extern void iov_iter_kvec(struct iov_iter *i, unsigned int direction,
                          const struct kvec *kvec, unsigned long nr_segs,
                          size_t count);
extern void iov_iter_bvec(struct iov_iter *i, unsigned int direction,
                          const struct bio_vec *bvec, unsigned long nr_segs,
                          size_t count);

/* Iterator advancement */
extern void iov_iter_advance(struct iov_iter *i, size_t bytes);
extern void iov_iter_revert(struct iov_iter *i, size_t bytes);
extern size_t iov_iter_count(const struct iov_iter *i);
extern bool iov_iter_is_bvec(const struct iov_iter *i);
extern bool iov_iter_is_kvec(const struct iov_iter *i);

/* Truncate */
static inline void iov_iter_truncate(struct iov_iter *i, size_t count)
{
    if (i->count > count)
        i->count = count;
}

/* XArray (simplified) */
struct xarray {
    void *xa_head;
};

/* Kiocb (kernel I/O control block) */
struct kiocb {
    struct file *ki_filp;
    loff_t ki_pos;
    void (*ki_complete)(struct kiocb *iocb, long ret);
    void *private;
    int ki_flags;
    u16 ki_ioprio;
};

/* Kiocb flags */
#define IOCB_EVENTFD    (1 << 0)
#define IOCB_APPEND     (1 << 1)
#define IOCB_DIRECT     (1 << 2)
#define IOCB_HIPRI      (1 << 3)
#define IOCB_DSYNC      (1 << 4)
#define IOCB_SYNC       (1 << 5)
#define IOCB_WRITE      (1 << 6)
#define IOCB_NOWAIT     (1 << 7)
#define IOCB_NOIO       (1 << 8)

/* Initialize kiocb */
static inline void init_sync_kiocb(struct kiocb *kiocb, struct file *filp)
{
    kiocb->ki_filp = filp;
    kiocb->ki_pos = filp->f_pos;
    kiocb->ki_complete = NULL;
    kiocb->private = NULL;
    kiocb->ki_flags = 0;
    kiocb->ki_ioprio = 0;
}

#endif /* _LINUX_UIO_H */
