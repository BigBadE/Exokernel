#![no_std]
#![feature(lang_items)]
// This is the bootloader. This project just exists to build the project artifacts
// That are passed along to the main build script.


pub fn main() {

}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
pub fn ignored() {}