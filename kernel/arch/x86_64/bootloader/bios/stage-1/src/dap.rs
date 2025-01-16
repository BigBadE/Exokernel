use core::arch::asm;

#[repr(C, packed)]
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
    lba: u64
}

impl DAP {
    pub fn new(size: u16, address: u32, lba: u64) -> Self {
        DAP {
            size: 0x10,
            zero: 0,
            sectors: size,
            offset: (address & 0x1111) as u16,
            segment: (address >> 4) as u16,
            lba
        }
    }

    pub unsafe fn load(&self, disk_number: u16) {
        let address = self as *const DAP as u16;
        asm!(
            // LLVM requires the si register, so save it
            "mov {1:x}, si",
            "mov si, {0:x}",
            "int 0x13",
            "jc fail",
            // Re-load si register
            "mov si, {1:x}",
        in(reg) address,
        out(reg) _,
        in("ax") 0x4200u16,
        in("dl") disk_number as u8);
    }
}