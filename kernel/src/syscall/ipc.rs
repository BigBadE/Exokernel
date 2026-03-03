//! IPC syscall implementations

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use spin::Mutex;

use exo_shared::{
    CapabilityHandle, ResourceDescriptor, Rights, SysError,
    ipc::{EndpointId, MAX_MESSAGE_SIZE, MAX_CAPS_PER_MESSAGE},
};

use crate::caps::{self, with_cap_table};
use crate::process::{current_pid, with_process_table, ProcessState, scheduler};

/// An IPC message in the kernel
struct Message {
    data: Vec<u8>,
    caps: Vec<CapabilityHandle>,
    sender_pid: u64,
    tag: u64,
}

/// An IPC endpoint
struct Endpoint {
    id: EndpointId,
    owner: u64,
    messages: VecDeque<Message>,
    waiting_receivers: VecDeque<u64>,  // PIDs waiting to receive
    waiting_senders: VecDeque<(u64, Message)>,  // PIDs waiting to send
}

impl Endpoint {
    fn new(id: EndpointId, owner: u64) -> Self {
        Self {
            id,
            owner,
            messages: VecDeque::new(),
            waiting_receivers: VecDeque::new(),
            waiting_senders: VecDeque::new(),
        }
    }
}

/// Global endpoint table
static ENDPOINTS: Mutex<Vec<Endpoint>> = Mutex::new(Vec::new());
static NEXT_ENDPOINT_ID: Mutex<u64> = Mutex::new(1);

/// Create an IPC endpoint
/// Args: flags
/// Returns: capability handle to endpoint
pub fn sys_ipc_create_endpoint(_flags: u64) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();

    // Allocate endpoint ID
    let endpoint_id = {
        let mut next_id = NEXT_ENDPOINT_ID.lock();
        let id = *next_id;
        *next_id += 1;
        EndpointId::new(id)
    };

    // Create endpoint
    {
        let mut endpoints = ENDPOINTS.lock();
        endpoints.push(Endpoint::new(endpoint_id, pid));
    }

    // Create capability
    let resource = ResourceDescriptor::ipc_endpoint(endpoint_id.0);
    let rights = Rights::SEND | Rights::RECEIVE | Rights::GRANT | Rights::DELEGATE;

    let handle = caps::create_root_cap(resource, rights, pid)?;
    Ok(handle.as_raw() as i64)
}

/// Send a message to an endpoint
/// Args: endpoint_cap, msg_ptr, msg_len, caps_ptr, caps_count
pub fn sys_ipc_send(
    endpoint_cap: u64,
    msg_ptr: u64,
    msg_len: u64,
    caps_ptr: u64,
    caps_count: u64,
) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();
    let cap_handle = CapabilityHandle::from_raw(endpoint_cap);

    // Validate capability
    let cap = caps::validate(cap_handle, pid, Rights::SEND)?;

    if cap.resource.resource_type != exo_shared::ResourceType::IpcEndpoint {
        return Err(SysError::InvalidArgument);
    }

    let endpoint_id = EndpointId::new(cap.resource.base);

    // Validate message
    if msg_len > MAX_MESSAGE_SIZE as u64 {
        return Err(SysError::MessageTooLarge);
    }

    if caps_count > MAX_CAPS_PER_MESSAGE as u64 {
        return Err(SysError::MessageTooLarge);
    }

    // Copy message data
    let data = if msg_len > 0 {
        if msg_ptr >= 0x0000_8000_0000_0000 {
            return Err(SysError::BadAddress);
        }
        unsafe {
            core::slice::from_raw_parts(msg_ptr as *const u8, msg_len as usize).to_vec()
        }
    } else {
        Vec::new()
    };

    // Copy capability handles
    let caps = if caps_count > 0 {
        if caps_ptr >= 0x0000_8000_0000_0000 {
            return Err(SysError::BadAddress);
        }
        unsafe {
            core::slice::from_raw_parts(
                caps_ptr as *const CapabilityHandle,
                caps_count as usize
            ).to_vec()
        }
    } else {
        Vec::new()
    };

    let message = Message {
        data,
        caps,
        sender_pid: pid,
        tag: 0,
    };

    // Find endpoint and deliver message
    let mut endpoints = ENDPOINTS.lock();
    let endpoint = endpoints
        .iter_mut()
        .find(|e| e.id == endpoint_id)
        .ok_or(SysError::EndpointNotFound)?;

    // Check if there's a waiting receiver
    if let Some(receiver_pid) = endpoint.waiting_receivers.pop_front() {
        // Direct transfer - wake up receiver
        endpoint.messages.push_back(message);
        drop(endpoints);
        scheduler::unblock(crate::process::ProcessId(receiver_pid));
    } else {
        // Queue the message
        endpoint.messages.push_back(message);
    }

    Ok(0)
}

/// Receive a message from an endpoint
/// Args: endpoint_cap, buf_ptr, buf_len, caps_buf_ptr, caps_buf_len
pub fn sys_ipc_recv(
    endpoint_cap: u64,
    buf_ptr: u64,
    buf_len: u64,
    caps_buf_ptr: u64,
    caps_buf_len: u64,
) -> Result<i64, SysError> {
    let pid = current_pid().as_u64();
    let cap_handle = CapabilityHandle::from_raw(endpoint_cap);

    // Validate capability
    let cap = caps::validate(cap_handle, pid, Rights::RECEIVE)?;

    if cap.resource.resource_type != exo_shared::ResourceType::IpcEndpoint {
        return Err(SysError::InvalidArgument);
    }

    let endpoint_id = EndpointId::new(cap.resource.base);

    // Try to receive
    loop {
        let mut endpoints = ENDPOINTS.lock();
        let endpoint = endpoints
            .iter_mut()
            .find(|e| e.id == endpoint_id)
            .ok_or(SysError::EndpointNotFound)?;

        if let Some(message) = endpoint.messages.pop_front() {
            // Got a message
            let copy_len = core::cmp::min(message.data.len(), buf_len as usize);

            if buf_ptr != 0 && copy_len > 0 {
                if buf_ptr >= 0x0000_8000_0000_0000 {
                    return Err(SysError::BadAddress);
                }
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        message.data.as_ptr(),
                        buf_ptr as *mut u8,
                        copy_len,
                    );
                }
            }

            // Copy capability handles
            let caps_copy_len = core::cmp::min(message.caps.len(), caps_buf_len as usize);
            if caps_buf_ptr != 0 && caps_copy_len > 0 {
                if caps_buf_ptr >= 0x0000_8000_0000_0000 {
                    return Err(SysError::BadAddress);
                }
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        message.caps.as_ptr(),
                        caps_buf_ptr as *mut CapabilityHandle,
                        caps_copy_len,
                    );
                }
            }

            return Ok(copy_len as i64);
        } else {
            // No message - block
            endpoint.waiting_receivers.push_back(pid);
            drop(endpoints);

            scheduler::block_current(ProcessState::BlockedRecv);

            // After waking up, try again
        }
    }
}

/// Synchronous call (send + wait for reply)
pub fn sys_ipc_call(
    _endpoint_cap: u64,
    _msg_ptr: u64,
    _msg_len: u64,
    _reply_ptr: u64,
    _reply_len: u64,
) -> Result<i64, SysError> {
    // TODO: Implement synchronous call
    Err(SysError::NotPermitted)
}

/// Reply to a call
pub fn sys_ipc_reply(_reply_ptr: u64, _reply_len: u64) -> Result<i64, SysError> {
    // TODO: Implement reply
    Err(SysError::NotPermitted)
}
