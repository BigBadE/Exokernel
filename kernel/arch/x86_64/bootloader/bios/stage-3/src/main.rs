#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::ptr;
use common::boot_info::BootInfo;

mod util;

const FOURTH_START: u32 = 0x20_000;

#[no_mangle]
#[link_section = ".third_stage"]
pub extern "C" fn third_stage(args: u32) {
    let boot_info = unsafe { ptr::read(args as *const BootInfo) };

    boot_info.video.write((0, 0), (255, 255, 255));
    loop {
        panic!("Fail!");
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}