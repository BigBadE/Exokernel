use crate::util::print::println;
use crate::FILE_BUFFER_SIZE;
use common::boot_info::VideoInfo;
use core::arch::asm;

#[repr(C, packed)]
pub struct VBEInfo {
    pub vbe_signature: [u8; 4],
    pub vbe_version: u16,
    pub oem_string_ptr: u32,
    pub capabilities: u32,
    pub video_mode_ptr: u32,
    pub total_memory: u16,
    oem: [u8; 512 - 0x14],
}

#[repr(C, align(256))]
#[derive(Clone, Copy)]
pub struct VBEModeInfo {
    // deprecated, only bit 7 should be of interest to you, and it indicates the mode supports a linear frame buffer.
    attributes: u16,
    // deprecated
    window_a: u8,
    // deprecated
    window_b: u8,
    // deprecated; used while calculating bank numbers
    granularity: u16,
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    // deprecated; used to switch banks from protected mode without returning to real mode
    win_func_ptr: u32,
    // number of bytes per horizontal line
    pitch: u16,
    // width in pixels
    width: u16,
    // height in pixels
    height: u16,
    // unused...
    w_char: u8,
    // ...
    y_char: u8,
    planes: u8,
    // bits per pixel in this mode
    bpp: u8,
    // deprecated; total number of banks in this mode
    banks: u8,
    memory_model: u8,
    // deprecated; size of a bank, almost always 64 KB but may be 16 KB...
    bank_size: u8,
    image_pages: u8,
    reserved0: u8,
    red_mask: u8,
    red_position: u8,
    green_mask: u8,
    green_position: u8,
    blue_mask: u8,
    blue_position: u8,
    reserved_mask: u8,
    reserved_position: u8,
    direct_color_attributes: u8,
    // physical address of the linear frame buffer; write here to draw to the screen
    framebuffer: u32,
    off_screen_mem_off: u32,
    // size of memory in the framebuffer but not being displayed on the screen
    off_screen_mem_size: u16,
    reserved1: [u8; 206],
}

impl VBEInfo {
    pub fn get_best_mode(&self) -> Option<VideoInfo> {
        let mut buffer = [0u8; 256];
        let mut best = None;
        let mut i = 0;
        while let Some(id) = self.get_mode_numb(i) {
            i += 1;

            let mode = match self.get_mode(&mut buffer, id) {
                Ok(result) => result,
                Err(_error) => {
                    println("Error!");
                    break;
                }
            };

            if mode.attributes & 0x90 != 0x90 {
                continue;
            }

            best = Some((mode, id));
        }

        match best {
            Some((best, id)) => Some(VideoInfo {
                mode: id,
                width: best.width,
                height: best.height,
                framebuffer: best.framebuffer,
                bytes_per_pixel: (best.bpp / 8) as u16,
                bytes_per_line: best.pitch,
            }),
            None => None,
        }
    }

    fn get_mode_numb(&self, index: usize) -> Option<u16> {
        let (segment, offset) = {
            let raw = self.video_mode_ptr;
            ((raw >> 16) as u16, raw as u16)
        };
        let video_mode_ptr = ((segment as u32) << 4) + offset as u32;
        let mode = unsafe { *((video_mode_ptr as *const u16).add(index)) };
        match mode {
            0xFFFF => None,
            _ => Some(mode),
        }
    }

    fn get_mode(&self, buffer: &mut [u8; 256], mode: u16) -> Result<VBEModeInfo, u16> {
        assert_eq!(size_of::<VBEModeInfo>(), 256);

        let slice = &mut buffer[..size_of::<VBEModeInfo>()];
        slice.fill(0);
        let block_ptr = slice.as_mut_ptr();

        let mut ret: u16;
        let mut target_addr = block_ptr as u32;
        let segment = target_addr >> 4;
        target_addr -= segment << 4;
        unsafe {
            asm!(
            "push es", "mov es, {:x}", "int 0x10", "pop es",
            in(reg) segment as u16,
            inout("ax") 0x4f01u16 => ret,
            in("cx") mode,
            in("di") target_addr as u16
            )
        };
        match ret {
            0x4F => Ok(unsafe { *(buffer.as_ptr() as *const VBEModeInfo) }),
            _ => Err(ret),
        }
    }
}

pub fn enable(info: &VideoInfo) -> Result<(), u16> {
    let code: u16;
    unsafe {
        asm!("push bx", "mov bx, {:x}", "int 0x10", "pop bx", in(reg) info.mode, inout("ax") 0x4F02u16 => code);
    }

    match code {
        0x4F => Ok(()),
        _ => Err(code),
    }
}

pub fn get_vbe_info(buffer: &mut [u8; FILE_BUFFER_SIZE]) -> &VBEInfo {
    assert_eq!(size_of::<VBEInfo>(), 512);
    buffer.fill(0);

    let block_ptr = buffer.as_mut_ptr();
    let ret: u16;
    unsafe {
        asm!("push es", "mov es, {:x}", "int 0x10", "pop es", in(reg)0, inout("ax") 0x4f00u16 => ret, in("di") block_ptr)
    };
    match ret {
        0x4f => unsafe { &*block_ptr.cast() },
        _ => panic!("Couldn't load VBE"),
    }
}
