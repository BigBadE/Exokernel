use core::arch::asm;

pub fn print(message: &str) {
    for char in message.bytes() {
        print_char(char);
    }
}

pub fn print_numb(mut number: u32) {
    if number == 0 {
        print_char(b'0');
    }
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

pub fn print_hex(mut number: u32) {
    let mut stack: [u8; 10] = [0; 10];
    let mut i = 0;
    if number == 0 {
        print_char(b'0');
    }
    while number > 0 {
        stack[i] = to_hex((number % 16) as u8);
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

fn to_hex(value: u8) -> u8 {
    if value > 9 {
        b'A' + value - 10
    } else {
        b'0' + value
    }
}

pub fn print_hex_buf(buf: &[u8]) {
    let mut out = 0;
    for i in 0..buf.len() {
        print_char(to_hex(buf[i] & 0x0F));
        print_char(to_hex((buf[i] & 0xF0) >> 4));
        print_char(b' ');
        out += 1;
        if out == 24 {
            print_char(b'\n');
            print_char(b'\r');
            out = 0;
        }
    }
}

pub fn printcharbuf(buf: &[u8]) {
    let mut out = 0;
    for i in 0..buf.len() {
        print_char(buf[i]);
        out += 1;
        if out == 32 {
            print_char(b'\n');
            print_char(b'\r');
            out = 0;
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
