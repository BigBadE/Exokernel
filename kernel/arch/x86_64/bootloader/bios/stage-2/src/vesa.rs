use core::arch::asm;
use core::ptr;
use common::boot_info::VideoInfo;
use crate::FILE_BUFFER_SIZE;
use crate::util::print::{print, printhex, println, printnumb};

#[repr(C)]
#[derive(Clone)]
pub struct VBEInfo {
    pub vbe_signature: [u8; 4],
    pub vbe_version: u16,
    pub oem_string_ptr: u32,
    pub capabilities: u32,
    pub video_mode_segment: u16,
    pub video_mode_offset: u16,
    pub total_memory: u16,
}

#[repr(C, align(256))]
#[derive(Clone)]
pub struct VBEModeInfo {
    attributes: u16,
    // deprecated, only bit 7 should be of interest to you, and it indicates the mode supports a linear frame buffer.
    window_a: u8,
    // deprecated
    window_b: u8,
    // deprecated
    granularity: u16,
    // deprecated; used while calculating bank numbers
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32,
    // deprecated; used to switch banks from protected mode without returning to real mode
    pitch: u16,
    // number of bytes per horizontal line
    width: u16,
    // width in pixels
    height: u16,
    // height in pixels
    w_char: u8,
    // unused...
    y_char: u8,
    // ...
    planes: u8,
    bpp: u8,
    // bits per pixel in this mode
    banks: u8,
    // deprecated; total number of banks in this mode
    memory_model: u8,
    bank_size: u8,
    // deprecated; size of a bank, almost always 64 KB but may be 16 KB...
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
    framebuffer: u32,
    // physical address of the linear frame buffer; write here to draw to the screen
    off_screen_mem_off: u32,
    off_screen_mem_size: u16,
    // size of memory in the framebuffer but not being displayed on the screen
    reserved1: [u8; 206],
}

impl VBEInfo {
    pub fn get_best_mode(&self, buffer: &mut [u8; FILE_BUFFER_SIZE]) -> Option<VideoInfo> {
        let mut best = None;
        let mut i = 0;
        while let Some(mode) = self.get_mode_numb(i) {
            i += 1;

            let mode = match self.get_mode(buffer, mode) {
                Ok(result) => result,
                Err(error) => {
                    print("Error getting VBE mode: ");
                    printhex(error as u32);
                    println("");
                    continue;
                }
            };

            best = Some(mode);
        }

        return best;
    }

    fn get_mode_numb(&self, index: u32) -> Option<u16> {
        let ptr = self.video_mode_segment as u32 >> 4 + self.video_mode_offset;
        let mode = unsafe { *((ptr + index) as *const u16) };
        return match mode {
            0xFFFF => None,
            _ => Some(mode)
        };
    }

    fn get_mode(&self, buffer: &mut [u8; FILE_BUFFER_SIZE], mode: u16) -> Result<VideoInfo, u16> {
        let mut buffer = &mut buffer[0..256];
        buffer.fill(0);

        let address = buffer.as_ptr() as u32;
        let offset = address - address >> 4;

        let code: u16;
        unsafe {
            asm!("push es", "mov es, {0}", "int 0x10", "pop es",
            in(reg) (address >> 4) as u16, inout("ax") 0x4F01u16 => code, in("cx") mode, in("di") offset);
        }

        return match code {
            0x4F => Ok(VideoInfo {}),
            _ => Err(code)
        };
    }
}

pub fn get_vbe_info(buffer: &mut [u8; FILE_BUFFER_SIZE]) -> VBEInfo {
    let buffer = buffer as *const [u8; FILE_BUFFER_SIZE] as u32;
    let output: u32;
    unsafe {
        asm!("push es", "int 0x10", "pop es", inout("ax") 0x4F00u32 => output, in("di") buffer);
        if output != 0x4f {
            panic!("Failed to fetch VBE");
        }

        return ptr::read(buffer as *const VBEInfo).clone();
    }
}