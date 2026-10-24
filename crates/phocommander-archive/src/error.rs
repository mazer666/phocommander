//! The error type of the archive crate.
//!
//! WHAT: the failures that the extraction and the container adapters report.
//! WHY: every failure of this workspace travels as a value. The error has to
//!       name the entry that failed and the reason, because the user
//!       interface shows the reason and the log records the code. The codes
//!       are the ones of ADR-0003, so that a test can cite `path.traversal`
//!       instead of quoting an error message.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::{ArchiveError, RefusalReason};
//!
//! let error = ArchiveError::Refused {
//!     name: "../escape.txt".to_string(),
//!     reason: RefusalReason::OutsideDestination,
//! };
//! assert_eq!(error.code(), "path.traversal");
//! ```

use std::io;
use std::path::{Path, PathBuf};

use crate::entry_name::EntryNameError;

/// A failure of the archive crate.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    /// The archive or the destination could not be read or written.
    #[error("the operation on {path} failed: {source}")]
    Io {
        /// The path the operation was working on.
        path: PathBuf,
        /// The error the operating system reported.
        #[source]
        source: io::Error,
    },

    /// The container could not be parsed.
    #[error("the archive could not be read: {reason}")]
    Format {
        /// What the parser reported.
        reason: String,
    },

    /// An entry of the archive was refused.
    #[error("the archive entry {name:?} was refused: {reason}")]
    Refused {
        /// The name of the entry that was refused.
        name: String,
        /// The reason of the refusal.
        reason: RefusalReason,
    },

    /// A limit of the extraction was exceeded.
    #[error("the limit for the {kind} was exceeded: {actual} is more than {limit}")]
    LimitExceeded {
        /// The limit that was exceeded.
        kind: LimitKind,
        /// The value that exceeded the limit.
        actual: u64,
        /// The limit itself.
        limit: u64,
    },
}

impl ArchiveError {
    /// Returns the code of the failure, as ADR-0003 names it.
    ///
    /// WHY: the code is the contract between this crate and the user
    ///       interface. The interface maps the code to a translated message;
    ///       it never parses the text of the error.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::{ArchiveError, LimitKind};
    ///
    /// let error = ArchiveError::LimitExceeded {
    ///     kind: LimitKind::TotalBytes,
    ///     actual: 10,
    ///     limit: 5,
    /// };
    /// assert_eq!(error.code(), "limit.too_large");
    /// ```
    pub fn code(&self) -> &'static str {
        match self {
            Self::Io { source, .. } => match source.kind() {
                io::ErrorKind::PermissionDenied => "io.permission",
                _ => "io.read",
            },
            Self::Format { .. } => "archive.format",
            Self::Refused { reason, .. } => reason.code(),
            Self::LimitExceeded { kind, .. } => kind.code(),
        }
    }

    /// Builds the error for an input or output failure.
    ///
    /// WHAT: wraps the error of the operating system together with the
    ///       thing the operation was working on.
    /// WHY: the adapters of this crate open a file, read an archive or
    ///       write an entry, and in every case the user wants to know
    ///       *which* file failed. The constructor keeps that path next to
    ///       the system error, so that no call site can lose it.
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::ArchiveError;
    /// use std::path::Path;
    ///
    /// let failure = ArchiveError::io(Path::new("/tmp/archive.zip"), std::io::Error::other("no"));
    /// assert_eq!(failure.code(), "io.read");
    /// ```
    pub fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

/// The limit that an extraction exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitKind {
    /// The total number of bytes that the extraction may write.
    TotalBytes,

    /// The number of entries that the extraction may write.
    EntryCount,

    /// The number of bytes that a single entry may have.
    SingleEntryBytes,
}

impl LimitKind {
    /// Returns the code of the limit, as ADR-0003 names it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::LimitKind;
    ///
    /// assert_eq!(LimitKind::EntryCount.code(), "limit.entries");
    /// ```
    pub const fn code(&self) -> &'static str {
        match self {
            Self::TotalBytes | Self::SingleEntryBytes => "limit.too_large",
            Self::EntryCount => "limit.entries",
        }
    }

    /// Returns the name of the limit, as it is shown in the message.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::LimitKind;
    ///
    /// assert_eq!(LimitKind::TotalBytes.label(), "total size");
    /// ```
    pub const fn label(&self) -> &'static str {
        match self {
            Self::TotalBytes => "total size",
            Self::EntryCount => "number of entries",
            Self::SingleEntryBytes => "size of a single entry",
        }
    }
}

impl std::fmt::Display for LimitKind {
    /// Writes the name of the limit, so that the limit can appear in a
    /// message without the caller naming it.
    ///
    /// WHY: the message of the error names the limit through this type.
    /// Implementing `Display` keeps that knowledge in one place instead of
    /// spreading a match over the call sites.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::LimitKind;
    ///
    /// assert_eq!(LimitKind::EntryCount.to_string(), "number of entries");
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.label())
    }
}

/// The reason why an entry of an archive was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RefusalReason {
    /// The name of the entry is not a name that may be written.
    #[error("the name of the entry is not valid: {0}")]
    Name(#[from] EntryNameError),

    /// The name of the entry could not be decoded without a loss.
    #[error("the name of the entry is not valid UTF-8 and would be written under a different name")]
    InvalidName,

    /// The target of the entry lies outside of the destination directory.
    #[error("the target of the entry lies outside of the destination directory")]
    OutsideDestination,

    /// The target of the entry exists as a symbolic link.
    #[error("the target of the entry exists as a symbolic link")]
    SymlinkTarget,

    /// The target of the entry exists as a directory.
    #[error("the target of the entry exists as a directory")]
    NotAFile,

    /// A component of the path of the entry exists as a symbolic link.
    #[error("a directory of the path of the entry exists as a symbolic link")]
    SymlinkInPath,

    /// The entry has no content although the container announced one.
    #[error("the entry has no content although the container announced one")]
    MissingContent,
}

impl RefusalReason {
    /// Returns the code of the refusal, as ADR-0003 names it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::RefusalReason;
    ///
    /// assert_eq!(RefusalReason::OutsideDestination.code(), "path.traversal");
    /// ```
    pub fn code(&self) -> &'static str {
        match self {
            Self::Name(EntryNameError::Absolute { .. }) => "path.absolute",
            Self::Name(EntryNameError::Traversal { .. }) => "path.traversal",
            Self::Name(EntryNameError::ReservedName { .. })
            | Self::Name(EntryNameError::AlternateDataStream { .. })
            | Self::Name(EntryNameError::ForbiddenCharacter { .. })
            | Self::Name(EntryNameError::ForbiddenEnding { .. }) => "path.reserved",
            Self::Name(_) => "path.invalid",
            Self::InvalidName => "path.invalid",
            Self::OutsideDestination | Self::SymlinkInPath => "path.traversal",
            Self::SymlinkTarget => "path.traversal",
            Self::NotAFile | Self::MissingContent => "path.conflict",
        }
    }
}
