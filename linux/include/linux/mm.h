/* SPDX-License-Identifier: GPL-2.0 */
#ifndef _LINUX_MM_H
#define _LINUX_MM_H

#include <linux/types.h>
#include <linux/gfp.h>
#include <linux/list.h>

/* Forward declarations */
struct page;
struct vm_area_struct;
struct mm_struct;
struct vm_fault;
struct file;

/* Page protection and offset types */
typedef unsigned long pgprot_t;
typedef unsigned long pgoff_t;

/* VM fault return type */
typedef unsigned int vm_fault_t;

/* PTE type (architecture specific) */
typedef unsigned long pte_t;

/* Page allocation (implemented in Rust) */
extern struct page *alloc_pages(gfp_t gfp, unsigned int order);
extern struct page *alloc_page(gfp_t gfp);
extern void __free_pages(struct page *page, unsigned int order);
extern void free_page(unsigned long addr);
extern unsigned long __get_free_pages(gfp_t gfp, unsigned int order);
extern unsigned long get_zeroed_page(gfp_t gfp);

/* Page reference counting */
extern void get_page(struct page *page);
extern void put_page(struct page *page);
extern int page_count(struct page *page);

/* Page address translation */
extern void *page_address(struct page *page);
extern struct page *virt_to_page(const void *addr);
extern unsigned long page_to_pfn(struct page *page);
extern struct page *pfn_to_page(unsigned long pfn);

/* High memory (no-op on 64-bit) */
static inline void *kmap(struct page *page)
{
    return page_address(page);
}

static inline void kunmap(struct page *page)
{
}

static inline void *kmap_local_page(struct page *page)
{
    return page_address(page);
}

static inline void kunmap_local(const void *addr)
{
}

static inline void *kmap_atomic(struct page *page)
{
    return page_address(page);
}

static inline void kunmap_atomic(void *addr)
{
}

/* VM area struct */
struct vm_area_struct {
    unsigned long vm_start;
    unsigned long vm_end;
    struct vm_area_struct *vm_next;
    struct mm_struct *vm_mm;
    pgprot_t vm_page_prot;
    unsigned long vm_flags;
    struct file *vm_file;
    pgoff_t vm_pgoff;
    void *vm_private_data;
};

/* VM flags */
#define VM_NONE         0x00000000
#define VM_READ         0x00000001
#define VM_WRITE        0x00000002
#define VM_EXEC         0x00000004
#define VM_SHARED       0x00000008
#define VM_MAYREAD      0x00000010
#define VM_MAYWRITE     0x00000020
#define VM_MAYEXEC      0x00000040
#define VM_MAYSHARE     0x00000080
#define VM_GROWSDOWN    0x00000100
#define VM_GROWSUP      0x00000200
#define VM_IO           0x00004000
#define VM_SEQ_READ     0x00008000
#define VM_RAND_READ    0x00010000
#define VM_DONTCOPY     0x00020000
#define VM_DONTEXPAND   0x00040000
#define VM_ACCOUNT      0x00100000
#define VM_NORESERVE    0x00200000
#define VM_HUGETLB      0x00400000
#define VM_DONTDUMP     0x04000000
#define VM_MIXEDMAP     0x10000000
#define VM_HUGEPAGE     0x20000000
#define VM_NOHUGEPAGE   0x40000000

/* VM operations */
struct vm_operations_struct {
    void (*open)(struct vm_area_struct *area);
    void (*close)(struct vm_area_struct *area);
    int (*may_split)(struct vm_area_struct *area, unsigned long addr);
    int (*mremap)(struct vm_area_struct *area);
    int (*mprotect)(struct vm_area_struct *vma, unsigned long start, unsigned long end, unsigned long newflags);
    vm_fault_t (*fault)(struct vm_fault *vmf);
    vm_fault_t (*huge_fault)(struct vm_fault *vmf, unsigned int order);
    vm_fault_t (*map_pages)(struct vm_fault *vmf, pgoff_t start_pgoff, pgoff_t end_pgoff);
    unsigned long (*pagesize)(struct vm_area_struct *area);
    vm_fault_t (*page_mkwrite)(struct vm_fault *vmf);
    vm_fault_t (*pfn_mkwrite)(struct vm_fault *vmf);
    int (*access)(struct vm_area_struct *vma, unsigned long addr, void *buf, int len, int write);
    const char *(*name)(struct vm_area_struct *vma);
    struct page *(*find_special_page)(struct vm_area_struct *vma, unsigned long addr);
};

/* VM fault codes */
#define VM_FAULT_OOM            0x000001
#define VM_FAULT_SIGBUS         0x000002
#define VM_FAULT_MAJOR          0x000004
#define VM_FAULT_WRITE          0x000008
#define VM_FAULT_HWPOISON       0x000010
#define VM_FAULT_HWPOISON_LARGE 0x000020
#define VM_FAULT_SIGSEGV        0x000040
#define VM_FAULT_NOPAGE         0x000100
#define VM_FAULT_LOCKED         0x000200
#define VM_FAULT_RETRY          0x000400
#define VM_FAULT_FALLBACK       0x000800
#define VM_FAULT_DONE_COW       0x001000
#define VM_FAULT_NEEDDSYNC      0x002000
#define VM_FAULT_COMPLETED      0x004000
#define VM_FAULT_HINDEX_MASK    0x0f0000

/* VM fault struct */
struct vm_fault {
    struct {
        struct vm_area_struct *vma;
        gfp_t gfp_mask;
        pgoff_t pgoff;
        unsigned long address;
        unsigned long real_address;
    };
    unsigned int flags;
    pte_t *pte;
    pte_t orig_pte;
    struct page *cow_page;
    struct page *page;
    pte_t *prealloc_pte;
};

#endif /* _LINUX_MM_H */
