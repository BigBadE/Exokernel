//! Mutex and semaphore implementation

use core::ffi::c_int;
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use linux_core::linux_export;

// ============================================================================
// Mutex
// ============================================================================

#[repr(C)]
pub struct mutex {
    locked: AtomicBool,
}

impl mutex {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
}

impl Default for mutex {
    fn default() -> Self {
        Self::new()
    }
}

#[linux_export]
fn mutex_init(lock: &mut mutex) {
    lock.locked = AtomicBool::new(false);
}

#[linux_export]
fn mutex_lock(lock: &mutex) {
    while lock
        .locked
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

#[linux_export]
fn mutex_unlock(lock: &mutex) {
    lock.locked.store(false, Ordering::Release);
}

#[linux_export]
fn mutex_trylock(lock: &mutex) -> c_int {
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
fn mutex_is_locked(lock: &mutex) -> c_int {
    if lock.locked.load(Ordering::Relaxed) { 1 } else { 0 }
}

#[linux_export]
fn mutex_lock_interruptible(lock: &mutex) -> c_int {
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
fn mutex_lock_killable(lock: &mutex) -> c_int {
    while lock
        .locked
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
    0
}

// ============================================================================
// Semaphore
// ============================================================================

#[repr(C)]
pub struct semaphore {
    count: AtomicI32,
}

impl semaphore {
    pub const fn new(val: c_int) -> Self {
        Self {
            count: AtomicI32::new(val),
        }
    }
}

#[linux_export]
fn sema_init(sem: &mut semaphore, val: c_int) {
    sem.count = AtomicI32::new(val);
}

#[linux_export]
fn down(sem: &semaphore) {
    loop {
        let count = sem.count.load(Ordering::Relaxed);
        if count > 0 {
            if sem
                .count
                .compare_exchange_weak(count, count - 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        core::hint::spin_loop();
    }
}

#[linux_export]
fn up(sem: &semaphore) {
    sem.count.fetch_add(1, Ordering::Release);
}

#[linux_export]
fn down_trylock(sem: &semaphore) -> c_int {
    let count = sem.count.load(Ordering::Relaxed);
    if count > 0 {
        if sem
            .count
            .compare_exchange(count, count - 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return 0;
        }
    }
    1
}

#[linux_export]
fn down_interruptible(sem: &semaphore) -> c_int {
    loop {
        let count = sem.count.load(Ordering::Relaxed);
        if count > 0 {
            if sem
                .count
                .compare_exchange_weak(count, count - 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        core::hint::spin_loop();
    }
    0
}
