//! The changes a user may make to a job that has not finished.
//!
//! WHAT: the changes themselves ([{Edit}]), the conflict policy they can
//!       carry ([{ConflictPolicy}]), the flags a running job shares with the
//!       queue ([{JobControl}]) and the rule that decides whether a change
//!       is allowed in the state the job is in ([{EditError}],
//!       [{check_edit}]).
//! WHY: a job that was submitted is not frozen. A queue of twenty copy
//!       operations often contains one with the wrong target or the wrong
//!       mode, and discovering that after the last one has run is the worst
//!       possible moment (the editable jobs of the specification). The rule
//!       of the states is written down once, so that the job list and the
//!       queue cannot disagree about what may be changed.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::edit::{Edit, EditError, check_edit};
//! use phocommander_ops::job::{JobState, Priority};
//!
//! // A queued job may be retargeted.
//! assert!(check_edit(&JobState::Queued, &Edit::Priority(Priority::High)).is_ok());
//!
//! // A running job may not.
//! assert!(matches!(
//!     check_edit(&JobState::Running, &Edit::Priority(Priority::High)),
//!     Err(EditError::WrongState { .. })
//! ));
//! ```

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::copy::CopyMode;
use crate::job::{JobState, Priority};

/// What the user may change in a job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
/// The job moves within the queue.
    Priority(Priority),

/// The job copies into another directory.
    Destination(PathBuf),

/// The job resolves a conflict in another way.
    ConflictPolicy(ConflictPolicy),

/// The job copies in another mode.
    VerificationMode(CopyMode),

/// The job stops at the next item boundary.
    Pause,

/// The job continues at the item it stopped at.
    Resume,

/// The job ends. What it has done already stays done.
    Cancel,

/// Only the failed items of the job run again.
    RetryFailed,
}

impl Edit {
/// The name of the edit, for the history of the job.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::Edit;
/// use phocommander_ops::job::Priority;
///
/// assert_eq!(Edit::Priority(Priority::High).as_str(), "priority");
/// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Edit::Priority(_) => "priority",
            Edit::Destination(_) => "destination",
            Edit::ConflictPolicy(_) => "conflict_policy",
            Edit::VerificationMode(_) => "verification_mode",
            Edit::Pause => "pause",
            Edit::Resume => "resume",
            Edit::Cancel => "cancel",
            Edit::RetryFailed => "retry_failed",
        }
    }
}

/// The conflict policy of a job.
///
/// WHY: the policy is part of the operation and not of the dialog that asks
/// the user, so that a job that was queued with a policy keeps it even when
/// the dialog is gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ConflictPolicy {
/// The user decides.
    #[default]
    Ask,

/// An existing file is never overwritten.
    Never,

/// An existing file is always overwritten.
    Always,

/// The copy receives a name that does not exist yet.
    KeepBoth,
}

impl ConflictPolicy {
/// The name of the policy.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::ConflictPolicy;
///
/// assert_eq!(ConflictPolicy::KeepBoth.as_str(), "keep_both");
/// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            ConflictPolicy::Ask => "ask",
            ConflictPolicy::Never => "never",
            ConflictPolicy::Always => "always",
            ConflictPolicy::KeepBoth => "keep_both",
        }
    }
}

/// The reason why a change was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EditError {
/// The job does not exist.
    #[error("there is no job with the identifier {id}")]
    UnknownJob {
/// The identifier that was asked for.
        id: u64,
    },

/// The change is not allowed in the state the job is in.
    #[error("the change {edit} is not allowed while the job is {state}")]
    WrongState {
/// The name of the change.
        edit: &'static str,
/// The state of the job.
        state: JobState,
    },

/// The change cannot be carried out.
    #[error("the change {edit} cannot be carried out: {reason}")]
    NotPossible {
/// The name of the change.
        edit: &'static str,
/// The reason why the change cannot be carried out.
        reason: String,
    },
}

/// Decides whether a change is allowed in the state the job is in.
///
/// WHY: the table of the specification says which change is allowed while:
//! the target, the priority, the conflict policy and the verification mode
/// while the job is queued, the pause and the resume while it runs, the
/// cancellation while it is queued or running, and the retry while it has
/// failed. Writing the rule down once keeps the job list and the queue from
/// disagreeing about it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::{Edit, check_edit};
/// use phocommander_ops::job::{JobState, Priority};
///
/// assert!(check_edit(&JobState::Queued, &Edit::Cancel).is_ok());
/// assert!(check_edit(&JobState::Running, &Edit::Cancel).is_ok());
/// assert!(check_edit(&JobState::Done, &Edit::Cancel).is_err());
/// ```
pub fn check_edit(state: &JobState, edit: &Edit) -> Result<(), EditError> {
    let allowed = match edit {
        Edit::Priority(_) | Edit::Destination(_) | Edit::ConflictPolicy(_) | Edit::VerificationMode(_) => {
            *state == JobState::Queued
        }
        Edit::Pause => *state == JobState::Running || *state == JobState::Waiting,
        Edit::Resume => *state == JobState::Running || *state == JobState::Waiting,
        Edit::Cancel => *state == JobState::Queued || *state == JobState::Running,
        Edit::RetryFailed => *state == JobState::Failed,
    };

    if allowed {
        return Ok(());
    }

    Err(EditError::WrongState {
        edit: edit.as_str(),
        state: *state,
    })
}

/// The flags a running job shares with the queue.
///
/// WHY: the queue and the worker both have to see a cancellation, and the
/// user interface must be able to set a pause while the worker is inside a
/// file. The flags therefore live in a structure that both sides hold, and
/// the change that a running job cannot take immediately — a new target, a
/// new mode — waits in a slot until the worker reaches the next item
/// boundary.
#[derive(Debug, Default)]
pub struct JobControl {
    cancelled: AtomicBool,
    paused: AtomicBool,
    pending: Mutex<Option<Edit>>,
}

impl JobControl {
/// Builds the flags of a job.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::JobControl;
///
/// let control = JobControl::new();
/// assert!(!control.is_cancelled());
/// assert!(!control.is_paused());
/// ```
    pub fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            pending: Mutex::new(None),
        }
    }

/// Ends the job.
///
/// WHY: the flag is set, never taken. A cancellation that a worker
/// notices late still ends the job, and the work that was done already
/// stays done.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::JobControl;
///
/// let control = JobControl::new();
/// control.cancel();
/// assert!(control.is_cancelled());
/// ```
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

/// Whether the job was cancelled.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::JobControl;
///
/// assert!(!JobControl::new().is_cancelled());
/// ```
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

/// Stops the job at the next item boundary.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::JobControl;
///
/// let control = JobControl::new();
/// control.pause();
/// assert!(control.is_paused());
/// ```
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

/// Lets the job continue.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::JobControl;
///
/// let control = JobControl::new();
/// control.pause();
/// control.resume();
/// assert!(!control.is_paused());
/// ```
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }

/// Whether the job is paused.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::JobControl;
///
/// assert!(!JobControl::new().is_paused());
/// ```
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

/// Stores a change that a running job takes over at its next boundary.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::{Edit, JobControl};
/// use phocommander_ops::job::Priority;
///
/// let control = JobControl::new();
/// control.request(Edit::Priority(Priority::High));
/// assert!(control.pending().is_some());
/// ```
    pub fn request(&self, edit: Edit) {
        let mut slot = self.pending.lock().unwrap_or_else(|error| error.into_inner());
        *slot = Some(edit);
    }

/// The change that waits for the next item boundary.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::{Edit, JobControl};
/// use phocommander_ops::job::Priority;
///
/// let control = JobControl::new();
/// control.request(Edit::Priority(Priority::High));
/// assert!(control.pending().is_some());
/// ```
    pub fn pending(&self) -> Option<Edit> {
        let slot = self.pending.lock().unwrap_or_else(|error| error.into_inner());
        slot.clone()
    }

/// Takes the waiting change and hands it to the worker.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::{Edit, JobControl};
/// use phocommander_ops::job::Priority;
///
/// let control = JobControl::new();
/// control.request(Edit::Priority(Priority::High));
/// assert!(control.take_pending().is_some());
/// assert!(control.take_pending().is_none());
/// ```
    pub fn take_pending(&self) -> Option<Edit> {
        let mut slot = self.pending.lock().unwrap_or_else(|error| error.into_inner());
        slot.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_queued_job_may_be_retargeted_and_reprioritised() {
        // act, assert
        assert!(check_edit(&JobState::Queued, &Edit::Priority(Priority::High)).is_ok());
        assert!(
            check_edit(&JobState::Queued, &Edit::Destination(PathBuf::from("/other"))).is_ok()
        );
        assert!(
            check_edit(&JobState::Queued, &Edit::ConflictPolicy(ConflictPolicy::Never)).is_ok()
        );
        assert!(
            check_edit(&JobState::Queued, &Edit::VerificationMode(CopyMode::Atomic)).is_ok()
        );
    }

    #[test]
    fn a_running_job_may_not_be_retargeted() {
        // act
        let result = check_edit(&JobState::Running, &Edit::Priority(Priority::High));

        // assert
        assert!(matches!(result, Err(EditError::WrongState { .. })), "{result:?}");
    }

    #[test]
    fn a_running_job_may_be_paused_resumed_and_cancelled() {
        // act, assert
        assert!(check_edit(&JobState::Running, &Edit::Pause).is_ok());
        assert!(check_edit(&JobState::Running, &Edit::Resume).is_ok());
        assert!(check_edit(&JobState::Running, &Edit::Cancel).is_ok());
        assert!(check_edit(&JobState::Queued, &Edit::Cancel).is_ok());
    }

    #[test]
    fn only_a_failed_job_may_retry_its_failed_items() {
        // act, assert
        assert!(check_edit(&JobState::Failed, &Edit::RetryFailed).is_ok());
        assert!(check_edit(&JobState::Done, &Edit::RetryFailed).is_err());
        assert!(check_edit(&JobState::Running, &Edit::RetryFailed).is_err());
    }

    #[test]
    fn the_flags_of_a_job_report_what_the_user_did() {
        // arrange
        let control = JobControl::new();

        // act
        control.cancel();
        control.pause();

        // assert
        assert!(control.is_cancelled());
        assert!(control.is_paused());
    }

    #[test]
    fn a_resumed_job_is_not_paused_any_more() {
        // arrange
        let control = JobControl::new();
        control.pause();

        // act
        control.resume();

        // assert
        assert!(!control.is_paused());
    }

    #[test]
    fn a_waiting_change_is_handed_out_once() {
        // arrange
        let control = JobControl::new();
        control.request(Edit::Priority(Priority::High));

        // act
        let first = control.take_pending();
        let second = control.take_pending();

        // assert
        assert!(matches!(first, Some(Edit::Priority(Priority::High))));
        assert!(second.is_none());
    }
}
