use core::arch::asm;
use core::{ptr, slice};

#[derive(Clone)]
pub struct DiskRead {
    //64KiB buffer used for reading bigger areas.
    buffer: u32,
    head: u64,
    disk: u16,
}

impl DiskRead {
    pub fn new(buffer: u32, head: u64, disk: u16) -> Self {
        return DiskRead {
            buffer,
            head,
            disk,
        };
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        return self.read_len(buffer.len(), buffer);
    }

    pub fn read_len(&mut self, mut size: usize, buffer: &mut [u8]) -> Option<usize> {
        let mut buffer = ptr::addr_of!(buffer) as u32;

        while size > 0 {
            let len = size.min(0xFFFF);
            unsafe {
                DAP::new(len as u16, self.buffer, self.head).load(self.disk);
                ptr::copy_nonoverlapping(self.buffer as *const u8,buffer as *mut u8, len);
            }
            size -= len;
            buffer += len as u32;
        }
        return Some(size);
    }

    pub fn seek(&mut self, seeking: u64) {
        self.head = seeking / 512;
    }
}

#[repr(C)]
pub struct DAP {
    //Packet size
    size: u8,
    //Always 0
    zero: u8,
    //# of sectors
    sectors: u16,
    //Transfer buffer in 16bit segment: 16bit offset format
    offset: u16,
    segment: u16,
    //LBA is 48 bits, so lower has first 32 and upper has last 16
    lba: u64,
}

//noinspection ALL
impl DAP {
    pub fn new(size: u16, buffer: u32, lba: u64) -> Self {
        return DAP {
            size: 16,
            zero: 0,
            sectors: size as u16,
            offset: (buffer & 0xF) as u16,
            segment: (buffer >> 4) as u16,
            lba,
        };
    }

    pub unsafe fn load(&self, disk_number: u16) {
        let address = self as *const DAP as u16;
        asm!(
        "mov {1:x}, si",
        "mov si, {0:x}",
        "int 0x13",
        "jc fail",
        "mov si, {1:x}",
        in(reg) address,
        out(reg) _,
        in("ah") 0x42u8,
        in("dl") disk_number as u8);
    }
}