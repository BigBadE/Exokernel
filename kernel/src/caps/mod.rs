//! Capability System - The Core of the Exokernel
//!
//! This module implements the kernel-side capability table and operations.
//! All resource access in the system is mediated through capabilities.

pub mod table;

use alloc::vec::Vec;
use spin::Mutex;

use exo_shared::{
    Capability, CapabilityHandle, CapabilityInfo, Generation,
    ResourceDescriptor, ResourceType, Rights, SysError, SysResult,
};

pub use table::CapabilityTable;

/// Maximum capabilities per process
pub const MAX_CAPS_PER_PROCESS: usize = 256;

/// Maximum total capabilities in the system
pub const MAX_TOTAL_CAPS: usize = 4096;

/// Global capability table
static CAPABILITY_TABLE: Mutex<Option<GlobalCapTable>> = Mutex::new(None);

/// Global capability table structure
pub struct GlobalCapTable {
    /// All capabilities in the system
    caps: Vec<CapabilityEntry>,

    /// Current generation (for revocation)
    current_generation: Generation,

    /// Free list of capability slots
    free_slots: Vec<usize>,
}

/// Entry in the capability table
#[derive(Clone)]
struct CapabilityEntry {
    cap: Capability,
    valid: bool,
}

impl GlobalCapTable {
    pub fn new() -> Self {
        let mut caps = Vec::with_capacity(MAX_TOTAL_CAPS);
        let mut free_slots = Vec::with_capacity(MAX_TOTAL_CAPS);

        // Initialize all slots as free
        for i in 0..MAX_TOTAL_CAPS {
            caps.push(CapabilityEntry {
                cap: Capability::null(),
                valid: false,
            });
            // Slot 0 is reserved as NULL
            if i > 0 {
                free_slots.push(i);
            }
        }

        GlobalCapTable {
            caps,
            current_generation: Generation::INITIAL,
            free_slots,
        }
    }

    /// Allocate a new capability slot
    fn alloc_slot(&mut self) -> Option<usize> {
        self.free_slots.pop()
    }

    /// Free a capability slot
    fn free_slot(&mut self, slot: usize) {
        if slot > 0 && slot < MAX_TOTAL_CAPS {
            self.caps[slot].valid = false;
            self.caps[slot].cap = Capability::null();
            self.free_slots.push(slot);
        }
    }

    /// Create a new root capability (kernel only)
    pub fn create_root_cap(
        &mut self,
        resource: ResourceDescriptor,
        rights: Rights,
        owner: u64,
    ) -> SysResult<CapabilityHandle> {
        let slot = self.alloc_slot().ok_or(SysError::CapabilityTableFull)?;

        let cap = Capability {
            resource,
            rights,
            generation: self.current_generation,
            parent: CapabilityHandle::NULL,
            owner,
        };

        self.caps[slot] = CapabilityEntry { cap, valid: true };

        Ok(CapabilityHandle(slot as u64))
    }

    /// Grant a capability (create derived capability)
    pub fn grant(
        &mut self,
        parent_handle: CapabilityHandle,
        resource: ResourceDescriptor,
        rights: Rights,
        target_owner: u64,
    ) -> SysResult<CapabilityHandle> {
        // Validate parent capability
        let parent = self.get(parent_handle)?;

        // Check GRANT right
        if !parent.rights.contains(Rights::GRANT) {
            return Err(SysError::InsufficientRights);
        }

        // Check that requested rights are a subset of parent's rights
        if !parent.rights.contains(rights) {
            return Err(SysError::InsufficientRights);
        }

        // Check that requested resource is contained within parent's resource
        if !parent.resource.contains(&resource) {
            return Err(SysError::InvalidArgument);
        }

        // Allocate new slot
        let slot = self.alloc_slot().ok_or(SysError::CapabilityTableFull)?;

        let cap = Capability {
            resource,
            rights,
            generation: self.current_generation,
            parent: parent_handle,
            owner: target_owner,
        };

        self.caps[slot] = CapabilityEntry { cap, valid: true };

        Ok(CapabilityHandle(slot as u64))
    }

    /// Delegate a capability to another process
    pub fn delegate(
        &mut self,
        handle: CapabilityHandle,
        requesting_pid: u64,
        target_pid: u64,
        new_rights: Rights,
    ) -> SysResult<CapabilityHandle> {
        // Validate the capability and extract needed data
        let (cap_resource, cap_rights, cap_owner) = {
            let cap = self.get(handle)?;
            (cap.resource, cap.rights, cap.owner)
        };

        // Check ownership
        if cap_owner != requesting_pid {
            return Err(SysError::NotPermitted);
        }

        // Check DELEGATE right
        if !cap_rights.contains(Rights::DELEGATE) {
            return Err(SysError::CannotDelegate);
        }

        // New rights must be a subset
        if !cap_rights.contains(new_rights) {
            return Err(SysError::InsufficientRights);
        }

        // Allocate new slot for delegated capability
        let slot = self.alloc_slot().ok_or(SysError::CapabilityTableFull)?;

        let new_cap = Capability {
            resource: cap_resource,
            rights: new_rights,
            generation: self.current_generation,
            parent: handle,
            owner: target_pid,
        };

        self.caps[slot] = CapabilityEntry {
            cap: new_cap,
            valid: true,
        };

        Ok(CapabilityHandle(slot as u64))
    }

    /// Revoke a capability and all derived capabilities
    pub fn revoke(&mut self, handle: CapabilityHandle, requesting_pid: u64) -> SysResult<()> {
        let slot = handle.0 as usize;

        if slot >= MAX_TOTAL_CAPS {
            return Err(SysError::InvalidCapability);
        }

        let entry = &self.caps[slot];
        if !entry.valid {
            return Err(SysError::InvalidCapability);
        }

        // Check ownership or REVOKE right
        if entry.cap.owner != requesting_pid && !entry.cap.rights.contains(Rights::REVOKE) {
            return Err(SysError::NotPermitted);
        }

        // Recursively revoke all capabilities derived from this one
        self.revoke_recursive(handle);

        Ok(())
    }

    /// Recursively revoke a capability and its children
    fn revoke_recursive(&mut self, handle: CapabilityHandle) {
        let slot = handle.0 as usize;

        if slot >= MAX_TOTAL_CAPS || !self.caps[slot].valid {
            return;
        }

        // Find and revoke all children
        let children: Vec<usize> = self
            .caps
            .iter()
            .enumerate()
            .filter(|(_, e)| e.valid && e.cap.parent == handle)
            .map(|(i, _)| i)
            .collect();

        for child_slot in children {
            self.revoke_recursive(CapabilityHandle(child_slot as u64));
        }

        // Now revoke this capability
        self.free_slot(slot);
    }

    /// Drop a capability voluntarily
    pub fn drop_cap(&mut self, handle: CapabilityHandle, requesting_pid: u64) -> SysResult<()> {
        let slot = handle.0 as usize;

        if slot >= MAX_TOTAL_CAPS {
            return Err(SysError::InvalidCapability);
        }

        let entry = &self.caps[slot];
        if !entry.valid {
            return Err(SysError::InvalidCapability);
        }

        // Must be owner to drop
        if entry.cap.owner != requesting_pid {
            return Err(SysError::NotPermitted);
        }

        // Just free this slot (don't revoke children - they become orphaned but still valid)
        self.free_slot(slot);

        Ok(())
    }

    /// Get a capability by handle
    pub fn get(&self, handle: CapabilityHandle) -> SysResult<&Capability> {
        let slot = handle.0 as usize;

        if slot >= MAX_TOTAL_CAPS {
            return Err(SysError::InvalidCapability);
        }

        let entry = &self.caps[slot];
        if !entry.valid {
            return Err(SysError::InvalidCapability);
        }

        // Check generation
        if !entry.cap.generation.is_valid(self.current_generation) {
            return Err(SysError::RevokedCapability);
        }

        Ok(&entry.cap)
    }

    /// Inspect a capability
    pub fn inspect(
        &self,
        handle: CapabilityHandle,
        requesting_pid: u64,
    ) -> SysResult<CapabilityInfo> {
        let cap = self.get(handle)?;

        // Can only inspect own capabilities
        if cap.owner != requesting_pid {
            return Err(SysError::NotPermitted);
        }

        Ok(CapabilityInfo {
            resource: cap.resource,
            rights: cap.rights,
            generation: cap.generation,
            has_parent: !cap.parent.is_null(),
        })
    }

    /// Validate that a process has a capability with given rights
    pub fn validate(
        &self,
        handle: CapabilityHandle,
        requesting_pid: u64,
        required_rights: Rights,
    ) -> SysResult<&Capability> {
        let cap = self.get(handle)?;

        // Check ownership
        if cap.owner != requesting_pid {
            return Err(SysError::NotPermitted);
        }

        // Check rights
        if !cap.rights.contains(required_rights) {
            return Err(SysError::InsufficientRights);
        }

        Ok(cap)
    }

    /// Get all capabilities owned by a process
    pub fn get_process_caps(&self, pid: u64) -> Vec<CapabilityHandle> {
        self.caps
            .iter()
            .enumerate()
            .filter(|(_, e)| e.valid && e.cap.owner == pid)
            .map(|(i, _)| CapabilityHandle(i as u64))
            .collect()
    }

    /// Bump generation (mass revocation)
    pub fn bump_generation(&mut self) -> Generation {
        self.current_generation = self.current_generation.next();
        self.current_generation
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        let used = MAX_TOTAL_CAPS - self.free_slots.len() - 1; // -1 for NULL slot
        let free = self.free_slots.len();
        (used, free)
    }
}

/// Initialize the capability system
pub fn init() {
    *CAPABILITY_TABLE.lock() = Some(GlobalCapTable::new());
}

/// Access the global capability table
pub fn with_cap_table<F, R>(f: F) -> R
where
    F: FnOnce(&mut GlobalCapTable) -> R,
{
    let mut guard = CAPABILITY_TABLE.lock();
    let table = guard.as_mut().expect("Capability table not initialized");
    f(table)
}

/// Create a root capability (kernel initialization only)
pub fn create_root_cap(
    resource: ResourceDescriptor,
    rights: Rights,
    owner: u64,
) -> SysResult<CapabilityHandle> {
    with_cap_table(|table| table.create_root_cap(resource, rights, owner))
}

/// Validate a capability
pub fn validate(
    handle: CapabilityHandle,
    pid: u64,
    required_rights: Rights,
) -> SysResult<Capability> {
    with_cap_table(|table| table.validate(handle, pid, required_rights).cloned())
}
