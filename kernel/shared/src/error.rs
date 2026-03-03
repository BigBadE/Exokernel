//! Error types for the Exokernel

/// System call result type
pub type SysResult<T> = Result<T, SysError>;

/// System error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum SysError {
    /// Operation succeeded (not an error)
    Success = 0,

    /// Invalid capability handle
    InvalidCapability = -1,

    /// Capability has been revoked
    RevokedCapability = -2,

    /// Insufficient rights for operation
    InsufficientRights = -3,

    /// Resource not found
    NotFound = -4,

    /// Resource already exists
    AlreadyExists = -5,

    /// Out of memory
    OutOfMemory = -6,

    /// Invalid argument
    InvalidArgument = -7,

    /// Operation not permitted
    NotPermitted = -8,

    /// Resource busy
    Busy = -9,

    /// Would block (for non-blocking operations)
    WouldBlock = -10,

    /// Invalid address
    BadAddress = -11,

    /// Capability table full
    CapabilityTableFull = -12,

    /// Cannot delegate (missing DELEGATE right)
    CannotDelegate = -13,

    /// Resource range overflow
    RangeOverflow = -14,

    /// Invalid syscall number
    InvalidSyscall = -15,

    /// Process not found
    ProcessNotFound = -16,

    /// IPC endpoint not found
    EndpointNotFound = -17,

    /// Message too large
    MessageTooLarge = -18,

    /// No message available
    NoMessage = -19,

    /// IRQ already bound
    IrqAlreadyBound = -20,

    /// Port access denied
    PortAccessDenied = -21,

    /// Page fault during operation
    PageFault = -22,

    /// Alignment error
    AlignmentError = -23,

    /// Internal kernel error (should not happen)
    InternalError = -99,
}

impl SysError {
    pub fn from_code(code: i64) -> Self {
        match code {
            0 => SysError::Success,
            -1 => SysError::InvalidCapability,
            -2 => SysError::RevokedCapability,
            -3 => SysError::InsufficientRights,
            -4 => SysError::NotFound,
            -5 => SysError::AlreadyExists,
            -6 => SysError::OutOfMemory,
            -7 => SysError::InvalidArgument,
            -8 => SysError::NotPermitted,
            -9 => SysError::Busy,
            -10 => SysError::WouldBlock,
            -11 => SysError::BadAddress,
            -12 => SysError::CapabilityTableFull,
            -13 => SysError::CannotDelegate,
            -14 => SysError::RangeOverflow,
            -15 => SysError::InvalidSyscall,
            -16 => SysError::ProcessNotFound,
            -17 => SysError::EndpointNotFound,
            -18 => SysError::MessageTooLarge,
            -19 => SysError::NoMessage,
            -20 => SysError::IrqAlreadyBound,
            -21 => SysError::PortAccessDenied,
            -22 => SysError::PageFault,
            -23 => SysError::AlignmentError,
            _ => SysError::InternalError,
        }
    }

    pub fn as_code(&self) -> i64 {
        *self as i64
    }

    pub fn is_success(&self) -> bool {
        matches!(self, SysError::Success)
    }
}

impl From<SysError> for i64 {
    fn from(err: SysError) -> i64 {
        err.as_code()
    }
}
