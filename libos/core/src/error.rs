//! Error types for libOS operations

use core::fmt;

/// Result type for libOS operations
pub type Result<T> = core::result::Result<T, Error>;

/// LibOS error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Error {
    /// Operation not permitted
    PermissionDenied = 1,
    /// No such file or directory
    NotFound = 2,
    /// I/O error
    Io = 5,
    /// Out of memory
    OutOfMemory = 12,
    /// Permission denied
    AccessDenied = 13,
    /// Resource busy
    Busy = 16,
    /// File exists
    AlreadyExists = 17,
    /// Not a directory
    NotDirectory = 20,
    /// Is a directory
    IsDirectory = 21,
    /// Invalid argument
    InvalidArgument = 22,
    /// Too many open files
    TooManyOpenFiles = 24,
    /// No space left on device
    NoSpace = 28,
    /// Read-only filesystem
    ReadOnlyFilesystem = 30,
    /// Operation not supported
    NotSupported = 38,
    /// Name too long
    NameTooLong = 36,
    /// Directory not empty
    DirectoryNotEmpty = 39,
    /// No data available
    NoData = 61,
    /// Invalid seek
    InvalidSeek = 29,
    /// Unknown error
    Unknown = 255,
}

impl Error {
    /// Convert from a negative errno value
    pub fn from_errno(errno: i32) -> Self {
        let code = if errno < 0 { -errno } else { errno };
        match code {
            1 => Error::PermissionDenied,
            2 => Error::NotFound,
            5 => Error::Io,
            12 => Error::OutOfMemory,
            13 => Error::AccessDenied,
            16 => Error::Busy,
            17 => Error::AlreadyExists,
            20 => Error::NotDirectory,
            21 => Error::IsDirectory,
            22 => Error::InvalidArgument,
            24 => Error::TooManyOpenFiles,
            28 => Error::NoSpace,
            29 => Error::InvalidSeek,
            30 => Error::ReadOnlyFilesystem,
            36 => Error::NameTooLong,
            38 => Error::NotSupported,
            39 => Error::DirectoryNotEmpty,
            61 => Error::NoData,
            _ => Error::Unknown,
        }
    }

    /// Convert to a negative errno value
    pub fn to_errno(self) -> i32 {
        -(self as i32)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PermissionDenied => write!(f, "Operation not permitted"),
            Error::NotFound => write!(f, "No such file or directory"),
            Error::Io => write!(f, "I/O error"),
            Error::OutOfMemory => write!(f, "Out of memory"),
            Error::AccessDenied => write!(f, "Permission denied"),
            Error::Busy => write!(f, "Resource busy"),
            Error::AlreadyExists => write!(f, "File exists"),
            Error::NotDirectory => write!(f, "Not a directory"),
            Error::IsDirectory => write!(f, "Is a directory"),
            Error::InvalidArgument => write!(f, "Invalid argument"),
            Error::TooManyOpenFiles => write!(f, "Too many open files"),
            Error::NoSpace => write!(f, "No space left on device"),
            Error::ReadOnlyFilesystem => write!(f, "Read-only filesystem"),
            Error::NotSupported => write!(f, "Operation not supported"),
            Error::NameTooLong => write!(f, "Name too long"),
            Error::DirectoryNotEmpty => write!(f, "Directory not empty"),
            Error::NoData => write!(f, "No data available"),
            Error::InvalidSeek => write!(f, "Invalid seek"),
            Error::Unknown => write!(f, "Unknown error"),
        }
    }
}
