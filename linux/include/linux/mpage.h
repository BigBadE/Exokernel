/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_MPAGE_H
#define _LINUX_MPAGE_H

#include <linux/types.h>
#include <linux/fs.h>

/* Forward declarations */
struct address_space;
struct writeback_control;
struct readahead_control;
struct folio;

/* Multi-page IO operations (implemented in Rust or stubs) */
extern void mpage_readahead(struct readahead_control *rac, get_block_t get_block);
extern int mpage_read_folio(struct folio *folio, get_block_t get_block);
extern int mpage_writepages(struct address_space *mapping, struct writeback_control *wbc, get_block_t get_block);
extern int mpage_writepage(struct page *page, get_block_t get_block, struct writeback_control *wbc);

#endif /* _LINUX_MPAGE_H */
