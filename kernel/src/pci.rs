//! PCI Bus Enumeration
//!
//! This module provides basic PCI configuration space access and device enumeration.
//! In a true exokernel, this would be in userspace, but we bootstrap by having the
//! kernel discover devices and grant capabilities for their MMIO regions.

use x86_64::instructions::port::{Port, PortWriteOnly, PortReadOnly};
use alloc::vec::Vec;

/// PCI configuration address port (write-only)
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
/// PCI configuration data port (read/write)
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// PCI device information
#[derive(Debug, Clone)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub bars: [u32; 6],
}

impl PciDevice {
    /// Check if this is a virtio device (vendor ID 0x1AF4)
    pub fn is_virtio(&self) -> bool {
        self.vendor_id == 0x1AF4
    }

    /// Get virtio device type from subsystem ID
    /// Returns None if not a virtio device
    pub fn virtio_device_type(&self) -> Option<VirtioDeviceType> {
        if !self.is_virtio() {
            return None;
        }
        // For transitional devices, device_id indicates type:
        // 0x1000 = network, 0x1001 = block, 0x1002 = balloon, etc.
        // For non-transitional, 0x1040+ maps to device type
        match self.device_id {
            0x1001 => Some(VirtioDeviceType::Block),
            0x1000 => Some(VirtioDeviceType::Network),
            0x1002 => Some(VirtioDeviceType::Balloon),
            0x1003 => Some(VirtioDeviceType::Console),
            0x1041 => Some(VirtioDeviceType::Block),  // Modern block
            0x1042 => Some(VirtioDeviceType::Scsi),
            _ => Some(VirtioDeviceType::Unknown(self.device_id)),
        }
    }

    /// Get the BAR0 address and type for the device
    pub fn bar0_address(&self) -> Option<BarInfo> {
        let bar0 = self.bars[0];
        if bar0 == 0 {
            return None;
        }

        // Check if memory or I/O
        if bar0 & 1 == 0 {
            // Memory BAR
            let addr = (bar0 & 0xFFFFFFF0) as u64;

            // Check if 64-bit BAR
            if (bar0 >> 1) & 0x3 == 2 {
                // 64-bit BAR, combine with BAR1
                let high = self.bars[1] as u64;
                Some(BarInfo::Memory64((high << 32) | addr))
            } else {
                Some(BarInfo::Memory32(addr))
            }
        } else {
            // I/O BAR
            Some(BarInfo::IoPort((bar0 & 0xFFFFFFFC) as u16))
        }
    }

    /// Get the BAR1 address (for virtio-pci, this is usually the MMIO region)
    pub fn bar1_address(&self) -> Option<BarInfo> {
        let bar1 = self.bars[1];
        if bar1 == 0 {
            return None;
        }

        // Check if memory or I/O
        if bar1 & 1 == 0 {
            let addr = (bar1 & 0xFFFFFFF0) as u64;
            // For BAR1, check if this is actually a 64-bit BAR (high part of BAR0)
            // by checking BAR0's type
            let bar0 = self.bars[0];
            if bar0 & 1 == 0 && (bar0 >> 1) & 0x3 == 2 {
                // BAR1 is high part of 64-bit BAR0, so check BAR2
                let bar2 = self.bars[2];
                if bar2 == 0 {
                    return None;
                }
                if bar2 & 1 == 0 {
                    let addr = (bar2 & 0xFFFFFFF0) as u64;
                    if (bar2 >> 1) & 0x3 == 2 {
                        let high = self.bars[3] as u64;
                        return Some(BarInfo::Memory64((high << 32) | addr));
                    } else {
                        return Some(BarInfo::Memory32(addr));
                    }
                } else {
                    return Some(BarInfo::IoPort((bar2 & 0xFFFFFFFC) as u16));
                }
            }
            Some(BarInfo::Memory32(addr))
        } else {
            Some(BarInfo::IoPort((bar1 & 0xFFFFFFFC) as u16))
        }
    }
}

/// BAR address information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarInfo {
    /// 32-bit memory-mapped I/O address
    Memory32(u64),
    /// 64-bit memory-mapped I/O address
    Memory64(u64),
    /// I/O port address
    IoPort(u16),
}

impl BarInfo {
    /// Get the MMIO address if this is a memory BAR
    pub fn mmio_address(&self) -> Option<u64> {
        match self {
            BarInfo::Memory32(addr) | BarInfo::Memory64(addr) => Some(*addr),
            BarInfo::IoPort(_) => None,
        }
    }

    /// Get the I/O port if this is a port BAR
    pub fn io_port(&self) -> Option<u16> {
        match self {
            BarInfo::IoPort(port) => Some(*port),
            _ => None,
        }
    }
}

/// Virtio device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioDeviceType {
    Network,
    Block,
    Console,
    Balloon,
    Scsi,
    Unknown(u16),
}

/// Read a 32-bit value from PCI configuration space
fn pci_config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = (1 << 31)  // Enable bit
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        let mut addr_port = PortWriteOnly::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        addr_port.write(address);
        data_port.read()
    }
}

/// Write a 32-bit value to PCI configuration space
#[allow(dead_code)]
fn pci_config_write(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address: u32 = (1 << 31)
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        let mut addr_port = PortWriteOnly::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        addr_port.write(address);
        data_port.write(value);
    }
}

/// Check if a PCI device exists at the given location
fn device_exists(bus: u8, device: u8, function: u8) -> bool {
    let vendor = pci_config_read(bus, device, function, 0) & 0xFFFF;
    vendor != 0xFFFF
}

/// Read device information from PCI configuration space
fn read_device(bus: u8, device: u8, function: u8) -> PciDevice {
    let id = pci_config_read(bus, device, function, 0);
    let class_info = pci_config_read(bus, device, function, 8);

    let mut bars = [0u32; 6];
    for i in 0..6 {
        bars[i] = pci_config_read(bus, device, function, 0x10 + (i as u8 * 4));
    }

    PciDevice {
        bus,
        device,
        function,
        vendor_id: (id & 0xFFFF) as u16,
        device_id: ((id >> 16) & 0xFFFF) as u16,
        class: ((class_info >> 24) & 0xFF) as u8,
        subclass: ((class_info >> 16) & 0xFF) as u8,
        prog_if: ((class_info >> 8) & 0xFF) as u8,
        bars,
    }
}

/// Check if device is multi-function
fn is_multifunction(bus: u8, device: u8) -> bool {
    let header = pci_config_read(bus, device, 0, 0x0C);
    (header >> 16) & 0x80 != 0
}

/// Enumerate all PCI devices
pub fn enumerate() -> Vec<PciDevice> {
    let mut devices = Vec::new();

    for bus in 0..=255u8 {
        for device in 0..32u8 {
            if !device_exists(bus, device, 0) {
                continue;
            }

            devices.push(read_device(bus, device, 0));

            // Check for multi-function device
            if is_multifunction(bus, device) {
                for function in 1..8u8 {
                    if device_exists(bus, device, function) {
                        devices.push(read_device(bus, device, function));
                    }
                }
            }
        }

        // Early exit if bus 0 scan complete and no devices on higher buses
        // (most systems only use bus 0)
        if bus == 0 && devices.is_empty() {
            break;
        }
    }

    devices
}

/// Find all virtio block devices
pub fn find_virtio_block_devices() -> Vec<PciDevice> {
    enumerate()
        .into_iter()
        .filter(|d| matches!(d.virtio_device_type(), Some(VirtioDeviceType::Block)))
        .collect()
}
