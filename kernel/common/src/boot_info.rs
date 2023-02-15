use core::ptr;

pub struct BootInfo {
    pub video: VideoInfo,
    pub memory: MemoryInfo
}

pub struct MemoryInfo {
    pub memory: &'static [MemoryRegion]
}

#[derive(Copy, Clone, Default)]
pub struct MemoryRegion {
    pub address: u64,
    pub length: u64,
    pub region_type: u32,
    pub extra_bits: u32
}

impl MemoryRegion {
    pub const fn default() -> MemoryRegion {
        return MemoryRegion {
            address: 0,
            length: 0,
            region_type: 0,
            extra_bits: 0
        }
    }
}

pub struct VideoInfo {
    pub mode: u16,
    pub width: u16,
    pub height: u16,
    pub framebuffer: u32,
    pub bytes_per_pixel: u16,
    pub bytes_per_line: u16
}

impl VideoInfo {
    pub fn write(&self, pixel: (u32, u32), value: (u8, u8, u8)) {
        unsafe {
            let pointer = (self.framebuffer as *mut u8).add((self.bytes_per_line as u32 * pixel.1) as usize);
            ptr::write(pointer as *mut (u8, u8, u8), value);
        }
    }
}