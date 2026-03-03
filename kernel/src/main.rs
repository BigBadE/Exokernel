//! Exokernel - A minimal capability-based exokernel
//!
//! This kernel provides only:
//! - Capability-based resource protection
//! - Physical memory allocation and mapping
//! - Process/CPU multiplexing
//! - IPC primitives
//! - I/O port/IRQ binding
//!
//! All higher-level abstractions (filesystems, networking, etc.)
//! are implemented in user-space libOS.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod allocator;
mod caps;
mod elf;
mod gdt;
mod interrupts;
mod memory;
mod pci;
mod process;
mod serial;
mod syscall;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use x86_64::VirtAddr;

use exo_shared::{ResourceDescriptor, Rights};

/// Bootloader configuration
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialize serial output
    serial::init();
    println!("Exokernel starting...");
    println!();

    // Initialize GDT
    gdt::init();
    println!("[OK] GDT initialized");

    // Initialize IDT
    interrupts::init();
    println!("[OK] IDT initialized");

    // Initialize memory management
    let phys_mem_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("Physical memory offset required");

    let (mut mapper, mut frame_allocator) = unsafe {
        memory::init(VirtAddr::new(phys_mem_offset), &boot_info.memory_regions)
    };

    // Initialize kernel heap first
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");
    println!("[OK] Kernel heap initialized");

    // Store memory manager for syscalls
    memory::store(mapper, frame_allocator);
    println!("[OK] Memory management initialized");

    // Initialize capability system
    caps::init();
    println!("[OK] Capability system initialized");

    // Initialize syscalls
    syscall::init();
    println!("[OK] Syscall mechanism initialized");

    // Initialize process subsystem
    process::init();
    println!("[OK] Process subsystem initialized");

    // Display framebuffer
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        println!("[OK] Framebuffer: {}x{}", info.width, info.height);

        for byte in framebuffer.buffer_mut() {
            *byte = 0x10;
        }
    }

    // Enumerate PCI devices
    println!();
    println!("Scanning PCI bus...");
    let pci_devices = pci::enumerate();
    for dev in &pci_devices {
        println!(
            "  PCI {:02x}:{:02x}.{}: {:04x}:{:04x} class={:02x}:{:02x}",
            dev.bus, dev.device, dev.function,
            dev.vendor_id, dev.device_id,
            dev.class, dev.subclass
        );
        if let Some(vtype) = dev.virtio_device_type() {
            println!("    -> Virtio device: {:?}", vtype);
            if let Some(bar0) = dev.bar0_address() {
                println!("    -> BAR0: {:?}", bar0);
            }
            // For modern virtio-pci, BAR1/BAR4 is the MMIO region
            if let Some(bar1) = dev.bar1_address() {
                println!("    -> BAR1: {:?}", bar1);
            }
        }
    }

    println!();
    println!("=== Exokernel initialized ===");
    println!();

    // Create init process (capability manager)
    // For now, create a simple test program
    run_init_process(&pci_devices);
}

/// Embedded init binary (compiled from init/ crate)
static INIT_ELF: &[u8] = include_bytes!(env!("INIT_ELF_PATH"));

/// Run the init process by loading the embedded ELF
fn run_init_process(pci_devices: &[pci::PciDevice]) -> ! {
    use process::context::jump_to_usermode;

    println!("Loading init process...");
    println!("  Init ELF size: {} bytes", INIT_ELF.len());

    // Use the embedded init binary
    let init_elf = INIT_ELF;

    // Create capabilities for init process (PID 1)
    let init_pid = 1u64;

    // Grant init full I/O port access (for drivers)
    let all_io_ports = ResourceDescriptor::io_port(0, 0x10000);
    let io_cap = caps::create_root_cap(
        all_io_ports,
        Rights::READ | Rights::WRITE,
        init_pid,
    ).expect("Failed to create I/O port capability");
    println!("  I/O port capability: {}", io_cap.as_raw());

    // Grant init IRQ capabilities (0-15)
    for irq in 0..16 {
        let irq_resource = ResourceDescriptor::irq(irq);
        let irq_cap = caps::create_root_cap(
            irq_resource,
            Rights::BIND | Rights::ACK,
            init_pid,
        ).expect("Failed to create IRQ capability");
        if irq == 0 {
            println!("  IRQ capabilities: {} (IRQ 0-15)", irq_cap.as_raw());
        }
    }

    // Grant capabilities for each virtio block device
    for dev in pci_devices {
        if let Some(pci::VirtioDeviceType::Block) = dev.virtio_device_type() {
            // Grant access based on BAR type
            if let Some(bar0) = dev.bar0_address() {
                match bar0 {
                    pci::BarInfo::IoPort(port) => {
                        // Virtio legacy uses I/O ports for config space (usually 256 bytes)
                        let io_resource = ResourceDescriptor::io_port(port as u64, 256);
                        let io_cap = caps::create_root_cap(
                            io_resource,
                            Rights::READ | Rights::WRITE,
                            init_pid,
                        ).expect("Failed to create I/O port capability for virtio");
                        println!(
                            "  Virtio block I/O cap: {} @ port {:#x}",
                            io_cap.as_raw(), port
                        );
                    }
                    pci::BarInfo::Memory32(addr) | pci::BarInfo::Memory64(addr) => {
                        // Modern virtio uses MMIO (typically 4KB)
                        let mmio_size = 0x1000u64;
                        let mmio_resource = ResourceDescriptor::device_mmio(addr, mmio_size);
                        let mmio_cap = caps::create_root_cap(
                            mmio_resource,
                            Rights::READ | Rights::WRITE | Rights::MAP,
                            init_pid,
                        ).expect("Failed to create MMIO capability for virtio");
                        println!(
                            "  Virtio block MMIO cap: {} @ {:#x} ({}B)",
                            mmio_cap.as_raw(), addr, mmio_size
                        );
                    }
                }
            }
        }
    }

    // Load the ELF
    let loaded = memory::with_mapper_and_allocator(|mapper, allocator| {
        elf::load_elf(&init_elf, mapper, allocator)
            .expect("Failed to load init ELF")
    });

    println!("  Entry point: {:#x}", loaded.entry_point);
    println!("  Stack top: {:#x}", loaded.stack_top);
    println!();
    println!("Jumping to init process...");
    println!();

    let selectors = gdt::selectors();

    unsafe {
        jump_to_usermode(
            selectors.user_code_selector.0 as u64,
            selectors.user_data_selector.0 as u64,
            loaded.entry_point,
            loaded.stack_top,
        )
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!();
    println!("!!! KERNEL PANIC !!!");
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
