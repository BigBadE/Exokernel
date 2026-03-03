//! Core Linux kernel types
//!
//! Fundamental types used throughout the kernel: atomic integers,
//! linked lists, time structures, and basic typedefs.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use core::ptr;
use core::sync::atomic::{AtomicI32, AtomicI64, AtomicIsize, Ordering};

// ============================================================================
// Basic type aliases
// ============================================================================

pub type size_t = usize;
pub type ssize_t = isize;
pub type loff_t = i64;
pub type sector_t = u64;
pub type blkcnt_t = u64;
pub type time64_t = i64;
pub type ktime_t = i64;
pub type umode_t = u16;
pub type kuid_t = u32;
pub type kgid_t = u32;
pub type dev_t = u32;
pub type fmode_t = u32;
pub type gfp_t = u32;
pub type ino_t = u64;
pub type wchar_t = u16;

// ============================================================================
// Atomic types
// ============================================================================

/// Atomic integer (uses AtomicI32 internally for safe atomic operations)
#[repr(C)]
pub struct atomic_t {
    counter: AtomicI32,
}

impl atomic_t {
    pub const fn new(val: c_int) -> Self {
        Self { counter: AtomicI32::new(val) }
    }

    /// Atomically read the value
    pub fn read(&self) -> c_int {
        self.counter.load(Ordering::SeqCst)
    }

    /// Atomically set the value
    pub fn set(&self, val: c_int) {
        self.counter.store(val, Ordering::SeqCst);
    }

    /// Atomically add and return the old value
    pub fn fetch_add(&self, val: c_int) -> c_int {
        self.counter.fetch_add(val, Ordering::SeqCst)
    }

    /// Atomically subtract and return the old value
    pub fn fetch_sub(&self, val: c_int) -> c_int {
        self.counter.fetch_sub(val, Ordering::SeqCst)
    }

    /// Atomically increment
    pub fn inc(&self) {
        self.fetch_add(1);
    }

    /// Atomically decrement and return true if result is zero
    pub fn dec_and_test(&self) -> bool {
        self.fetch_sub(1) == 1
    }

    /// Compare and exchange: if current == old, set to new and return old
    /// Otherwise return current value
    pub fn cmpxchg(&self, old: c_int, new: c_int) -> c_int {
        match self.counter.compare_exchange(old, new, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(val) => val,
            Err(val) => val,
        }
    }

    /// Atomically swap value and return old value
    pub fn xchg(&self, new: c_int) -> c_int {
        self.counter.swap(new, Ordering::SeqCst)
    }
}

impl Default for atomic_t {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Atomic long (uses AtomicIsize internally for safe atomic operations)
#[repr(C)]
#[derive(Debug)]
pub struct atomic_long_t {
    counter: AtomicIsize,
}

impl atomic_long_t {
    pub const fn new(val: c_long) -> Self {
        Self { counter: AtomicIsize::new(val as isize) }
    }

    pub fn read(&self) -> c_long {
        self.counter.load(Ordering::SeqCst) as c_long
    }

    pub fn set(&self, val: c_long) {
        self.counter.store(val as isize, Ordering::SeqCst);
    }

    pub fn fetch_add(&self, val: c_long) -> c_long {
        self.counter.fetch_add(val as isize, Ordering::SeqCst) as c_long
    }

    pub fn fetch_sub(&self, val: c_long) -> c_long {
        self.counter.fetch_sub(val as isize, Ordering::SeqCst) as c_long
    }
}

impl Default for atomic_long_t {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Atomic 64-bit (uses AtomicI64 internally for safe atomic operations)
#[repr(C)]
pub struct atomic64_t {
    counter: AtomicI64,
}

impl atomic64_t {
    pub const fn new(val: i64) -> Self {
        Self { counter: AtomicI64::new(val) }
    }

    pub fn read(&self) -> i64 {
        self.counter.load(Ordering::SeqCst)
    }

    pub fn set(&self, val: i64) {
        self.counter.store(val, Ordering::SeqCst);
    }

    pub fn fetch_add(&self, val: i64) -> i64 {
        self.counter.fetch_add(val, Ordering::SeqCst)
    }

    pub fn fetch_sub(&self, val: i64) -> i64 {
        self.counter.fetch_sub(val, Ordering::SeqCst)
    }
}

impl Default for atomic64_t {
    fn default() -> Self {
        Self::new(0)
    }
}

// ============================================================================
// Time types
// ============================================================================

/// Time specification (64-bit seconds)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct timespec64 {
    pub tv_sec: time64_t,
    pub tv_nsec: c_long,
}

// ============================================================================
// List types (Linux intrusive linked lists)
// ============================================================================

/// Doubly linked list head
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct list_head {
    pub next: *mut list_head,
    pub prev: *mut list_head,
}

impl list_head {
    pub const fn new() -> Self {
        Self {
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }

    pub fn init(&mut self) {
        self.next = self as *mut _;
        self.prev = self as *mut _;
    }

    pub fn is_empty(&self) -> bool {
        self.next == self as *const _ as *mut _
    }
}

/// Hash list head
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct hlist_head {
    pub first: *mut hlist_node,
}

/// Hash list node
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct hlist_node {
    pub next: *mut hlist_node,
    pub pprev: *mut *mut hlist_node,
}

/// Hash list for block layer
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct hlist_bl_head {
    pub first: *mut hlist_bl_node,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct hlist_bl_node {
    pub next: *mut hlist_bl_node,
    pub pprev: *mut *mut hlist_bl_node,
}

// ============================================================================
// RCU types
// ============================================================================

/// RCU callback head
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct rcu_head {
    pub next: *mut rcu_head,
    pub func: Option<unsafe extern "C" fn(*mut rcu_head)>,
}

// ============================================================================
// Misc kernel types
// ============================================================================

/// Lockref - combined lock and reference count
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct lockref {
    pub lock: u32,
    pub count: c_uint,
}

/// Sequence lock
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct seqlock_t {
    pub sequence: c_uint,
    pub lock: u32,
}

/// Quick string (hashed name)
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct qstr {
    pub hash: c_uint,
    pub len: c_uint,
    pub name: *const u8,
}

/// Wait queue head
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct wait_queue_head_t {
    pub lock: u32,
    pub head: list_head,
}

/// Completion
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct completion {
    pub done: c_uint,
    pub wait: wait_queue_head_t,
}

// ============================================================================
// Mode constants
// ============================================================================

pub const S_IFMT: umode_t = 0o170000;
pub const S_IFSOCK: umode_t = 0o140000;
pub const S_IFLNK: umode_t = 0o120000;
pub const S_IFREG: umode_t = 0o100000;
pub const S_IFBLK: umode_t = 0o060000;
pub const S_IFDIR: umode_t = 0o040000;
pub const S_IFCHR: umode_t = 0o020000;
pub const S_IFIFO: umode_t = 0o010000;
pub const S_ISUID: umode_t = 0o004000;
pub const S_ISGID: umode_t = 0o002000;
pub const S_ISVTX: umode_t = 0o001000;

pub const S_IRWXU: umode_t = 0o0700;
pub const S_IRUSR: umode_t = 0o0400;
pub const S_IWUSR: umode_t = 0o0200;
pub const S_IXUSR: umode_t = 0o0100;
pub const S_IRWXG: umode_t = 0o0070;
pub const S_IRGRP: umode_t = 0o0040;
pub const S_IWGRP: umode_t = 0o0020;
pub const S_IXGRP: umode_t = 0o0010;
pub const S_IRWXO: umode_t = 0o0007;
pub const S_IROTH: umode_t = 0o0004;
pub const S_IWOTH: umode_t = 0o0002;
pub const S_IXOTH: umode_t = 0o0001;

/// File type check macros
#[inline]
pub fn S_ISREG(mode: umode_t) -> bool {
    (mode & S_IFMT) == S_IFREG
}

#[inline]
pub fn S_ISDIR(mode: umode_t) -> bool {
    (mode & S_IFMT) == S_IFDIR
}

#[inline]
pub fn S_ISCHR(mode: umode_t) -> bool {
    (mode & S_IFMT) == S_IFCHR
}

#[inline]
pub fn S_ISBLK(mode: umode_t) -> bool {
    (mode & S_IFMT) == S_IFBLK
}

#[inline]
pub fn S_ISFIFO(mode: umode_t) -> bool {
    (mode & S_IFMT) == S_IFIFO
}

#[inline]
pub fn S_ISLNK(mode: umode_t) -> bool {
    (mode & S_IFMT) == S_IFLNK
}

#[inline]
pub fn S_ISSOCK(mode: umode_t) -> bool {
    (mode & S_IFMT) == S_IFSOCK
}

// ============================================================================
// Error pointer utilities
// ============================================================================

const MAX_ERRNO: usize = 4095;

#[inline]
pub fn IS_ERR<T>(ptr: *const T) -> bool {
    (ptr as usize) >= (usize::MAX - MAX_ERRNO)
}

#[inline]
pub fn IS_ERR_OR_NULL<T>(ptr: *const T) -> bool {
    ptr.is_null() || IS_ERR(ptr)
}

#[inline]
pub fn PTR_ERR<T>(ptr: *const T) -> c_long {
    ptr as c_long
}

#[inline]
pub fn ERR_PTR<T>(error: c_long) -> *mut T {
    error as *mut T
}

#[inline]
pub fn ERR_CAST<T, U>(ptr: *const T) -> *mut U {
    ptr as *mut U
}

// ============================================================================
// GFP flags (memory allocation)
// ============================================================================

pub const GFP_KERNEL: gfp_t = 0x0;
pub const GFP_ATOMIC: gfp_t = 0x1;
pub const GFP_NOWAIT: gfp_t = 0x2;
pub const GFP_NOIO: gfp_t = 0x4;
pub const GFP_NOFS: gfp_t = 0x8;
pub const GFP_USER: gfp_t = 0x10;
pub const GFP_HIGHUSER: gfp_t = 0x20;
pub const GFP_DMA: gfp_t = 0x40;
pub const GFP_ZERO: gfp_t = 0x100;

// ============================================================================
// C exports for atomic operations
// ============================================================================

use crate::linux_export;

#[linux_export]
fn atomic_read(v: &atomic_t) -> c_int {
    v.read()
}

#[linux_export]
fn atomic_set(v: &atomic_t, i: c_int) {
    v.set(i)
}

#[linux_export]
fn atomic_add(i: c_int, v: &atomic_t) {
    v.fetch_add(i);
}

#[linux_export]
fn atomic_sub(i: c_int, v: &atomic_t) {
    v.fetch_sub(i);
}

#[linux_export]
fn atomic_inc(v: &atomic_t) {
    v.inc();
}

#[linux_export]
fn atomic_dec(v: &atomic_t) {
    v.fetch_sub(1);
}

#[linux_export]
fn atomic_dec_and_test(v: &atomic_t) -> c_int {
    v.dec_and_test() as c_int
}

#[linux_export]
fn atomic_inc_return(v: &atomic_t) -> c_int {
    v.fetch_add(1) + 1
}

// ============================================================================
// C exports for list operations
// ============================================================================

#[linux_export]
fn list_empty(head: &list_head) -> c_int {
    head.is_empty() as c_int
}

#[linux_export]
fn list_del_init(entry: &mut list_head) {
    let prev = entry.prev;
    let next = entry.next;
    unsafe {
        if !prev.is_null() {
            (*prev).next = next;
        }
        if !next.is_null() {
            (*next).prev = prev;
        }
    }
    entry.next = entry as *mut _;
    entry.prev = entry as *mut _;
}

#[linux_export]
fn list_move(entry: &mut list_head, head: &mut list_head) {
    // Remove from current position
    let prev = entry.prev;
    let next = entry.next;
    unsafe {
        if !prev.is_null() {
            (*prev).next = next;
        }
        if !next.is_null() {
            (*next).prev = prev;
        }
    }
    // Add after head
    let head_next = head.next;
    entry.next = head_next;
    entry.prev = head as *mut _;
    head.next = entry as *mut _;
    unsafe {
        if !head_next.is_null() {
            (*head_next).prev = entry as *mut _;
        }
    }
}

#[linux_export]
fn hlist_add_head(n: &mut hlist_node, h: &mut hlist_head) {
    let first = h.first;
    n.next = first;
    unsafe {
        if !first.is_null() {
            (*first).pprev = &mut n.next;
        }
    }
    h.first = n as *mut _;
    n.pprev = &mut h.first;
}

#[linux_export]
fn hlist_del_init(n: &mut hlist_node) {
    let next = n.next;
    let pprev = n.pprev;
    unsafe {
        if !pprev.is_null() {
            *pprev = next;
        }
        if !next.is_null() {
            (*next).pprev = pprev;
        }
    }
    n.next = ptr::null_mut();
    n.pprev = ptr::null_mut();
}
