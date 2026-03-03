//! Init Process - Capability Manager
//!
//! The init process is the first user-space process and serves as the
//! root of the capability hierarchy. It:
//! 1. Receives hardware capabilities from the kernel
//! 2. Spawns essential system services (fs-server, etc.)
//! 3. Passes capabilities to child processes
//! 4. Acts as a naming service for IPC endpoint discovery

#![no_std]
#![no_main]

extern crate libexo;

use core::sync::atomic::{AtomicUsize, Ordering};
use libexo::{CapabilityHandle, ipc, syscall};

// Embedded fs-server binary (built separately)
// For now, we'll skip embedding and just demonstrate the IPC flow
// static FS_SERVER_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fs-server"));

// =============================================================================
// Service Registry (Naming Service)
// =============================================================================

/// Maximum number of registered services
const MAX_SERVICES: usize = 64;

/// Service registry using raw pointers to avoid static mut issues
struct ServiceRegistry {
    endpoints: [CapabilityHandle; MAX_SERVICES],
    count: AtomicUsize,
}

impl ServiceRegistry {
    const fn new() -> Self {
        Self {
            endpoints: [CapabilityHandle::NULL; MAX_SERVICES],
            count: AtomicUsize::new(0),
        }
    }

    fn register(&self, endpoint: CapabilityHandle) -> bool {
        let idx = self.count.fetch_add(1, Ordering::SeqCst);
        if idx >= MAX_SERVICES {
            self.count.fetch_sub(1, Ordering::SeqCst);
            return false;
        }
        // Use raw pointer to avoid mutable reference
        unsafe {
            let ptr = self.endpoints.as_ptr() as *mut CapabilityHandle;
            ptr.add(idx).write(endpoint);
        }
        true
    }
}

// Global service registry
static SERVICE_REGISTRY: ServiceRegistry = ServiceRegistry::new();

// =============================================================================
// Init Entry Point
// =============================================================================

// =============================================================================
// Virtio Device Discovery
// =============================================================================

/// Well-known virtio-blk I/O port base from QEMU
/// The kernel prints "Virtio block I/O cap: X @ port 0xc000" during boot
/// In a full implementation, we'd parse the capability table to find this
const VIRTIO_BLK_IO_PORT: u16 = 0xc000;

/// Filesystem server endpoint ID (must match fs-server)
const FS_SERVER_ENDPOINT_ID: u64 = 0x4653; // "FS"

// =============================================================================
// Init Main
// =============================================================================

/// Init main function
fn init_main() -> ! {
    // Print startup message
    let _ = syscall::debug_print("=== Exokernel Init Starting ===\n");

    // Get our PID (should be 1)
    let pid = syscall::get_pid();
    let _ = syscall::debug_print("Init PID: ");
    print_num(pid);
    let _ = syscall::debug_print("\n");

    // In a full implementation, the kernel would have granted us capabilities
    // for all hardware resources before starting us.

    let _ = syscall::debug_print("Init: Creating naming service endpoint...\n");

    match ipc::create_endpoint(0) {
        Ok(endpoint) => {
            let _ = syscall::debug_print("Init: Naming service endpoint created\n");
            SERVICE_REGISTRY.register(endpoint);
        }
        Err(_) => {
            let _ = syscall::debug_print("Init: Failed to create naming service endpoint\n");
        }
    }

    // Dump capability table for debugging
    let _ = syscall::debug_dump_caps();

    // Print virtio-blk I/O port for fs-server
    let _ = syscall::debug_print("Init: Virtio-blk I/O port at ");
    print_hex(VIRTIO_BLK_IO_PORT as u64);
    let _ = syscall::debug_print("\n");

    // In a full implementation, we would:
    // 1. Load fs-server ELF from embedded binary
    // 2. Spawn it as a child process
    // 3. Send the virtio I/O port via IPC
    //
    // For now, we demonstrate by creating an endpoint and waiting
    // When fs-server is loaded separately, it will receive this info

    let _ = syscall::debug_print("Init: Ready to communicate with fs-server\n");
    let _ = syscall::debug_print("Init: Filesystem server should receive I/O port ");
    print_hex(VIRTIO_BLK_IO_PORT as u64);
    let _ = syscall::debug_print(" to access disk\n");

    // Main loop - handle service requests
    let _ = syscall::debug_print("Init: Entering main loop\n");

    let mut counter = 0u64;
    loop {
        // Periodically print to show we're alive
        if counter % 10000000 == 0 {
            let _ = syscall::debug_print(".");
        }
        counter = counter.wrapping_add(1);

        // Yield to other processes
        syscall::yield_now();
    }
}

// =============================================================================
// Entry Point
// =============================================================================

/// Entry point for init process
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init_main()
}

// =============================================================================
// Panic Handler
// =============================================================================

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = syscall::debug_print("!!! PANIC in init !!!\n");
    if let Some(location) = info.location() {
        let _ = syscall::debug_print("  at ");
        let _ = syscall::debug_print(location.file());
        let _ = syscall::debug_print(":");
        print_num(location.line() as u64);
        let _ = syscall::debug_print("\n");
    }
    syscall::exit(-1)
}

// =============================================================================
// Utilities
// =============================================================================

/// Print a number (simple implementation)
fn print_num(n: u64) {
    if n == 0 {
        let _ = syscall::debug_print("0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut n = n;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    // Reverse
    let mut j = 0;
    while j < i / 2 {
        buf.swap(j, i - 1 - j);
        j += 1;
    }

    // Print
    if let Ok(s) = core::str::from_utf8(&buf[..i]) {
        let _ = syscall::debug_print(s);
    }
}

/// Print a hex number
fn print_hex(n: u64) {
    let _ = syscall::debug_print("0x");
    if n == 0 {
        let _ = syscall::debug_print("0");
        return;
    }

    let mut buf = [0u8; 16];
    let mut i = 0;
    let mut n = n;

    while n > 0 {
        let digit = (n & 0xF) as u8;
        buf[i] = if digit < 10 { b'0' + digit } else { b'a' + digit - 10 };
        n >>= 4;
        i += 1;
    }

    // Reverse
    let mut j = 0;
    while j < i / 2 {
        buf.swap(j, i - 1 - j);
        j += 1;
    }

    if let Ok(s) = core::str::from_utf8(&buf[..i]) {
        let _ = syscall::debug_print(s);
    }
}
