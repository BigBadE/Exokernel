//! Buffer cache implementation
//!
//! Provides a buffer cache for block I/O using idiomatic Rust types.

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicI32, Ordering};

use libos_core::{DevId, Sector, Size, Result};
use libos_sync::{Spinlock, Lazy};
use crate::block::{read_sectors_into, write_sectors};

/// Buffer state flags
pub mod flags {
    pub const UPTODATE: u32 = 1 << 0;
    pub const DIRTY: u32 = 1 << 1;
    pub const LOCKED: u32 = 1 << 2;
    pub const MAPPED: u32 = 1 << 3;
    pub const NEW: u32 = 1 << 4;
}

/// Buffer - represents a cached block
pub struct Buffer {
    /// State flags
    state: AtomicU32,
    /// Block number
    block: Sector,
    /// Block size
    size: Size,
    /// Device ID
    dev: DevId,
    /// Buffer data
    data: Vec<u8>,
    /// Reference count
    refcount: AtomicI32,
}

impl Buffer {
    /// Create a new buffer
    pub fn new(dev: DevId, block: Sector, block_size: Size) -> Self {
        Self {
            state: AtomicU32::new(0),
            block,
            size: block_size,
            dev,
            data: alloc::vec![0u8; block_size],
            refcount: AtomicI32::new(1),
        }
    }

    /// Get the block number
    pub fn block(&self) -> Sector {
        self.block
    }

    /// Get the device ID
    pub fn dev(&self) -> DevId {
        self.dev
    }

    /// Get the buffer size
    pub fn size(&self) -> Size {
        self.size
    }

    /// Check if buffer is up to date
    pub fn is_uptodate(&self) -> bool {
        (self.state.load(Ordering::Relaxed) & flags::UPTODATE) != 0
    }

    /// Check if buffer is dirty
    pub fn is_dirty(&self) -> bool {
        (self.state.load(Ordering::Relaxed) & flags::DIRTY) != 0
    }

    /// Check if buffer is locked
    pub fn is_locked(&self) -> bool {
        (self.state.load(Ordering::Relaxed) & flags::LOCKED) != 0
    }

    /// Set uptodate flag
    pub fn set_uptodate(&self) {
        self.state.fetch_or(flags::UPTODATE, Ordering::SeqCst);
    }

    /// Clear uptodate flag
    pub fn clear_uptodate(&self) {
        self.state.fetch_and(!flags::UPTODATE, Ordering::SeqCst);
    }

    /// Set dirty flag
    pub fn set_dirty(&self) {
        self.state.fetch_or(flags::DIRTY, Ordering::SeqCst);
    }

    /// Clear dirty flag
    pub fn clear_dirty(&self) {
        self.state.fetch_and(!flags::DIRTY, Ordering::SeqCst);
    }

    /// Mark this buffer dirty
    pub fn mark_dirty(&self) {
        self.set_dirty();
    }

    /// Lock the buffer
    pub fn lock(&self) {
        while self.state.fetch_or(flags::LOCKED, Ordering::Acquire) & flags::LOCKED != 0 {
            core::hint::spin_loop();
        }
    }

    /// Unlock the buffer
    pub fn unlock(&self) {
        self.state.fetch_and(!flags::LOCKED, Ordering::Release);
    }

    /// Try to lock
    pub fn try_lock(&self) -> bool {
        (self.state.fetch_or(flags::LOCKED, Ordering::Acquire) & flags::LOCKED) == 0
    }

    /// Get data as slice
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get data as mutable slice
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get reference count
    pub fn refcount(&self) -> i32 {
        self.refcount.load(Ordering::Relaxed)
    }

    /// Increment reference count
    pub fn get(&self) {
        self.refcount.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement reference count
    pub fn put(&self) {
        self.refcount.fetch_sub(1, Ordering::SeqCst);
    }

    /// Sync this buffer to disk
    pub fn sync(&mut self) -> Result<()> {
        if !self.is_dirty() {
            return Ok(());
        }

        write_sectors(self.dev, self.block, &self.data)?;
        self.clear_dirty();
        Ok(())
    }

    /// Read from disk into this buffer
    pub fn read_from_disk(&mut self) -> Result<()> {
        read_sectors_into(self.dev, self.block, &mut self.data)?;
        self.set_uptodate();
        Ok(())
    }

    /// Submit I/O (read or write)
    pub fn submit(&mut self, write: bool) -> Result<()> {
        if write {
            self.sync()
        } else {
            self.read_from_disk()
        }
    }

    /// Forget this buffer (clear flags)
    pub fn forget(&self) {
        self.clear_uptodate();
        self.clear_dirty();
    }
}

/// A shared buffer reference
pub type SharedBuffer = Arc<Spinlock<Buffer>>;

/// Buffer cache
pub struct BufferCache {
    buffers: Spinlock<BTreeMap<(DevId, Sector), SharedBuffer>>,
}

impl BufferCache {
    /// Create a new buffer cache
    pub fn new() -> Self {
        Self {
            buffers: Spinlock::new(BTreeMap::new()),
        }
    }

    /// Get or create a buffer for a block
    fn get_or_create(&self, dev: DevId, block: Sector, block_size: Size) -> SharedBuffer {
        let mut cache = self.buffers.lock();

        if let Some(buf) = cache.get(&(dev, block)) {
            return Arc::clone(buf);
        }

        let buffer = Arc::new(Spinlock::new(Buffer::new(dev, block, block_size)));
        cache.insert((dev, block), Arc::clone(&buffer));
        buffer
    }

    /// Read a block from device
    pub fn bread(&self, dev: DevId, block: Sector, block_size: Size) -> Result<SharedBuffer> {
        let buffer = self.get_or_create(dev, block, block_size);

        {
            let mut buf = buffer.lock();
            if !buf.is_uptodate() {
                buf.read_from_disk()?;
            }
        }

        Ok(buffer)
    }

    /// Get a block (allocate but don't read)
    pub fn getblk(&self, dev: DevId, block: Sector, block_size: Size) -> SharedBuffer {
        self.get_or_create(dev, block, block_size)
    }

    /// Sync all dirty buffers for a device
    pub fn sync_device(&self, dev: DevId) -> Result<()> {
        let cache = self.buffers.lock();

        for ((d, _), buffer) in cache.iter() {
            if *d == dev {
                let mut buf = buffer.lock();
                if buf.is_dirty() {
                    buf.sync()?;
                }
            }
        }

        Ok(())
    }

    /// Sync all dirty buffers
    pub fn sync_all(&self) -> Result<()> {
        let cache = self.buffers.lock();

        for (_, buffer) in cache.iter() {
            let mut buf = buffer.lock();
            if buf.is_dirty() {
                buf.sync()?;
            }
        }

        Ok(())
    }

    /// Invalidate all buffers for a device
    pub fn invalidate_device(&self, dev: DevId) {
        let mut cache = self.buffers.lock();
        cache.retain(|(d, _), _| *d != dev);
    }

    /// Get buffer count
    pub fn buffer_count(&self) -> usize {
        self.buffers.lock().len()
    }
}

impl Default for BufferCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Global buffer cache (lazily initialized)
static BUFFER_CACHE: Lazy<BufferCache> = Lazy::new(BufferCache::new);

fn get_cache() -> &'static BufferCache {
    &BUFFER_CACHE
}

// =============================================================================
// Public API
// =============================================================================

/// Read a block from device
pub fn bread(dev: DevId, block: Sector, block_size: Size) -> Result<SharedBuffer> {
    get_cache().bread(dev, block, block_size)
}

/// Get a block (allocate but don't read)
pub fn getblk(dev: DevId, block: Sector, block_size: Size) -> SharedBuffer {
    get_cache().getblk(dev, block, block_size)
}

/// Sync all dirty buffers for a device
pub fn sync_device(dev: DevId) -> Result<()> {
    get_cache().sync_device(dev)
}

/// Sync all dirty buffers
pub fn sync_all() -> Result<()> {
    get_cache().sync_all()
}

/// Invalidate all buffers for a device
pub fn invalidate_device(dev: DevId) {
    get_cache().invalidate_device(dev)
}
