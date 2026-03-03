//! I/O syscalls
//!
//! Provides user-space access to hardware I/O ports and IRQ handling.
//! All operations require appropriate capabilities.

use exo_shared::{CapabilityHandle, SysError, SyscallNumber};
use crate::{syscall1, syscall2, syscall4};

// =============================================================================
// Port I/O
// =============================================================================

/// Read from an I/O port (requires IoPort capability)
pub fn port_read(port: u16, size: u8) -> Result<u32, SysError> {
    let ret = unsafe {
        syscall4(
            SyscallNumber::IoPort as u64,
            port as u64,
            0,  // value (unused for read)
            0,  // is_write = false
            size as u64,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(ret as u32)
    }
}

/// Write to an I/O port (requires IoPort capability)
pub fn port_write(port: u16, value: u32, size: u8) -> Result<(), SysError> {
    let ret = unsafe {
        syscall4(
            SyscallNumber::IoPort as u64,
            port as u64,
            value as u64,
            1,  // is_write = true
            size as u64,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

// =============================================================================
// IRQ Handling
// =============================================================================

/// Bind an IRQ to an upcall handler
///
/// When the IRQ fires, the kernel will deliver an upcall to the process.
/// The process must have set up an upcall handler via `set_upcall`.
///
/// Arguments:
/// - `irq_cap`: Capability for the IRQ (ResourceType::Irq)
/// - `handler_addr`: User-space handler address (passed to upcall)
pub fn irq_bind(irq_cap: CapabilityHandle, handler_addr: u64) -> Result<(), SysError> {
    let ret = unsafe {
        syscall2(
            SyscallNumber::IrqBind as u64,
            irq_cap.as_raw(),
            handler_addr,
        )
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

/// Acknowledge an IRQ after handling it
///
/// Must be called after handling an IRQ to allow subsequent interrupts.
pub fn irq_ack(irq_cap: CapabilityHandle) -> Result<(), SysError> {
    let ret = unsafe {
        syscall1(SyscallNumber::IrqAck as u64, irq_cap.as_raw())
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(())
    }
}

// =============================================================================
// DMA
// =============================================================================

/// Allocate a DMA buffer
///
/// Returns a capability for physically contiguous memory suitable for DMA.
pub fn dma_alloc(size: u64, flags: u64) -> Result<CapabilityHandle, SysError> {
    let ret = unsafe {
        syscall2(SyscallNumber::DmaAlloc as u64, size, flags)
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(CapabilityHandle::from_raw(ret as u64))
    }
}

/// Get the physical address of a DMA buffer
pub fn dma_get_phys(dma_cap: CapabilityHandle) -> Result<u64, SysError> {
    let ret = unsafe {
        syscall1(SyscallNumber::DmaGetPhys as u64, dma_cap.as_raw())
    };
    if ret < 0 {
        Err(SysError::from_code(ret))
    } else {
        Ok(ret as u64)
    }
}
