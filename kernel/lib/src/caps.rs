//! Capability management

use exo_shared::{CapabilityHandle, CapabilityInfo, Rights, SysError, SyscallNumber};
use crate::{syscall1, syscall2, syscall3, syscall4};

/// Grant a capability to another process
pub fn grant(
    cap: CapabilityHandle,
    target_pid: u64,
    rights: Rights,
) -> Result<CapabilityHandle, SysError> {
    let ret = unsafe {
        syscall4(
            SyscallNumber::CapGrant as u64,
            cap.as_raw(),
            target_pid,
            rights.bits(),
            0,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(CapabilityHandle::from_raw(ret as u64))
    }
}

/// Delegate a capability (grant with reduced rights)
pub fn delegate(
    cap: CapabilityHandle,
    target_pid: u64,
    rights: Rights,
) -> Result<CapabilityHandle, SysError> {
    let ret = unsafe {
        syscall4(
            SyscallNumber::CapDelegate as u64,
            cap.as_raw(),
            target_pid,
            rights.bits(),
            0,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(CapabilityHandle::from_raw(ret as u64))
    }
}

/// Revoke a capability
pub fn revoke(cap: CapabilityHandle) -> Result<(), SysError> {
    let ret = unsafe { syscall2(SyscallNumber::CapRevoke as u64, cap.as_raw(), 0) };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Drop a capability (release without revocation)
pub fn drop_cap(cap: CapabilityHandle) -> Result<(), SysError> {
    let ret = unsafe { syscall2(SyscallNumber::CapDrop as u64, cap.as_raw(), 0) };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Inspect a capability
pub fn inspect(cap: CapabilityHandle, info_ptr: &mut CapabilityInfo) -> Result<(), SysError> {
    let ret = unsafe {
        syscall3(
            SyscallNumber::CapInspect as u64,
            cap.as_raw(),
            info_ptr as *mut CapabilityInfo as u64,
            0,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}
