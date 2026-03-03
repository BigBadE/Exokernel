//! Read-write semaphore implementation

use core::ffi::c_int;
use core::sync::atomic::{AtomicI32, Ordering};

use linux_core::linux_export;

// ============================================================================
// RW Semaphore
// ============================================================================

#[repr(C)]
pub struct rw_semaphore {
    // Positive = number of readers, -1 = writer holds it
    count: AtomicI32,
}

impl rw_semaphore {
    pub const fn new() -> Self {
        Self {
            count: AtomicI32::new(0),
        }
    }
}

impl Default for rw_semaphore {
    fn default() -> Self {
        Self::new()
    }
}

#[linux_export]
fn init_rwsem(sem: &mut rw_semaphore) {
    sem.count = AtomicI32::new(0);
}

#[linux_export]
fn down_read(sem: &rw_semaphore) {
    loop {
        let count = sem.count.load(Ordering::Relaxed);
        if count >= 0 {
            if sem
                .count
                .compare_exchange_weak(count, count + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        core::hint::spin_loop();
    }
}

#[linux_export]
fn up_read(sem: &rw_semaphore) {
    sem.count.fetch_sub(1, Ordering::Release);
}

#[linux_export]
fn down_write(sem: &rw_semaphore) {
    while sem
        .count
        .compare_exchange_weak(0, -1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

#[linux_export]
fn up_write(sem: &rw_semaphore) {
    sem.count.store(0, Ordering::Release);
}

#[linux_export]
fn down_read_trylock(sem: &rw_semaphore) -> c_int {
    let count = sem.count.load(Ordering::Relaxed);
    if count >= 0 {
        if sem
            .count
            .compare_exchange(count, count + 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return 1;
        }
    }
    0
}

#[linux_export]
fn down_write_trylock(sem: &rw_semaphore) -> c_int {
    if sem
        .count
        .compare_exchange(0, -1, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        1
    } else {
        0
    }
}

#[linux_export]
fn downgrade_write(sem: &rw_semaphore) {
    sem.count.store(1, Ordering::Release);
}

#[linux_export]
fn down_read_killable(sem: &rw_semaphore) -> c_int {
    loop {
        let count = sem.count.load(Ordering::Relaxed);
        if count >= 0 {
            if sem
                .count
                .compare_exchange_weak(count, count + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        core::hint::spin_loop();
    }
    0
}

#[linux_export]
fn down_write_killable(sem: &rw_semaphore) -> c_int {
    while sem
        .count
        .compare_exchange_weak(0, -1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
    0
}
