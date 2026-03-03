//! IPC types and constants

use crate::caps::CapabilityHandle;

/// Maximum message size for IPC
pub const MAX_MESSAGE_SIZE: usize = 4096;

/// Maximum number of capabilities that can be transferred in one message
pub const MAX_CAPS_PER_MESSAGE: usize = 8;

/// IPC endpoint identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EndpointId(pub u64);

impl EndpointId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

/// IPC message buffer for sending/receiving
#[derive(Clone)]
#[repr(C)]
pub struct MessageBuffer {
    /// Message data
    pub data: [u8; MAX_MESSAGE_SIZE],

    /// Actual length of data
    pub data_len: usize,

    /// Capabilities being transferred
    pub caps: [CapabilityHandle; MAX_CAPS_PER_MESSAGE],

    /// Number of capabilities
    pub cap_count: usize,

    /// Message tag (application-defined)
    pub tag: u64,

    /// Sender PID (filled in by kernel on receive)
    pub sender_pid: u64,
}

impl MessageBuffer {
    pub const fn new() -> Self {
        Self {
            data: [0; MAX_MESSAGE_SIZE],
            data_len: 0,
            caps: [CapabilityHandle::NULL; MAX_CAPS_PER_MESSAGE],
            cap_count: 0,
            tag: 0,
            sender_pid: 0,
        }
    }

    pub fn set_data(&mut self, data: &[u8]) -> bool {
        if data.len() > MAX_MESSAGE_SIZE {
            return false;
        }
        self.data[..data.len()].copy_from_slice(data);
        self.data_len = data.len();
        true
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data[..self.data_len]
    }

    pub fn add_cap(&mut self, cap: CapabilityHandle) -> bool {
        if self.cap_count >= MAX_CAPS_PER_MESSAGE {
            return false;
        }
        self.caps[self.cap_count] = cap;
        self.cap_count += 1;
        true
    }
}

impl Default for MessageBuffer {
    fn default() -> Self {
        Self::new()
    }
}
