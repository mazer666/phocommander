//! The execution of a synchronisation plan.
//!
//! WHAT: the execution of a plan ([{execute_sync}]).
//! WHY: the plan is executed only when the user confirmed it. A plan that
//!       was not confirmed changes nothing at all — not a byte and not a
//!       deletion. The execution walks the plan in a fixed order: the
//!       deletions run from the deepest entry to the shallowest one, so that
//!       a directory is removed after the files it contains, and the copies
//!       run afterwards.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::copy::CopyMode;
//! use phocommander_ops::sync::{Approval, build_plan, execute_sync};
//! use std::path::Path;
//!
//! # fn run(fs: &dyn phocommander_ops::port::FileSystem, context: &mut phocommander_ops::progress::JobContext) -> Result<(), phocommander_ops::error::OpError> {
//! let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncMirror)?;
//! let outcome = execute_sync(fs, &plan, Approval::Granted, context)?;
//! assert_eq!(outcome.errors.len(), 0);
//! # Ok(())
//! # }
//! ```
//! BOUNDARY: this module works through the [{crate::port::FileSystem}] of the
//!       operations. It never touches a file system on its own.

use std::path::Path;

use crate::copy::{CopyMode, CopyOutcome, copy_one_file};
use crate::error::{JobError, OpError};
use crate::port::FileSystem;
use crate::progress::JobContext;
use crate::sync::{Approval, SyncAction, SyncPlan};

/// Executes the plan of a synchronisation.
///
/// WHY: the plan is executed only when the user confirmed it. A plan that
///       was not confirmed changes nothing at all — not a byte and not a
///       deletion.
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
    fs: &dyn FileSystem,
    plan: &SyncPlan,
    approval: Approval,
    context: &mut JobContext,
) -> Result<CopyOutcome, OpError> {
    if approval == Approval::Withheld {
        return Err(OpError::NotConfirmed);
    }

    let mut outcome = CopyOutcome::default();
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
    let mut order: Vec<&crate::sync::SyncPlanEntry> = plan.entries.iter().collect();
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
                // WHY: the deletion of a synchronisation is the one
                //       deletion that does not go into the trash. The user
                //       has seen the plan and confirmed it, and the plan
                //       names every file that goes. A trash that holds a
                //       copy of the target of every synchronisation would
                //       fill the medium and would hold the files that the
                //       user wanted gone.
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
                        &crate::copy::CopyPolicy::strict(),
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
                        &crate::copy::CopyPolicy::strict(),
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
                outcome.errors.push(JobError::new(
                    source_path.clone(),
                    error.code(),
                    error.to_string(),
                    true,
                ));
                context.finish_item();
            }
        }
        context.report();
    }

    Ok(outcome)
}

/// The path of an entry of the plan, on the side of the source.
///
/// WHY: the helper names the side of the plan, so that a caller cannot
///       confuse the source with the destination.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::sync::execute::source_path;
/// use phocommander_ops::sync::{SyncPlan, build_plan};
/// use phocommander_ops::copy::CopyMode;
/// use std::path::Path;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncPreview)?;
/// let entry = plan.entries().first();
/// if let Some(entry) = entry {
///     assert_eq!(source_path(&plan, &entry.relative_path), Path::new("/photos").join(&entry.relative_path));
/// }
/// # Ok(())
/// # }
/// ```
pub fn source_path(plan: &SyncPlan, relative: &Path) -> std::path::PathBuf {
    plan.source.join(relative)
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::edit::JobControl;
    use crate::fake_fs::FakeFileSystem;
    use crate::progress::{ProgressSink, ProgressUpdate};
    use std::sync::Arc;

    /// A sink that discards every report.
    struct Silent;

    impl ProgressSink for Silent {
        fn report(&mut self, _update: ProgressUpdate) {}
    }

    /// Builds a context for a job that runs without a queue.
    fn context() -> JobContext {
        JobContext::new(Arc::new(JobControl::new()), Box::new(Silent))
    }

    #[test]
    fn a_plan_that_was_not_confirmed_moves_nothing() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        fs.write_file(Path::new("/backup/old.txt"), b"old")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        let plan = crate::sync::build_plan(
            &fs,
            Path::new("/photos"),
            Path::new("/backup"),
            CopyMode::SyncMirror,
        );
        let plan = plan.unwrap_or_default();
        let mut context = context();

        // act
        let result = execute_sync(&fs, &plan, Approval::Withheld, &mut context);

        // assert
        assert!(matches!(result, Err(OpError::NotConfirmed)), "{result:?}");
        assert!(!fs.exists(Path::new("/backup/a.txt")));
        assert!(fs.exists(Path::new("/backup/old.txt")));
    }

    #[test]
    fn a_confirmed_mirror_plan_copies_and_deletes() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        fs.write_file(Path::new("/backup/old.txt"), b"old")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        let plan = crate::sync::build_plan(
            &fs,
            Path::new("/photos"),
            Path::new("/backup"),
            CopyMode::SyncMirror,
        );
        let plan = plan.unwrap_or_default();
        let mut context = context();

        // act
        let outcome = execute_sync(&fs, &plan, Approval::Granted, &mut context);

        // assert
        let outcome = outcome.unwrap_or_default();
        assert_eq!(outcome.errors.len(), 0, "{:?}", outcome.errors);
        assert!(fs.exists(Path::new("/backup/a.txt")));
        assert!(!fs.exists(Path::new("/backup/old.txt")));
    }
}
