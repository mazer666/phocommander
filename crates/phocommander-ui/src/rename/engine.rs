//! The seam to the engine that forms the new names.
//!
//! WHAT: the trait that the interface asks for the new names
//!       ([`RenameEngine`]), the question ([`RenameRequest`]), the facts of a
//!       file ([`RenameFile`]), the answer ([`RenamePreview`]), a row of the
//!       answer ([`PreviewRow`]), the reason why a row failed
//!       ([`PreviewProblem`], [`ProblemKind`]), a conflict ([`Conflict`]) and
//!       the kinds of the conflicts ([`ConflictKind`]).
//! WHY: the dialog shows a live preview of every new name before the rename
//!       runs (FR-34), and the preview needs the engine that forms the
//!       names. The engine lives in the crate of the operations, and the
//!       interface crate may not depend on a sibling crate of the same level.
//!       The seam is therefore a trait, and the composition root hands the
//!       implementation over — the same arrangement as the texts of the
//!       catalogue and the formats of a locale.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::rename::{RenameEngine, RenamePreview, RenameRequest};
//!
//! # struct Engine;
//! # impl RenameEngine for Engine {
//! #     fn preview(&self, _request: &RenameRequest) -> RenamePreview {
//! #         RenamePreview::empty()
//! #     }
//! # }
//! let engine = Engine;
//! let request = RenameRequest::new();
//! assert!(engine.preview(&request).rows.is_empty());
//! ```

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::i18n::TextKey;
use crate::rename::rule::RenameRule;

/// The engine that forms the new names of a selection.
///
/// WHY: the interface asks the engine for the new names and for the
///       conflicts of the rename, and it never asks how the engine works.
///       The implementation lives in the composition root, because the engine
///       of the product belongs to the crate of the operations.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::rename::{RenameEngine, RenamePreview, RenameRequest};
///
/// # struct Engine;
/// # impl RenameEngine for Engine {
/// #     fn preview(&self, _request: &RenameRequest) -> RenamePreview {
/// #         RenamePreview::empty()
/// #     }
/// # }
///
/// fn names(engine: &dyn RenameEngine, request: &RenameRequest) -> usize {
///     engine.preview(request).rows.len()
/// }
///
/// assert_eq!(names(&Engine, &RenameRequest::new()), 0);
/// ```
pub trait RenameEngine {
    /// The new names of the files of the selection and the conflicts of the
    /// rename.
    ///
    /// WHY: the answer carries the conflicts of the rename as well, because
    ///       the user has to see them before the rename runs (FR-35). The
    ///       engine never renames a file here — it only says what the names
    ///       would be.
    fn preview(&self, request: &RenameRequest) -> RenamePreview;
}

/// The question that the dialog asks the engine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenameRequest {
    /// The rule that forms the names.
    pub rule: RenameRule,

    /// The files of the selection.
    pub files: Vec<RenameFile>,
}

impl RenameRequest {
    /// Builds an empty question.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenameRequest;
    ///
    /// let request = RenameRequest::new();
    /// assert!(request.files.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a question with a rule and a selection.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{RenameFile, RenameRequest, RenameRule};
    ///
    /// let request = RenameRequest::with_files(
    ///     RenameRule::new("[name].[ext]"),
    ///     vec![RenameFile::new("/photos/sunset.jpg")],
    /// );
    /// assert_eq!(request.files.len(), 1);
    /// ```
    pub fn with_files(rule: RenameRule, files: Vec<RenameFile>) -> Self {
        Self { rule, files }
    }
}

/// The facts of a file of the selection.
///
/// WHY: the formation reads no file. The size, the time of the last change
///       and the checksum are handed to it by the caller, so that the same
///       rule always yields the same names (FR-33) and so that the reading of
///       a large file stays out of the loop that draws the preview.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenameFile {
    /// The path of the file that is renamed.
    pub path: PathBuf,

    /// The size of the file in bytes.
    pub size: Option<u64>,

    /// The time of the last change of the file.
    pub modified: Option<SystemTime>,

    /// The checksum of the content of the file.
    pub checksum: Option<u64>,
}

impl RenameFile {
    /// Builds the facts of a file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenameFile;
    ///
    /// let file = RenameFile::new("/photos/sunset.jpg");
    /// assert_eq!(file.path.to_string_lossy(), "/photos/sunset.jpg");
    /// assert!(file.size.is_none());
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            size: None,
            modified: None,
            checksum: None,
        }
    }

    /// Adds the size of the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenameFile;
    ///
    /// let file = RenameFile::new("/photos/sunset.jpg").with_size(4096);
    /// assert_eq!(file.size, Some(4096));
    /// ```
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Adds the time of the last change of the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenameFile;
    /// use std::time::{Duration, SystemTime};
    ///
    /// let time = SystemTime::UNIX_EPOCH + Duration::from_secs(60);
    /// let file = RenameFile::new("/photos/sunset.jpg").with_modified(time);
    /// assert_eq!(file.modified, Some(time));
    /// ```
    pub fn with_modified(mut self, modified: SystemTime) -> Self {
        self.modified = Some(modified);
        self
    }

    /// Adds the checksum of the content of the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenameFile;
    ///
    /// let file = RenameFile::new("/photos/sunset.jpg").with_checksum(7);
    /// assert_eq!(file.checksum, Some(7));
    /// ```
    pub fn with_checksum(mut self, checksum: u64) -> Self {
        self.checksum = Some(checksum);
        self
    }
}

/// The answer of the engine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenamePreview {
    /// One row per file of the selection.
    pub rows: Vec<PreviewRow>,

    /// The conflicts of the rename.
    pub conflicts: Vec<Conflict>,

    /// The reason why no name could be formed at all.
    ///
    /// WHY: a rule without a format and a broken regular expression stop the
    ///       whole formation. The rows then carry no name, and the dialog
    ///       shows the reason above the list.
    pub problem: Option<PreviewProblem>,
}

impl RenamePreview {
    /// Builds an empty answer.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenamePreview;
    ///
    /// let preview = RenamePreview::empty();
    /// assert!(preview.rows.is_empty());
    /// assert!(preview.conflicts.is_empty());
    /// ```
    pub fn empty() -> Self {
        Self::default()
    }

    /// Whether the rename may run.
    ///
    /// WHY: the rename of a selection is atomic — either every file is
    ///       renamed or none of them is. A selection with a file that could
    ///       not be named and a selection with a conflict are therefore both
    ///       refused as a whole.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenamePreview;
    ///
    /// assert!(!RenamePreview::empty().may_rename());
    /// ```
    pub fn may_rename(&self) -> bool {
        if self.problem.is_some() || !self.conflicts.is_empty() {
            return false;
        }
        self.rows
            .iter()
            .all(|row| row.name.is_some() && row.problem.is_none())
    }
}

/// One row of the preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewRow {
    /// The path the file carries now.
    pub source: PathBuf,

    /// The path the file would receive.
    pub target: Option<PathBuf>,

    /// The new name of the file.
    pub name: Option<String>,

    /// The reason why the file could not be named.
    pub problem: Option<PreviewProblem>,
}

impl PreviewRow {
    /// Builds a row of a file that received a new name.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::PreviewRow;
    ///
    /// let row = PreviewRow::named("/photos/a.jpg", "a_001.jpg");
    /// assert_eq!(row.name.as_deref(), Some("a_001.jpg"));
    /// ```
    pub fn named(source: impl Into<PathBuf>, name: impl Into<String>) -> Self {
        let source: PathBuf = source.into();
        let name = name.into();
        let target = match source.parent() {
            Some(parent) => parent.join(name.as_str()),
            None => PathBuf::from(name.as_str()),
        };
        Self {
            source,
            target: Some(target),
            name: Some(name),
            problem: None,
        }
    }

    /// Builds a row of a file that could not be named.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{PreviewProblem, PreviewRow, ProblemKind};
    ///
    /// let row = PreviewRow::failed("/photos/a.jpg", PreviewProblem::empty_format());
    /// assert!(row.name.is_none());
    /// assert_eq!(row.problem.map(|problem| problem.kind), Some(ProblemKind::EmptyFormat));
    /// ```
    pub fn failed(source: impl Into<PathBuf>, problem: PreviewProblem) -> Self {
        Self {
            source: source.into(),
            target: None,
            name: None,
            problem: Some(problem),
        }
    }

    /// The name of the file without its directory.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::PreviewRow;
    ///
    /// let row = PreviewRow::named("/photos/a.jpg", "a_001.jpg");
    /// assert_eq!(row.old_name(), "a.jpg");
    /// ```
    pub fn old_name(&self) -> String {
        file_name_of(&self.source)
    }
}

/// The reason why a name could not be formed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewProblem {
    /// The kind of the reason.
    pub kind: ProblemKind,

    /// The value that the reason names — a placeholder, a pattern or a name.
    ///
    /// WHY: the value is data and not a text of the interface. The message
    ///       that explains it comes from the catalogue, and it takes the value
    ///       as an argument (FR-05).
    pub detail: String,
}

impl PreviewProblem {
    /// Builds a reason of a kind without a value.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::PreviewProblem;
    ///
    /// let problem = PreviewProblem::empty_format();
    /// assert!(problem.detail.is_empty());
    /// ```
    pub fn empty_format() -> Self {
        Self {
            kind: ProblemKind::EmptyFormat,
            detail: String::new(),
        }
    }

    /// Builds a reason that names a value.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{PreviewProblem, ProblemKind};
    ///
    /// let problem = PreviewProblem::of(ProblemKind::UnknownValue, "hash");
    /// assert_eq!(problem.detail, "hash");
    /// ```
    pub fn of(kind: ProblemKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    /// The key of the catalogue that carries the message of this reason.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{PreviewProblem, ProblemKind};
    /// use phocommander_ui::TextKey;
    ///
    /// let problem = PreviewProblem::empty_format();
    /// assert_eq!(problem.message_key(), TextKey::RenameProblemEmptyFormat);
    /// ```
    pub fn message_key(&self) -> TextKey {
        self.kind.message_key()
    }
}

/// The kinds of the reasons why a name could not be formed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemKind {
    /// The rule carries no format.
    EmptyFormat,

    /// The value of a placeholder is not known.
    UnknownValue,

    /// The pattern of a replacement rule is not a valid regular expression.
    InvalidPattern,

    /// The formed name cannot be used as a file name.
    InvalidName,
}

impl ProblemKind {
    /// The key of the catalogue that carries the message of this kind.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::ProblemKind;
    /// use phocommander_ui::TextKey;
    ///
    /// assert_eq!(
    ///     ProblemKind::InvalidPattern.message_key(),
    ///     TextKey::RenameProblemInvalidPattern
    /// );
    /// ```
    pub const fn message_key(&self) -> TextKey {
        match self {
            ProblemKind::EmptyFormat => TextKey::RenameProblemEmptyFormat,
            ProblemKind::UnknownValue => TextKey::RenameProblemUnknownValue,
            ProblemKind::InvalidPattern => TextKey::RenameProblemInvalidPattern,
            ProblemKind::InvalidName => TextKey::RenameProblemInvalidName,
        }
    }
}

/// A conflict that stops the rename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// The kind of the conflict.
    pub kind: ConflictKind,

    /// The path the conflict is about.
    pub path: PathBuf,
}

impl Conflict {
    /// Builds a conflict.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{Conflict, ConflictKind};
    ///
    /// let conflict = Conflict::new(ConflictKind::TargetExists, "/photos/b.jpg");
    /// assert_eq!(conflict.kind, ConflictKind::TargetExists);
    /// ```
    pub fn new(kind: ConflictKind, path: impl Into<PathBuf>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }

    /// The key of the catalogue that carries the message of this conflict.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{Conflict, ConflictKind};
    /// use phocommander_ui::TextKey;
    ///
    /// let conflict = Conflict::new(ConflictKind::DuplicateTarget, "/photos/b.jpg");
    /// assert_eq!(conflict.message_key(), TextKey::RenameConflictDuplicateTarget);
    /// ```
    pub const fn message_key(&self) -> TextKey {
        self.kind.message_key()
    }
}

/// The kinds of the conflicts of a rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConflictKind {
    /// The target of the rename is occupied by another file.
    TargetExists,

    /// Two files of the selection would receive the same name.
    DuplicateTarget,

    /// The file that is to be renamed does not exist.
    MissingSource,
}

impl ConflictKind {
    /// The name of the kind, as the operations write it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::ConflictKind;
    ///
    /// assert_eq!(ConflictKind::TargetExists.as_str(), "target-exists");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            ConflictKind::TargetExists => "target-exists",
            ConflictKind::DuplicateTarget => "duplicate-target",
            ConflictKind::MissingSource => "missing-source",
        }
    }

    /// The key of the catalogue that carries the message of this kind.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::ConflictKind;
    /// use phocommander_ui::TextKey;
    ///
    /// assert_eq!(
    ///     ConflictKind::MissingSource.message_key(),
    ///     TextKey::RenameConflictMissingSource
    /// );
    /// ```
    pub const fn message_key(&self) -> TextKey {
        match self {
            ConflictKind::TargetExists => TextKey::RenameConflictTargetExists,
            ConflictKind::DuplicateTarget => TextKey::RenameConflictDuplicateTarget,
            ConflictKind::MissingSource => TextKey::RenameConflictMissingSource,
        }
    }
}

/// One file of a multiple rename.
///
/// WHY: the interface names the pairs of the rename on its own, because the
///       type of the operations is not known to it. The composition root
///       translates the pair into the pair of the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePair {
    /// The path the file carries now.
    pub source: PathBuf,

    /// The path the file receives.
    pub target: PathBuf,
}

impl RenamePair {
    /// Builds a pair of a rename.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenamePair;
    ///
    /// let pair = RenamePair::new("/photos/a.jpg", "/photos/a_001.jpg");
    /// assert_eq!(pair.source.to_string_lossy(), "/photos/a.jpg");
    /// ```
    pub fn new(source: impl Into<PathBuf>, target: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }

    /// The name of the file without its directory.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenamePair;
    ///
    /// let pair = RenamePair::new("/photos/a.jpg", "/photos/a_001.jpg");
    /// assert_eq!(pair.target_name(), "a_001.jpg");
    /// ```
    pub fn target_name(&self) -> String {
        file_name_of(&self.target)
    }
}

/// The name of a path without its directory.
fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;
