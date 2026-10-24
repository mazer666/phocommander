//! The error type of the core layer and the stable codes of its failures.
//!
//! WHAT: the failures that the core layer reports ([{Error}]), the reason a
//!       file system call failed ([{IoOperation}]) and the stable code that
//!       names the failure ([{ErrorCode}]).
//! WHY: every fallible operation in this workspace returns a {Result}. The
//!       error has to name what failed and carry the cause, otherwise the
//!       user interface can only show "something went wrong". The code is
//!       the contract with the user interface (ADR-0003): the interface
//!       matches on the code and shows the message of its translation
//!       catalogue, the log records the code, and the acceptance
//!       documentation cites the code instead of a sentence.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::Error;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Error> {
//! let path = Path::new("/etc/hosts");
//! let metadata = std::fs::metadata(path).map_err(|source| Error::io(path, source))?;
//! println!("{} bytes", metadata.len());
//! # Ok(())
//! # }
//! ```

use std::io;
use std::path::{Path, PathBuf};

use crate::path::PathError;

/// The reason why a file system call failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoOperation {
    /// The file or the directory was read.
    Read,

    /// The file or the directory was written, renamed or deleted.
    Write,
}

impl IoOperation {
    /// The stable code of the operation.
    ///
    /// WHY: a failure to read and a failure to write lead to different
    /// messages, so the code has to tell them apart.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::error::IoOperation;
    ///
    /// assert_eq!(IoOperation::Read.as_str(), "io.read");
    /// assert_eq!(IoOperation::Write.as_str(), "io.write");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            IoOperation::Read => "io.read",
            IoOperation::Write => "io.write",
        }
    }
}

/// The stable code of a failure of the core layer.
///
/// WHY: the codes are part of the public surface of the project. A new
/// failure gets a new code, and the meaning of an existing code never
/// changes (ADR-0003). The number is the numeric form of the same contract:
/// it is what a log record and an acceptance document carry, and it stays
/// stable even if the name of a code is spelled differently somewhere.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::error::ErrorCode;
///
/// assert_eq!(ErrorCode::IoRead.as_str(), "io.read");
/// assert_eq!(ErrorCode::PathTraversal.number(), 1102);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// A file or a directory could not be read.
    IoRead,

    /// A file could not be written.
    IoWrite,

    /// The operating system refused the access.
    IoPermission,

    /// A path was absolute where a relative one was required.
    PathAbsolute,

    /// A path tried to leave the destination directory.
    PathTraversal,

    /// A name is not valid on every target platform.
    PathReserved,

    /// A file or an archive exceeded a limit.
    LimitTooLarge,

    /// An archive has too many entries.
    LimitEntries,

    /// The operating system reported no configuration directory.
    PlatformNoConfigDirectory,
}

impl ErrorCode {
    /// The name of the code, as the table of ADR-0003 writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::error::ErrorCode;
    ///
    /// assert_eq!(ErrorCode::LimitTooLarge.as_str(), "limit.too_large");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::IoRead => "io.read",
            ErrorCode::IoWrite => "io.write",
            ErrorCode::IoPermission => "io.permission",
            ErrorCode::PathAbsolute => "path.absolute",
            ErrorCode::PathTraversal => "path.traversal",
            ErrorCode::PathReserved => "path.reserved",
            ErrorCode::LimitTooLarge => "limit.too_large",
            ErrorCode::LimitEntries => "limit.entries",
            ErrorCode::PlatformNoConfigDirectory => "platform.no_config_directory",
        }
    }

    /// The number of the code.
    ///
    /// WHY: the number is what a log record carries and what an acceptance
    /// document cites. It is stable: a code that is never used again keeps
    /// its number, and a new code gets the next free number of its group.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::error::ErrorCode;
    ///
    /// assert_eq!(ErrorCode::IoRead.number(), 1001);
    /// assert_eq!(ErrorCode::LimitEntries.number(), 1202);
    /// ```
    pub fn number(&self) -> u16 {
        match self {
            ErrorCode::IoRead => 1001,
            ErrorCode::IoWrite => 1002,
            ErrorCode::IoPermission => 1003,
            ErrorCode::PathAbsolute => 1101,
            ErrorCode::PathTraversal => 1102,
            ErrorCode::PathReserved => 1103,
            ErrorCode::LimitTooLarge => 1201,
            ErrorCode::LimitEntries => 1202,
            ErrorCode::PlatformNoConfigDirectory => 1901,
        }
    }
}

/// A failure of the core layer.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operating system did not report a directory for user configuration.
    #[error("the platform did not report a configuration directory")]
    NoConfigDirectory,

    /// A file or directory was larger than the limit that applies to it.
    #[error("{path} is {size} bytes, the limit is {limit} bytes")]
    TooLarge {
        /// The path of the file that was too large.
        path: PathBuf,
        /// The size that was measured.
        size: u64,
        /// The limit that was exceeded.
        limit: u64,
    },

    /// A filesystem operation failed.
    #[error("filesystem operation failed for {path}: {source}")]
    Io {
        /// The path the operation was working on.
        path: PathBuf,
        /// The kind of operation that failed.
        operation: IoOperation,
        /// The error the operating system reported.
        #[source]
        source: io::Error,
    },

    /// A path was refused because it would leave the destination directory.
    #[error(transparent)]
    Path(#[from] PathError),
}

impl Error {
    /// Builds an [{Error::Io}] for a read operation.
    ///
    /// WHY: the path is usually lost when an [{io::Error}] travels upwards.
    /// The path is what the user needs in order to understand the failure, so
    /// it is carried along.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::Error;
    /// use std::path::Path;
    ///
    /// let error = Error::io(Path::new("/does/not/exist"), std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
    /// assert!(error.to_string().contains("missing"));
    /// ```
    pub fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            operation: IoOperation::Read,
            source,
        }
    }

    /// Builds an [{Error::Io}] for a write operation.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::Error;
    /// use std::path::Path;
    ///
    /// let error = Error::write(Path::new("/root/x"), std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"));
    /// assert_eq!(error.code().as_str(), "io.permission");
    /// ```
    pub fn write(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            operation: IoOperation::Write,
            source,
        }
    }

    /// Builds an [{Error::TooLarge}] for a path, the measured size and the
    /// limit that was exceeded.
    ///
    /// WHY: the viewer and the archive adapter have to tell the user why they
    /// refused a file. Naming the size and the limit makes the refusal
    /// explainable instead of mysterious.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::Error;
    /// use std::path::Path;
    ///
    /// let error = Error::too_large(Path::new("movie.mkv"), 20_000_000, 10_485_760);
    /// assert!(error.to_string().contains("movie.mkv"));
    /// ```
    pub fn too_large(path: impl AsRef<Path>, size: u64, limit: u64) -> Self {
        Self::TooLarge {
            path: path.as_ref().to_path_buf(),
            size,
            limit,
        }
    }

    /// The stable code of the failure.
    ///
    /// WHY: the user interface matches on the code and never on the text of
    /// the error. The text is written for the log, the code is written for
    /// the message catalogue.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::Error;
    /// use std::path::Path;
    ///
    /// let error = Error::io(Path::new("/root/x"), std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"));
    /// assert_eq!(error.code().as_str(), "io.permission");
    /// ```
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::NoConfigDirectory => ErrorCode::PlatformNoConfigDirectory,
            Error::TooLarge { .. } => ErrorCode::LimitTooLarge,
            Error::Path(error) => error.code(),
            Error::Io {
                operation, source, ..
            } => {
                if source.kind() == io::ErrorKind::PermissionDenied {
                    ErrorCode::IoPermission
                } else {
                    match operation {
                        IoOperation::Read => ErrorCode::IoRead,
                        IoOperation::Write => ErrorCode::IoWrite,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
