//! Capability System Types
//!
//! Capabilities are unforgeable tokens that grant specific rights to specific resources.
//! The kernel tracks capabilities in a capability table, and all resource access
//! must go through capability checks.

/// A capability handle - an index into the process's capability table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CapabilityHandle(pub u64);

impl CapabilityHandle {
    pub const NULL: Self = Self(0);

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(&self) -> u64 {
        self.0
    }
}

/// Generation counter for capability revocation
/// When a capability is revoked, the generation is bumped, invalidating all derived caps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Generation(pub u64);

impl Generation {
    pub const INITIAL: Self = Self(1);

    pub fn next(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn is_valid(&self, current: Generation) -> bool {
        self.0 == current.0
    }
}

/// Types of resources that can be protected by capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceType {
    /// Null/invalid capability
    Null = 0,

    /// Physical memory frames
    PhysicalMemory = 1,

    /// Virtual address space region
    VirtualMemory = 2,

    /// I/O port range
    IoPort = 3,

    /// IRQ line
    Irq = 4,

    /// IPC endpoint
    IpcEndpoint = 5,

    /// Process handle
    Process = 6,

    /// DMA buffer
    DmaBuffer = 7,

    /// Device MMIO region
    DeviceMmio = 8,
}

/// Rights that can be granted on a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Rights(pub u32);

impl Rights {
    pub const NONE: Self = Self(0);

    // Basic rights
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXECUTE: Self = Self(1 << 2);

    // Memory-specific rights
    pub const MAP: Self = Self(1 << 3);
    pub const GRANT: Self = Self(1 << 4);     // Can grant to others
    pub const DELEGATE: Self = Self(1 << 5);   // Can delegate with attenuation

    // Process rights
    pub const KILL: Self = Self(1 << 6);
    pub const SUSPEND: Self = Self(1 << 7);
    pub const RESUME: Self = Self(1 << 8);

    // IPC rights
    pub const SEND: Self = Self(1 << 9);
    pub const RECEIVE: Self = Self(1 << 10);
    pub const CALL: Self = Self(1 << 11);      // Synchronous call

    // IRQ rights
    pub const BIND: Self = Self(1 << 12);
    pub const ACK: Self = Self(1 << 13);

    // Special rights
    pub const REVOKE: Self = Self(1 << 14);    // Can revoke this cap and derived caps

    // Combined rights
    pub const READ_WRITE: Self = Self(Self::READ.0 | Self::WRITE.0);
    pub const ALL: Self = Self(0xFFFFFFFF);

    pub fn contains(&self, other: Rights) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn union(&self, other: Rights) -> Rights {
        Rights(self.0 | other.0)
    }

    pub fn intersection(&self, other: Rights) -> Rights {
        Rights(self.0 & other.0)
    }

    pub fn subtract(&self, other: Rights) -> Rights {
        Rights(self.0 & !other.0)
    }

    pub fn bits(&self) -> u64 {
        self.0 as u64
    }

    pub fn from_bits(bits: u64) -> Self {
        Self(bits as u32)
    }
}

impl core::ops::BitOr for Rights {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for Rights {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

/// A resource descriptor - identifies a specific resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ResourceDescriptor {
    /// Type of resource
    pub resource_type: ResourceType,

    /// Resource-specific identifier
    /// For PhysicalMemory: base frame number
    /// For IoPort: base port number
    /// For Irq: IRQ number
    /// For IpcEndpoint: endpoint ID
    /// For Process: process ID
    pub base: u64,

    /// Size/count (resource-specific)
    /// For PhysicalMemory: number of frames
    /// For IoPort: number of ports
    /// For others: typically 1
    pub size: u64,
}

impl ResourceDescriptor {
    pub const fn null() -> Self {
        Self {
            resource_type: ResourceType::Null,
            base: 0,
            size: 0,
        }
    }

    pub const fn physical_memory(base_frame: u64, frame_count: u64) -> Self {
        Self {
            resource_type: ResourceType::PhysicalMemory,
            base: base_frame,
            size: frame_count,
        }
    }

    pub const fn io_port(base_port: u64, port_count: u64) -> Self {
        Self {
            resource_type: ResourceType::IoPort,
            base: base_port,
            size: port_count,
        }
    }

    pub const fn irq(irq_num: u64) -> Self {
        Self {
            resource_type: ResourceType::Irq,
            base: irq_num,
            size: 1,
        }
    }

    pub const fn ipc_endpoint(endpoint_id: u64) -> Self {
        Self {
            resource_type: ResourceType::IpcEndpoint,
            base: endpoint_id,
            size: 1,
        }
    }

    pub const fn process(pid: u64) -> Self {
        Self {
            resource_type: ResourceType::Process,
            base: pid,
            size: 1,
        }
    }

    pub const fn virtual_memory(base_addr: u64, size_bytes: u64) -> Self {
        Self {
            resource_type: ResourceType::VirtualMemory,
            base: base_addr,
            size: size_bytes,
        }
    }

    pub const fn device_mmio(base_addr: u64, size_bytes: u64) -> Self {
        Self {
            resource_type: ResourceType::DeviceMmio,
            base: base_addr,
            size: size_bytes,
        }
    }

    /// Check if this descriptor contains another (for delegation validation)
    pub fn contains(&self, other: &ResourceDescriptor) -> bool {
        if self.resource_type != other.resource_type {
            return false;
        }

        other.base >= self.base && (other.base + other.size) <= (self.base + self.size)
    }
}

/// Full capability structure (kernel-side representation)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Capability {
    /// The resource this capability grants access to
    pub resource: ResourceDescriptor,

    /// Rights granted by this capability
    pub rights: Rights,

    /// Generation counter for revocation
    pub generation: Generation,

    /// Parent capability (for revocation tree)
    /// NULL if this is a root capability
    pub parent: CapabilityHandle,

    /// Owner process ID
    pub owner: u64,
}

impl Capability {
    pub const fn null() -> Self {
        Self {
            resource: ResourceDescriptor::null(),
            rights: Rights::NONE,
            generation: Generation::INITIAL,
            parent: CapabilityHandle::NULL,
            owner: 0,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self.resource.resource_type, ResourceType::Null)
    }

    /// Check if this capability allows the given rights
    pub fn allows(&self, rights: Rights) -> bool {
        self.rights.contains(rights)
    }

    /// Check if this capability can be delegated with the given rights
    pub fn can_delegate(&self, rights: Rights) -> bool {
        self.rights.contains(Rights::DELEGATE) && self.rights.contains(rights)
    }
}

/// Information about a capability (returned by cap_inspect)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityInfo {
    pub resource: ResourceDescriptor,
    pub rights: Rights,
    pub generation: Generation,
    pub has_parent: bool,
}
