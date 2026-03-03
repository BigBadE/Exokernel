//! Init Process - Capability Manager
//!
//! The init process is the first user-space process and serves as the
//! root of the capability hierarchy. It is responsible for:
//!
//! 1. **Bootstrap**: Receives ALL hardware capabilities from the kernel
//! 2. **Resource Distribution**: Delegates capabilities to other processes
//! 3. **Process Spawning**: Creates and starts device drivers and applications
//! 4. **Naming Service**: Maps names to IPC endpoints for service discovery
//!
//! ## Exokernel Philosophy
//!
//! In a traditional OS, the kernel manages resources directly.
//! In the exokernel model:
//! - Kernel provides raw hardware access via capabilities
//! - Init receives all capabilities and delegates them to user-space
//! - Drivers run in user-space with delegated hardware access
//! - Applications request resources from drivers via IPC
//!
//! ## Boot Sequence
//!
//! 1. Kernel loads init (the only ELF the kernel loads)
//! 2. Kernel grants init capabilities for: all physical memory, all IRQs,
//!    all I/O ports, and the ability to create processes
//! 3. Init spawns device drivers with specific capabilities:
//!    - Keyboard driver: IRQ1, keyboard I/O ports
//!    - Disk driver: IRQ14/15, ATA I/O ports
//!    - Console driver: framebuffer memory
//! 4. Init spawns applications that communicate with drivers via IPC

#![no_std]

extern crate libexo;

use core::sync::atomic::{AtomicUsize, Ordering};
use libexo::CapabilityHandle;
use libexo::syscall;

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

/// Init main function - called by _start
pub fn init_main() -> ! {
    // Print startup message
    let _ = syscall::debug_print("=== Exokernel Init Starting ===\n");

    // Get our PID (should be 1)
    let pid = syscall::get_pid();
    let _ = syscall::debug_print("Init PID: ");
    print_num(pid);
    let _ = syscall::debug_print("\n");

    // In a full implementation, the kernel would have granted us capabilities
    // for all hardware resources before starting us. For now, we just
    // demonstrate the capability manager structure.

    let _ = syscall::debug_print("Init: Waiting for hardware capabilities from kernel...\n");

    // TODO: Receive initial capabilities from kernel via IPC or special syscall
    // For now, we'll create our own IPC endpoint to act as the naming service

    let _ = syscall::debug_print("Init: Creating naming service endpoint...\n");

    match libexo::ipc::create_endpoint(0) {
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

    // Main loop - handle service requests
    let _ = syscall::debug_print("Init: Entering main loop\n");

    loop {
        // In a real implementation:
        // 1. Wait for IPC messages
        // 2. Handle capability delegation requests
        // 3. Handle service registration
        // 4. Handle service lookup

        // For now, just yield
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
    let _ = syscall::debug_print("PANIC in init: ");
    if let Some(msg) = info.message().as_str() {
        let _ = syscall::debug_print(msg);
    }
    let _ = syscall::debug_print("\n");
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
