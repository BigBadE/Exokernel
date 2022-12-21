#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::borrow::Borrow;
use core::ptr;
use crate::dap::DAP;

global_asm!(include_str!("bootloader.s"));

mod dap;

extern "C" {
    pub static _second_stage: u8;
}

//See bootloader/bios/README.md for why these values are here.
//const SECOND_STAGE_START: u16 = 0x7E00;
const PARTITION_TABLE: *const u8 = 0x7DB9 as *const u8;

#[no_mangle]
pub extern "C" fn first_stage(disk_number: u16) {
    print(b'1');
    let SECOND_STAGE_START = unsafe { ptr::read(&_second_stage as *const u8 as *const u16) };
    let dap = DAP::new(env!("second_stage_length").parse::<u16>().unwrap(), SECOND_STAGE_START, 1);
    print(b'2');
    unsafe {
        dap.load(disk_number);

        print(b'3');
        let second_stage: fn(disk_number: u16, partition_table: *const u8) = core::mem::transmute(SECOND_STAGE_START as *const u8);
        print(b'4');
        second_stage(disk_number, PARTITION_TABLE);
    }
    print(b'5');
}

#[no_mangle]
pub extern "C" fn fail() -> ! {
    print(b'!');
    loop{}
}

pub fn print(char: u8) {
    unsafe {
        let char = char as u16 | 0xE00;
        asm!("push bx", "mov bx, 0", "int 0x10", "pop bx", in("ax") char);
    }
}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    print(b'?');
    loop {}
}