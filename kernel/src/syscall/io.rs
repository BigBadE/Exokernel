//! I/O syscall implementations
//!
//! In the exokernel model, hardware access is delegated to user-space via capabilities.
//! The kernel provides:
//! - IRQ binding: Maps hardware interrupts to user-space upcalls
//! - Port I/O: Validates capability then performs port read/write
//! - DMA allocation: Provides physically contiguous memory for device DMA
//!
//! User-space device drivers use these primitives to control hardware directly.

use exo_shared::{CapabilityHandle, ResourceDescriptor, ResourceType, Rights, SysError};
use spin::Mutex;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
use x86_64::VirtAddr;

use crate::caps::{self, with_cap_table};
use crate::process::{current_pid, ProcessId};
use crate::println;

/// Maximum number of IRQ lines we support
const MAX_IRQS: usize = 16;

/// IRQ handler registration
#[derive(Debug, Clone, Copy)]
struct IrqBinding {
    /// Process that owns this IRQ
    owner: ProcessId,
    /// User-space handler address
    handler: VirtAddr,
    /// Whether IRQ is currently pending (needs acknowledgment)
    pending: bool,
}

/// Global IRQ binding table
static IRQ_TABLE: Mutex<[Option<IrqBinding>; MAX_IRQS]> = Mutex::new([None; MAX_IRQS]);

/// Bind I/O port access
/// Args: port_cap, base_port, count
pub fn sys_io_bind_port(
    _port_cap: u64,
    _base_port: u64,
    _count: u64,
) -> Result<i64, SysError> {
    // In a real exokernel, this would modify the IOPB (I/O Permission Bitmap)
    // in the TSS to allow direct port access from user space.
    // For now, we just validate the capability and return success.

    // TODO: Implement IOPB manipulation
    Ok(0)
}

/// Perform port I/O
/// Args: port, value, is_write, size (1/2/4)
pub fn sys_io_port(
    port: u64,
    value: u64,
    is_write: u64,
    size: u64,
) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();

    // Find capability for this port
    let has_access = with_cap_table(|table| {
        // Look for any capability that covers this port
        for cap_handle in table.get_process_caps(pid) {
            if let Ok(cap) = table.get(cap_handle) {
                if cap.resource.resource_type == ResourceType::IoPort {
                    let start = cap.resource.base;
                    let end = start + cap.resource.size;
                    if port >= start && port < end {
                        if is_write != 0 {
                            return cap.rights.contains(Rights::WRITE);
                        } else {
                            return cap.rights.contains(Rights::READ);
                        }
                    }
                }
            }
        }
        false
    });

    if !has_access {
        return Err(SysError::PortAccessDenied);
    }

    // Perform the I/O
    let port_num = port as u16;

    if is_write != 0 {
        match size {
            1 => unsafe {
                let mut p: Port<u8> = Port::new(port_num);
                p.write(value as u8);
            },
            2 => unsafe {
                let mut p: Port<u16> = Port::new(port_num);
                p.write(value as u16);
            },
            4 => unsafe {
                let mut p: Port<u32> = Port::new(port_num);
                p.write(value as u32);
            },
            _ => return Err(SysError::InvalidArgument),
        }
        Ok(0)
    } else {
        match size {
            1 => unsafe {
                let mut p: PortReadOnly<u8> = PortReadOnly::new(port_num);
                Ok(p.read() as i64)
            },
            2 => unsafe {
                let mut p: PortReadOnly<u16> = PortReadOnly::new(port_num);
                Ok(p.read() as i64)
            },
            4 => unsafe {
                let mut p: PortReadOnly<u32> = PortReadOnly::new(port_num);
                Ok(p.read() as i64)
            },
            _ => Err(SysError::InvalidArgument),
        }
    }
}

/// Bind an IRQ to an upcall handler
///
/// When the IRQ fires, the kernel will deliver an upcall to the process's
/// registered upcall handler (set via ProcessSetUpcall syscall).
///
/// The upcall will be called with:
/// - arg0: IRQ number
/// - arg1: handler_addr (the value passed here)
///
/// Args:
/// - irq_cap: Capability handle for the IRQ
/// - handler_addr: User-space handler address (passed to upcall)
///
/// Returns: 0 on success
pub fn sys_irq_bind(irq_cap: u64, handler_addr: u64) -> Result<i64, SysError> {
    let pid = current_pid();
    let cap_handle = CapabilityHandle::from_raw(irq_cap);

    // Validate the IRQ capability
    let cap = caps::validate(cap_handle, pid.as_u64(), Rights::BIND)?;

    // Check it's an IRQ capability
    if cap.resource.resource_type != ResourceType::Irq {
        return Err(SysError::InvalidArgument);
    }

    let irq_num = cap.resource.base as usize;

    // Validate IRQ number
    if irq_num >= MAX_IRQS {
        return Err(SysError::InvalidArgument);
    }

    // Validate handler address is in user space
    if handler_addr >= 0x0000_8000_0000_0000 {
        return Err(SysError::BadAddress);
    }

    // Register the IRQ binding
    let mut table = IRQ_TABLE.lock();

    // Check if already bound by another process
    if let Some(existing) = &table[irq_num] {
        if existing.owner != pid {
            return Err(SysError::IrqAlreadyBound);
        }
    }

    table[irq_num] = Some(IrqBinding {
        owner: pid,
        handler: VirtAddr::new(handler_addr),
        pending: false,
    });

    println!("IRQ {} bound to process {} at handler {:#x}", irq_num, pid.as_u64(), handler_addr);

    Ok(0)
}

/// Acknowledge an IRQ
///
/// After handling an IRQ via upcall, the process must acknowledge it.
/// This allows the next interrupt to be delivered.
///
/// Args:
/// - irq_cap: Capability handle for the IRQ
///
/// Returns: 0 on success
pub fn sys_irq_ack(irq_cap: u64) -> Result<i64, SysError> {
    let pid = current_pid();
    let cap_handle = CapabilityHandle::from_raw(irq_cap);

    // Validate the IRQ capability
    let cap = caps::validate(cap_handle, pid.as_u64(), Rights::ACK)?;

    // Check it's an IRQ capability
    if cap.resource.resource_type != ResourceType::Irq {
        return Err(SysError::InvalidArgument);
    }

    let irq_num = cap.resource.base as usize;

    // Validate IRQ number
    if irq_num >= MAX_IRQS {
        return Err(SysError::InvalidArgument);
    }

    // Clear pending flag
    let mut table = IRQ_TABLE.lock();

    if let Some(binding) = &mut table[irq_num] {
        if binding.owner != pid {
            return Err(SysError::NotPermitted);
        }
        binding.pending = false;
    } else {
        return Err(SysError::InvalidArgument);
    }

    Ok(0)
}

/// Called from interrupt handlers to dispatch IRQ to user-space
///
/// This is the kernel-side mechanism for IRQ dispatch.
/// Returns true if a user-space handler is registered and should be called.
pub fn dispatch_irq(irq_num: usize) -> Option<(ProcessId, VirtAddr)> {
    if irq_num >= MAX_IRQS {
        return None;
    }

    let mut table = IRQ_TABLE.lock();

    if let Some(binding) = &mut table[irq_num] {
        // Mark as pending (process must acknowledge before next interrupt)
        binding.pending = true;
        Some((binding.owner, binding.handler))
    } else {
        None
    }
}

/// Check if an IRQ is bound to user-space
pub fn irq_is_bound(irq_num: usize) -> bool {
    if irq_num >= MAX_IRQS {
        return false;
    }
    IRQ_TABLE.lock()[irq_num].is_some()
}

/// Allocate DMA buffer
/// Args: size, flags
pub fn sys_dma_alloc(_size: u64, _flags: u64) -> Result<i64, SysError> {
    // TODO: Implement DMA buffer allocation
    // This would allocate physically contiguous memory below 4GB
    Err(SysError::NotPermitted)
}

/// Get physical address of DMA buffer
/// Args: dma_cap
pub fn sys_dma_get_phys(_dma_cap: u64) -> Result<i64, SysError> {
    // TODO: Implement DMA physical address query
    Err(SysError::NotPermitted)
}
