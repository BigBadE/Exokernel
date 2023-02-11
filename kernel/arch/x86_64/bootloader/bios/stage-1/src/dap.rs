use core::arch::asm;

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
    lba: u64
}

impl DAP {
    pub fn new(size: u16, segment: u32, lba: u64) -> Self {
        return DAP {
            size: 16,
            zero: 0,
            sectors: size as u16,
            offset: 0,
            segment: (segment >> 4) as u16,
            lba
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