//! Syscall numbers and structures for the Exokernel
//!
//! The exokernel has a minimal syscall interface focused on:
//! - Capability management
//! - Memory binding
//! - Process/CPU management
//! - IPC
//! - I/O binding

use crate::caps::{CapabilityHandle, Rights, ResourceDescriptor};

/// Syscall numbers
///
/// Unlike traditional kernels, we don't have open/read/write/etc.
/// Those are abstractions that belong in user-space libOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Syscall {
    // === Capability Management (0x00-0x0F) ===

    /// Grant a new capability (kernel only, or from existing cap with GRANT right)
    /// Args: resource_type, base, size, rights
    /// Returns: CapabilityHandle
    CapGrant = 0x00,

    /// Revoke a capability and all derived capabilities
    /// Args: cap_handle
    /// Returns: 0 on success
    CapRevoke = 0x01,

    /// Delegate a capability to another process (with possible attenuation)
    /// Args: cap_handle, target_pid, new_rights (must be subset of current)
    /// Returns: 0 on success
    CapDelegate = 0x02,

    /// Inspect a capability's properties
    /// Args: cap_handle, info_ptr (output)
    /// Returns: 0 on success
    CapInspect = 0x03,

    /// Drop a capability (give it up voluntarily)
    /// Args: cap_handle
    /// Returns: 0 on success
    CapDrop = 0x04,

    // === Memory Management (0x10-0x1F) ===

    /// Allocate physical memory frames
    /// Args: frame_count, flags
    /// Returns: CapabilityHandle to physical frames
    MemAllocPhys = 0x10,

    /// Map physical frames to virtual address
    /// Args: phys_cap, virt_addr, flags
    /// Returns: 0 on success
    MemMap = 0x11,

    /// Unmap a virtual address range
    /// Args: virt_addr, size
    /// Returns: 0 on success
    MemUnmap = 0x12,

    /// Change protection on a virtual address range
    /// Args: virt_addr, size, new_rights
    /// Returns: 0 on success
    MemProtect = 0x13,

    /// Query physical address for a virtual address
    /// Args: virt_addr
    /// Returns: physical address or error
    MemQuery = 0x14,

    // === Process Management (0x20-0x2F) ===

    /// Create a new process
    /// Args: entry_point, stack_cap, flags
    /// Returns: CapabilityHandle to new process
    ProcessCreate = 0x20,

    /// Destroy a process
    /// Args: process_cap
    /// Returns: 0 on success
    ProcessDestroy = 0x21,

    /// Get current process ID
    /// Args: none
    /// Returns: current PID
    ProcessGetPid = 0x22,

    /// Exit current process
    /// Args: exit_code
    /// Returns: does not return
    ProcessExit = 0x23,

    /// Yield CPU to scheduler
    /// Args: none
    /// Returns: 0
    ProcessYield = 0x24,

    /// Set upcall handler for async events
    /// Args: handler_addr, stack_addr
    /// Returns: 0 on success
    ProcessSetUpcall = 0x25,

    /// Start a created process
    /// Args: process_cap, entry_point, stack_top
    /// Returns: 0 on success
    ProcessStart = 0x26,

    /// Set syscall handler for LibOS forwarding
    /// When an unknown syscall is executed, instead of returning ENOSYS,
    /// the kernel will call this handler with the syscall arguments.
    /// Args: handler_addr, stack_addr
    /// Returns: 0 on success
    /// Handler signature: fn(num: u64, arg1-6: u64) -> i64
    ProcessSetSyscallHandler = 0x27,

    // === IPC (0x30-0x3F) ===

    /// Create an IPC endpoint
    /// Args: flags
    /// Returns: CapabilityHandle to endpoint
    IpcCreateEndpoint = 0x30,

    /// Send a message to an endpoint
    /// Args: endpoint_cap, msg_ptr, msg_len, caps_ptr, caps_count
    /// Returns: 0 on success
    IpcSend = 0x31,

    /// Receive a message from an endpoint
    /// Args: endpoint_cap, buf_ptr, buf_len, caps_buf_ptr, caps_buf_len
    /// Returns: message length on success
    IpcRecv = 0x32,

    /// Synchronous call (send + wait for reply)
    /// Args: endpoint_cap, msg_ptr, msg_len, reply_ptr, reply_len
    /// Returns: reply length on success
    IpcCall = 0x33,

    /// Reply to a call
    /// Args: reply_ptr, reply_len
    /// Returns: 0 on success
    IpcReply = 0x34,

    // === I/O Management (0x40-0x4F) ===

    /// Bind I/O port access
    /// Args: port_cap, base_port, count
    /// Returns: 0 on success
    IoBindPort = 0x40,

    /// Perform port I/O (if capability held)
    /// Args: port, value, is_write, size (1/2/4)
    /// Returns: read value or 0 for write
    IoPort = 0x41,

    /// Bind an IRQ to an upcall
    /// Args: irq_cap, handler_addr
    /// Returns: 0 on success
    IrqBind = 0x42,

    /// Acknowledge an IRQ
    /// Args: irq_cap
    /// Returns: 0 on success
    IrqAck = 0x43,

    /// Allocate DMA buffer
    /// Args: size, flags
    /// Returns: CapabilityHandle to DMA buffer
    DmaAlloc = 0x44,

    /// Get physical address of DMA buffer
    /// Args: dma_cap
    /// Returns: physical address
    DmaGetPhys = 0x45,

    // === Debug (0xF0-0xFF) ===

    /// Debug print (temporary, for early development)
    /// Args: string_ptr, length
    /// Returns: 0
    DebugPrint = 0xF0,

    /// Debug: dump capability table
    DebugDumpCaps = 0xF1,
}

impl Syscall {
    pub fn from_num(num: u64) -> Option<Self> {
        match num {
            0x00 => Some(Syscall::CapGrant),
            0x01 => Some(Syscall::CapRevoke),
            0x02 => Some(Syscall::CapDelegate),
            0x03 => Some(Syscall::CapInspect),
            0x04 => Some(Syscall::CapDrop),

            0x10 => Some(Syscall::MemAllocPhys),
            0x11 => Some(Syscall::MemMap),
            0x12 => Some(Syscall::MemUnmap),
            0x13 => Some(Syscall::MemProtect),
            0x14 => Some(Syscall::MemQuery),

            0x20 => Some(Syscall::ProcessCreate),
            0x21 => Some(Syscall::ProcessDestroy),
            0x22 => Some(Syscall::ProcessGetPid),
            0x23 => Some(Syscall::ProcessExit),
            0x24 => Some(Syscall::ProcessYield),
            0x25 => Some(Syscall::ProcessSetUpcall),
            0x26 => Some(Syscall::ProcessStart),
            0x27 => Some(Syscall::ProcessSetSyscallHandler),

            0x30 => Some(Syscall::IpcCreateEndpoint),
            0x31 => Some(Syscall::IpcSend),
            0x32 => Some(Syscall::IpcRecv),
            0x33 => Some(Syscall::IpcCall),
            0x34 => Some(Syscall::IpcReply),

            0x40 => Some(Syscall::IoBindPort),
            0x41 => Some(Syscall::IoPort),
            0x42 => Some(Syscall::IrqBind),
            0x43 => Some(Syscall::IrqAck),
            0x44 => Some(Syscall::DmaAlloc),
            0x45 => Some(Syscall::DmaGetPhys),

            0xF0 => Some(Syscall::DebugPrint),
            0xF1 => Some(Syscall::DebugDumpCaps),

            _ => None,
        }
    }

    pub fn as_num(&self) -> u64 {
        *self as u64
    }
}

/// Flags for memory allocation
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct MemFlags(pub u64);

impl MemFlags {
    pub const NONE: Self = Self(0);

    /// Memory must be contiguous in physical space
    pub const CONTIGUOUS: Self = Self(1 << 0);

    /// Memory must be below 4GB (for legacy DMA)
    pub const DMA32: Self = Self(1 << 1);

    /// Memory should be zeroed
    pub const ZEROED: Self = Self(1 << 2);

    /// Memory is uncacheable (for MMIO)
    pub const UNCACHEABLE: Self = Self(1 << 3);

    /// Memory is write-combining
    pub const WRITE_COMBINING: Self = Self(1 << 4);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::ops::BitOr for MemFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Flags for process creation
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ProcessFlags(pub u64);

impl ProcessFlags {
    pub const NONE: Self = Self(0);

    /// Process should start suspended
    pub const SUSPENDED: Self = Self(1 << 0);

    /// Process has access to all I/O ports (dangerous!)
    pub const IOPL3: Self = Self(1 << 1);
}

/// IPC message header (used in shared memory)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IpcMessage {
    /// Length of message data
    pub data_len: u32,

    /// Number of capabilities being transferred
    pub cap_count: u32,

    /// Sender PID (filled in by kernel)
    pub sender_pid: u64,

    /// Message tag (application-defined)
    pub tag: u64,
}
