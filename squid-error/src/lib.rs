#![forbid(unsafe_code)]
#![deny(
    dead_code,
    unused_imports,
    unused_mut,
    missing_docs,
    missing_debug_implementations
)]
//! internal library to provide structures for errors in Squid.
//!
//! # Examples
//! ```rust
//! use squid_error::Result;
//!
//! fn main() -> Result<()> {
//!     Ok(())
//! }
//! ```

use std::error::Error as StdError;
use std::fmt;

/// Boxed error to bypass specific [Error](StdError).
type BError = Box<dyn StdError + Send + Sync>;
/// anyhow-like error handler.
pub type Result<T> = core::result::Result<T, BError>;

/// The struct that represents an error
#[derive(Debug)]
pub struct Error {
    /// The error type.
    pub etype: ErrorType,
    /// The cause of this error.
    pub cause: Option<BError>,
    /// Explains the context in which the error occurs.
    pub context: Option<String>,
}

impl Error {
    /// Throw an [`Error`].
    pub fn new(
        etype: ErrorType,
        cause: Option<BError>,
        context: Option<String>,
    ) -> Self {
        Error {
            etype,
            cause,
            context,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.etype)
    }
}
impl StdError for Error {}

/// Errors in Squid.
#[derive(Debug)]
pub enum ErrorType {
    /// Generic error that returns no additional information.
    Unspecified,
    /// Errors related to `squid-db`.
    Database(DatabaseError),
    /// IO errors, especially due to std::fs.
    InputOutput(IoError),
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorType::Unspecified => {
                write!(f, "An error has occurred, but no further information is provided.")
            },
            ErrorType::Database(error) => write!(f, "{:?}", error),
            ErrorType::InputOutput(error) => write!(f, "{:?}", error),
        }
    }
}
impl StdError for ErrorType {}

/// Errors related to `squid-db`.
#[derive(Debug)]
pub enum DatabaseError {
    /// File compression failed.
    FailedCompression,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::FailedCompression => {
                write!(f, "File compression failed.")
            },
        }
    }
}
impl StdError for DatabaseError {}

/// Errors related to [`std`].
#[derive(Debug)]
pub enum IoError {
    /// Deserialization failed.
    DeserializationError,
    /// Serialization failed.
    SerializationError,
    /// Data are corrupted or not in the correct format (UTF-8).
    ReadingError,
    /// Failed unwrap Rwlock or Mutex for writing.
    WritingError,
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IoError::DeserializationError => {
                write!(f, "Deserialization failed.")
            },
            IoError::SerializationError => write!(f, "Serialization failed."),
            IoError::ReadingError => write!(
                f,
                "Data are corrupted or not in the correct format (UTF-8)."
            ),
            IoError::WritingError => {
                write!(f, "Failed unwrap Rwlock or Mutex for writing.")
            },
        }
    }
}
impl StdError for IoError {}
