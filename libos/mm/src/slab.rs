//! Slab allocator implementation using Rust types
//!
//! This provides a type-safe slab cache for fixed-size allocations.

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use core::marker::PhantomData;

use libos_core::Result;

/// A slab cache for allocating objects of type T
pub struct SlabCache<T> {
    /// Cache name for debugging
    name: String,
    /// Free list of objects
    free_list: Vec<Box<T>>,
    /// Number of allocated objects
    allocated: usize,
    /// Phantom data for T
    _marker: PhantomData<T>,
}

impl<T: Default> SlabCache<T> {
    /// Create a new slab cache
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            free_list: Vec::new(),
            allocated: 0,
            _marker: PhantomData,
        }
    }

    /// Get the cache name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the object size
    pub fn object_size(&self) -> usize {
        core::mem::size_of::<T>()
    }

    /// Get the number of allocated objects
    pub fn allocated(&self) -> usize {
        self.allocated
    }

    /// Allocate an object from the cache
    pub fn alloc(&mut self) -> Result<Box<T>> {
        self.allocated += 1;
        if let Some(obj) = self.free_list.pop() {
            Ok(obj)
        } else {
            Ok(Box::new(T::default()))
        }
    }

    /// Free an object back to the cache
    pub fn free(&mut self, obj: Box<T>) {
        if self.allocated > 0 {
            self.allocated -= 1;
        }
        self.free_list.push(obj);
    }

    /// Shrink the cache by freeing unused objects
    pub fn shrink(&mut self) {
        self.free_list.clear();
    }
}

impl<T> SlabCache<T> {
    /// Create a new slab cache with a custom initializer
    pub fn with_init<F: Fn() -> T + 'static>(name: &str, init: F) -> SlabCacheWithInit<T, F> {
        SlabCacheWithInit {
            name: String::from(name),
            free_list: Vec::new(),
            allocated: 0,
            init,
        }
    }
}

/// A slab cache with a custom initializer function
pub struct SlabCacheWithInit<T, F: Fn() -> T> {
    name: String,
    free_list: Vec<Box<T>>,
    allocated: usize,
    init: F,
}

impl<T, F: Fn() -> T> SlabCacheWithInit<T, F> {
    /// Allocate an object from the cache
    pub fn alloc(&mut self) -> Result<Box<T>> {
        self.allocated += 1;
        if let Some(obj) = self.free_list.pop() {
            Ok(obj)
        } else {
            Ok(Box::new((self.init)()))
        }
    }

    /// Free an object back to the cache
    pub fn free(&mut self, obj: Box<T>) {
        if self.allocated > 0 {
            self.allocated -= 1;
        }
        self.free_list.push(obj);
    }
}
