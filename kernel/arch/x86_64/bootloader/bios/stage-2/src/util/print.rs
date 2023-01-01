use core::arch::asm;

pub fn print(message: &'static str) {
    for char in message.bytes() {
        print_char(char);
    }
}

pub fn printnumb(mut number: u32) {
    let mut stack: [u8; 10] = [0; 10];
    let mut i = 0;
    while number > 0 {
        stack[i] = b'0' + (number % 10) as u8;
        i += 1;
        number /= 10;
    }

    loop {
        print_char(stack[i]);
        if i == 0 {
            break;
        } else {
            i -= 1;
        }
    }
}

pub fn printhex(mut number: u32) {
    let mut stack: [u8; 10] = [0; 10];
    let mut i = 0;
    while number > 0 {
        let adding = (number % 16) as u8;
        if adding > 9 {
            stack[i] = b'A' + adding - 10;
        } else {
            stack[i] = b'0' + adding;
        }
        i += 1;
        number /= 16;
    }

    loop {
        print_char(stack[i]);
        if i == 0 {
            break;
        } else {
            i -= 1;
        }
    }
}

pub fn println(message: &'static str) {
    print(message);
    print_char(b'\n');
    print_char(b'\r');
}

fn print_char(char: u8) {
    let char = char as u16 | 0xE00;
    unsafe {
        asm!("push bx", "mov bx, 0", "int 0x10", "pop bx", in("ax") char);
    }
}
