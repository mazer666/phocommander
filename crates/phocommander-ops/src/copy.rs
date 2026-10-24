//! The copy: the modes of the specification and the comparison of two trees.
//!
//! WHAT: the modes of a copy ([{CopyMode}]), the request of a copy
//!       ([{CopyRequest}]), what a copy did ([{CopyOutcome}]), the execution
//!       of a copy ([{execute_copy}]) and the comparison of two directory
//!       trees ([{compare_trees}]).
//! WHY: a copy is not one operation but a family of them, and the difference
//!       between them is the difference between a fast copy and a copy that
//!       can be trusted (section 6.9 of the specification). The user chooses
//!       the mode; the default is the safe one, because the failure of a copy
//!       is discovered far too late otherwise. The comparison behind every
//!       synchronisation is the same and produces the same plan: per entry
//!       the state is identical, different, missing-in-destination or
//!       extra-in-destination.
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
//! BOUNDARY: this module works through the [{crate::port::FileSystem}] of the
//!       operations. It never touches a file system on its own.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::edit::ConflictPolicy;
use crate::error::{JobError, OpError};
use crate::progress::JobContext;
use crate::sync::{SyncAction, SyncPlanEntry, SyncState};

/// The counter that makes the name of a temporary file unique.
static TEMPORARY_COUNTER: AtomicU64 = AtomicU64::new(0);

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
            CopyMode::SyncMirror | CopyMode::SyncUpdate | CopyMode::SyncTwoWay | CopyMode::SyncPreview
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
}

/// Lists the files and the directories of a tree.
///
/// WHY: the copy has to know the size of the whole tree before it copies the
/// first file, otherwise the progress of the job would be a guess. The walk
/// follows the entries of the source and never follows a symbolic link.
pub fn collect_items(
    fs: &dyn crate::port::FileSystem,
    root: &Path,
) -> Result<Vec<TreeItem>, OpError> {
    let mut items = Vec::new();
    collect_below(fs, root, Path::new(""), &mut items)?;
    items.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(items)
}

/// Adds the entries of one directory to the list.
fn collect_below(
    fs: &dyn crate::port::FileSystem,
    root: &Path,
    prefix: &Path,
    items: &mut Vec<TreeItem>,
) -> Result<(), OpError> {
    let directory = root.join(prefix);
    let entries = fs.read_dir(&directory)?;

    for entry in entries {
        let relative_path = prefix.join(&entry.name);
        let path = root.join(&relative_path);

        if entry.kind.is_directory() {
            items.push(TreeItem {
                relative_path: relative_path.clone(),
                path: path.clone(),
                size: 0,
                modified: entry.modified,
                is_directory: true,
            });
            collect_below(fs, root, &relative_path, items)?;
            continue;
        }

        let size = match entry.size {
            Some(size) => size,
            None => fs.metadata(&path)?.size,
        };

        items.push(TreeItem {
            relative_path,
            path,
            size,
            modified: entry.modified,
            is_directory: false,
        });
    }

    Ok(())
}

/// Copies a file in the mode of the request.
///
/// WHY: the three modes differ in what happens after the bytes are written.
/// The quick mode stops there, the verified mode compares the checksum of the
/// source with the checksum of the destination, and the atomic mode writes
/// into a temporary file, verifies that file and renames it into place — so
/// that a program which reads the destination never sees a half-written
/// file.
pub fn copy_one_file(
    fs: &dyn crate::port::FileSystem,
    source: &Path,
    target: &Path,
    mode: CopyMode,
    context: &mut JobContext,
) -> Result<u64, OpError> {
    match mode {
        CopyMode::Quick => {
            let result = fs.copy_file(source, target)?;
            Ok(result.bytes)
        }
        CopyMode::Verified => {
            let result = fs.copy_file(source, target)?;
            let destination_checksum = fs.checksum(target)?;
            if destination_checksum != result.checksum {
                context.begin_verification(1, result.bytes);
                context.add_verified_bytes(result.bytes);
                return Err(OpError::ChecksumMismatch {
                    path: target.to_path_buf(),
                });
            }
            Ok(result.bytes)
        }
        CopyMode::Atomic => {
            let temporary = temporary_path(target);
            let result = fs.copy_file(source, &temporary)?;
            let temporary_checksum = fs.checksum(&temporary)?;
            if temporary_checksum != result.checksum {
                let _ = fs.remove_file(&temporary);
                context.begin_verification(1, result.bytes);
                context.add_verified_bytes(result.bytes);
                return Err(OpError::ChecksumMismatch {
                    path: target.to_path_buf(),
                });
            }
            fs.rename(&temporary, target)?;
            Ok(result.bytes)
        }
        _ => Err(OpError::UnsupportedMode {
            mode: mode.as_str().to_string(),
        }),
    }
}

/// The name of the temporary file of the atomic mode.
///
/// WHY: the temporary file has to be in the same directory as the target,
/// because a rename cannot cross a file system. The name starts with a dot
/// and ends with a suffix that no user file carries, so that the temporary
/// file is not mistaken for a result.
fn temporary_path(target: &Path) -> PathBuf {
    let counter = TEMPORARY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let parent = target.parent().map(Path::to_path_buf).unwrap_or_default();
    parent.join(format!(".{name}.phocommander-{counter}.part"))
}

/// Finds a name that does not exist yet.
///
/// WHY: the policy "keep both" has to produce a name that is free, and the
/// name has to be found without asking the user — the operation runs in the
/// background.
pub fn free_name(
    fs: &dyn crate::port::FileSystem,
    target: &Path,
) -> Result<PathBuf, OpError> {
    if fs.metadata(target).is_err() {
        return Ok(target.to_path_buf());
    }

    let stem = target
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let extension = target
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();
    let parent = target.parent().map(Path::to_path_buf).unwrap_or_default();

    for number in 1..10_000 {
        let candidate = parent.join(format!("{stem} ({number}){extension}"));
        if fs.metadata(&candidate).is_err() {
            return Ok(candidate);
        }
    }

    Err(OpError::AlreadyExists {
        path: target.to_path_buf(),
    })
}

/// Copies a file or a directory tree.
///
/// WHY: the copy collects its failures instead of stopping at the first one
/// (FR-25), and it reports its progress after every file. The destination
/// and the mode can be changed while the copy runs; the change takes effect
/// at the next file, never in the middle of one.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::copy::{CopyMode, CopyRequest, execute_copy};
/// use phocommander_ops::edit::JobControl;
/// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
/// use std::sync::Arc;
///
/// struct Silent;
/// impl ProgressSink for Silent {
///     fn report(&mut self, _update: ProgressUpdate) {}
/// }
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let control = Arc::new(JobControl::new());
/// let mut context = JobContext::new(control, Box::new(Silent));
/// let request = CopyRequest::new("/photos", "/backup/photos").with_mode(CopyMode::Verified);
/// let outcome = execute_copy(fs, &request, &mut context)?;
/// assert_eq!(outcome.errors.len(), 0);
/// # Ok(())
/// # }
/// ```
pub fn execute_copy(
    fs: &dyn crate::port::FileSystem,
    request: &CopyRequest,
    context: &mut JobContext,
) -> Result<CopyOutcome, OpError> {
    if request.mode.is_sync() {
        return Err(OpError::UnsupportedMode {
            mode: request.mode.as_str().to_string(),
        });
    }

    let items = collect_items(fs, &request.source)?;
    let total_bytes: u64 = items.iter().map(|item| item.size).sum();
    context.set_totals(items.len() as u64, total_bytes);

    let mut outcome = CopyOutcome::default();
    let mut destination = request.destination.clone();
    let mut mode = request.mode;

    for item in items {
        match context.checkpoint() {
            Ok(Some(edit)) => {
                if let crate::edit::Edit::Destination(path) = edit {
                    destination = path;
                }
                if let crate::edit::Edit::VerificationMode(new_mode) = edit {
                    mode = new_mode;
                }
            }
            Ok(None) => {}
            Err(error) => return Err(error),
        }

        let name = item
            .relative_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| item.relative_path.to_string_lossy().to_string());
        context.begin_item(&name);

        let target = destination.join(&item.relative_path);

        if item.is_directory {
            if let Err(error) = fs.create_dir_all(&target) {
                outcome
                    .errors
                    .push(JobError::new(target.clone(), error.code(), error.to_string(), true));
            }
            context.finish_item();
            context.report();
            continue;
        }

        if let Some(parent) = target.parent() {
            let _ = fs.create_dir_all(parent);
        }

        let target = match request.policy {
            ConflictPolicy::Never | ConflictPolicy::Ask => {
                if fs.metadata(&target).is_ok() {
                    outcome.files_skipped += 1;
                    context.finish_item();
                    context.report();
                    continue;
                }
                target
            }
            ConflictPolicy::KeepBoth => free_name(fs, &target)?,
            ConflictPolicy::Always => target,
        };

        match copy_one_file(fs, &item.path, &target, mode, context) {
            Ok(bytes) => {
                outcome.files_copied += 1;
                outcome.bytes_copied += bytes;
                context.add_bytes(bytes);
            }
            Err(error) => {
                outcome.errors.push(JobError::new(
                    item.path.clone(),
                    error.code(),
                    error.to_string(),
                    true,
                ));
            }
        }

        context.finish_item();
        context.report();
    }

    Ok(outcome)
}

/// Compares two directory trees.
///
/// WHY: the comparison behind every synchronisation is the same and produces
/// the same plan. Per entry the state is identical, different,
/// missing-in-destination or extra-in-destination, together with the size
/// and the time of both sides. The action that the mode derives from the
/// state is part of the entry, so that the plan shows what the operation
/// would do before anything is moved.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::copy::{CopyMode, compare_trees};
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let entries = compare_trees(fs, std::path::Path::new("/photos"), std::path::Path::new("/backup"), CopyMode::SyncPreview)?;
/// for entry in entries {
///     println!("{:?} {:?}", entry.state, entry.relative_path);
/// }
/// # Ok(())
/// # }
/// ```
pub fn compare_trees(
    fs: &dyn crate::port::FileSystem,
    source: &Path,
    destination: &Path,
    mode: CopyMode,
) -> Result<Vec<SyncPlanEntry>, OpError> {
    let source_items = collect_items(fs, source)?;
    let destination_items = collect_items(fs, destination).unwrap_or_default();

    let mut entries: Vec<SyncPlanEntry> = Vec::new();

    for item in &source_items {
        let counterpart = destination_items
            .iter()
            .find(|other| other.relative_path == item.relative_path);

        match counterpart {
            None => {
                entries.push(SyncPlanEntry {
                    relative_path: item.relative_path.clone(),
                    kind: crate::port::EntryKind::File,
                    state: SyncState::MissingInDestination,
                    source_size: Some(item.size),
                    destination_size: None,
                    source_modified: item.modified,
                    destination_modified: None,
                    checksum: None,
                    action: SyncAction::Copy,
                });
            }
            Some(other) => {
                let identical = item.size == other.size
                    && item.modified.is_some()
                    && item.modified == other.modified;
                let state = if identical {
                    SyncState::Identical
                } else {
                    SyncState::Different
                };
                let action = if identical {
                    SyncAction::Keep
                } else {
                    match mode {
                        CopyMode::SyncTwoWay => {
                            if newer(item.modified, other.modified) {
                                SyncAction::CopyToSource
                            } else {
                                SyncAction::Overwrite
                            }
                        }
                        _ => SyncAction::Overwrite,
                    }
                };
                entries.push(SyncPlanEntry {
                    relative_path: item.relative_path.clone(),
                    kind: crate::port::EntryKind::File,
                    state,
                    source_size: Some(item.size),
                    destination_size: Some(other.size),
                    source_modified: item.modified,
                    destination_modified: other.modified,
                    checksum: None,
                    action,
                });
            }
        }
    }

    for item in &destination_items {
        let known = source_items
            .iter()
            .any(|other| other.relative_path == item.relative_path);
        if known {
            continue;
        }
        let action = if mode.deletes() {
            SyncAction::Delete
        } else {
            SyncAction::Keep
        };
        entries.push(SyncPlanEntry {
            relative_path: item.relative_path.clone(),
            kind: crate::port::EntryKind::File,
            state: SyncState::ExtraInDestination,
            source_size: None,
            destination_size: Some(item.size),
            source_modified: None,
            destination_modified: item.modified,
            checksum: None,
            action,
        });
    }

    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(entries)
}

/// Whether the first time is the newer one.
fn newer(first: Option<std::time::SystemTime>, second: Option<std::time::SystemTime>) -> bool {
    match (first, second) {
        (Some(first), Some(second)) => first > second,
        (Some(_), None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_of_a_copy_is_the_verified_mode() {
        // act
        let mode = CopyMode::default();

        // assert
        assert_eq!(mode, CopyMode::Verified);
        assert!(mode.verifies());
    }

    #[test]
    fn only_the_mirror_mode_deletes() {
        // act, assert
        assert!(CopyMode::SyncMirror.deletes());
        assert!(!CopyMode::SyncUpdate.deletes());
        assert!(!CopyMode::SyncTwoWay.deletes());
        assert!(!CopyMode::SyncPreview.deletes());
        assert!(!CopyMode::Quick.deletes());
    }

    #[test]
    fn the_modes_know_which_of_them_are_synchronisations() {
        // act, assert
        assert!(CopyMode::SyncMirror.is_sync());
        assert!(CopyMode::SyncUpdate.is_sync());
        assert!(CopyMode::SyncTwoWay.is_sync());
        assert!(CopyMode::SyncPreview.is_sync());
        assert!(!CopyMode::Quick.is_sync());
        assert!(!CopyMode::Verified.is_sync());
        assert!(!CopyMode::Atomic.is_sync());
    }

    #[test]
    fn the_modes_have_the_names_the_configuration_uses() {
        // act, assert
        assert_eq!(CopyMode::Quick.as_str(), "quick");
        assert_eq!(CopyMode::Verified.as_str(), "verified");
        assert_eq!(CopyMode::Atomic.as_str(), "atomic");
        assert_eq!(CopyMode::SyncMirror.as_str(), "sync-mirror");
        assert_eq!(CopyMode::SyncUpdate.as_str(), "sync-update");
        assert_eq!(CopyMode::SyncTwoWay.as_str(), "sync-two-way");
        assert_eq!(CopyMode::SyncPreview.as_str(), "sync-preview");
    }

    #[test]
    fn the_temporary_file_of_the_atomic_mode_sits_next_to_the_target() {
        // arrange
        let target = Path::new("/backup/photos/sunset.jpg");

        // act
        let temporary = temporary_path(target);

        // assert
        assert_eq!(temporary.parent(), target.parent());
        let name = temporary
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        assert!(name.starts_with(".sunset.jpg."), "{name}");
        assert!(name.ends_with(".part"), "{name}");
    }
}
