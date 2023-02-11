use core::arch::asm;
use core::{ptr, slice};
use crate::{print, println};
use crate::util::print::{printhex, printnumb};

#[derive(Clone)]
pub struct DiskRead {
    base: u64,
    head: u64,
    disk: u16
}

impl DiskRead {
    pub fn new(base: u64, disk: u16) -> Self {
        return DiskRead {
            base,
            head: base,
            disk,
        };
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        let mut temp_buffer = [0u8; 512];
        let value = self.read_len(buffer.len(), &mut temp_buffer);
        unsafe {
            ptr::copy_nonoverlapping(temp_buffer.as_ptr_range().start, buffer.as_mut_ptr_range().start, buffer.len());
        }
        return value;
    }

    pub fn read_len(&mut self, mut size: usize, buffer: &mut [u8]) -> Option<usize> {
        assert_eq!(buffer.len() % 512, 0);
        let mut buffer = buffer.as_ptr_range().start as u32;
        while size > 0 {
            //Get the offset (from (0, 512)) on the LBA
            let offset = (self.head & 0x1FF) as u32;
            //Subtract the offset from the length
            let len = size.min((0xFFFF - offset) as usize);
            //Get the LBA from the head and read it
            DAP::new(((len - 1) / 512 + 1) as u16, buffer, self.head >> 9).load(self.disk);
            self.head += len as u64;
            size -= len;
            buffer += len as u32;
        }
        return Some(size);
    }

    pub fn seek(&mut self, seeking: u64) {
        self.head = self.base + seeking;
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

    pub fn load(&self, disk_number: u16) {
        let address = self as *const DAP as u16;
        unsafe {
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
}