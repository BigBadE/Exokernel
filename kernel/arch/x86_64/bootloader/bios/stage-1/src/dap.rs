#[repr(C)]
pub struct DAP {
    //Packet size
    size: u8,
    //Always 0
    zero: u8,
    //# of sectors
    sectors: u16,
    //Transfer buffer in 16bit segment: 16bit offset format
    buffer: u32,
    //LBA is 48 bits, so lower has first 32 and upper has last 16
    lower_lba: u32,
    upper_lba: u32
}

//noinspection ALL
impl DAP {
    pub fn new(size: u16, buffer: u16, lba: u64) -> Self {
        return DAP {
            size: 16,
            zero: 0,
            sectors: size as u16,
            buffer: buffer as u32,
            lower_lba: (lba & 0xFFFF) as u32,
            upper_lba: ((lba & 0xFF0000) >> 32) as u32
        }
    }

    pub unsafe fn load(&self, disk_number: u16) {

    }
}