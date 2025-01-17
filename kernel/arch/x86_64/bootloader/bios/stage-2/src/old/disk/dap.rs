use core::arch::asm;

#[derive(Clone)]
pub struct DiskRead {
    base: u64,
    head: u64,
    disk: u16
}

impl DiskRead {
    pub fn new(base: u64, disk: u16) -> Self {
        DiskRead {
            base,
            head: base,
            disk,
        }
    }

    pub fn read(&mut self, len: usize) -> &[u8] {
        let current_sector_offset = usize::try_from(self.head % 512).unwrap();

        static mut TMP_BUF: [u8; 512*2] = [0; 512 * 2];
        let buf = unsafe { &mut TMP_BUF };
        assert!(current_sector_offset + len <= buf.len());

        self.read_len(buf.len(), buf);

        &buf[current_sector_offset..][..len]
    }

    pub fn read_len(&mut self, size: usize, buffer: &mut [u8]) {
        assert_eq!(size % 512, 0);
        let buf = &mut buffer[..size];

        let end_addr = self.base + self.head + buf.len() as u64;
        let mut start_lba = (self.base + self.head) / 512;
        let end_lba = (end_addr - 1) / 512;

        let mut number_of_sectors = end_lba + 1 - start_lba;
        let mut target_addr = buf.as_ptr_range().start as u32;

        loop {
            let sectors = u64::min(number_of_sectors, 32) as u16;
            DAP::new(sectors, target_addr, start_lba).load(self.disk);

            start_lba += sectors as u64;
            number_of_sectors -= sectors as u64;
            target_addr += sectors as u32 * 512;

            if number_of_sectors == 0 {
                break;
            }
        }

        self.head = end_addr;
    }

    pub fn seek(&mut self, seeking: u64) {
        self.head = seeking;
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
            size: 0x10,
            zero: 0,
            sectors: size as u16,
            offset: (buffer & 0b1111) as u16,
            segment: (buffer >> 4) as u16,
            lba,
        };
    }

    pub fn load(&self, disk_number: u16) {
        let address = self as *const Self as u16;
        unsafe {
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

    pub fn load_old(&self, disk_number: u16) {
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

