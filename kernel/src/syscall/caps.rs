//! Capability syscall implementations

use exo_shared::{
    CapabilityHandle, CapabilityInfo, ResourceDescriptor, ResourceType,
    Rights, SysError,
};
use crate::caps::{self, with_cap_table};
use crate::process::current_pid;

/// Grant a capability
/// Args: resource_type, base, size, rights
/// Returns: capability handle
pub fn sys_cap_grant(
    resource_type: u64,
    base: u64,
    size: u64,
    rights: u64,
) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();

    let res_type = match resource_type {
        1 => ResourceType::PhysicalMemory,
        2 => ResourceType::VirtualMemory,
        3 => ResourceType::IoPort,
        4 => ResourceType::Irq,
        5 => ResourceType::IpcEndpoint,
        6 => ResourceType::Process,
        7 => ResourceType::DmaBuffer,
        8 => ResourceType::DeviceMmio,
        _ => return Err(SysError::InvalidArgument),
    };

    let resource = ResourceDescriptor {
        resource_type: res_type,
        base,
        size,
    };

    let rights = Rights(rights as u32);

    // For now, only kernel (pid 0) can create root capabilities
    // User processes need to use delegation
    if pid != 0 {
        return Err(SysError::NotPermitted);
    }

    let handle = caps::create_root_cap(resource, rights, pid)?;
    Ok(handle.as_raw() as i64)
}

/// Revoke a capability
pub fn sys_cap_revoke(handle: u64) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();
    let handle = CapabilityHandle::from_raw(handle);

    with_cap_table(|table| table.revoke(handle, pid))?;
    Ok(0)
}

/// Delegate a capability to another process
/// Args: handle, target_pid, new_rights
pub fn sys_cap_delegate(
    handle: u64,
    target_pid: u64,
    new_rights: u64,
) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();
    let handle = CapabilityHandle::from_raw(handle);
    let rights = Rights(new_rights as u32);

    let new_handle = with_cap_table(|table| {
        table.delegate(handle, pid, target_pid, rights)
    })?;

    Ok(new_handle.as_raw() as i64)
}

/// Inspect a capability
/// Args: handle, info_ptr
pub fn sys_cap_inspect(handle: u64, info_ptr: u64) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();
    let handle = CapabilityHandle::from_raw(handle);

    // Validate pointer
    if info_ptr >= 0x0000_8000_0000_0000 || info_ptr == 0 {
        return Err(SysError::BadAddress);
    }

    let info = with_cap_table(|table| table.inspect(handle, pid))?;

    // Write info to user memory
    unsafe {
        let ptr = info_ptr as *mut CapabilityInfo;
        *ptr = info;
    }

    Ok(0)
}

/// Drop a capability
pub fn sys_cap_drop(handle: u64) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();
    let handle = CapabilityHandle::from_raw(handle);

    with_cap_table(|table| table.drop_cap(handle, pid))?;
    Ok(0)
}
