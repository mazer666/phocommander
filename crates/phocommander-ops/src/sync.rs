//! The plan of a synchronisation and the execution of the plan.
//!
//! WHAT: the state of an entry in the comparison ([{SyncState}]), what the
//!       synchronisation does with it ([{SyncAction}]), one entry of the plan
//!       ([{SyncPlanEntry}]), the plan itself ([{SyncPlan}]), the
//!       confirmation of the user ([{Approval}]) and the execution of the
//!       plan ([{execute_sync}]).
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

use crate::copy::{CopyMode, copy_one_file, compare_trees};
use crate::error::{JobError, OpError};
use crate::port::EntryKind;
use crate::progress::JobContext;

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

/// One entry of a synchronisation plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPlanEntry {
/// The path of the entry, relative to the two directories.
    pub relative_path: PathBuf,

/// The kind of the entry.
    pub kind: EntryKind,

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

/// The checksum of the entry, if one was computed.
    pub checksum: Option<u64>,

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
///     action: SyncAction::Delete,
/// };
/// assert!(entry.is_deletion());
/// ```
    pub fn is_deletion(&self) -> bool {
        matches!(self.action, SyncAction::Delete)
    }
}

/// The plan of a synchronisation.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    let entries = compare_trees(fs, source, destination, mode)?;
    Ok(SyncPlan {
        mode,
        source: source.to_path_buf(),
        destination: destination.to_path_buf(),
        entries,
    })
}

/// Executes the plan of a synchronisation.
///
/// WHY: the plan is executed only when the user confirmed it. A plan that
/// was not confirmed changes nothing at all — not a byte and not a deletion.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::copy::CopyMode;
/// use phocommander_ops::sync::{Approval, build_plan, execute_sync};
/// use std::path::Path;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem, context: &mut phocommander_ops::progress::JobContext) -> Result<(), phocommander_ops::error::OpError> {
/// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncMirror)?;
/// let outcome = execute_sync(fs, &plan, Approval::Granted, context)?;
/// assert_eq!(outcome.errors.len(), 0);
/// # Ok(())
/// # }
/// ```
pub fn execute_sync(
    fs: &dyn crate::port::FileSystem,
    plan: &SyncPlan,
    approval: Approval,
    context: &mut JobContext,
) -> Result<crate::copy::CopyOutcome, OpError> {
    if approval == Approval::Withheld {
        return Err(OpError::NotConfirmed);
    }

    let mut outcome = crate::copy::CopyOutcome::default();
    let total_bytes: u64 = plan
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.action,
                SyncAction::Copy | SyncAction::Overwrite | SyncAction::CopyToSource
            )
        })
        .filter_map(|entry| entry.source_size.or(entry.destination_size))
        .sum();
    context.set_totals(plan.entries.len() as u64, total_bytes);

    // The deletions run from the deepest entry to the shallowest one, so that
    // a directory is removed after the files it contains.
    let mut order: Vec<&SyncPlanEntry> = plan.entries.iter().collect();
    order.sort_by_key(|entry| {
        (
            !entry.is_deletion(),
            std::cmp::Reverse(entry.relative_path.components().count()),
            entry.relative_path.clone(),
        )
    });

    for entry in order {
        match context.checkpoint() {
            Ok(_) => {}
            Err(error) => return Err(error),
        }

        let name = entry
            .relative_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.relative_path.to_string_lossy().to_string());
        context.begin_item(&name);

        let source_path = plan.source.join(&entry.relative_path);
        let destination_path = plan.destination.join(&entry.relative_path);

        let result = match entry.action {
            SyncAction::Keep => {
                context.finish_item();
                context.report();
                continue;
            }
            SyncAction::Delete => {
                if entry.kind.is_directory() {
                    fs.remove_dir(&destination_path)
                } else {
                    fs.remove_file(&destination_path)
                }
            }
            SyncAction::Copy | SyncAction::Overwrite => {
                if entry.kind.is_directory() {
                    fs.create_dir_all(&destination_path)
                } else {
                    if let Some(parent) = destination_path.parent() {
                        let _ = fs.create_dir_all(parent);
                    }
                    copy_one_file(
                        fs,
                        &source_path,
                        &destination_path,
                        CopyMode::Verified,
                        context,
                    )
                    .map(|_| ())
                }
            }
            SyncAction::CopyToSource => {
                if entry.kind.is_directory() {
                    fs.create_dir_all(&source_path)
                } else {
                    if let Some(parent) = source_path.parent() {
                        let _ = fs.create_dir_all(parent);
                    }
                    copy_one_file(
                        fs,
                        &destination_path,
                        &source_path,
                        CopyMode::Verified,
                        context,
                    )
                    .map(|_| ())
                }
            }
        };

        match result {
            Ok(()) => {
                outcome.files_copied += 1;
                context.finish_item();
            }
            Err(error) => {
                outcome
                    .errors
                    .push(JobError::new(source_path.clone(), error.code(), error.to_string(), true));
                context.finish_item();
            }
        }
        context.report();
    }

    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

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
                action: SyncAction::Delete,
            }],
        };

        // act
        let deletions = plan.deletions();

        // assert
        assert_eq!(deletions.len(), 1);
        assert_eq!(deletions[0].relative_path, PathBuf::from("old.txt"));
    }
}
