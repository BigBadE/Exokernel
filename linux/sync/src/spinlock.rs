//! Spinlock implementation

use core::ffi::{c_int, c_ulong};
use core::sync::atomic::{AtomicBool, Ordering};

use linux_core::linux_export;

// ============================================================================
// Spinlock
// ============================================================================

#[repr(C)]
pub struct spinlock_t {
    locked: AtomicBool,
}

impl spinlock_t {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
}

impl Default for spinlock_t {
    fn default() -> Self {
        Self::new()
    }
}

#[linux_export]
fn spin_lock_init(lock: &mut spinlock_t) {
    lock.locked = AtomicBool::new(false);
}

#[linux_export]
fn spin_lock(lock: &spinlock_t) {
    while lock
        .locked
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

#[linux_export]
fn spin_unlock(lock: &spinlock_t) {
    lock.locked.store(false, Ordering::Release);
}

#[linux_export]
fn spin_trylock(lock: &spinlock_t) -> c_int {
    if lock
        .locked
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        1
    } else {
        0
    }
}

#[linux_export]
fn spin_lock_irq(lock: &spinlock_t) {
    while lock
        .locked
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

#[linux_export]
fn spin_unlock_irq(lock: &spinlock_t) {
    lock.locked.store(false, Ordering::Release);
}

#[linux_export]
fn spin_lock_irqsave(lock: &spinlock_t, _flags: c_ulong) -> c_ulong {
    while lock
        .locked
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
    0
}

#[linux_export]
fn spin_unlock_irqrestore(lock: &spinlock_t, _flags: c_ulong) {
    lock.locked.store(false, Ordering::Release);
}

#[linux_export]
fn spin_lock_bh(lock: &spinlock_t) {
    while lock
        .locked
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

#[linux_export]
fn spin_unlock_bh(lock: &spinlock_t) {
    lock.locked.store(false, Ordering::Release);
}

// ============================================================================
// Raw spinlock (same as spinlock in our implementation)
// ============================================================================

#[repr(C)]
pub struct raw_spinlock_t {
    locked: AtomicBool,
}

impl raw_spinlock_t {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
}

impl Default for raw_spinlock_t {
    fn default() -> Self {
        Self::new()
    }
}

#[linux_export]
fn raw_spin_lock_init(lock: &mut raw_spinlock_t) {
    lock.locked = AtomicBool::new(false);
}

#[linux_export]
fn raw_spin_lock(lock: &raw_spinlock_t) {
    while lock
        .locked
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

#[linux_export]
fn raw_spin_unlock(lock: &raw_spinlock_t) {
    lock.locked.store(false, Ordering::Release);
}
