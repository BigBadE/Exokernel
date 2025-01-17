use core::arch::asm;
use core::mem::size_of;
use common::boot_info::BootInfo;

#[repr(C)]
pub struct GDT {
    pub empty: u64,
    pub code: u64,
    pub data: u64
}

#[repr(C, packed(2))]
pub struct GDTPointer {
    pub length: u16,
    pub base: *const GDT
}

impl GDT {
    pub fn new() -> Self {
        return GDT {
            empty: 0,
            //Structure: Limit Lower (16), Base Lower (24), Access (8), Limit Upper (4), Flags (4), Base Upper (8)
            //From https://wiki.osdev.org/Unreal_Mode
            code: 0xFFFF_000000_9A_0_0_0,
            data: 0xFFFF_000000_92_C_F_0,
        };
    }

    pub unsafe fn load(&self) {
        let pointer = GDTPointer {
            //Size of table
            length: (3 * size_of::<u64>() - 1) as u16,
            base: self as *const GDT,
        };
        asm!(
        //Disable interrupts
        "cli",
        //Load GDT
        "lgdt [{}]", in(reg) &pointer)
    }

    pub unsafe fn enter_unreal() {
        let gdt = GDT::new();
        gdt.load();

        asm!(
        //Save data segment
        "push ds",
        //Enter protected mode
        "mov eax, cr0", "or al, 1", "mov cr0, eax",
        //Load GDT
        "mov bx, 0x10", "mov ds, bx",
        //Exit protected mode
        "and al, 0xFE", "mov cr0, eax",
        //Load data segment
        "pop ds",
        //Enable interrupts
        "sti");
    }

    pub unsafe fn enter_protected_jump(jumping: u32, args: &mut BootInfo) -> ! {
        asm!(
        //Disable interrupts
        "cli",
        //Enter protected mode
        "mov eax, cr0", "or al, 1", "mov cr0, eax",
        // align the stack
        "and esp, 0xffffff00",
        //Push args
        "push {0:e}",
        "push {1:e}",
        in(reg) args as *const BootInfo as u32,
        in(reg) jumping);
        //Explainer: https://stackoverflow.com/questions/49438550/assembly-executing-a-long-jump-with-an-offset-with-different-syntax
        //Jump to protected mode
        asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
        asm!(
        //32-bit mode!
        ".code32",

        //Setup segment registers
        "mov ax, 0x10", "mov ds, ax", "mov es, ax", "mov ss, ax",
        //Call
        "pop {0}",
        //"call {0}",
        out(reg) _);

        loop {
            asm!("hlt");
        }
    }
}