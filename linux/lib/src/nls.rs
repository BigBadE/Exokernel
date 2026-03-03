//! Linux kernel NLS (Native Language Support) for character set conversion

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use linux_core::{linux_export, wchar_t, EINVAL, ENAMETOOLONG};

/// NLS table - character set conversion table
#[repr(C)]
pub struct nls_table {
    pub charset: *const c_char,
    pub alias: *const c_char,
    pub uni2char: Option<unsafe extern "C" fn(wchar_t, *mut u8, c_int) -> c_int>,
    pub char2uni: Option<unsafe extern "C" fn(*const u8, c_int, *mut wchar_t) -> c_int>,
    pub charset2lower: *const u8,
    pub charset2upper: *const u8,
    pub owner: *mut c_void,
    pub next: *mut nls_table,
}

// ============================================================================
// Character tables
// ============================================================================

static ASCII_LOWER: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = if i >= b'A' as usize && i <= b'Z' as usize {
            (i + 32) as u8
        } else {
            i as u8
        };
        i += 1;
    }
    table
};

static ASCII_UPPER: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = if i >= b'a' as usize && i <= b'z' as usize {
            (i - 32) as u8
        } else {
            i as u8
        };
        i += 1;
    }
    table
};

static DEFAULT_CHARSET: &[u8] = b"default\0";
static UTF8_CHARSET: &[u8] = b"utf8\0";
static CP437_CHARSET: &[u8] = b"cp437\0";

// ============================================================================
// Conversion functions (callbacks - must remain unsafe extern "C")
// ============================================================================

unsafe extern "C" fn default_uni2char(uni: wchar_t, out: *mut u8, boundlen: c_int) -> c_int {
    if out.is_null() || boundlen < 1 {
        return -ENAMETOOLONG;
    }

    if uni < 0x100 {
        *out = uni as u8;
        1
    } else {
        *out = b'?';
        1
    }
}

unsafe extern "C" fn default_char2uni(rawstring: *const u8, boundlen: c_int, uni: *mut wchar_t) -> c_int {
    if rawstring.is_null() || uni.is_null() || boundlen < 1 {
        return -EINVAL;
    }

    *uni = *rawstring as wchar_t;
    1
}

unsafe extern "C" fn utf8_uni2char(uni: wchar_t, out: *mut u8, boundlen: c_int) -> c_int {
    if out.is_null() {
        return -ENAMETOOLONG;
    }

    let uni = uni as u32;

    if uni < 0x80 {
        if boundlen < 1 {
            return -ENAMETOOLONG;
        }
        *out = uni as u8;
        1
    } else if uni < 0x800 {
        if boundlen < 2 {
            return -ENAMETOOLONG;
        }
        *out = (0xC0 | (uni >> 6)) as u8;
        *out.add(1) = (0x80 | (uni & 0x3F)) as u8;
        2
    } else {
        if boundlen < 3 {
            return -ENAMETOOLONG;
        }
        *out = (0xE0 | (uni >> 12)) as u8;
        *out.add(1) = (0x80 | ((uni >> 6) & 0x3F)) as u8;
        *out.add(2) = (0x80 | (uni & 0x3F)) as u8;
        3
    }
}

unsafe extern "C" fn utf8_char2uni(rawstring: *const u8, boundlen: c_int, uni: *mut wchar_t) -> c_int {
    if rawstring.is_null() || uni.is_null() || boundlen < 1 {
        return -EINVAL;
    }

    let c0 = *rawstring;

    if c0 < 0x80 {
        *uni = c0 as wchar_t;
        1
    } else if c0 < 0xE0 {
        if boundlen < 2 {
            return -EINVAL;
        }
        let c1 = *rawstring.add(1);
        if (c1 & 0xC0) != 0x80 {
            return -EINVAL;
        }
        *uni = (((c0 & 0x1F) as u16) << 6) | ((c1 & 0x3F) as u16);
        2
    } else if c0 < 0xF0 {
        if boundlen < 3 {
            return -EINVAL;
        }
        let c1 = *rawstring.add(1);
        let c2 = *rawstring.add(2);
        if (c1 & 0xC0) != 0x80 || (c2 & 0xC0) != 0x80 {
            return -EINVAL;
        }
        *uni = (((c0 & 0x0F) as u16) << 12)
            | (((c1 & 0x3F) as u16) << 6)
            | ((c2 & 0x3F) as u16);
        3
    } else {
        -EINVAL
    }
}

// ============================================================================
// Static NLS tables
// ============================================================================

static mut DEFAULT_NLS: nls_table = nls_table {
    charset: DEFAULT_CHARSET.as_ptr() as *const c_char,
    alias: ptr::null(),
    uni2char: Some(default_uni2char),
    char2uni: Some(default_char2uni),
    charset2lower: ASCII_LOWER.as_ptr(),
    charset2upper: ASCII_UPPER.as_ptr(),
    owner: ptr::null_mut(),
    next: ptr::null_mut(),
};

static mut UTF8_NLS: nls_table = nls_table {
    charset: UTF8_CHARSET.as_ptr() as *const c_char,
    alias: ptr::null(),
    uni2char: Some(utf8_uni2char),
    char2uni: Some(utf8_char2uni),
    charset2lower: ASCII_LOWER.as_ptr(),
    charset2upper: ASCII_UPPER.as_ptr(),
    owner: ptr::null_mut(),
    next: ptr::null_mut(),
};

static mut CP437_NLS: nls_table = nls_table {
    charset: CP437_CHARSET.as_ptr() as *const c_char,
    alias: ptr::null(),
    uni2char: Some(default_uni2char),
    char2uni: Some(default_char2uni),
    charset2lower: ASCII_LOWER.as_ptr(),
    charset2upper: ASCII_UPPER.as_ptr(),
    owner: ptr::null_mut(),
    next: ptr::null_mut(),
};

// ============================================================================
// NLS API
// ============================================================================

unsafe fn cmp_charset(s1: *const c_char, s2: *const c_char) -> bool {
    if s1.is_null() || s2.is_null() {
        return false;
    }

    let mut p1 = s1;
    let mut p2 = s2;

    while *p1 != 0 && *p2 != 0 {
        let c1 = (*p1 as u8).to_ascii_lowercase();
        let c2 = (*p2 as u8).to_ascii_lowercase();
        if c1 != c2 {
            return false;
        }
        p1 = p1.add(1);
        p2 = p2.add(1);
    }

    *p1 == 0 && *p2 == 0
}

#[linux_export]
unsafe fn load_nls(charset: *const c_char) -> *mut nls_table {
    if charset.is_null() {
        return &raw mut DEFAULT_NLS;
    }

    if cmp_charset(charset, b"utf8\0".as_ptr() as *const c_char) {
        return &raw mut UTF8_NLS;
    }

    if cmp_charset(charset, b"cp437\0".as_ptr() as *const c_char)
        || cmp_charset(charset, b"cp850\0".as_ptr() as *const c_char)
    {
        return &raw mut CP437_NLS;
    }

    &raw mut DEFAULT_NLS
}

#[linux_export]
unsafe fn load_nls_default() -> *mut nls_table {
    &raw mut DEFAULT_NLS
}

#[linux_export]
fn unload_nls(_nls: *mut nls_table) {
}

#[linux_export]
unsafe fn get_default_nls() -> *mut nls_table {
    &raw mut DEFAULT_NLS
}

// ============================================================================
// NLS helper functions
// ============================================================================

#[linux_export]
unsafe fn nls_uni16s_to_nls(
    nls: *const nls_table,
    uni: *const wchar_t,
    unilen: c_int,
    out: *mut u8,
    outlen: c_int,
) -> c_int {
    if nls.is_null() || uni.is_null() || out.is_null() {
        return -EINVAL;
    }

    let uni2char = match (*nls).uni2char {
        Some(f) => f,
        None => return -EINVAL,
    };

    let mut i = 0;
    let mut o = 0;

    while i < unilen && o < outlen {
        let ch = *uni.add(i as usize);
        let ret = uni2char(ch, out.add(o as usize), outlen - o);
        if ret < 0 {
            return ret;
        }
        o += ret;
        i += 1;
    }

    o
}

#[linux_export]
unsafe fn nls_nls_to_uni16s(
    nls: *const nls_table,
    input: *const u8,
    inlen: c_int,
    uni: *mut wchar_t,
    unilen: c_int,
) -> c_int {
    if nls.is_null() || input.is_null() || uni.is_null() {
        return -EINVAL;
    }

    let char2uni = match (*nls).char2uni {
        Some(f) => f,
        None => return -EINVAL,
    };

    let mut i = 0;
    let mut o = 0;

    while i < inlen && o < unilen {
        let mut ch: wchar_t = 0;
        let ret = char2uni(input.add(i as usize), inlen - i, &mut ch);
        if ret < 0 {
            return ret;
        }
        *uni.add(o as usize) = ch;
        i += ret;
        o += 1;
    }

    o
}

#[linux_export]
unsafe fn nls_tolower(nls: *const nls_table, c: c_uint) -> c_uint {
    if nls.is_null() {
        return c;
    }
    if !(*nls).charset2lower.is_null() && c < 256 {
        *(*nls).charset2lower.add(c as usize) as c_uint
    } else {
        c
    }
}

#[linux_export]
unsafe fn nls_toupper(nls: *const nls_table, c: c_uint) -> c_uint {
    if nls.is_null() {
        return c;
    }
    if !(*nls).charset2upper.is_null() && c < 256 {
        *(*nls).charset2upper.add(c as usize) as c_uint
    } else {
        c
    }
}

#[linux_export]
unsafe fn nls_strlen(nls: *const nls_table, s: *const u8) -> c_int {
    if s.is_null() {
        return 0;
    }

    let char2uni = if !nls.is_null() {
        (*nls).char2uni
    } else {
        Some(default_char2uni as unsafe extern "C" fn(*const u8, c_int, *mut wchar_t) -> c_int)
    };

    let char2uni = match char2uni {
        Some(f) => f,
        None => return 0,
    };

    let mut len = 0;
    let mut p = s;

    while *p != 0 {
        let mut uni: wchar_t = 0;
        let mut remaining = 0;
        let mut q = p;
        while *q != 0 {
            remaining += 1;
            q = q.add(1);
        }

        let ret = char2uni(p, remaining, &mut uni);
        if ret <= 0 {
            break;
        }
        len += 1;
        p = p.add(ret as usize);
    }

    len
}
