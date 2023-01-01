#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::ptr;
use crate::dap::DAP;

global_asm!(include_str!("bootloader.s"));

mod dap;

extern "C" {
    pub static second_stage: u16;
}

//See bootloader/bios/README.md for why these values are here.
const SECOND_STAGE_START: u32 = 0x7E00;
const PARTITION_TABLE: *const u8 = 0x7DBE as *const u8;

#[no_mangle]
pub extern "C" fn first_stage(disk_number: u16) {
    unsafe {
        let partitions = ptr::read(PARTITION_TABLE as *const [PartitionTableEntry; 4]);
        let dap = DAP::new(partitions[1].sectors as u16, SECOND_STAGE_START, 1);
        enable_a20();

        dap.load(disk_number);

        let call_second_stage: fn(disk_number: u16) = core::mem::transmute(SECOND_STAGE_START);
        call_second_stage(disk_number);
    }
}

#[no_mangle]
pub extern "C" fn fail() -> ! {
    print(b'!');
    loop{}
}

pub unsafe fn enable_a20() {
    asm!("in al, 0x92",
    "or al, 2",
    "out 0x92, al");
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

#[repr(C, packed(2))]
pub struct PartitionTableEntry {
    pub bootable: u8,
    pub partition_type: u8,
    pub lba: u32,
    pub sectors: u32,
}