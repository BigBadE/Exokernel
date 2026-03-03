//! IPC syscalls

use exo_shared::{CapabilityHandle, SysError, SyscallNumber};
use crate::{syscall1, syscall5};

/// Create an IPC endpoint
pub fn create_endpoint(flags: u64) -> Result<CapabilityHandle, SysError> {
    let ret = unsafe {
        syscall1(SyscallNumber::IpcCreateEndpoint as u64, flags)
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(CapabilityHandle::from_raw(ret as u64))
    }
}

/// Send a message
pub fn send(
    endpoint_cap: CapabilityHandle,
    msg: &[u8],
    caps: &[CapabilityHandle],
) -> Result<(), SysError> {
    let ret = unsafe {
        syscall5(
            SyscallNumber::IpcSend as u64,
            endpoint_cap.as_raw(),
            msg.as_ptr() as u64,
            msg.len() as u64,
            caps.as_ptr() as u64,
            caps.len() as u64,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Receive a message
pub fn recv(
    endpoint_cap: CapabilityHandle,
    buf: &mut [u8],
    caps_buf: &mut [CapabilityHandle],
) -> Result<usize, SysError> {
    let ret = unsafe {
        syscall5(
            SyscallNumber::IpcRecv as u64,
            endpoint_cap.as_raw(),
            buf.as_mut_ptr() as u64,
            buf.len() as u64,
            caps_buf.as_mut_ptr() as u64,
            caps_buf.len() as u64,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(ret as usize)
    }
}
