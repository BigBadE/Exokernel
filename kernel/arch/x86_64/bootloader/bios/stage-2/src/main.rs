#![no_std]
#![no_main]
#![feature(lang_items)]
use core::arch::asm;
use crate::dap::DAP;

mod dap;

#[no_mangle]
pub extern "C" fn second_stage() {
    print("Testing!");

    let buffer = [0; 512*8];
    let packet = DAP::new(8, buffer as *const u8, 1);

}

pub fn print(message: &'static str) {
    unsafe {
        for char in message.bytes() {
            asm!("mov ah, 0xE", "int 0x10")
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