//! The copy: the modes of the specification.
//!
//! WHAT: the execution of a copy ([{execute_copy}]) and the copy of a single
//!       file ([{copy_one_file}]). The vocabulary of the copy — the mode, the
//!       request, the outcome, the item of a tree — lives in the file
//!       `types.rs` of this directory, the rules of the security section in
//!       the file `policy.rs` and the atomic mode in the file `atomic.rs`.
//! WHY: a copy is not one operation but a family of them, and the difference
//!       between them is the difference between a fast copy and a copy that
//!       can be trusted (section 6.9 of the specification). The user chooses
//!       the mode; the default is the safe one, because the failure of a copy
//!       is discovered far too late otherwise. The comparison of two trees
//!       lives in the module [{crate::compare}], because the synchronisation
//!       and the comparison of two directories use the same one.
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

use crate::error::{JobError, OpError};
use crate::progress::JobContext;

pub mod atomic;
mod policy;
mod types;

pub use policy::CopyPolicy;
// WHY: the policy of the conflicts belongs to the copy, but it is defined
// with the other rules of a job. It is re-exported here so that the copy
// can be used without knowing where the rule lives.
pub use crate::edit::ConflictPolicy;
pub use types::{CopyMode, CopyOutcome, CopyRequest, TreeItem};

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
                kind: entry.kind,
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
            kind: entry.kind,
        });
    }

    Ok(())
}

/// Copies a file in the mode of the request.
///
/// WHY: the three modes differ in what happens after the bytes are written.
/// The quick mode stops there, the verified mode compares the checksum of the
/// source with the checksum of the destination — unless the rules switched
/// that comparison off, in which case the destination is not read a second
/// time at all — and the atomic mode writes into a temporary file, verifies
/// that file and renames it into place, so that a program which reads the
/// destination never sees a half-written file.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::copy::{CopyMode, CopyPolicy, copy_one_file};
/// use phocommander_ops::edit::JobControl;
/// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
/// use std::path::Path;
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
/// let bytes = copy_one_file(
///     fs,
///     Path::new("/photos/a.txt"),
///     Path::new("/backup/a.txt"),
///     CopyMode::Verified,
///     &CopyPolicy::strict(),
///     &mut context,
/// )?;
/// assert!(bytes > 0);
/// # Ok(())
/// # }
/// ```
pub fn copy_one_file(
    fs: &dyn crate::port::FileSystem,
    source: &Path,
    target: &Path,
    mode: CopyMode,
    rules: &CopyPolicy,
    context: &mut JobContext,
) -> Result<u64, OpError> {
    match mode {
        CopyMode::Quick => {
            let result = fs.copy_file(source, target)?;
            Ok(result.bytes)
        }
        CopyMode::Verified => {
            let result = fs.copy_file(source, target)?;
            if !rules.verifies_checksums() {
                // WHY: the rule "checksum_on_copy" is switched off, so the
                // comparison is not executed. The destination is not read a
                // second time, and the read volume of the copy stays at the
                // volume that was copied.
                return Ok(result.bytes);
            }
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
        CopyMode::Atomic => atomic::copy_atomically(fs, source, target, context),
        _ => Err(OpError::UnsupportedMode {
            mode: mode.as_str().to_string(),
        }),
    }
}

/// Limits a list of items to the items of a restriction.
///
/// WHY: a job that repeats only its failed items carries the list of the
///       items that failed. The list of the items of the tree is then cut
///       down to those items, so that the files that were copied already are
///       not copied a second time. An item that is not in the tree any more
///       is simply not part of the copy.
fn restrict_to(items: Vec<TreeItem>, restricted: Option<&[PathBuf]>) -> Vec<TreeItem> {
    let Some(restricted) = restricted else {
        return items;
    };
    items
        .into_iter()
        .filter(|item| restricted.iter().any(|path| path == &item.path))
        .collect()
}

/// Finds a name that does not exist yet.
///
/// WHY: the policy "keep both" has to produce a name that is free, and the
/// name has to be found without asking the user — the operation runs in the
/// background.
pub fn free_name(fs: &dyn crate::port::FileSystem, target: &Path) -> Result<PathBuf, OpError> {
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
    let items = restrict_to(items, request.restricted_items.as_deref());
    let total_bytes: u64 = items.iter().map(|item| item.size).sum();
    context.set_totals(items.len() as u64, total_bytes);

    let mut outcome = CopyOutcome::default();
    let mut destination = request.destination.clone();
    let mut mode = request.mode;
    let rules = request.rules;

    for item in items {
        match context.checkpoint() {
            Ok(Some(edit)) => {
                if let crate::edit::Edit::Destination(ref path) = edit {
                    destination = path.to_path_buf();
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
                outcome.errors.push(JobError::new(
                    target.clone(),
                    error.code(),
                    error.to_string(),
                    true,
                ));
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

        // WHY: a symbolic link is copied as a link, unless the rules allow
        // the copy to dereference it. A copy that dereferences the link
        // would copy a file the user never selected and would lose the link
        // itself.
        if item.kind.is_symlink() && !rules.follows_symlinks() {
            match fs.copy_symlink(&item.path, &target) {
                Ok(()) => {
                    outcome.files_copied += 1;
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
            continue;
        }

        match copy_one_file(fs, &item.path, &target, mode, &rules, context) {
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

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;
