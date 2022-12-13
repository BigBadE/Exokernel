#![no_std]
#![no_main]
#![feature(lang_items)]
use core::arch::asm;

#[no_mangle]
pub extern "C" fn second_stage() {
    print("Testing!");
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