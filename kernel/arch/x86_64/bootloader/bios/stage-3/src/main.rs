#![no_std]
#![no_main]
#![feature(panic_info_message)]

use crate::util::print::{print, println};

mod util;

#[no_mangle]
#[link_section = ".third_stage"]
pub extern "C" fn third_stage(args: u16) {
    println("Third stage!");
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    print(info.message().unwrap().as_str().unwrap());
    loop {}
}