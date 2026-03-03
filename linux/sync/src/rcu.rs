//! RCU (Read-Copy-Update) and memory barriers - Linux wrappers

use core::ffi::c_int;
use core::sync::atomic::Ordering;

use linux_core::{linux_export, rcu_head, atomic_t};
use libos_sync::rcu as libos_rcu;

// Re-export memory barriers with Linux names
#[linux_export]
fn smp_mb() {
    libos_rcu::barrier::smp_mb();
}

#[linux_export]
fn smp_rmb() {
    libos_rcu::barrier::smp_rmb();
}

#[linux_export]
fn smp_wmb() {
    libos_rcu::barrier::smp_wmb();
}

#[linux_export]
fn mb() {
    libos_rcu::barrier::mb();
}

#[linux_export]
fn rmb() {
    libos_rcu::barrier::rmb();
}

#[linux_export]
fn wmb() {
    libos_rcu::barrier::wmb();
}

// RCU operations
#[linux_export]
fn rcu_read_lock() {
    libos_rcu::rcu_read_lock();
}

#[linux_export]
fn rcu_read_unlock() {
    libos_rcu::rcu_read_unlock();
}

#[linux_export]
fn synchronize_rcu() {
    libos_rcu::synchronize_rcu();
}

#[linux_export]
unsafe fn call_rcu(head: *mut rcu_head, func: Option<unsafe extern "C" fn(*mut rcu_head)>) {
    // Simplified: just call immediately
    if let Some(f) = func {
        if !head.is_null() {
            f(head);
        }
    }
}

#[linux_export]
fn rcu_barrier() {
    libos_rcu::rcu_barrier();
}

// Atomic operations
#[linux_export]
fn atomic_dec_return(v: &atomic_t) -> c_int {
    v.fetch_sub(1) - 1
}

#[linux_export]
fn atomic_add_return(i: c_int, v: &atomic_t) -> c_int {
    v.fetch_add(i) + i
}

#[linux_export]
fn atomic_cmpxchg(v: &atomic_t, old: c_int, new: c_int) -> c_int {
    v.cmpxchg(old, new)
}

#[linux_export]
fn atomic_xchg(v: &atomic_t, new: c_int) -> c_int {
    v.xchg(new)
}
