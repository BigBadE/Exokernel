//! LibOS synchronization primitives
//!
//! Re-exports from the `spin` crate for no_std synchronization.

#![no_std]

extern crate alloc;

// Re-export spin crate primitives
pub use spin::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use spin::{Once, Lazy};

// Spinlock is just an alias for Mutex in spin crate
pub type Spinlock<T> = spin::Mutex<T>;
pub type SpinlockGuard<'a, T> = spin::MutexGuard<'a, T>;

pub mod barrier {
    //! Memory barrier operations

    use core::sync::atomic::Ordering;

    /// Full memory barrier
    #[inline]
    pub fn mb() {
        core::sync::atomic::fence(Ordering::SeqCst);
    }

    /// Read memory barrier
    #[inline]
    pub fn rmb() {
        core::sync::atomic::fence(Ordering::Acquire);
    }

    /// Write memory barrier
    #[inline]
    pub fn wmb() {
        core::sync::atomic::fence(Ordering::Release);
    }

    /// SMP full memory barrier
    #[inline]
    pub fn smp_mb() {
        core::sync::atomic::fence(Ordering::SeqCst);
    }

    /// SMP read memory barrier
    #[inline]
    pub fn smp_rmb() {
        core::sync::atomic::fence(Ordering::Acquire);
    }

    /// SMP write memory barrier
    #[inline]
    pub fn smp_wmb() {
        core::sync::atomic::fence(Ordering::Release);
    }
}
