#![no_std]
#![no_main]

use core::arch::global_asm;

global_asm!(include_str!("bootloader.s"));

#[no_mangle]
pub extern "C" fn main() {

}