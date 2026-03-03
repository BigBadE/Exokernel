/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_RANDOM_H
#define _LINUX_RANDOM_H

#include <linux/types.h>

/* Random number generation (implemented in Rust) */
extern void get_random_bytes(void *buf, size_t len);
extern u8 get_random_u8(void);
extern u16 get_random_u16(void);
extern u32 get_random_u32(void);
extern u64 get_random_u64(void);

/* Bounded random */
static inline u32 get_random_u32_below(u32 ceil)
{
    if (ceil == 0)
        return 0;
    return get_random_u32() % ceil;
}

static inline u32 get_random_u32_inclusive(u32 floor, u32 ceil)
{
    return floor + get_random_u32_below(ceil - floor + 1);
}

/* Legacy names */
#define get_random_int()    get_random_u32()
#define get_random_long()   get_random_u64()

/* Entropy pool */
extern void add_device_randomness(const void *buf, size_t len);
extern void add_input_randomness(unsigned int type, unsigned int code, unsigned int value);
extern void add_interrupt_randomness(int irq);
extern void add_disk_randomness(void *disk);

/* UUID generation */
extern void generate_random_uuid(unsigned char uuid[16]);
extern void generate_random_guid(unsigned char guid[16]);

#endif /* _LINUX_RANDOM_H */
