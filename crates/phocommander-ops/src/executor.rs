//! The execution of a job: what a job kind means in terms of file operations.
//!
//! WHAT: the result of an execution ([{JobResult}]) and the function that
//!       carries out the work of a job ([{execute}]).
//! WHY: the queue knows about jobs and about workers, but not about files.
//!       The mapping from the kind of a job to the file operations belongs
//!       in one place, so that a new kind of job is added here and nowhere
//!       else. The execution collects the failures instead of stopping at the
//!       first one, because a job that copied 400 of 500 files and failed on
//!       the other 100 is a completed job with errors.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::executor::execute;
//! use phocommander_ops::job::{Job, JobKind, Priority};
//! use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
//! use std::sync::Arc;
//!
//! struct Silent;
//! impl ProgressSink for Silent {
//!     fn report(&mut self, _update: ProgressUpdate) {}
//! }
//!
//! # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
//! let control = Arc::new(phocommander_ops::edit::JobControl::new());
//! let mut context = JobContext::new(control, Box::new(Silent));
//! let job = Job::new(
//!     JobKind::CreateDirectory { path: "/photos".into() },
//!     Priority::Normal,
//! );
//! let result = execute(fs, &job, &mut context)?;
//! assert!(result.errors.is_empty());
//! # Ok(())
//! # }
//! ```
//! BOUNDARY: this module works through the [{crate::port::FileSystem}] of the
//!       operations. It never touches a file system on its own.

use std::path::Path;

use crate::copy::{CopyRequest, execute_copy};
use crate::error::{JobError, OpError};
use crate::job::{Job, JobKind, JobState};
use crate::port::FileSystem;
use crate::progress::JobContext;
use crate::sync::{Approval, SyncPlan, execute_sync};

/// What the execution of a job produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobResult {
    /// The state the job ends in.
    pub state: JobState,

    /// The failures the job collected.
    pub errors: Vec<JobError>,

    /// The number of items that are done.
    pub items_done: u64,

    /// The number of bytes that are done.
    pub bytes_done: u64,
}

/// Carries out the work of a job.
///
/// WHY: every kind of job ends in the same place: the failures are
/// collected, the progress is reported and the state of the job is decided.
/// The cancellation is the one failure that ends the job instead of being
/// collected, because a cancelled job does not continue with the next item.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::executor::execute;
/// use phocommander_ops::job::{Job, JobKind, Priority};
/// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
/// use std::sync::Arc;
///
/// struct Silent;
/// impl ProgressSink for Silent {
///     fn report(&mut self, _update: ProgressUpdate) {}
/// }
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let control = Arc::new(phocommander_ops::edit::JobControl::new());
/// let mut context = JobContext::new(control, Box::new(Silent));
/// let job = Job::new(JobKind::Delete { paths: Vec::new() }, Priority::Normal);
/// let result = execute(fs, &job, &mut context)?;
/// assert_eq!(result.state, phocommander_ops::job::JobState::Done);
/// # Ok(())
/// # }
/// ```
pub fn execute(
    fs: &dyn FileSystem,
    job: &Job,
    context: &mut JobContext,
) -> Result<JobResult, OpError> {
    match &job.kind {
        JobKind::Copy {
            source,
            destination,
            mode,
        } => {
            let request = CopyRequest::new(source.clone(), destination.clone())
                .with_mode(*mode)
                .with_restriction(job.restricted_items.clone());
            let outcome = execute_copy(fs, &request, context)?;
            Ok(JobResult {
                state: state_of(&outcome.errors),
                errors: outcome.errors,
                items_done: context.items_done(),
                bytes_done: context.bytes_done(),
            })
        }
        JobKind::Move {
            source,
            destination,
            mode,
        } => {
            let request = CopyRequest::new(source.clone(), destination.clone())
                .with_mode(*mode)
                .with_restriction(job.restricted_items.clone());
            let outcome = execute_copy(fs, &request, context)?;
            let mut errors = outcome.errors;
            if errors.is_empty()
                && job.restricted_items.is_none()
                && let Err(error) = remove_entry(fs, source)
            {
                errors.push(
                    JobError::new(source.clone(), error.code(), error.to_string(), false)
                        .with_arguments(error.message_arguments()),
                );
            }
            Ok(JobResult {
                state: state_of(&errors),
                errors,
                items_done: context.items_done(),
                bytes_done: context.bytes_done(),
            })
        }
        JobKind::Sync { plan } => {
            let outcome = execute_sync(fs, plan, Approval::Granted, context)?;
            Ok(JobResult {
                state: state_of(&outcome.errors),
                errors: outcome.errors,
                items_done: context.items_done(),
                bytes_done: context.bytes_done(),
            })
        }
        JobKind::Rename {
            source,
            destination,
        } => match fs.rename(source, destination) {
            Ok(()) => Ok(JobResult {
                state: JobState::Done,
                errors: Vec::new(),
                items_done: context.items_done(),
                bytes_done: context.bytes_done(),
            }),
            Err(error) => Ok(JobResult {
                state: JobState::Failed,
                errors: vec![
                    JobError::new(source.clone(), error.code(), error.to_string(), false)
                        .with_arguments(error.message_arguments()),
                ],
                items_done: context.items_done(),
                bytes_done: context.bytes_done(),
            }),
        },
        JobKind::Delete { paths } => {
            let mut errors = Vec::new();
            for path in paths {
                if let Err(error) = remove_entry(fs, path) {
                    errors.push(
                        JobError::new(path.clone(), error.code(), error.to_string(), true)
                            .with_arguments(error.message_arguments()),
                    );
                }
            }
            Ok(JobResult {
                state: state_of(&errors),
                errors,
                items_done: context.items_done(),
                bytes_done: context.bytes_done(),
            })
        }
        JobKind::CreateDirectory { path } => match fs.create_dir_all(path) {
            Ok(()) => Ok(JobResult {
                state: JobState::Done,
                errors: Vec::new(),
                items_done: context.items_done(),
                bytes_done: context.bytes_done(),
            }),
            Err(error) => Ok(JobResult {
                state: JobState::Failed,
                errors: vec![
                    JobError::new(path.clone(), error.code(), error.to_string(), false)
                        .with_arguments(error.message_arguments()),
                ],
                items_done: context.items_done(),
                bytes_done: context.bytes_done(),
            }),
        },
    }
}

/// The state a job ends in, given the failures it collected.
///
/// WHY: a job that collected a failure is a failed job, but it is a job that
/// ran — the difference matters for the job cue, which shows "failed" with
/// the number of failures and not an empty progress.
fn state_of(errors: &[JobError]) -> JobState {
    if errors.is_empty() {
        JobState::Done
    } else {
        JobState::Failed
    }
}

/// Moves a file or a directory into the trash, whatever it is.
///
/// WHY: the deletion of a file goes into the trash and not into the void
///       (FR-27). The user can undo it there, and the file keeps its data
///       until the trash is emptied. A directory goes as one piece, with
///       everything it holds.
fn remove_entry(fs: &dyn FileSystem, path: &Path) -> Result<(), OpError> {
    match fs.metadata(path) {
        Ok(_) => fs.move_to_trash(path),
        Err(error) => Err(error),
    }
}

/// Builds the job of a synchronisation from a plan that the user confirmed.
///
/// WHY: the plan is the document the user confirms, and no data moves before
/// the plan is confirmed. The job therefore carries the confirmed plan, and
/// a plan that was not confirmed cannot become a job at all.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::copy::CopyMode;
/// use phocommander_ops::executor::confirmed_job;
/// use phocommander_ops::job::Priority;
/// use phocommander_ops::sync::{Approval, SyncPlan};
///
/// # fn run() -> Result<(), phocommander_ops::error::OpError> {
/// let plan = SyncPlan {
///     mode: CopyMode::SyncMirror,
///     source: "/photos".into(),
///     destination: "/backup".into(),
///     entries: Vec::new(),
/// };
/// let refused = confirmed_job(plan.clone(), Approval::Withheld, Priority::Normal);
/// assert!(refused.is_err());
/// let job = confirmed_job(plan, Approval::Granted, Priority::Normal)?;
/// assert_eq!(job.id().get(), 1);
/// # Ok(())
/// # }
/// ```
pub fn confirmed_job(
    plan: SyncPlan,
    approval: Approval,
    priority: crate::job::Priority,
) -> Result<crate::job::Job, OpError> {
    if approval == Approval::Withheld {
        return Err(OpError::NotConfirmed);
    }
    Ok(crate::job::Job::new(JobKind::Sync { plan }, priority))
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::copy::CopyMode;
    use crate::fake_fs::FakeFileSystem;
    use crate::job::{JobState, Priority};
    use std::path::Path;
    use std::sync::Arc;

    /// The sink that discards the progress.
    struct Silent;

    impl crate::progress::ProgressSink for Silent {
        fn report(&mut self, _update: crate::progress::ProgressUpdate) {}
    }

    /// A context of a running job.
    fn context() -> crate::progress::JobContext {
        crate::progress::JobContext::new(Arc::new(crate::edit::JobControl::new()), Box::new(Silent))
    }

    #[test]
    fn the_deletion_of_a_file_goes_into_the_trash() {
        // arrange
        // WHY: the deletion of a file is the one operation of a file
        //       manager that can destroy data that the user cannot get back.
        //       The specification therefore names the trash as the default
        //       path of the deletion (FR-27).
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file should have been written: {error}"));
        let mut context = context();
        let job = Job::new(
            JobKind::Delete {
                paths: vec![Path::new("/photos/a.txt").to_path_buf()],
            },
            Priority::Normal,
        );

        // act
        let result = execute(&fs, &job, &mut context)
            .unwrap_or_else(|error| panic!("the job should have worked: {error}"));

        // assert
        assert_eq!(result.state, JobState::Done);
        // WHY: the file is in the trash and not gone.
        assert_eq!(fs.trashed(), vec![Path::new("/photos/a.txt").to_path_buf()]);
        // WHY: and it is not in the file system any more.
        assert!(fs.metadata(Path::new("/photos/a.txt")).is_err());
    }

    #[test]
    fn the_deletion_of_a_directory_goes_into_the_trash_as_one_piece() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/ordner/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file should have been written: {error}"));
        fs.create_dir_all(Path::new("/photos/ordner"))
            .unwrap_or_else(|error| panic!("the directory should have been created: {error}"));
        let mut context = context();
        let job = Job::new(
            JobKind::Delete {
                paths: vec![Path::new("/photos/ordner").to_path_buf()],
            },
            Priority::Normal,
        );

        // act
        let result = execute(&fs, &job, &mut context)
            .unwrap_or_else(|error| panic!("the job should have been worked: {error}"));

        // assert
        assert_eq!(result.state, JobState::Done);
        // WHY: the directory goes with everything it holds, and not as a
        //       pile of empty folders.
        let mut trashed = fs.trashed();
        trashed.sort();
        assert_eq!(
            trashed,
            vec![
                Path::new("/photos/ordner").to_path_buf(),
                Path::new("/photos/ordner/a.txt").to_path_buf(),
            ]
        );
    }

    #[test]
    fn a_file_that_cannot_be_moved_into_the_trash_makes_the_job_fail() {
        // arrange
        let fs = FakeFileSystem::new();
        let mut context = context();
        let job = Job::new(
            JobKind::Delete {
                paths: vec![Path::new("/photos/gibt-es-nicht.txt").to_path_buf()],
            },
            Priority::Normal,
        );

        // act
        let result = execute(&fs, &job, &mut context)
            .unwrap_or_else(|error| panic!("the job should have been worked: {error}"));

        // assert
        // WHY: a deletion that did not happen has to be reported. A job
        //       that says "done" while the file is still there teaches the
        //       user that the file is gone.
        assert_eq!(result.state, JobState::Failed);
        assert_eq!(result.errors.len(), 1);
        assert!(fs.trashed().is_empty());
    }

    #[test]
    fn the_move_of_a_file_leaves_the_source_in_the_trash() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file should have been written: {error}"));
        let mut context = context();
        let job = Job::new(
            JobKind::Move {
                source: Path::new("/photos/a.txt").to_path_buf(),
                destination: Path::new("/backup/a.txt").to_path_buf(),
                mode: CopyMode::Verified,
            },
            Priority::Normal,
        );

        // act
        let result = execute(&fs, &job, &mut context)
            .unwrap_or_else(|error| panic!("the job should have worked: {error}"));

        // assert
        // WHY: the move copies the file and then removes the source. The
        //       removal goes into the trash, so that the user can undo the
        //       move.
        assert_eq!(result.state, JobState::Done);
        assert_eq!(fs.trashed(), vec![Path::new("/photos/a.txt").to_path_buf()]);
    }
}
