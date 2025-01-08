use crate::{
    task::TaskLocalErr,
};
use log::SetLoggerError;
use std::{
    panic::Location,
    error::Error as StdError,
    io::{Error as IoError, ErrorKind as IoErrorKind},
    ffi::NulError,
    sync::PoisonError,
    fmt,
};

/// Result type for this crate
pub type Result<T> = std::result::Result<T, ErrorKind>;

/// A list of possible errors that can occur in this library
///
/// This list is intended to be grow over time and is not recommended to
/// be exhaustively matched.
///
/// It is used with the [`Result`](type@crate::Result) type to signal that an error occurred.
///
/// The `IoError` variant is used to catch any underlying [`std::io::Errors`](std::io::Error)
/// without mapping them to internal error(s). This allows a better control over underling
/// errors instead of "hiding" them.
///
/// # Handling errors and matching `ErrorKind`
///
/// In application code `match` against the expected `ErrorKind`, use `_` to match all
/// other Errors.
///
/// ```rust
/// use hclog::{ErrorKind};
///
/// match hclog::dump(&mut std::io::stdout()) {
///    Ok(_) => {},
///    Err(ErrorKind::IoError(e)) => eprintln!("I/O Error: {:?}", e),
///    Err(_) => panic!("Unexpected error"),
/// }
/// ```
#[derive(PartialEq, Eq, Ord, PartialOrd)]
pub enum ErrorKind {
    /// Failed to lock the context
    ContextLock,
    /// Context is inconsistent
    ContextInconsistent,
    /// Log-Module isn't initialized
    ScopeNotInitialized,
    /// Submodule is not initialized
    KeyNotInitialized,
    /// Parse environment variable failed
    ParseEnv,
    /// Parse commandline argument string failed
    ParseArg,
    /// Environment variable has unexpected type
    EnvType,
    /// Loglevel is unknown
    UnknownLogLevel,
    /// Failed to write logstring via [`Facade`](type@crate::FacadeVariant)
    WriteFailed,
    /// Logging string contained non utf8 characters
    InvalFmtString,
    /// Log Compat is already initialized
    LogCompatInitialized,
    /// Error in task local access
    TaskLocal(TaskLocalErr),
    /// I/O Error while writing
    ///
    /// Wraps the [`std::io::ErrorKind`] thrown by the underlying I/O operation.
    IoError(IoErrorKind),
}
impl StdError for ErrorKind {}
impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ContextLock => write!(f, "Failed to lock Context"),
            Self::ContextInconsistent => write!(f, "Context is inconsistent"),
            Self::ScopeNotInitialized => f.write_str("Scope not initialized"),
            Self::KeyNotInitialized => write!(f, "LogKey is not initialized"),
            Self::ParseEnv => write!(f, "Parse environment variable failed"),
            Self::ParseArg => write!(f, "Parse argument string failed"),
            Self::EnvType => write!(f, "Environment variable has unexpected type"),
            Self::UnknownLogLevel => write!(f, "Loglevel is unknown"),
            Self::WriteFailed => write!(f, "Failed to write logstring"),
            Self::InvalFmtString => write!(f, "Logging string contained non utf8 characters"),
            Self::LogCompatInitialized => write!(f, "Log Compat is already initialized"),
            Self::TaskLocal(ref e) => write!(f, "Error '{}' in task local access", e),
            Self::IoError(ref i) => write!(f, "IoError while writing: {:?}", i),
        }
    }
}
impl fmt::Debug for ErrorKind {
    #[track_caller]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::TaskLocal(ref e) => write!(f, "{}", e),
            _ => write!(f, "{} in {}", self, Location::caller()),
        }
    }
}
/*
 * map every other possible error to its according ErrorKind Variant
 */
impl From<TaskLocalErr> for ErrorKind {
    fn from(tle: TaskLocalErr) -> Self {
        Self::TaskLocal(tle)
    }
}
impl From<IoError> for ErrorKind {
    fn from(io: IoError) -> Self {
        Self::IoError(io.kind())
    }
}
impl<T: 'static> From<PoisonError<T>> for ErrorKind {
    fn from(_: PoisonError<T>) -> Self {
        Self::ContextLock
    }
}
impl From<NulError> for ErrorKind {
    fn from(_: NulError) -> Self {
        Self::InvalFmtString
    }
}
impl From<SetLoggerError> for ErrorKind {
    fn from(_: SetLoggerError) -> Self {
        Self::LogCompatInitialized
    }
}

#[cfg(test)]
mod test {
    use super::ErrorKind::{self, *};
    use crate::task::TaskLocalErr::*;
    use std::io::{Error as IoError, ErrorKind::BrokenPipe};

    #[test]
    fn error_eq() {
        assert_eq!(ParseEnv == ParseEnv, true);
        assert_eq!(TaskLocal(BorrowError) == TaskLocal(BorrowError), true);
    }

    #[test]
    fn error_ne() {
        assert_eq!(ParseEnv == EnvType, false);
        assert_eq!(TaskLocal(AccessError) == TaskLocal(BorrowError), false);
    }

    #[test]
    fn from_other_error() {
        assert_eq!(ErrorKind::from(IoError::from(BrokenPipe)), IoError(BrokenPipe));
    }
}
