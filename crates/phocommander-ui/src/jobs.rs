//! The list of the jobs.
//!
//! WHAT: the state of a job as the list shows it ([`JobState`]), the work a
//!       job does ([`JobKind`]), one row of the list ([`JobSummary`]) and the
//!       source that fills the list ([`JobSource`]).
//! WHY: the interface may not depend on the operations crate — no crate
//!       below `crates/` may depend on a sibling — so the queue of the
//!       operations reaches the surface as a list of summaries. The surface
//!       therefore draws rows and knows nothing about the queue.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::jobs::{JobKind, JobState, JobSummary};
//!
//! let summary = JobSummary::new(7, JobKind::Copy, JobState::Running, 3, 10);
//! assert_eq!(summary.fraction(), 0.3);
//! ```

use crate::i18n::TextKey;

/// The state of a job, as the list shows it.
///
/// WHY: the states are the ones the job cue of the specification names. The
/// verification is a state of its own, because the phase has its own
/// progress and the user has to see that the copy is not stuck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// Every state, in a stable order.
    pub const fn all() -> [JobState; 7] {
        [
            JobState::Queued,
            JobState::Running,
            JobState::Verifying,
            JobState::Waiting,
            JobState::Done,
            JobState::Failed,
            JobState::Cancelled,
        ]
    }

    /// The translation key of the name of the state.
    ///
    /// WHY: the name of a state is a visible text and goes through a
    /// translation key (FR-05).
    pub const fn text_key(&self) -> TextKey {
        match self {
            JobState::Queued => TextKey::JobStateQueued,
            JobState::Running => TextKey::JobStateRunning,
            JobState::Verifying => TextKey::JobStateVerifying,
            JobState::Waiting => TextKey::JobStateWaiting,
            JobState::Done => TextKey::JobStateDone,
            JobState::Failed => TextKey::JobStateFailed,
            JobState::Cancelled => TextKey::JobStateCancelled,
        }
    }
}

/// The work a job does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JobKind {
    /// The job copies files or a directory tree.
    Copy,

    /// The job moves files or a directory tree.
    Move,

    /// The job deletes files or directories.
    Delete,

    /// The job renames a file.
    Rename,

    /// The job brings two directories into the same state.
    Sync,

    /// The job creates a directory.
    CreateDirectory,
}

impl JobKind {
    /// Every kind, in a stable order.
    pub const fn all() -> [JobKind; 6] {
        [
            JobKind::Copy,
            JobKind::Move,
            JobKind::Delete,
            JobKind::Rename,
            JobKind::Sync,
            JobKind::CreateDirectory,
        ]
    }

    /// The translation key of the name of the kind.
    ///
    /// WHY: the name of a kind is a visible text and goes through a
    /// translation key (FR-05).
    pub const fn text_key(&self) -> TextKey {
        match self {
            JobKind::Copy => TextKey::JobKindCopy,
            JobKind::Move => TextKey::JobKindMove,
            JobKind::Delete => TextKey::JobKindDelete,
            JobKind::Rename => TextKey::JobKindRename,
            JobKind::Sync => TextKey::JobKindSync,
            JobKind::CreateDirectory => TextKey::JobKindCreateDirectory,
        }
    }
}

/// The failure of a job, as the surface needs it.
///
/// WHY: the surface knows nothing about the crate of the operations. The
///       failure therefore travels as a key of the catalogue and the values
///       that the message needs, and the surface reads the text in the
///       language of the interface (FR-05, FR-124).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobError {
    /// The key of the catalogue that carries the message.
    key: String,

    /// The values that the message carries, with the name that the message
    /// of the catalogue expects.
    arguments: Vec<(String, String)>,
}

impl JobError {
    /// Builds the failure of a job.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::jobs::JobError;
    ///
    /// let error = JobError::new(
    ///     "path-too-long",
    ///     vec![("path".to_string(), "/a/b".to_string())],
    /// );
    /// assert_eq!(error.key(), "path-too-long");
    /// ```
    pub fn new(key: impl Into<String>, arguments: Vec<(String, String)>) -> Self {
        Self {
            key: key.into(),
            arguments,
        }
    }

    /// The key of the catalogue that carries the message.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The values that the message carries, with their names.
    pub fn arguments(&self) -> &[(String, String)] {
        &self.arguments
    }
}

/// One row of the list of the jobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSummary {
    id: u64,
    kind: JobKind,
    state: JobState,
    items_done: u64,
    items_total: u64,
    rate: Option<u64>,
    error: Option<JobError>,
}

impl JobSummary {
    /// A row of the list.
    pub const fn new(
        id: u64,
        kind: JobKind,
        state: JobState,
        items_done: u64,
        items_total: u64,
    ) -> Self {
        Self {
            id,
            kind,
            state,
            items_done,
            items_total,
            rate: None,
            error: None,
        }
    }

    /// The row with the failure of the job.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::jobs::{JobError, JobSummary};
    ///
    /// let error = JobError::new("path-too-long", Vec::new());
    /// let summary = JobSummary::new(1, phocommander_ui::jobs::JobKind::Copy, phocommander_ui::jobs::JobState::Failed, 0, 0)
    ///     .with_error(error);
    /// assert!(summary.error().is_some());
    /// ```
    pub fn with_error(mut self, error: JobError) -> Self {
        self.error = Some(error);
        self
    }

    /// The failure of the job, if it failed.
    pub fn error(&self) -> Option<&JobError> {
        self.error.as_ref()
    }

    /// The row with the rate of the job.
    pub const fn with_rate(mut self, rate: Option<u64>) -> Self {
        self.rate = rate;
        self
    }

    /// The number of the job.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// The work the job does.
    pub const fn kind(&self) -> JobKind {
        self.kind
    }

    /// The state of the job.
    pub const fn state(&self) -> JobState {
        self.state
    }

    /// The number of items that are done.
    pub const fn items_done(&self) -> u64 {
        self.items_done
    }

    /// The number of items that have to be done.
    pub const fn items_total(&self) -> u64 {
        self.items_total
    }

    /// The rate of the job in bytes per second.
    pub const fn rate(&self) -> Option<u64> {
        self.rate
    }

    /// The share of the work that is done, between zero and one.
    ///
    /// WHY: a job without a known total has no share. The answer is zero
    /// then, and not a number that pretends to know something.
    pub fn fraction(&self) -> f32 {
        if self.items_total == 0 {
            return 0.0;
        }
        (self.items_done as f32 / self.items_total as f32).clamp(0.0, 1.0)
    }
}

/// The source that fills the list of the jobs.
///
/// WHY: the trait is the seam between the surface and the queue of the
/// operations. The surface asks for the rows it draws and the source
/// answers with them.
pub trait JobSource {
    /// The rows of the list of the jobs.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::jobs::{JobSource, JobSummary};
    ///
    /// # struct Empty;
    /// # impl JobSource for Empty {
    /// #     fn summaries(&self) -> Vec<JobSummary> { Vec::new() }
    /// # }
    /// # let source = Empty;
    /// assert!(source.summaries().is_empty());
    /// ```
    fn summaries(&self) -> Vec<JobSummary>;
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_share_of_a_job_without_a_total_is_zero() {
        // act
        let summary = JobSummary::new(1, JobKind::Copy, JobState::Queued, 0, 0);

        // assert
        assert_eq!(summary.fraction(), 0.0);
    }

    #[test]
    fn the_share_of_a_job_never_leaves_the_range() {
        // act
        let summary = JobSummary::new(1, JobKind::Copy, JobState::Running, 30, 10);

        // assert
        assert_eq!(summary.fraction(), 1.0);
    }

    #[test]
    fn every_state_and_every_kind_has_a_translation_key() {
        // act
        let states: Vec<TextKey> = JobState::all().iter().map(JobState::text_key).collect();
        let kinds: Vec<TextKey> = JobKind::all().iter().map(JobKind::text_key).collect();

        // assert
        let mut unique_states = states.clone();
        unique_states.sort_unstable();
        unique_states.dedup();
        assert_eq!(unique_states.len(), states.len());

        let mut unique_kinds = kinds.clone();
        unique_kinds.sort_unstable();
        unique_kinds.dedup();
        assert_eq!(unique_kinds.len(), kinds.len());
    }
}
