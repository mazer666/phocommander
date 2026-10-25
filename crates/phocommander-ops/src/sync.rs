//! The plan of a synchronisation and the confirmation of the user.
//!
//! WHAT: the state of an entry in the comparison ([{SyncState}]), what the
//!       synchronisation does with it ([{SyncAction}]), one entry of the plan
//!       ([{SyncPlanEntry}]), the plan itself ([{SyncPlan}]), the
//!       confirmation of the user ([{Approval}]) and the building of the plan
//!       ([{build_plan}]).
//! WHY: the plan is the document the user confirms, and the same document
//!       that the acceptance of a synchronisation refers to. No data moves
//!       before the plan is confirmed, and 'sync-mirror' — the only mode that
//!       deletes — puts every deletion into the plan before it runs. The
//!       confirmation is therefore an argument of the operation and not a
//!       remark in the documentation.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::sync::{Approval, SyncPlan, build_plan};
//! use phocommander_ops::copy::CopyMode;
//! use std::path::Path;
//!
//! # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
//! let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncMirror)?;
//! println!("{} deletions", plan.deletions().len());
//! # Ok(())
//! # }
//! ```
//! BOUNDARY: this module works through the [{crate::port::FileSystem}] of the
//!       operations. It never touches a file system on its own.

use std::path::{Path, PathBuf};

use crate::compare::ComparisonDepth;
use crate::copy::CopyMode;
use crate::error::OpError;

pub mod document;
pub mod execute;

pub use document::{FNV_OFFSET, FNV_PRIME, write_document};
pub use execute::execute_sync;

/// The state of an entry in the comparison of two directories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncState {
    /// The entry exists on both sides and is the same.
    Identical,

    /// The entry exists on both sides and differs.
    Different,

    /// The entry exists in the source and not in the destination.
    MissingInDestination,

    /// The entry exists in the destination and not in the source.
    ExtraInDestination,
}

impl SyncState {
    /// The name of the state.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::SyncState;
    ///
    /// assert_eq!(SyncState::ExtraInDestination.as_str(), "extra-in-destination");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncState::Identical => "identical",
            SyncState::Different => "different",
            SyncState::MissingInDestination => "missing-in-destination",
            SyncState::ExtraInDestination => "extra-in-destination",
        }
    }
}

/// What the synchronisation does with an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncAction {
    /// The entry is copied to the destination.
    Copy,

    /// The entry in the destination is overwritten by the source.
    Overwrite,

    /// The entry is deleted in the destination.
    Delete,

    /// The entry is left alone.
    Keep,

    /// The entry in the source is overwritten by the destination.
    CopyToSource,
}

impl SyncAction {
    /// The name of the action.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::SyncAction;
    ///
    /// assert_eq!(SyncAction::Delete.as_str(), "delete");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncAction::Copy => "copy",
            SyncAction::Overwrite => "overwrite",
            SyncAction::Delete => "delete",
            SyncAction::Keep => "keep",
            SyncAction::CopyToSource => "copy-to-source",
        }
    }

    /// Whether the action moves data.
    ///
    /// WHY: the plan names the copies and the deletions apart, because the
    ///       deletions are the part that the user has to see before the
    ///       operation runs.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::SyncAction;
    ///
    /// assert!(SyncAction::Copy.moves_data());
    /// assert!(!SyncAction::Keep.moves_data());
    /// ```
    pub fn moves_data(&self) -> bool {
        !matches!(self, SyncAction::Keep)
    }
}

/// One entry of a synchronisation plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPlanEntry {
    /// The path of the entry, relative to the two directories.
    pub relative_path: PathBuf,

    /// The kind of the entry.
    pub kind: crate::port::EntryKind,

    /// The state of the entry in the comparison.
    pub state: SyncState,

    /// The size of the entry in the source.
    pub source_size: Option<u64>,

    /// The size of the entry in the destination.
    pub destination_size: Option<u64>,

    /// The time of the last change in the source.
    pub source_modified: Option<std::time::SystemTime>,

    /// The time of the last change in the destination.
    pub destination_modified: Option<std::time::SystemTime>,

    /// The checksum of the entry in the source, if one was computed.
    pub checksum: Option<u64>,

    /// The checksum of the entry in the destination, if one was computed.
    pub destination_checksum: Option<u64>,

    /// What the synchronisation does with the entry.
    pub action: SyncAction,
}

impl SyncPlanEntry {
    /// Whether the entry is deleted by the plan.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::{SyncAction, SyncPlanEntry, SyncState};
    /// use phocommander_ops::port::EntryKind;
    /// use std::path::PathBuf;
    ///
    /// let entry = SyncPlanEntry {
    ///     relative_path: PathBuf::from("a.txt"),
    ///     kind: EntryKind::File,
    ///     state: SyncState::ExtraInDestination,
    ///     source_size: None,
    ///     destination_size: Some(1),
    ///     source_modified: None,
    ///     destination_modified: None,
    ///     checksum: None,
    ///     destination_checksum: None,
    ///     action: SyncAction::Delete,
    /// };
    /// assert!(entry.is_deletion());
    /// ```
    pub fn is_deletion(&self) -> bool {
        matches!(self.action, SyncAction::Delete)
    }
}

/// The plan of a synchronisation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncPlan {
    /// The mode the plan was built for.
    pub mode: CopyMode,

    /// The directory that is the reference of the synchronisation.
    pub source: PathBuf,

    /// The directory that is brought into the state of the reference.
    pub destination: PathBuf,

    /// The entries of the plan.
    pub entries: Vec<SyncPlanEntry>,
}

impl SyncPlan {
    /// The entries of the plan.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::{SyncPlan, build_plan};
    /// use phocommander_ops::copy::CopyMode;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
    /// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncPreview)?;
    /// assert_eq!(plan.entries().len(), plan.entries.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn entries(&self) -> &[SyncPlanEntry] {
        &self.entries
    }

    /// The entries the plan deletes.
    ///
    /// WHY: the deletions are the dangerous part of a synchronisation, and
    /// they have to be visible before the operation runs.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::{SyncPlan, build_plan};
    /// use phocommander_ops::copy::CopyMode;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
    /// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncMirror)?;
    /// for entry in plan.deletions() {
    ///     println!("{}", entry.relative_path.display());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn deletions(&self) -> Vec<&SyncPlanEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.is_deletion())
            .collect()
    }

    /// The entries the plan copies.
    ///
    /// WHY: the plan tells the copies and the deletions apart, so that the
    ///       user can see what the operation moves and what it removes.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::{SyncPlan, build_plan};
    /// use phocommander_ops::copy::CopyMode;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
    /// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncUpdate)?;
    /// assert_eq!(plan.copies().len(), plan.entries().len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn copies(&self) -> Vec<&SyncPlanEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.action.moves_data() && !entry.is_deletion())
            .collect()
    }

    /// The document of the plan.
    ///
    /// WHY: the plan has to be quotable in an acceptance, so that the
    ///       acceptance can name the plan it refers to. The document is a
    ///       stable text: the same plan always produces the same document.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::{SyncPlan, build_plan};
    /// use phocommander_ops::copy::CopyMode;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
    /// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncPreview)?;
    /// assert!(plan.to_document().contains("phocommander-sync-plan"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_document(&self) -> String {
        document::write_document(self)
    }

    /// The fingerprint of the plan.
    ///
    /// WHY: the fingerprint names the plan in one line. Two plans with the
    ///       same fingerprint are the same plan, and the acceptance can cite
    ///       the fingerprint instead of the whole document.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::sync::{SyncPlan, build_plan};
    /// use phocommander_ops::copy::CopyMode;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
    /// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncPreview)?;
    /// assert_eq!(plan.fingerprint(), plan.fingerprint());
    /// # Ok(())
    /// # }
    /// ```
    pub fn fingerprint(&self) -> u64 {
        document::fingerprint(self)
    }
}

/// Whether the user confirmed the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Approval {
    /// The user confirmed the plan and the operation may run.
    Granted,

    /// The user did not confirm the plan. Nothing is moved and nothing is
    /// deleted.
    Withheld,
}

/// Compares two directories and builds the plan of the synchronisation.
///
/// WHY: the plan is built before anything is moved. The user sees the state
/// of every entry and confirms the plan; only then does the operation run.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::copy::CopyMode;
/// use phocommander_ops::sync::build_plan;
/// use std::path::Path;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncPreview)?;
/// assert!(plan.deletions().is_empty());
/// # Ok(())
/// # }
/// ```
pub fn build_plan(
    fs: &dyn crate::port::FileSystem,
    source: &Path,
    destination: &Path,
    mode: CopyMode,
) -> Result<SyncPlan, OpError> {
    build_plan_with(fs, source, destination, mode, ComparisonDepth::Metadata)
}

/// Compares two directories and builds the plan of the synchronisation.
///
/// WHY: the depth decides whether the content of the entries is compared as
///       well. The plan of a synchronisation is built with the depth of the
///       metadata, because a synchronisation copies everything that differs
///       anyway; the plan of a *comparison* (FR-79) is built with the depth
///       of the content, because the user asks whether the two directories
///       hold the same files.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::compare::ComparisonDepth;
/// use phocommander_ops::copy::CopyMode;
/// use phocommander_ops::sync::build_plan_with;
/// use std::path::Path;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let plan = build_plan_with(
///     fs,
///     Path::new("/photos"),
///     Path::new("/backup"),
///     CopyMode::SyncPreview,
///     ComparisonDepth::Content,
/// )?;
/// assert_eq!(plan.entries().len(), plan.entries.len());
/// # Ok(())
/// # }
/// ```
pub fn build_plan_with(
    fs: &dyn crate::port::FileSystem,
    source: &Path,
    destination: &Path,
    mode: CopyMode,
    depth: ComparisonDepth,
) -> Result<SyncPlan, OpError> {
    crate::compare::compare_directories(fs, source, destination, mode, depth)
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::port::EntryKind;

    #[test]
    fn the_states_have_the_names_the_plan_shows() {
        // act, assert
        assert_eq!(SyncState::Identical.as_str(), "identical");
        assert_eq!(SyncState::Different.as_str(), "different");
        assert_eq!(
            SyncState::MissingInDestination.as_str(),
            "missing-in-destination"
        );
        assert_eq!(
            SyncState::ExtraInDestination.as_str(),
            "extra-in-destination"
        );
    }

    #[test]
    fn a_plan_knows_which_entries_it_deletes() {
        // arrange
        let plan = SyncPlan {
            mode: CopyMode::SyncMirror,
            source: PathBuf::from("/photos"),
            destination: PathBuf::from("/backup"),
            entries: vec![SyncPlanEntry {
                relative_path: PathBuf::from("old.txt"),
                kind: EntryKind::File,
                state: SyncState::ExtraInDestination,
                source_size: None,
                destination_size: Some(3),
                source_modified: None,
                destination_modified: None,
                checksum: None,
                destination_checksum: None,
                action: SyncAction::Delete,
            }],
        };

        // act
        let deletions = plan.deletions();

        // assert
        assert_eq!(deletions.len(), 1);
        assert_eq!(deletions[0].relative_path, PathBuf::from("old.txt"));
        assert!(plan.copies().is_empty());
    }
}
