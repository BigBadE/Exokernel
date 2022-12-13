#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

global_asm!(include_str!("bootloader.s"));

#[no_mangle]
pub extern "C" fn main() {
    unsafe {
        print("Testing!");
    }
}

pub unsafe fn print(message: &'static str) {
    for char in message {
        asm!("mov ah, 0xE", "int 0x10")
    }
}