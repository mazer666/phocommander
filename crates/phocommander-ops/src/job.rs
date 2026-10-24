//! The job: what it does, how far it is and how it is named.
//!
//! WHAT: the identity of a job ([{JobId}]), the work it does ([{JobKind}]),
//!       its state ([{JobState}]), its priority ([{Priority}]), its progress
//!       ([{JobProgress}], [{VerificationProgress}]) and the state the user
//!       interface reads ([{JobSnapshot}]).
//! WHY: every operation that can take longer than a frame is a job, and the
//!       job is the only thing the interface knows about it. The interface
//!       reads a snapshot and never reaches into the queue, so that the
//!       queue can change its internals without touching the interface
//!       (the concurrency model of the specification). The snapshot is a copy
//!       of plain data, so the interface can hold it while the job runs.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::copy::CopyMode;
//! use phocommander_ops::job::{Job, JobKind, JobState, Priority};
//! use std::path::PathBuf;
//!
//! let job = Job::new(
//!     JobKind::Copy {
//!         source: PathBuf::from("/photos"),
//!         destination: PathBuf::from("/backup/photos"),
//!         mode: CopyMode::default(),
//!     },
//!     Priority::Normal,
//! );
//! assert_eq!(job.state(), JobState::Queued);
//! ```
//! BOUNDARY: this module owns the state of a job. It performs no file
//!       operation and knows no user interface. The channel a running job
//!       reports through lives in [{crate::progress}].

use std::path::PathBuf;
use std::time::SystemTime;

use crate::copy::CopyMode;
use crate::error::JobError;

/// The identifier of a job.
///
/// WHY: the identifier is what the user interface holds on to. It is a
/// number and not a pointer, so that the interface can keep it while the
/// queue moves the job around.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobId(u64);

impl JobId {
/// Builds an identifier.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::JobId;
///
/// let id = JobId::new(7);
/// assert_eq!(id.get(), 7);
/// ```
    pub fn new(value: u64) -> Self {
        Self(value)
    }

/// The number of the identifier.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::JobId;
///
/// assert_eq!(JobId::new(7).get(), 7);
/// ```
    pub fn get(&self) -> u64 {
        self.0
    }
}

/// The work a job does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobKind {
/// Copy a file or a directory tree.
    Copy {
/// The directory or the file that is copied.
        source: PathBuf,
/// The directory or the file that receives the copy.
        destination: PathBuf,
/// The mode of the copy.
        mode: CopyMode,
    },

/// Move a file or a directory tree.
    Move {
/// The directory or the file that is moved.
        source: PathBuf,
/// The directory or the file that receives the move.
        destination: PathBuf,
/// The mode of the copy that the move performs.
        mode: CopyMode,
    },

/// Bring two directories into the same state.
///
/// WHY: the job carries the plan and not the two directories, because
/// the plan is what the user confirmed. A synchronisation that was not
/// confirmed cannot become a job at all.
    Sync {
/// The plan the user confirmed.
        plan: crate::sync::SyncPlan,
    },

/// Rename a file.
    Rename {
/// The file that is renamed.
        source: PathBuf,
/// The name the file receives.
        destination: PathBuf,
    },

/// Delete files or directories.
    Delete {
/// The paths that are deleted.
        paths: Vec<PathBuf>,
    },

/// Create a directory.
    CreateDirectory {
/// The directory that is created.
        path: PathBuf,
    },
}

/// The state of a job.
///
/// WHY: the states are the ones the job cue of the specification names:
/// queued, running, verifying, waiting, done, failed and cancelled. The
/// verification is a state of its own, because the phase has its own
/// progress and the user has to see that the copy is not stuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobState {
/// The job waits for a worker.
    Queued,

/// The job works.
    Running,

/// The job compares the source and the destination.
    Verifying,

/// The job waits — for the user, for a permission or for the pause.
    Waiting,

/// The job finished without an error.
    Done,

/// The job finished with errors.
    Failed,

/// The job was cancelled.
    Cancelled,
}

impl JobState {
/// The name of the state, as the job cue shows it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::JobState;
///
/// assert_eq!(JobState::Verifying.as_str(), "verifying");
/// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            JobState::Queued => "queued",
            JobState::Running => "running",
            JobState::Verifying => "verifying",
            JobState::Waiting => "waiting",
            JobState::Done => "done",
            JobState::Failed => "failed",
            JobState::Cancelled => "cancelled",
        }
    }

/// Whether the job has finished, whatever the outcome was.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::JobState;
///
/// assert!(JobState::Done.is_finished());
/// assert!(!JobState::Running.is_finished());
/// ```
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            JobState::Done | JobState::Failed | JobState::Cancelled
        )
    }
}

/// The priority of a job in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
/// The job runs after the jobs of the normal and the high priority.
    Low,

/// The job runs in the order it was submitted.
    #[default]
    Normal,

/// The job runs before the jobs of the normal and the low priority.
    High,
}

impl Priority {
/// The name of the priority.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::Priority;
///
/// assert_eq!(Priority::High.as_str(), "high");
/// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Normal => "normal",
            Priority::High => "high",
        }
    }
}

/// The progress of the verification phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VerificationProgress {
/// The number of files that were compared.
    pub items_done: u64,

/// The number of files that have to be compared.
    pub items_total: u64,

/// The number of bytes that were read again.
    pub bytes_done: u64,

/// The number of bytes that have to be read again.
    pub bytes_total: u64,
}

/// The progress of a job.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JobProgress {
/// The number of items that are done.
    pub items_done: u64,

/// The number of items that have to be done.
    pub items_total: u64,

/// The number of bytes that are done.
    pub bytes_done: u64,

/// The number of bytes that have to be done.
    pub bytes_total: u64,

/// The item the job is working on.
    pub current_item: Option<String>,

/// The progress of the verification, if the mode verifies.
    pub verification: Option<VerificationProgress>,
}

/// One entry of the history of a job.
///
/// WHY: the history records what was changed in the job, so that an
/// interrupted and corrected operation stays traceable (the editable jobs of
/// the specification).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
/// The time of the entry.
    pub at: SystemTime,

/// What happened.
    pub message: String,
}

/// The state of a job, as the user interface reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSnapshot {
/// The identifier of the job.
    pub id: JobId,

/// The work the job does.
    pub kind: JobKind,

/// The state of the job.
    pub state: JobState,

/// The priority of the job.
    pub priority: Priority,

/// The progress of the job.
    pub progress: JobProgress,

/// The failures the job collected.
    pub errors: Vec<JobError>,

/// The history of the job.
    pub history: Vec<HistoryEntry>,
}

/// A job of the queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
/// The identifier of the job.
    pub id: JobId,

/// The work the job does.
    pub kind: JobKind,

/// The state of the job.
    pub state: JobState,

/// The priority of the job.
    pub priority: Priority,

/// The progress of the job.
    pub progress: JobProgress,

/// The failures the job collected.
    pub errors: Vec<JobError>,

/// The history of the job.
    pub history: Vec<HistoryEntry>,
}

impl Job {
/// Builds a queued job.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::{Job, JobKind, Priority};
///
/// let job = Job::new(JobKind::Delete { paths: Vec::new() }, Priority::Normal);
/// assert!(job.errors().is_empty());
/// ```
    pub fn new(kind: JobKind, priority: Priority) -> Self {
        Self {
            id: JobId::new(0),
            kind,
            state: JobState::Queued,
            priority,
            progress: JobProgress::default(),
            errors: Vec::new(),
            history: Vec::new(),
        }
    }

/// The identifier of the job.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::{Job, JobKind, Priority};
///
/// let job = Job::new(JobKind::Delete { paths: Vec::new() }, Priority::Normal);
/// assert_eq!(job.id(), JobId::new(0));
/// ```
    pub fn id(&self) -> JobId {
        self.id
    }

/// The state of the job.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::{Job, JobKind, JobState, Priority};
///
/// let job = Job::new(JobKind::Delete { paths: Vec::new() }, Priority::Normal);
/// assert_eq!(job.state(), JobState::Queued);
/// ```
    pub fn state(&self) -> JobState {
        self.state
    }

/// The failures the job collected.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::{Job, JobKind, Priority};
///
/// let job = Job::new(JobKind::Delete { paths: Vec::new() }, Priority::Normal);
/// assert!(job.errors().is_empty());
/// ```
    pub fn errors(&self) -> &[JobError] {
        &self.errors
    }

/// The state the user interface reads.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::job::{Job, JobKind, Priority};
///
/// let job = Job::new(JobKind::Delete { paths: Vec::new() }, Priority::Normal);
/// assert_eq!(job.snapshot().state, JobState::Queued);
/// ```
    pub fn snapshot(&self) -> JobSnapshot {
        JobSnapshot {
            id: self.id,
            kind: self.kind.clone(),
            state: self.state,
            priority: self.priority,
            progress: self.progress.clone(),
            errors: self.errors.clone(),
            history: self.history.clone(),
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::job::JobKind::Delete;

    #[test]
    fn a_new_job_is_queued_and_has_no_failures() {
        // act
        let job = Job::new(Delete { paths: Vec::new() }, Priority::Normal);

        // assert
        assert_eq!(job.state(), JobState::Queued);
        assert!(job.errors().is_empty());
        assert_eq!(job.priority, Priority::Normal);
    }

    #[test]
    fn the_snapshot_carries_the_state_of_the_job() {
        // arrange
        let mut job = Job::new(Delete { paths: Vec::new() }, Priority::High);
        job.state = JobState::Running;

        // act
        let snapshot = job.snapshot();

        // assert
        assert_eq!(snapshot.state, JobState::Running);
        assert_eq!(snapshot.priority, Priority::High);
        assert_eq!(snapshot.id, job.id);
    }

    #[test]
    fn the_states_know_which_of_them_are_finished() {
        // act, assert
        assert!(JobState::Done.is_finished());
        assert!(JobState::Failed.is_finished());
        assert!(JobState::Cancelled.is_finished());
        assert!(!JobState::Queued.is_finished());
        assert!(!JobState::Running.is_finished());
        assert!(!JobState::Verifying.is_finished());
        assert!(!JobState::Waiting.is_finished());
    }

    #[test]
    fn the_priority_orders_the_queue() {
        // act, assert
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
        assert_eq!(Priority::default(), Priority::Normal);
    }

    #[test]
    fn the_states_have_the_names_the_job_cue_shows() {
        // act, assert
        assert_eq!(JobState::Queued.as_str(), "queued");
        assert_eq!(JobState::Running.as_str(), "running");
        assert_eq!(JobState::Verifying.as_str(), "verifying");
        assert_eq!(JobState::Waiting.as_str(), "waiting");
        assert_eq!(JobState::Done.as_str(), "done");
        assert_eq!(JobState::Failed.as_str(), "failed");
        assert_eq!(JobState::Cancelled.as_str(), "cancelled");
    }
}
