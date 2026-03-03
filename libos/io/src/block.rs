//! Block device abstraction
//!
//! Provides traits and types for block device I/O.

use alloc::vec::Vec;

use libos_core::{DevId, Sector, Result, Error};
use libos_sync::Once;

/// Block device information
#[derive(Debug, Clone)]
pub struct BlockDeviceInfo {
    /// Device ID
    pub dev_id: DevId,
    /// Block size in bytes
    pub block_size: u32,
    /// Total number of sectors
    pub sectors: Sector,
    /// Read-only flag
    pub read_only: bool,
}

impl BlockDeviceInfo {
    /// Create a new block device info
    pub fn new(dev_id: DevId) -> Self {
        Self {
            dev_id,
            block_size: 512,
            sectors: 0,
            read_only: false,
        }
    }

    /// Get logical block size
    pub fn logical_block_size(&self) -> u32 {
        if self.block_size > 0 {
            self.block_size
        } else {
            512
        }
    }

    /// Get number of sectors
    pub fn nr_sectors(&self) -> Sector {
        self.sectors
    }

    /// Get size in bytes
    pub fn nr_bytes(&self) -> u64 {
        self.sectors * 512
    }

    /// Check if read-only
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Set block size
    pub fn set_block_size(&mut self, size: u32) -> Result<()> {
        if size < 512 || size > 4096 || (size & (size - 1)) != 0 {
            return Err(Error::InvalidArgument);
        }
        self.block_size = size;
        Ok(())
    }
}

impl Default for BlockDeviceInfo {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Block I/O operations trait
///
/// This trait must be implemented by the platform to provide
/// actual block device access.
pub trait BlockOps: Send + Sync {
    /// Read sectors from device into buffer
    fn read(&self, dev: DevId, sector: Sector, buffer: &mut [u8]) -> Result<()>;

    /// Write sectors to device from buffer
    fn write(&self, dev: DevId, sector: Sector, buffer: &[u8]) -> Result<()>;

    /// Flush device
    fn flush(&self, dev: DevId) -> Result<()>;

    /// Get device info
    fn info(&self, dev: DevId) -> Result<BlockDeviceInfo>;
}

/// Global block operations holder
static BLOCK_OPS: Once<&'static dyn BlockOps> = Once::new();

/// Set the global block operations
///
/// Can only be called once. Subsequent calls are ignored.
pub fn set_block_ops(ops: &'static dyn BlockOps) {
    BLOCK_OPS.call_once(|| ops);
}

/// Get the global block operations
pub fn get_block_ops() -> Option<&'static dyn BlockOps> {
    BLOCK_OPS.get().copied()
}

/// Read sectors into a new buffer
pub fn read_sectors(dev: DevId, sector: Sector, count: u32) -> Result<Vec<u8>> {
    let ops = get_block_ops().ok_or(Error::NotSupported)?;
    let size = (count as usize) * 512;
    let mut buffer = alloc::vec![0u8; size];
    ops.read(dev, sector, &mut buffer)?;
    Ok(buffer)
}

/// Read sectors into an existing buffer
pub fn read_sectors_into(dev: DevId, sector: Sector, buffer: &mut [u8]) -> Result<()> {
    let ops = get_block_ops().ok_or(Error::NotSupported)?;
    ops.read(dev, sector, buffer)
}

/// Write sectors from buffer
pub fn write_sectors(dev: DevId, sector: Sector, buffer: &[u8]) -> Result<()> {
    let ops = get_block_ops().ok_or(Error::NotSupported)?;
    ops.write(dev, sector, buffer)
}

/// Flush device
pub fn flush_device(dev: DevId) -> Result<()> {
    let ops = get_block_ops().ok_or(Error::NotSupported)?;
    ops.flush(dev)
}

/// Get device info
pub fn device_info(dev: DevId) -> Result<BlockDeviceInfo> {
    let ops = get_block_ops().ok_or(Error::NotSupported)?;
    ops.info(dev)
}

/// A block device handle
pub struct BlockDevice {
    info: BlockDeviceInfo,
}

impl BlockDevice {
    /// Open a block device
    pub fn open(dev_id: DevId) -> Result<Self> {
        let info = device_info(dev_id)?;
        Ok(Self { info })
    }

    /// Get device info
    pub fn info(&self) -> &BlockDeviceInfo {
        &self.info
    }

    /// Get device ID
    pub fn dev_id(&self) -> DevId {
        self.info.dev_id
    }

    /// Read sectors
    pub fn read(&self, sector: Sector, count: u32) -> Result<Vec<u8>> {
        read_sectors(self.info.dev_id, sector, count)
    }

    /// Read sectors into buffer
    pub fn read_into(&self, sector: Sector, buffer: &mut [u8]) -> Result<()> {
        read_sectors_into(self.info.dev_id, sector, buffer)
    }

    /// Write sectors
    pub fn write(&self, sector: Sector, buffer: &[u8]) -> Result<()> {
        write_sectors(self.info.dev_id, sector, buffer)
    }

    /// Flush
    pub fn flush(&self) -> Result<()> {
        flush_device(self.info.dev_id)
    }
}
