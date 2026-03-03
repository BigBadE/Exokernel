//! Per-process capability table view
//!
//! Each process has a view into the global capability table,
//! containing only the capabilities it owns.

use alloc::vec::Vec;
use exo_shared::{CapabilityHandle, SysResult, SysError};

/// Per-process capability table
/// This is a compact view of capabilities owned by a single process
pub struct CapabilityTable {
    /// Process ID
    pid: u64,

    /// Local handle to global handle mapping
    /// Index is local handle, value is global handle
    local_to_global: Vec<CapabilityHandle>,

    /// Maximum local handles
    max_caps: usize,
}

impl CapabilityTable {
    pub const DEFAULT_MAX_CAPS: usize = 64;

    pub fn new(pid: u64) -> Self {
        Self::with_capacity(pid, Self::DEFAULT_MAX_CAPS)
    }

    pub fn with_capacity(pid: u64, max_caps: usize) -> Self {
        let mut local_to_global = Vec::with_capacity(max_caps);
        // Slot 0 is NULL
        local_to_global.push(CapabilityHandle::NULL);

        CapabilityTable {
            pid,
            local_to_global,
            max_caps,
        }
    }

    /// Add a capability to this process's table
    pub fn add(&mut self, global_handle: CapabilityHandle) -> SysResult<CapabilityHandle> {
        if self.local_to_global.len() >= self.max_caps {
            return Err(SysError::CapabilityTableFull);
        }

        let local_handle = CapabilityHandle(self.local_to_global.len() as u64);
        self.local_to_global.push(global_handle);

        Ok(local_handle)
    }

    /// Remove a capability from this process's table
    pub fn remove(&mut self, local_handle: CapabilityHandle) -> SysResult<CapabilityHandle> {
        let idx = local_handle.0 as usize;

        if idx == 0 || idx >= self.local_to_global.len() {
            return Err(SysError::InvalidCapability);
        }

        let global = self.local_to_global[idx];
        if global.is_null() {
            return Err(SysError::InvalidCapability);
        }

        self.local_to_global[idx] = CapabilityHandle::NULL;
        Ok(global)
    }

    /// Translate local handle to global handle
    pub fn translate(&self, local_handle: CapabilityHandle) -> SysResult<CapabilityHandle> {
        let idx = local_handle.0 as usize;

        if idx >= self.local_to_global.len() {
            return Err(SysError::InvalidCapability);
        }

        let global = self.local_to_global[idx];
        if global.is_null() {
            return Err(SysError::InvalidCapability);
        }

        Ok(global)
    }

    /// Get all valid local handles
    pub fn all_handles(&self) -> Vec<CapabilityHandle> {
        self.local_to_global
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.is_null())
            .map(|(i, _)| CapabilityHandle(i as u64))
            .collect()
    }

    /// Get the process ID
    pub fn pid(&self) -> u64 {
        self.pid
    }

    /// Get number of capabilities
    pub fn count(&self) -> usize {
        self.local_to_global.iter().filter(|h| !h.is_null()).count()
    }
}
