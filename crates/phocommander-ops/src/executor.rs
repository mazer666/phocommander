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
                && let Err(error) = remove_tree(fs, source)
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

/// Removes a file or a directory, whatever it is.
fn remove_entry(fs: &dyn FileSystem, path: &Path) -> Result<(), OpError> {
    match fs.metadata(path) {
        Ok(info) => {
            if info.kind.is_directory() {
                fs.remove_dir(path)
            } else {
                fs.remove_file(path)
            }
        }
        Err(error) => Err(error),
    }
}

/// Removes a directory with everything it contains.
fn remove_tree(fs: &dyn FileSystem, root: &Path) -> Result<(), OpError> {
    let entries = match fs.read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            // The source of a move may be a single file.
            return fs.remove_file(root).map_err(|_| error);
        }
    };

    for entry in entries {
        let path = root.join(&entry.name);
        if entry.kind.is_directory() {
            remove_tree(fs, &path)?;
        } else {
            fs.remove_file(&path)?;
        }
    }

    fs.remove_dir(root)
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
