//! Character type functions

/// Check if character is alphanumeric
pub fn isalnum(c: u8) -> bool {
    isalpha(c) || isdigit(c)
}

/// Check if character is alphabetic
pub fn isalpha(c: u8) -> bool {
    isupper(c) || islower(c)
}

/// Check if character is ASCII
pub fn isascii(c: u8) -> bool {
    c < 128
}

/// Check if character is blank (space or tab)
pub fn isblank(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check if character is control character
pub fn iscntrl(c: u8) -> bool {
    c < 32 || c == 127
}

/// Check if character is digit
pub fn isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Check if character is graphical (printable, not space)
pub fn isgraph(c: u8) -> bool {
    c > 32 && c < 127
}

/// Check if character is lowercase
pub fn islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

/// Check if character is printable
pub fn isprint(c: u8) -> bool {
    c >= 32 && c < 127
}

/// Check if character is punctuation
pub fn ispunct(c: u8) -> bool {
    isgraph(c) && !isalnum(c)
}

/// Check if character is whitespace
pub fn isspace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c')
}

/// Check if character is uppercase
pub fn isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Check if character is hex digit
pub fn isxdigit(c: u8) -> bool {
    isdigit(c) || (c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')
}

/// Convert to lowercase
pub fn tolower(c: u8) -> u8 {
    if isupper(c) {
        c + 32
    } else {
        c
    }
}

/// Convert to uppercase
pub fn toupper(c: u8) -> u8 {
    if islower(c) {
        c - 32
    } else {
        c
    }
}

/// Convert hex digit to value
pub fn hex_to_val(c: u8) -> Option<u8> {
    if c >= b'0' && c <= b'9' {
        Some(c - b'0')
    } else if c >= b'a' && c <= b'f' {
        Some(c - b'a' + 10)
    } else if c >= b'A' && c <= b'F' {
        Some(c - b'A' + 10)
    } else {
        None
    }
}
