//! The failures of the operations and the stable codes of the jobs.
//!
//! WHAT: the failure of a single file operation ([{OpError}]), the failure a
//!       job collects ([{JobError}]) and the code that names it
//!       ([{JobErrorCode}]).
//! WHY: a job does not stop at the first failure — it collects the failures
//!       and continues with the next item (FR-25). Every collected failure
//!       therefore has to name the item it belongs to and say whether the job
//!       continues. The code is the contract with the user interface: the
//!       interface matches on the code and shows the message of its
//!       translation catalogue, and the log records the code and never the
//!       path of the user (ADR-0003).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::error::{JobError, JobErrorCode};
//! use std::path::PathBuf;
//!
//! let error = JobError::new(
//!     PathBuf::from("/photos/sunset.jpg"),
//!     JobErrorCode::ChecksumMismatch,
//!     "the checksum of the destination differs",
//!     true,
//! );
//! assert!(error.continues());
//! assert_eq!(error.code().as_str(), "job.checksum_mismatch");
//! ```

use std::io;
use std::path::PathBuf;

/// The reason why a name cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameProblem {
/// The name contains a character that Windows forbids.
    ForbiddenCharacter,

/// The name is a device name that Windows reserves.
    ReservedDeviceName,

/// The name is empty.
    Empty,

/// The name contains a separator and would name a directory.
    PathSeparator,
}

impl NameProblem {
/// The stable code of the problem.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::error::NameProblem;
///
/// assert_eq!(NameProblem::ForbiddenCharacter.as_str(), "path.reserved");
/// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            NameProblem::ForbiddenCharacter => "path.reserved",
            NameProblem::ReservedDeviceName => "path.reserved",
            NameProblem::Empty => "path.reserved",
            NameProblem::PathSeparator => "path.reserved",
        }
    }
}

/// The failure of a single file operation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OpError {
/// The file system refused the operation.
    #[error("the file system refused the operation for {path}: {source}")]
    Io {
/// The path the operation was working on.
        path: PathBuf,
/// The error the operating system reported.
        #[source]
        source: io::Error,
    },

/// The path does not exist.
    #[error("{path} does not exist")]
    NotFound {
/// The path that does not exist.
        path: PathBuf,
    },

/// The path exists already.
    #[error("{path} exists already")]
    AlreadyExists {
/// The path that exists.
        path: PathBuf,
    },

/// The name cannot be used on the target platform.
    #[error("the name {name} cannot be used: {problem}")]
    InvalidName {
/// The name that was refused.
        name: String,
/// The reason why the name was refused.
        problem: NameProblem,
    },

/// The plan of a synchronisation was not confirmed.
    #[error("the synchronisation plan was not confirmed")]
    NotConfirmed,

/// The source and the destination differ after a verified copy.
    #[error("the checksum of {path} differs from the checksum of the source")]
    ChecksumMismatch {
/// The path of the destination that was checked.
        path: PathBuf,
    },

/// The mode does not belong to the operation.
    #[error("the mode {mode} does not belong to this operation")]
    UnsupportedMode {
/// The name of the mode.
        mode: String,
    },

/// The operation was cancelled.
    #[error("the operation was cancelled")]
    Cancelled,
}

impl OpError {
/// The stable code of the failure.
///
/// WHY: the code decides which message the user interface shows and
/// whether the job may continue.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::error::{JobErrorCode, OpError};
/// use std::path::PathBuf;
///
/// let error = OpError::NotFound { path: PathBuf::from("a.txt") };
/// assert_eq!(error.code(), JobErrorCode::NotFound);
/// ```
    pub fn code(&self) -> JobErrorCode {
        match self {
            OpError::Io { source, .. } => {
                if source.kind() == io::ErrorKind::PermissionDenied {
                    JobErrorCode::IoPermission
                } else {
                    JobErrorCode::IoWrite
                }
            }
            OpError::NotFound { .. } => JobErrorCode::NotFound,
            OpError::AlreadyExists { .. } => JobErrorCode::AlreadyExists,
            OpError::InvalidName { problem, .. } => match problem {
                NameProblem::ForbiddenCharacter
                | NameProblem::ReservedDeviceName
                | NameProblem::Empty
                | NameProblem::PathSeparator => JobErrorCode::NameInvalid,
            },
            OpError::NotConfirmed => JobErrorCode::NotConfirmed,
            OpError::ChecksumMismatch { .. } => JobErrorCode::ChecksumMismatch,
            OpError::UnsupportedMode { .. } => JobErrorCode::ModeUnsupported,
            OpError::Cancelled => JobErrorCode::Cancelled,
        }
    }
}

/// The stable code of a failure that a job collected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobErrorCode {
/// A file or a directory could not be read.
    IoRead,

/// A file could not be written.
    IoWrite,

/// The operating system refused the access.
    IoPermission,

/// The source and the destination differ after a verified copy.
    ChecksumMismatch,

/// The name of a file is not usable.
    NameInvalid,

/// The operation was cancelled.
    Cancelled,

/// The plan of a synchronisation was not confirmed.
    NotConfirmed,

/// The path does not exist.
    NotFound,

/// The path exists already.
    AlreadyExists,

/// The mode does not belong to the operation.
    ModeUnsupported,
}

impl JobErrorCode {
/// The name of the code.
///
/// WHY: the codes of ADR-0003 are the contract with the user interface.
/// A code that this crate adds keeps the same shape, so that the
/// interface can map every code with one table.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::error::JobErrorCode;
///
/// assert_eq!(JobErrorCode::IoRead.as_str(), "io.read");
/// assert_eq!(JobErrorCode::ChecksumMismatch.as_str(), "job.checksum_mismatch");
/// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            JobErrorCode::IoRead => "io.read",
            JobErrorCode::IoWrite => "io.write",
            JobErrorCode::IoPermission => "io.permission",
            JobErrorCode::ChecksumMismatch => "job.checksum_mismatch",
            JobErrorCode::NameInvalid => "path.reserved",
            JobErrorCode::Cancelled => "job.cancelled",
            JobErrorCode::NotConfirmed => "job.not_confirmed",
            JobErrorCode::NotFound => "io.read",
            JobErrorCode::AlreadyExists => "job.already_exists",
            JobErrorCode::ModeUnsupported => "job.unsupported_mode",
        }
    }

/// The number of the code.
///
/// WHY: the number is what a log record carries. It stays stable, even
/// when a new code is added next to it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::error::JobErrorCode;
///
/// assert_eq!(JobErrorCode::ChecksumMismatch.number(), 1301);
/// ```
    pub fn number(&self) -> u16 {
        match self {
            JobErrorCode::IoRead => 1001,
            JobErrorCode::IoWrite => 1002,
            JobErrorCode::IoPermission => 1003,
            JobErrorCode::ChecksumMismatch => 1301,
            JobErrorCode::NameInvalid => 1103,
            JobErrorCode::Cancelled => 1302,
            JobErrorCode::NotConfirmed => 1303,
            JobErrorCode::NotFound => 1004,
            JobErrorCode::AlreadyExists => 1304,
            JobErrorCode::ModeUnsupported => 1305,
        }
    }
}

/// A failure that a job collected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobError {
/// The item the operation was working on.
    pub item: PathBuf,

/// The stable code of the failure.
    pub code: JobErrorCode,

/// The message for the log.
///
/// WHY: the message names the cause and never the user. The user
/// interface does not show it — it shows the message of the translation
/// catalogue that belongs to the code.
    pub message: String,

/// Whether the job continues with the next item.
    pub continues: bool,
}

impl JobError {
/// Builds a collected failure.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::error::{JobError, JobErrorCode};
/// use std::path::PathBuf;
///
/// let error = JobError::new(
///     PathBuf::from("sunset.jpg"),
///     JobErrorCode::IoWrite,
///     "the file system refused the operation",
///     true,
/// );
/// assert_eq!(error.item(), PathBuf::from("sunset.jpg"));
/// ```
    pub fn new(
        item: PathBuf,
        code: JobErrorCode,
        message: impl Into<String>,
        continues: bool,
    ) -> Self {
        Self {
            item,
            code,
            message: message.into(),
            continues,
        }
    }

/// The item the operation was working on.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::error::{JobError, JobErrorCode};
/// use std::path::PathBuf;
///
/// let error = JobError::new(PathBuf::from("a.txt"), JobErrorCode::NotFound, "missing", true);
/// assert_eq!(error.item(), &PathBuf::from("a.txt"));
/// ```
    pub fn item(&self) -> &std::path::Path {
        &self.item
    }

/// The stable code of the failure.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::error::{JobError, JobErrorCode};
/// use std::path::PathBuf;
///
/// let error = JobError::new(PathBuf::from("a.txt"), JobErrorCode::NotFound, "missing", true);
/// assert_eq!(error.code(), JobErrorCode::NotFound);
/// ```
    pub fn code(&self) -> JobErrorCode {
        self.code
    }

/// Whether the job continues with the next item.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::error::{JobError, JobErrorCode};
/// use std::path::PathBuf;
///
/// let error = JobError::new(PathBuf::from("a.txt"), JobErrorCode::NotFound, "missing", true);
/// assert!(error.continues());
/// ```
    pub fn continues(&self) -> bool {
        self.continues
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn a_collected_failure_names_the_item_and_the_code() {
        // act
        let error = JobError::new(
            PathBuf::from("photos/sunset.jpg"),
            JobErrorCode::ChecksumMismatch,
            "the checksums differ",
            true,
        );

        // assert
        assert_eq!(error.item(), Path::new("photos/sunset.jpg"));
        assert_eq!(error.code(), JobErrorCode::ChecksumMismatch);
        assert!(error.continues());
        assert_eq!(error.message, "the checksums differ");
    }

    #[test]
    fn a_refused_access_carries_the_permission_code() {
        // act
        let error = OpError::Io {
            path: PathBuf::from("a.txt"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        };

        // assert
        assert_eq!(error.code(), JobErrorCode::IoPermission);
    }

    #[test]
    fn the_failures_of_the_operations_carry_their_own_codes() {
        // act
        let not_found = OpError::NotFound {
            path: PathBuf::from("a.txt"),
        };
        let exists = OpError::AlreadyExists {
            path: PathBuf::from("a.txt"),
        };
        let not_confirmed = OpError::NotConfirmed;
        let cancelled = OpError::Cancelled;

        // assert
        assert_eq!(not_found.code(), JobErrorCode::NotFound);
        assert_eq!(exists.code(), JobErrorCode::AlreadyExists);
        assert_eq!(not_confirmed.code(), JobErrorCode::NotConfirmed);
        assert_eq!(cancelled.code(), JobErrorCode::Cancelled);
    }

    #[test]
    fn an_invalid_name_carries_the_reserved_code() {
        // act
        let error = OpError::InvalidName {
            name: "NUL".to_string(),
            problem: NameProblem::ReservedDeviceName,
        };

        // assert
        assert_eq!(error.code(), JobErrorCode::NameInvalid);
        assert_eq!(error.code().as_str(), "path.reserved");
    }

    #[test]
    fn every_job_code_has_a_name_and_a_number() {
        // arrange
        let codes = [
            JobErrorCode::IoRead,
            JobErrorCode::IoWrite,
            JobErrorCode::IoPermission,
            JobErrorCode::ChecksumMismatch,
            JobErrorCode::NameInvalid,
            JobErrorCode::Cancelled,
            JobErrorCode::NotConfirmed,
            JobErrorCode::NotFound,
            JobErrorCode::AlreadyExists,
        ];

        // act
        let names: Vec<&str> = codes.iter().map(JobErrorCode::as_str).collect();

        // assert
        for name in names {
            assert!(!name.is_empty());
        }
        for (index, code) in codes.iter().enumerate() {
            let number = code.number();
            assert!(
                !codes[..index].iter().any(|other| other.number() == number),
                "the number {number} is used twice"
            );
        }
    }
}
