use crate::disk::dap::DAP;
use crate::disk::{AlignedArrayBuffer, AlignedBuffer, Read, Seek, SeekFrom};
use crate::util::print::{print, print_hex, print_numb, println};

pub struct DiskReader {
    pub disk_number: u16,
    pub base: u64,
    pub offset: u64,
}

impl Read for DiskReader {
    unsafe fn read_exact(&mut self, len: usize) -> &[u8] {
        /*// Only works with a single buffer's worth
        assert!(len < 513);
        static mut BUFFER: AlignedArrayBuffer<512> = AlignedArrayBuffer {
            buffer: [0; 512],
        };
        let buf = unsafe { &mut BUFFER };
        self.read_exact_into(len, buf);
        &buf.buffer[..len]*/
        let current_sector_offset = usize::try_from(self.offset % 512).unwrap();
        static mut TMP_BUF: AlignedArrayBuffer<1024> = AlignedArrayBuffer {
            buffer: [0; 512 * 2],
        };
        let buf = unsafe { &mut TMP_BUF };
        assert!(current_sector_offset + len <= buf.buffer.len());

        self.read_exact_into(buf.buffer.len(), buf);

        &buf.buffer[current_sector_offset..][..len]
    }

    fn read_exact_into(&mut self, len: usize, buf: &mut dyn AlignedBuffer) {
        /*let mut sectors_left = len / 512;
        let buf = buf.slice_mut();
        let mut buf_target = buf.as_ptr_range().start as u32;
        while sectors_left > 0 {
            let reading_sectors = sectors_left.max(127);
            let dap = DAP::new(reading_sectors as u16, buf_target, self.base / 512 + self.offset / 512);
            dap.load(self.disk_number);
            sectors_left -= reading_sectors;
            buf_target += reading_sectors as u32 * 512;
            self.offset += reading_sectors as u64 * 512;
        }*/
        assert_eq!(len % 512, 0);
        let buf = &mut buf.slice_mut()[..len];

        println("1");
        let end_addr = self.base + self.offset + u64::try_from(buf.len()).unwrap();
        let mut start_lba = (self.base + self.offset) / 512;
        let end_lba = (end_addr - 1) / 512;
        print("3: ");
        print_numb(self.base as u32);
        print(", ");
        print_hex(self.offset as u32);
        loop {

        }
        let mut number_of_sectors = end_lba + 1 - start_lba;
        let mut target_addr = buf.as_ptr_range().start as u32;

        loop {
            let sectors = u64::min(number_of_sectors, 32) as u16;
            let dap = DAP::new(
                sectors,
                target_addr,
                start_lba
            );
            println("3-1");
            dap.load(self.disk_number);
            println("4");

            start_lba += u64::from(sectors);
            number_of_sectors -= u64::from(sectors);
            target_addr += u32::from(sectors) * 512;

            if number_of_sectors == 0 {
                break;
            }
            println("5");
        }

        self.offset = end_addr;
        println("Here!");
        loop {

        }
    }
}

impl Seek for DiskReader {
    fn seek(&mut self, pos: SeekFrom) -> u64 {
        self.offset = match pos {
            SeekFrom::Start(offset) => offset
        };
        self.offset
    }
}
/*
#[derive(Clone)]
pub struct DiskAccess {
    pub disk_number: u16,
    pub base_offset: u64,
    pub current_offset: u64,
}

impl Read for DiskAccess {
    unsafe fn read_exact(&mut self, len: usize) -> &[u8] {
            /*
        let current_sector_offset = usize::try_from(self.current_offset % 512).unwrap();

        static mut TMP_BUF: AlignedArrayBuffer<1024> = AlignedArrayBuffer {
            buffer: [0; 512 * 2],
        };
        let buf = unsafe { &mut TMP_BUF };
        assert!(current_sector_offset + len <= buf.buffer.len());

        self.read_exact_into(buf.buffer.len(), buf);

        &buf.buffer[current_sector_offset..][..len]*/
    }

    fn read_exact_into(&mut self, len: usize, buf: &mut dyn AlignedBuffer) {
        assert_eq!(len % 512, 0);
        let buf = &mut buf.slice_mut()[..len];

        let end_addr = self.base_offset + self.current_offset + u64::try_from(buf.len()).unwrap();
        let mut start_lba = (self.base_offset + self.current_offset) / 512;
        let end_lba = (end_addr - 1) / 512;

        let mut number_of_sectors = end_lba + 1 - start_lba;
        let mut target_addr = buf.as_ptr_range().start as u32;

        loop {
            let sectors = u64::min(number_of_sectors, 32) as u16;
            let dap = DAP::new(
                sectors,
                target_addr,
                start_lba
            );
            dap.load(self.disk_number);

            start_lba += u64::from(sectors);
            number_of_sectors -= u64::from(sectors);
            target_addr += u32::from(sectors) * 512;

            if number_of_sectors == 0 {
                break;
            }
        }

        self.current_offset = end_addr;
    }
}

impl Seek for DiskAccess {
    fn seek(&mut self, pos: SeekFrom) -> u64 {
        match pos {
            SeekFrom::Start(offset) => {
                self.current_offset = offset;
                self.current_offset
            }
        }
    }
}*/