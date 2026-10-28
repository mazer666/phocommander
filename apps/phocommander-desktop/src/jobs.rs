//! The source of the list of the jobs.
//!
//! WHAT: the adapter that reads the waiting room of the operations and hands
//!       the rows of the list to the surface ([`QueueJobSource`]).
//! WHY: the surface knows nothing about the crate of the operations — it
//!       asks a source for the rows and reads the state of every row. The
//!       adapter is the one place where the two meet, so that neither crate
//!       has to know the other.
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::jobs::QueueJobSource;
//! use phocommander_ops::queue::JobQueue;
//! use phocommander_ui::jobs::JobSource;
//! use std::sync::Arc;
//!
//! let queue = Arc::new(JobQueue::new(Arc::new(phocommander_ops::fake_fs::FakeFileSystem::new()), 1));
//! let source = QueueJobSource::new(queue);
//! assert!(source.summaries().is_empty());
//! ```

use std::sync::Arc;

use phocommander_ops::job::{JobKind as OpsJobKind, JobSnapshot, JobState as OpsJobState};
use phocommander_ops::queue::JobQueue;
use phocommander_ui::jobs::{JobError, JobKind, JobSource, JobState, JobSummary};

/// The list of the jobs, read from the waiting room of the operations.
#[derive(Clone)]
pub struct QueueJobSource {
    /// The waiting room that carries the jobs.
    queue: Arc<JobQueue>,
}

impl QueueJobSource {
    /// Builds the source of the list of the jobs.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_desktop::jobs::QueueJobSource;
    /// use phocommander_ops::queue::JobQueue;
    /// use phocommander_ui::jobs::JobSource;
    /// use std::sync::Arc;
    ///
    /// let queue = Arc::new(JobQueue::new(Arc::new(phocommander_ops::fake_fs::FakeFileSystem::new()), 1));
    /// let source = QueueJobSource::new(queue);
    /// assert_eq!(source.summaries().len(), 0);
    /// ```
    pub fn new(queue: Arc<JobQueue>) -> Self {
        Self { queue }
    }
}

impl JobSource for QueueJobSource {
    fn summaries(&self) -> Vec<JobSummary> {
        self.queue.snapshot().iter().map(summary_of).collect()
    }
}

/// The row of the list that belongs to a snapshot of a job.
///
/// WHY: the conversion stands on its own, so that it can be tested without a
///       waiting room that runs a job. The surface knows the kind of the work
///       and the state of the job; the failure travels as the key of the
///       catalogue and the values of the message (FR-05).
fn summary_of(snapshot: &JobSnapshot) -> JobSummary {
    let progress = &snapshot.progress;
    let mut summary = JobSummary::new(
        snapshot.id.get(),
        kind_of(&snapshot.kind),
        state_of(snapshot.state),
        progress.items_done,
        progress.items_total,
    )
    .with_rate(progress.bytes_per_second)
    .with_eta(progress.eta_seconds);

    // WHY: the failure of a job is shown in its row. The error of the
    //       operations carries the key of the catalogue and the values that
    //       the message needs, and the surface reads the text from the
    //       catalogue.
    if let Some(error) = snapshot.errors.first() {
        summary = summary.with_error(JobError::new(
            error.code().message_key(),
            error.arguments().to_vec(),
        ));
    }

    summary
}

/// The work of the job, as the surface names it.
///
/// WHY: the work of the operations carries the paths and the mode, and the
///       surface names only the kind of the work. The row of the list shows
///       the kind, and the details of the work belong to the job itself.
fn kind_of(kind: &OpsJobKind) -> JobKind {
    match kind {
        OpsJobKind::Copy { .. } => JobKind::Copy,
        OpsJobKind::Move { .. } => JobKind::Move,
        OpsJobKind::Delete { .. } => JobKind::Delete,
        OpsJobKind::Rename { .. } => JobKind::Rename,
        OpsJobKind::Sync { .. } => JobKind::Sync,
        OpsJobKind::CreateDirectory { .. } => JobKind::CreateDirectory,
    }
}

/// The state of the job, as the surface names it.
fn state_of(state: OpsJobState) -> JobState {
    match state {
        OpsJobState::Queued => JobState::Queued,
        OpsJobState::Running => JobState::Running,
        OpsJobState::Verifying => JobState::Verifying,
        OpsJobState::Waiting => JobState::Waiting,
        OpsJobState::Done => JobState::Done,
        OpsJobState::Failed => JobState::Failed,
        OpsJobState::Cancelled => JobState::Cancelled,
    }
}

#[cfg(test)]
// WHY: the tests of the module live in the module itself, because the
//       adapter is the one place where the two crates meet.
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use phocommander_core::path::length::PathLengthLimit;
    use phocommander_ops::error::{JobError, JobErrorCode};
    use phocommander_ops::job::{JobId, JobProgress, Priority};
    use std::path::PathBuf;

    /// A snapshot of a job that failed because its path is too long.
    fn failed_snapshot() -> JobSnapshot {
        let long = PathBuf::from("/photos/ein-sehr-langer-ordner-der-nicht-mehr-passt");
        JobSnapshot {
            id: JobId::new(7),
            kind: phocommander_ops::job::JobKind::Copy {
                source: PathBuf::from("/photos/a.txt"),
                destination: long,
                mode: phocommander_ops::copy::CopyMode::Quick,
            },
            state: phocommander_ops::job::JobState::Failed,
            priority: Priority::Normal,
            progress: JobProgress {
                items_done: 0,
                items_total: 1,
                bytes_done: 0,
                bytes_total: 0,
                current_item: None,
                verification: None,
                bytes_per_second: None,
                eta_seconds: None,
            },
            errors: vec![
                JobError::new(
                    PathBuf::from("/photos/a.txt"),
                    JobErrorCode::PathTooLong,
                    "the path is too long",
                    false,
                )
                .with_arguments(vec![
                    ("path".to_string(), "/photos/a.txt".to_string()),
                    ("amount".to_string(), "4200".to_string()),
                    ("unit".to_string(), "bytes".to_string()),
                    (
                        "limit".to_string(),
                        PathLengthLimit::linux().amount().to_string(),
                    ),
                ]),
            ],
            history: Vec::new(),
            failed_items: Vec::new(),
            restricted_items: None,
        }
    }

    #[test]
    fn the_row_of_a_failed_job_carries_the_failure_and_its_values() {
        // act
        let row = summary_of(&failed_snapshot());

        // assert
        // WHY: the row of the list is the only place where the user learns
        //       that a job failed. It therefore carries the key of the
        //       message and the values that the message needs (FR-05).
        let error = row.error().expect("the row carries the failure");
        assert_eq!(error.key(), "path-too-long");
        let arguments = error.arguments();
        assert_eq!(arguments.len(), 4, "the message names the values");
        assert_eq!(arguments[0].0, "path");
        assert_eq!(arguments[3].0, "limit");
    }

    #[test]
    fn the_key_of_the_catalogue_is_the_name_of_the_code_with_a_hyphen() {
        // WHY: the codes of ADR-0003 carry a point, because they name a case
        //       in a table of cases. The keys of the catalogue carry a
        //       hyphen, because the format of the catalogue allows no point.
        assert_eq!(JobErrorCode::PathTooLong.as_str(), "path.too_long");
        assert_eq!(JobErrorCode::PathTooLong.message_key(), "path-too-long");
        assert_eq!(JobErrorCode::IoRead.message_key(), "io-read");
    }

    #[test]
    fn the_row_of_a_job_carries_the_rate_and_the_estimate() {
        // arrange
        let mut snapshot = failed_snapshot();
        snapshot.progress.bytes_per_second = Some(1_048_576);
        snapshot.progress.eta_seconds = Some(120);

        // act
        let row = summary_of(&snapshot);

        // assert
        // WHY: the rate and the estimate tell the user how long the job will
        //       run. Without them a job that copies ten gigabytes looks as if
        //       it hangs (FR-24, FR-22g).
        assert_eq!(row.rate(), Some(1_048_576));
        assert_eq!(row.eta(), Some(120));
    }

    #[test]
    fn a_job_that_has_not_run_long_enough_has_no_rate() {
        // arrange
        let mut snapshot = failed_snapshot();
        snapshot.progress.bytes_per_second = None;
        snapshot.progress.eta_seconds = None;

        // act
        let row = summary_of(&snapshot);

        // assert
        // WHY: the cue shows nothing instead of a number that would be wrong.
        assert_eq!(row.rate(), None);
        assert_eq!(row.eta(), None);
    }
}
