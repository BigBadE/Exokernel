#![no_std]
#![no_main]

use crate::dap::DAP;
use core::arch::{asm, global_asm};
use core::{mem, ptr};

global_asm!(include_str!("bootloader.s"));

mod dap;

extern "C" {
    pub static _partition_table: u8;
    pub static _second_stage: u8;
}

#[no_mangle]
pub extern "C" fn first_stage(disk_number: u16) {
    unsafe {
        let partitions = ptr::read(&_partition_table as *const u8 as *const [PartitionTableEntry; 4]);
        let dap = DAP::new(
            partitions[0].sectors as u16,
            &_second_stage as *const u8 as u32,
            partitions[0].lba as u64,
        );
        enable_a20();
        dap.load(disk_number);

        let call_second_stage: fn(disk_number: u16, partition_table: *const u8) =
            mem::transmute(&_second_stage as *const u8 as u32);
        call_second_stage(disk_number, &_partition_table as *const u8);

        fail(b'R');
    }
}

#[no_mangle]
pub extern "C" fn fail(code: u8) -> ! {
    print(b'!');
    print(code);
    loop {}
}

pub unsafe fn enable_a20() {
    asm!("in al, 0x92", "or al, 2", "out 0x92, al");
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PartitionTableEntry {
    pub bootable: u32,
    pub partition_type: u32,
    pub lba: u32,
    pub sectors: u32,
}