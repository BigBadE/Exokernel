//! Linux kernel error codes and result types
//!
//! This module provides:
//! - `Errno`: A type-safe wrapper around Linux error codes
//! - `KernelResult<T>`: An alias for `Result<T, Errno>`
//! - Conversion traits for FFI boundary handling

use core::ffi::c_int;
use core::ptr;

// ============================================================================
// Errno type
// ============================================================================

/// Linux kernel error code
///
/// This is a type-safe wrapper around errno values. It can be converted to:
/// - Negative integers (for functions returning c_int)
/// - Error pointers (for functions returning pointers, using ERR_PTR convention)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Errno(c_int);

impl Errno {
    /// Create an Errno from a raw error code (positive value)
    #[inline]
    pub const fn new(code: c_int) -> Self {
        Self(code)
    }

    /// Get the raw error code (positive value)
    #[inline]
    pub const fn code(self) -> c_int {
        self.0
    }

    /// Convert to negative int for C return values
    #[inline]
    pub const fn to_int(self) -> c_int {
        -self.0
    }

    /// Convert to an error pointer (ERR_PTR convention)
    #[inline]
    pub fn to_ptr<T>(self) -> *const T {
        (-self.0 as isize) as *const T
    }

    /// Convert to a mutable error pointer (ERR_PTR convention)
    #[inline]
    pub fn to_ptr_mut<T>(self) -> *mut T {
        (-self.0 as isize) as *mut T
    }

    /// Check if a pointer is an error pointer
    #[inline]
    pub fn is_err_ptr<T>(ptr: *const T) -> bool {
        let val = ptr as isize;
        val < 0 && val >= -4095
    }

    /// Extract errno from an error pointer
    #[inline]
    pub fn from_ptr<T>(ptr: *const T) -> Option<Self> {
        if Self::is_err_ptr(ptr) {
            Some(Self(-(ptr as isize) as c_int))
        } else {
            None
        }
    }

    // Common error codes as associated constants
    pub const EPERM: Self = Self(1);
    pub const ENOENT: Self = Self(2);
    pub const ESRCH: Self = Self(3);
    pub const EINTR: Self = Self(4);
    pub const EIO: Self = Self(5);
    pub const ENXIO: Self = Self(6);
    pub const E2BIG: Self = Self(7);
    pub const ENOEXEC: Self = Self(8);
    pub const EBADF: Self = Self(9);
    pub const ECHILD: Self = Self(10);
    pub const EAGAIN: Self = Self(11);
    pub const ENOMEM: Self = Self(12);
    pub const EACCES: Self = Self(13);
    pub const EFAULT: Self = Self(14);
    pub const ENOTBLK: Self = Self(15);
    pub const EBUSY: Self = Self(16);
    pub const EEXIST: Self = Self(17);
    pub const EXDEV: Self = Self(18);
    pub const ENODEV: Self = Self(19);
    pub const ENOTDIR: Self = Self(20);
    pub const EISDIR: Self = Self(21);
    pub const EINVAL: Self = Self(22);
    pub const ENFILE: Self = Self(23);
    pub const EMFILE: Self = Self(24);
    pub const ENOTTY: Self = Self(25);
    pub const ETXTBSY: Self = Self(26);
    pub const EFBIG: Self = Self(27);
    pub const ENOSPC: Self = Self(28);
    pub const ESPIPE: Self = Self(29);
    pub const EROFS: Self = Self(30);
    pub const EMLINK: Self = Self(31);
    pub const EPIPE: Self = Self(32);
    pub const EDOM: Self = Self(33);
    pub const ERANGE: Self = Self(34);
    pub const EDEADLK: Self = Self(35);
    pub const ENAMETOOLONG: Self = Self(36);
    pub const ENOLCK: Self = Self(37);
    pub const ENOSYS: Self = Self(38);
    pub const ENOTEMPTY: Self = Self(39);
    pub const ELOOP: Self = Self(40);
    pub const EWOULDBLOCK: Self = Self::EAGAIN;
    pub const ENOMSG: Self = Self(42);
    pub const EIDRM: Self = Self(43);
    pub const ENODATA: Self = Self(61);
    pub const ETIME: Self = Self(62);
    pub const EOVERFLOW: Self = Self(75);
    pub const EILSEQ: Self = Self(84);
    pub const ENOTSOCK: Self = Self(88);
    pub const EOPNOTSUPP: Self = Self(95);
    pub const ETIMEDOUT: Self = Self(110);
    pub const ESTALE: Self = Self(116);
    pub const EDQUOT: Self = Self(122);
}

// ============================================================================
// KernelResult type
// ============================================================================

/// Result type for kernel operations
pub type KernelResult<T> = Result<T, Errno>;

// ============================================================================
// Conversion traits for KernelResult
// ============================================================================

/// Extension trait for converting KernelResult to FFI types
pub trait KernelResultExt<T> {
    /// Convert to c_int (Ok = 0, Err = -errno)
    fn to_int(self) -> c_int;

    /// Convert to const pointer (Ok = ptr, Err = ERR_PTR)
    fn to_ptr(self) -> *const T;

    /// Convert to mutable pointer (Ok = ptr, Err = ERR_PTR)
    fn to_ptr_mut(self) -> *mut T;
}

impl KernelResultExt<()> for KernelResult<()> {
    #[inline]
    fn to_int(self) -> c_int {
        match self {
            Ok(()) => 0,
            Err(e) => e.to_int(),
        }
    }

    #[inline]
    fn to_ptr(self) -> *const () {
        match self {
            Ok(()) => ptr::null(),
            Err(e) => e.to_ptr(),
        }
    }

    #[inline]
    fn to_ptr_mut(self) -> *mut () {
        match self {
            Ok(()) => ptr::null_mut(),
            Err(e) => e.to_ptr_mut(),
        }
    }
}

impl<'a, T> KernelResultExt<T> for KernelResult<&'a T> {
    #[inline]
    fn to_int(self) -> c_int {
        match self {
            Ok(_) => 0,
            Err(e) => e.to_int(),
        }
    }

    #[inline]
    fn to_ptr(self) -> *const T {
        match self {
            Ok(r) => r as *const T,
            Err(e) => e.to_ptr(),
        }
    }

    #[inline]
    fn to_ptr_mut(self) -> *mut T {
        match self {
            Ok(r) => r as *const T as *mut T,
            Err(e) => e.to_ptr_mut(),
        }
    }
}

impl<'a, T> KernelResultExt<T> for KernelResult<&'a mut T> {
    #[inline]
    fn to_int(self) -> c_int {
        match self {
            Ok(_) => 0,
            Err(e) => e.to_int(),
        }
    }

    #[inline]
    fn to_ptr(self) -> *const T {
        match self {
            Ok(r) => r as *const T,
            Err(e) => e.to_ptr(),
        }
    }

    #[inline]
    fn to_ptr_mut(self) -> *mut T {
        match self {
            Ok(r) => r as *mut T,
            Err(e) => e.to_ptr_mut(),
        }
    }
}

// ============================================================================
// Option extensions
// ============================================================================

/// Extension trait for converting Option to KernelResult
pub trait OptionExt<T> {
    /// Convert None to ENOMEM
    fn ok_or_enomem(self) -> KernelResult<T>;

    /// Convert None to EINVAL
    fn ok_or_einval(self) -> KernelResult<T>;

    /// Convert None to ENOENT
    fn ok_or_enoent(self) -> KernelResult<T>;

    /// Convert None to specified error
    fn ok_or_err(self, err: Errno) -> KernelResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn ok_or_enomem(self) -> KernelResult<T> {
        self.ok_or(Errno::ENOMEM)
    }

    #[inline]
    fn ok_or_einval(self) -> KernelResult<T> {
        self.ok_or(Errno::EINVAL)
    }

    #[inline]
    fn ok_or_enoent(self) -> KernelResult<T> {
        self.ok_or(Errno::ENOENT)
    }

    #[inline]
    fn ok_or_err(self, err: Errno) -> KernelResult<T> {
        self.ok_or(err)
    }
}

// ============================================================================
// Legacy constants (for backwards compatibility with existing code)
// ============================================================================

pub const EPERM: c_int = 1;
pub const ENOENT: c_int = 2;
pub const ESRCH: c_int = 3;
pub const EINTR: c_int = 4;
pub const EIO: c_int = 5;
pub const ENXIO: c_int = 6;
pub const E2BIG: c_int = 7;
pub const ENOEXEC: c_int = 8;
pub const EBADF: c_int = 9;
pub const ECHILD: c_int = 10;
pub const EAGAIN: c_int = 11;
pub const ENOMEM: c_int = 12;
pub const EACCES: c_int = 13;
pub const EFAULT: c_int = 14;
pub const ENOTBLK: c_int = 15;
pub const EBUSY: c_int = 16;
pub const EEXIST: c_int = 17;
pub const EXDEV: c_int = 18;
pub const ENODEV: c_int = 19;
pub const ENOTDIR: c_int = 20;
pub const EISDIR: c_int = 21;
pub const EINVAL: c_int = 22;
pub const ENFILE: c_int = 23;
pub const EMFILE: c_int = 24;
pub const ENOTTY: c_int = 25;
pub const ETXTBSY: c_int = 26;
pub const EFBIG: c_int = 27;
pub const ENOSPC: c_int = 28;
pub const ESPIPE: c_int = 29;
pub const EROFS: c_int = 30;
pub const EMLINK: c_int = 31;
pub const EPIPE: c_int = 32;
pub const EDOM: c_int = 33;
pub const ERANGE: c_int = 34;
pub const EDEADLK: c_int = 35;
pub const ENAMETOOLONG: c_int = 36;
pub const ENOLCK: c_int = 37;
pub const ENOSYS: c_int = 38;
pub const ENOTEMPTY: c_int = 39;
pub const ELOOP: c_int = 40;
pub const EWOULDBLOCK: c_int = EAGAIN;
pub const ENOMSG: c_int = 42;
pub const EIDRM: c_int = 43;
pub const ECHRNG: c_int = 44;
pub const EL2NSYNC: c_int = 45;
pub const EL3HLT: c_int = 46;
pub const EL3RST: c_int = 47;
pub const ELNRNG: c_int = 48;
pub const EUNATCH: c_int = 49;
pub const ENOCSI: c_int = 50;
pub const EL2HLT: c_int = 51;
pub const EBADE: c_int = 52;
pub const EBADR: c_int = 53;
pub const EXFULL: c_int = 54;
pub const ENOANO: c_int = 55;
pub const EBADRQC: c_int = 56;
pub const EBADSLT: c_int = 57;
pub const EDEADLOCK: c_int = EDEADLK;
pub const EBFONT: c_int = 59;
pub const ENOSTR: c_int = 60;
pub const ENODATA: c_int = 61;
pub const ETIME: c_int = 62;
pub const ENOSR: c_int = 63;
pub const ENONET: c_int = 64;
pub const ENOPKG: c_int = 65;
pub const EREMOTE: c_int = 66;
pub const ENOLINK: c_int = 67;
pub const EADV: c_int = 68;
pub const ESRMNT: c_int = 69;
pub const ECOMM: c_int = 70;
pub const EPROTO: c_int = 71;
pub const EMULTIHOP: c_int = 72;
pub const EDOTDOT: c_int = 73;
pub const EBADMSG: c_int = 74;
pub const EOVERFLOW: c_int = 75;
pub const ENOTUNIQ: c_int = 76;
pub const EBADFD: c_int = 77;
pub const EREMCHG: c_int = 78;
pub const ELIBACC: c_int = 79;
pub const ELIBBAD: c_int = 80;
pub const ELIBSCN: c_int = 81;
pub const ELIBMAX: c_int = 82;
pub const ELIBEXEC: c_int = 83;
pub const EILSEQ: c_int = 84;
pub const ERESTART: c_int = 85;
pub const ESTRPIPE: c_int = 86;
pub const EUSERS: c_int = 87;
pub const ENOTSOCK: c_int = 88;
pub const EDESTADDRREQ: c_int = 89;
pub const EMSGSIZE: c_int = 90;
pub const EPROTOTYPE: c_int = 91;
pub const ENOPROTOOPT: c_int = 92;
pub const EPROTONOSUPPORT: c_int = 93;
pub const ESOCKTNOSUPPORT: c_int = 94;
pub const EOPNOTSUPP: c_int = 95;
pub const EPFNOSUPPORT: c_int = 96;
pub const EAFNOSUPPORT: c_int = 97;
pub const EADDRINUSE: c_int = 98;
pub const EADDRNOTAVAIL: c_int = 99;
pub const ENETDOWN: c_int = 100;
pub const ENETUNREACH: c_int = 101;
pub const ENETRESET: c_int = 102;
pub const ECONNABORTED: c_int = 103;
pub const ECONNRESET: c_int = 104;
pub const ENOBUFS: c_int = 105;
pub const EISCONN: c_int = 106;
pub const ENOTCONN: c_int = 107;
pub const ESHUTDOWN: c_int = 108;
pub const ETOOMANYREFS: c_int = 109;
pub const ETIMEDOUT: c_int = 110;
pub const ECONNREFUSED: c_int = 111;
pub const EHOSTDOWN: c_int = 112;
pub const EHOSTUNREACH: c_int = 113;
pub const EALREADY: c_int = 114;
pub const EINPROGRESS: c_int = 115;
pub const ESTALE: c_int = 116;
pub const EUCLEAN: c_int = 117;
pub const ENOTNAM: c_int = 118;
pub const ENAVAIL: c_int = 119;
pub const EISNAM: c_int = 120;
pub const EREMOTEIO: c_int = 121;
pub const EDQUOT: c_int = 122;
pub const ENOMEDIUM: c_int = 123;
pub const EMEDIUMTYPE: c_int = 124;
pub const ECANCELED: c_int = 125;
pub const ENOKEY: c_int = 126;
pub const EKEYEXPIRED: c_int = 127;
pub const EKEYREVOKED: c_int = 128;
pub const EKEYREJECTED: c_int = 129;
pub const EOWNERDEAD: c_int = 130;
pub const ENOTRECOVERABLE: c_int = 131;
pub const ERFKILL: c_int = 132;
pub const EHWPOISON: c_int = 133;
