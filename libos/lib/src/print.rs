//! Print operations

use libos_sync::Once;

/// Print operations trait
///
/// This must be implemented by the platform.
pub trait PrintOps: Send + Sync {
    /// Print a string
    fn print(&self, s: &str);
}

/// Global print operations
static PRINT_OPS: Once<&'static dyn PrintOps> = Once::new();

/// Set the global print operations
///
/// Can only be called once. Subsequent calls are ignored.
pub fn set_print_ops(ops: &'static dyn PrintOps) {
    PRINT_OPS.call_once(|| ops);
}

/// Get the global print operations
pub fn get_print_ops() -> Option<&'static dyn PrintOps> {
    PRINT_OPS.get().copied()
}

/// Print a string
pub fn print(s: &str) {
    if let Some(ops) = get_print_ops() {
        ops.print(s);
    }
}

/// Print bytes (lossy UTF-8 conversion)
pub fn print_bytes(s: &[u8]) {
    if let Ok(s) = core::str::from_utf8(s) {
        print(s);
    }
}

/// Print a byte as hex
pub fn print_hex_byte(b: u8) {
    const HEX: &[u8] = b"0123456789abcdef";
    let chars = [HEX[(b >> 4) as usize], HEX[(b & 0xf) as usize]];
    if let Ok(s) = core::str::from_utf8(&chars) {
        print(s);
    }
}

/// Print a number as hex
pub fn print_hex(n: u64) {
    print("0x");
    if n == 0 {
        print("0");
        return;
    }

    // Find first non-zero nibble
    let mut started = false;
    for i in (0..16).rev() {
        let nibble = ((n >> (i * 4)) & 0xf) as u8;
        if nibble != 0 || started {
            print_hex_byte(nibble);
            started = true;
        }
    }
}

/// Print a number as decimal
pub fn print_dec(mut n: u64) {
    if n == 0 {
        print("0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    // Reverse
    let slice = &mut buf[..i];
    slice.reverse();
    if let Ok(s) = core::str::from_utf8(slice) {
        print(s);
    }
}

/// Print signed decimal
pub fn print_signed(n: i64) {
    if n < 0 {
        print("-");
        print_dec((-n) as u64);
    } else {
        print_dec(n as u64);
    }
}

/// Print newline
pub fn println() {
    print("\n");
}

/// Print with newline
pub fn println_str(s: &str) {
    print(s);
    println();
}
