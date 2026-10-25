//! The data of a copy: the mode, the request, the outcome and the items of
//! a tree.
//!
//! WHAT: the mode of a copy ([`CopyMode`]), the request ([`CopyRequest`]),
//!       what a copy did ([`CopyOutcome`]) and one file of a tree
//!       ([`TreeItem`]).
//! WHY: the copy is a family of operations, and the difference between them
//!       is the difference between a fast copy and a copy that can be
//!       trusted (section 6.9 of the specification). The types live in their
//!       own file, so that the copy itself stays readable: the file that
//!       does the work carries the work and not the vocabulary.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::copy::{CopyMode, CopyRequest};
//! use std::path::PathBuf;
//!
//! // The default of a copy is the mode that verifies.
//! assert_eq!(CopyMode::default(), CopyMode::Verified);
//!
//! let request = CopyRequest::new(PathBuf::from("/photos"), PathBuf::from("/backup"));
//! assert_eq!(request.mode, CopyMode::Verified);
//! ```

use std::path::PathBuf;

use crate::edit::ConflictPolicy;
use crate::error::JobError;

use super::policy::CopyPolicy;

/// The mode of a copy.
///
/// WHY: the modes are the ones of the specification: quick, verified,
/// atomic, sync-mirror, sync-update, sync-two-way and sync-preview. The
/// default of a copy is {verified} and the default of a synchronisation is
/// 'sync-preview', so that the destructive case always requires an explicit
/// act.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CopyMode {
    /// Byte copy, no verification.
    Quick,

    /// Byte copy, then a checksum of source and destination compared.
    #[default]
    Verified,

    /// Write to a temporary file next to the destination, verify it, then
    /// rename it into place.
    Atomic,

    /// The destination becomes an exact copy of the source.
    SyncMirror,

    /// The destination is brought up to date with the source, nothing is
    /// deleted.
    SyncUpdate,

    /// The newer version wins in both directions.
    SyncTwoWay,

    /// No data is moved. The comparison produces a plan.
    SyncPreview,
}

impl CopyMode {
    /// The name of the mode, as the configuration file writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyMode;
    ///
    /// assert_eq!(CopyMode::SyncMirror.as_str(), "sync-mirror");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            CopyMode::Quick => "quick",
            CopyMode::Verified => "verified",
            CopyMode::Atomic => "atomic",
            CopyMode::SyncMirror => "sync-mirror",
            CopyMode::SyncUpdate => "sync-update",
            CopyMode::SyncTwoWay => "sync-two-way",
            CopyMode::SyncPreview => "sync-preview",
        }
    }

    /// Whether the mode belongs to a synchronisation.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyMode;
    ///
    /// assert!(CopyMode::SyncPreview.is_sync());
    /// assert!(!CopyMode::Verified.is_sync());
    /// ```
    pub fn is_sync(&self) -> bool {
        matches!(
            self,
            CopyMode::SyncMirror
                | CopyMode::SyncUpdate
                | CopyMode::SyncTwoWay
                | CopyMode::SyncPreview
        )
    }

    /// Whether the mode deletes anything.
    ///
    /// WHY: 'sync-mirror' is the only mode that deletes. The answer decides
    /// whether the operation may be started without a confirmation.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyMode;
    ///
    /// assert!(CopyMode::SyncMirror.deletes());
    /// assert!(!CopyMode::SyncUpdate.deletes());
    /// ```
    pub fn deletes(&self) -> bool {
        matches!(self, CopyMode::SyncMirror)
    }

    /// Whether the mode compares the source and the destination after the
    /// copy.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyMode;
    ///
    /// assert!(CopyMode::Verified.verifies());
    /// assert!(CopyMode::Atomic.verifies());
    /// assert!(!CopyMode::Quick.verifies());
    /// ```
    pub fn verifies(&self) -> bool {
        matches!(self, CopyMode::Verified | CopyMode::Atomic)
    }
}

/// The request of a copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyRequest {
    /// The directory or the file that is copied.
    pub source: PathBuf,

    /// The directory or the file that receives the copy.
    pub destination: PathBuf,

    /// The mode of the copy.
    pub mode: CopyMode,

    /// The policy for a file that exists already.
    pub policy: ConflictPolicy,

    /// The rules that decide how the copy treats the files it copies.
    pub rules: CopyPolicy,

    /// The items the copy is limited to, if it is limited to some.
    ///
    /// WHY: a job that repeats only its failed items (FR-22k) carries the
    ///       list of the items that failed. The copy then touches those
    ///       items and nothing else — the files that were copied already are
    ///       not copied a second time.
    pub restricted_items: Option<Vec<PathBuf>>,
}

impl CopyRequest {
    /// Builds a request with the default mode and the default policy.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::{CopyMode, CopyRequest};
    /// use std::path::PathBuf;
    ///
    /// let request = CopyRequest::new(PathBuf::from("/photos"), PathBuf::from("/backup"));
    /// assert_eq!(request.mode, CopyMode::Verified);
    /// ```
    pub fn new(source: impl Into<PathBuf>, destination: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            destination: destination.into(),
            mode: CopyMode::default(),
            policy: ConflictPolicy::default(),
            rules: CopyPolicy::strict(),
            restricted_items: None,
        }
    }

    /// Sets the mode of the copy.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::{CopyMode, CopyRequest};
    /// use std::path::PathBuf;
    ///
    /// let request = CopyRequest::new(PathBuf::from("/photos"), PathBuf::from("/backup"))
    ///     .with_mode(CopyMode::Quick);
    /// assert_eq!(request.mode, CopyMode::Quick);
    /// ```
    pub fn with_mode(mut self, mode: CopyMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the policy for a file that exists already.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyRequest;
    /// use phocommander_ops::edit::ConflictPolicy;
    /// use std::path::PathBuf;
    ///
    /// let request = CopyRequest::new(PathBuf::from("/photos"), PathBuf::from("/backup"))
    ///     .with_policy(ConflictPolicy::KeepBoth);
    /// assert_eq!(request.policy, ConflictPolicy::KeepBoth);
    /// ```
    pub fn with_policy(mut self, policy: ConflictPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Sets the rules that decide how the copy treats the files it copies.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::{CopyPolicy, CopyRequest};
    /// use std::path::PathBuf;
    ///
    /// let request = CopyRequest::new(PathBuf::from("/photos"), PathBuf::from("/backup"))
    ///     .with_rules(CopyPolicy::new(false, false));
    /// assert!(!request.rules.verifies_checksums());
    /// ```
    pub fn with_rules(mut self, rules: CopyPolicy) -> Self {
        self.rules = rules;
        self
    }

    /// Limits the copy to the given items.
    ///
    /// WHY: the request of a retry carries the items that failed. The copy
    ///       then touches those items and nothing else.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyRequest;
    /// use std::path::PathBuf;
    ///
    /// let request = CopyRequest::new("/photos", "/backup")
    ///     .with_restricted_items(vec![PathBuf::from("/photos/a.txt")]);
    /// assert_eq!(request.restricted_items.map(|items| items.len()), Some(1));
    /// ```
    pub fn with_restricted_items(mut self, items: Vec<PathBuf>) -> Self {
        self.restricted_items = Some(items);
        self
    }

    /// Limits the copy to the given items, if there are any.
    ///
    /// WHY: the executor builds the request of a job, and a job knows whether
    ///       it is limited to some items. The method takes the value as it
    ///       is, so that the executor does not have to ask twice.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::copy::CopyRequest;
    /// use std::path::PathBuf;
    ///
    /// let request = CopyRequest::new("/photos", "/backup")
    ///     .with_restriction(Some(vec![PathBuf::from("/photos/a.txt")]));
    /// assert_eq!(request.restricted_items.map(|items| items.len()), Some(1));
    /// ```
    pub fn with_restriction(mut self, items: Option<Vec<PathBuf>>) -> Self {
        self.restricted_items = items;
        self
    }
}

/// What a copy did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CopyOutcome {
    /// The number of files that were copied.
    pub files_copied: u64,

    /// The number of bytes that were copied.
    pub bytes_copied: u64,

    /// The number of files that were left alone.
    pub files_skipped: u64,

    /// The failures the copy collected.
    pub errors: Vec<JobError>,
}

/// One file of a tree, as the copy sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeItem {
    /// The path of the file, relative to the root of the tree.
    pub relative_path: PathBuf,

    /// The path of the file.
    pub path: PathBuf,

    /// The size of the file in bytes.
    pub size: u64,

    /// The time of the last change of the file.
    pub modified: Option<std::time::SystemTime>,

    /// Whether the item is a directory.
    pub is_directory: bool,

    /// What kind of entry the item is.
    ///
    /// WHY: a symbolic link is not a file. The copy has to know the
    /// difference, because a link is copied as a link unless the rules say
    /// that it may be dereferenced.
    pub kind: crate::port::EntryKind,
}
