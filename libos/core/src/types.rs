//! Core types used throughout libOS

use core::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, Ordering};

/// Size type
pub type Size = usize;

/// Signed size type
pub type SSize = isize;

/// File offset type
pub type Offset = i64;

/// Sector number type
pub type Sector = u64;

/// Device ID type
pub type DevId = u32;

/// User ID type
pub type Uid = u32;

/// Group ID type
pub type Gid = u32;

/// File mode type
pub type Mode = u32;

/// Time in seconds since epoch
pub type Time = i64;

/// Time in nanoseconds
pub type TimeNs = u64;

/// Timestamp with nanosecond precision
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Timespec {
    pub sec: Time,
    pub nsec: i64,
}

impl Timespec {
    pub const fn new(sec: Time, nsec: i64) -> Self {
        Self { sec, nsec }
    }

    pub const fn zero() -> Self {
        Self { sec: 0, nsec: 0 }
    }

    pub fn now() -> Self {
        // Will be filled in by timekeeping
        Self::zero()
    }
}

/// Allocation flags
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AllocFlags(pub u32);

impl AllocFlags {
    pub const NONE: AllocFlags = AllocFlags(0);
    pub const ZERO: AllocFlags = AllocFlags(1 << 0);
    pub const ATOMIC: AllocFlags = AllocFlags(1 << 1);
    pub const DMA: AllocFlags = AllocFlags(1 << 2);

    pub fn zero(self) -> bool {
        (self.0 & Self::ZERO.0) != 0
    }

    pub fn atomic(self) -> bool {
        (self.0 & Self::ATOMIC.0) != 0
    }
}

impl core::ops::BitOr for AllocFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        AllocFlags(self.0 | rhs.0)
    }
}

/// Atomic 32-bit signed integer
#[repr(transparent)]
pub struct Atomic32(AtomicI32);

impl Atomic32 {
    pub const fn new(val: i32) -> Self {
        Self(AtomicI32::new(val))
    }

    pub fn load(&self) -> i32 {
        self.0.load(Ordering::Relaxed)
    }

    pub fn store(&self, val: i32) {
        self.0.store(val, Ordering::Relaxed);
    }

    pub fn fetch_add(&self, val: i32) -> i32 {
        self.0.fetch_add(val, Ordering::SeqCst)
    }

    pub fn fetch_sub(&self, val: i32) -> i32 {
        self.0.fetch_sub(val, Ordering::SeqCst)
    }

    pub fn inc(&self) {
        self.fetch_add(1);
    }

    pub fn dec(&self) -> i32 {
        self.fetch_sub(1) - 1
    }

    pub fn cmpxchg(&self, old: i32, new: i32) -> i32 {
        match self.0.compare_exchange(old, new, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(v) => v,
            Err(v) => v,
        }
    }

    pub fn xchg(&self, new: i32) -> i32 {
        self.0.swap(new, Ordering::SeqCst)
    }
}

impl Default for Atomic32 {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Atomic unsigned 32-bit integer
#[repr(transparent)]
pub struct AtomicU(AtomicU32);

impl AtomicU {
    pub const fn new(val: u32) -> Self {
        Self(AtomicU32::new(val))
    }

    pub fn load(&self) -> u32 {
        self.0.load(Ordering::Relaxed)
    }

    pub fn store(&self, val: u32) {
        self.0.store(val, Ordering::Relaxed);
    }

    pub fn fetch_or(&self, val: u32) -> u32 {
        self.0.fetch_or(val, Ordering::SeqCst)
    }

    pub fn fetch_and(&self, val: u32) -> u32 {
        self.0.fetch_and(val, Ordering::SeqCst)
    }
}

impl Default for AtomicU {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Atomic 64-bit integer
#[repr(transparent)]
pub struct Atomic64(AtomicU64);

impl Atomic64 {
    pub const fn new(val: u64) -> Self {
        Self(AtomicU64::new(val))
    }

    pub fn load(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    pub fn store(&self, val: u64) {
        self.0.store(val, Ordering::Relaxed);
    }

    pub fn fetch_add(&self, val: u64) -> u64 {
        self.0.fetch_add(val, Ordering::SeqCst)
    }
}

impl Default for Atomic64 {
    fn default() -> Self {
        Self::new(0)
    }
}
