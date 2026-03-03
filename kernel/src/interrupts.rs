use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use spin::Lazy;
use crate::{gdt, println};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // CPU Exceptions
    idt.divide_error.set_handler_fn(divide_error_handler);
    idt.debug.set_handler_fn(debug_handler);
    idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.overflow.set_handler_fn(overflow_handler);
    idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.device_not_available.set_handler_fn(device_not_available_handler);

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.segment_not_present.set_handler_fn(segment_not_present_handler);
    idt.stack_segment_fault.set_handler_fn(stack_segment_handler);
    idt.general_protection_fault.set_handler_fn(general_protection_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
    idt.alignment_check.set_handler_fn(alignment_check_handler);
    idt.machine_check.set_handler_fn(machine_check_handler);
    idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
    idt.virtualization.set_handler_fn(virtualization_handler);
    idt.security_exception.set_handler_fn(security_exception_handler);

    // Hardware interrupts (PIC)
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);

    idt
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

use spin::Mutex;

pub static PICS: Mutex<Pics> = Mutex::new(Pics::new(PIC_1_OFFSET, PIC_2_OFFSET));

pub struct Pics {
    pic1_offset: u8,
    pic2_offset: u8,
}

impl Pics {
    const fn new(offset1: u8, offset2: u8) -> Self {
        Pics {
            pic1_offset: offset1,
            pic2_offset: offset2,
        }
    }

    pub unsafe fn init(&mut self) {
        use x86_64::instructions::port::Port;

        unsafe {
            let mut wait_port: Port<u8> = Port::new(0x80);
            let mut wait = || wait_port.write(0);

            let mut pic1_cmd: Port<u8> = Port::new(0x20);
            let mut pic1_data: Port<u8> = Port::new(0x21);
            let mut pic2_cmd: Port<u8> = Port::new(0xA0);
            let mut pic2_data: Port<u8> = Port::new(0xA1);

            // Save masks
            let mask1 = pic1_data.read();
            let mask2 = pic2_data.read();

            // Start initialization sequence (ICW1)
            pic1_cmd.write(0x11);
            wait();
            pic2_cmd.write(0x11);
            wait();

            // Set vector offsets (ICW2)
            pic1_data.write(self.pic1_offset);
            wait();
            pic2_data.write(self.pic2_offset);
            wait();

            // Tell PICs about each other (ICW3)
            pic1_data.write(4); // PIC2 at IRQ2
            wait();
            pic2_data.write(2); // Cascade identity
            wait();

            // Set 8086 mode (ICW4)
            pic1_data.write(0x01);
            wait();
            pic2_data.write(0x01);
            wait();

            // Restore saved masks
            pic1_data.write(mask1);
            pic2_data.write(mask2);
        }
    }

    pub unsafe fn end_of_interrupt(&mut self, irq: u8) {
        use x86_64::instructions::port::Port;

        unsafe {
            if irq >= self.pic2_offset {
                let mut pic2_cmd: Port<u8> = Port::new(0xA0);
                pic2_cmd.write(0x20);
            }

            let mut pic1_cmd: Port<u8> = Port::new(0x20);
            pic1_cmd.write(0x20);
        }
    }
}

pub fn init() {
    IDT.load();

    unsafe {
        PICS.lock().init();
        x86_64::instructions::interrupts::enable();
    }
}

// Exception handlers
extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: NON-MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: INVALID TSS (error: {})\n{:#?}", error_code, stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: SEGMENT NOT PRESENT (error: {})\n{:#?}", error_code, stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn stack_segment_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: STACK SEGMENT FAULT (error: {})\n{:#?}", error_code, stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT (error: {})\n{:#?}", error_code, stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: x87 FLOATING POINT\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: ALIGNMENT CHECK (error: {})\n{:#?}", error_code, stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    println!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn security_exception_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: SECURITY EXCEPTION (error: {})\n{:#?}", error_code, stack_frame);
    loop { x86_64::instructions::hlt(); }
}

// Hardware interrupt handlers

/// Timer interrupt handler - MECHANISM ONLY
///
/// This handler is intentionally thin:
/// 1. Acknowledge the interrupt (required by hardware)
/// 2. Tell the scheduler "time's up" for current process
///
/// The scheduler decides policy (who runs next).
/// The timer just provides the preemption mechanism.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Acknowledge interrupt FIRST (before any potential context switch)
    unsafe {
        PICS.lock().end_of_interrupt(InterruptIndex::Timer.as_u8());
    }

    // Notify scheduler - it decides what to do (policy)
    crate::process::scheduler::timer_tick();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // Read scancode (must read to clear interrupt)
    let mut port: Port<u8> = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // Check if user-space has bound this IRQ
    // Keyboard is IRQ 1 (IRQ 0 is timer)
    const KEYBOARD_IRQ: usize = 1;

    if let Some((pid, handler)) = crate::syscall::io::dispatch_irq(KEYBOARD_IRQ) {
        // User-space handler is registered
        // TODO: Deliver upcall to the process with scancode
        // For now, just log it
        println!("[IRQ1->PID{}] Keyboard scancode: {} (upcall pending)", pid.as_u64(), scancode);
    } else {
        // No user-space handler, just print for debugging
        println!("Keyboard scancode: {}", scancode);
    }

    // Acknowledge interrupt
    unsafe {
        PICS.lock().end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
