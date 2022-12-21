#![no_std]
#![no_main]
#![feature(panic_info_message)]
use core::arch::{asm, global_asm};
use core::ptr;
use crate::dap::DAP;

pub mod dap;
pub mod partitions;

//global_asm!(include_str!("setup.s"));

#[no_mangle]
pub extern "C" fn second_stage(disk_number: u16, partition_table: *const u8) {
    print("Entered stage 2");
}

pub fn print(message: &'static str) {
    unsafe {
        for char in message.bytes() {
            let char = char as u16 | 0xE00;
            asm!("push bx", "mov bx, 0", "int 0x10", "pop bx", in("ax") char);
        }

        let char = '\n' as u16 | 0xE00;
        asm!("push bx", "mov bx, 0", "int 0x10", "pop bx", in("ax") char);
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    print(info.message().unwrap().as_str().unwrap());
    loop {}
}
