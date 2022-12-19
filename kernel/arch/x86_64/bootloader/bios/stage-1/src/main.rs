#![no_std]
#![no_main]
#![feature(lang_items)]

use core::arch::{asm, global_asm};

global_asm!(include_str!("bootloader.s"));

#[no_mangle]
pub extern "C" fn first_stage(disk_number: u16) {
    print("Testing!");
}

pub fn print(message: &'static str) {
    unsafe {
        for char in message.bytes() {
            let char = char as u16 | 0xE00;
            asm!("push bx", "mov bx, 0", "int 0x10", "pop bx", in("ax") char)
        }
    }
}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    print("Panic!");
    loop {}
}

#[lang = "eh_personality"]
pub fn ignored() {}