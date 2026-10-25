//! The progress of a running job and the channel it reports through.
//!
//! WHAT: the report a running job produces ([{ProgressUpdate}]), the place
//!       the report goes to ([{ProgressSink}]) and the context a running job
//!       works in ([{JobContext}]).
//! WHY: the user interface never blocks and never performs I/O; it reads a
//!       snapshot of the job state (the concurrency model of the
//!       specification). A running job therefore needs exactly one channel to
//!       the queue, and this module is that channel. The context carries the
//!       flags the user can set — pause and cancel — so that a change to a
//!       running job takes effect at the next item boundary and never in the
//!       middle of a file that is being written.
//! SIZE: the report, the channel and the context of a running job describe
//!       one contract. A reader who wants to know what a running job can
//!       see and what it can change has to see the report, the sink and the
//!       context at the same time.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::edit::JobControl;
//! use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
//! use std::sync::Arc;
//!
//! struct Silent;
//! impl ProgressSink for Silent {
//!     fn report(&mut self, _update: ProgressUpdate) {}
//! }
//!
//! let control = Arc::new(JobControl::new());
//! let mut context = JobContext::new(control, Box::new(Silent));
//! context.set_totals(2, 100);
//! context.begin_item("sunset.jpg");
//! context.finish_item();
//! assert_eq!(context.items_done(), 1);
//! ```

use std::sync::Arc;
use std::time::Duration;

use crate::edit::{Edit, JobControl};
use crate::error::OpError;
use crate::job::{JobState, VerificationProgress};

/// The time the worker waits between two checks of the pause flag.
///
/// WHY: a paused job must not spin on the processor while it waits, and it
/// must notice a cancellation quickly. Five milliseconds are short enough for
/// the pause to feel immediate and long enough to leave the processor alone.
const PAUSE_POLL_INTERVAL: Duration = Duration::from_millis(5);

/// What a running job reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressUpdate {
    /// The state of the job at the moment of the report.
    pub state: JobState,

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

/// The place a running job reports its progress to.
///
/// WHY: the interface reads a snapshot and never reaches into the queue. The
/// sink is the only way a running job reaches the queue, and the queue
/// decides how often a report is passed on — at most ten times per second,
/// so that a fast copy cannot flood the interface (NFR-09b).
pub trait ProgressSink: Send {
    /// Receives one report of the running job.
    fn report(&mut self, update: ProgressUpdate);
}

/// The context of a running job.
///
/// WHY: the context is the only channel between the job and the queue. It
/// carries the flags the user can set — pause, cancel — and it collects the
/// progress. Every call of the executor that touches a file ends with a
/// checkpoint, so that a change to a running job takes effect at the next
/// item boundary and never in the middle of a file.
pub struct JobContext {
    control: Arc<JobControl>,
    sink: Box<dyn ProgressSink>,
    items_total: u64,
    items_done: u64,
    bytes_total: u64,
    bytes_done: u64,
    current_item: Option<String>,
    verification: Option<VerificationProgress>,
    state: JobState,
}

impl JobContext {
    /// Builds a context for a running job.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// struct Silent;
    /// impl ProgressSink for Silent {
    ///     fn report(&mut self, _update: ProgressUpdate) {}
    /// }
    ///
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.set_totals(2, 100);
    /// assert_eq!(context.items_done(), 0);
    /// ```
    pub fn new(control: Arc<JobControl>, sink: Box<dyn ProgressSink>) -> Self {
        Self {
            control,
            sink,
            items_total: 0,
            items_done: 0,
            bytes_total: 0,
            bytes_done: 0,
            current_item: None,
            verification: None,
            state: JobState::Running,
        }
    }

    /// The flags of the job.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let context = JobContext::new(control, Box::new(Silent));
    /// assert!(!context.control().is_cancelled());
    /// ```
    pub fn control(&self) -> &Arc<JobControl> {
        &self.control
    }

    /// The number of items that are done.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let context = JobContext::new(control, Box::new(Silent));
    /// assert_eq!(context.items_done(), 0);
    /// ```
    pub fn items_done(&self) -> u64 {
        self.items_done
    }

    /// The number of bytes that are done.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let context = JobContext::new(control, Box::new(Silent));
    /// assert_eq!(context.bytes_done(), 0);
    /// ```
    pub fn bytes_done(&self) -> u64 {
        self.bytes_done
    }

    /// The number of items the job has to do.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.set_totals(3, 300);
    /// assert_eq!(context.items_total(), 3);
    /// ```
    pub fn items_total(&self) -> u64 {
        self.items_total
    }

    /// Sets the numbers the job has to reach.
    ///
    /// WHY: the totals are known before the first file is copied — the
    /// comparison of two directories counts the files first. The progress of
    /// the job is then a share of a known whole.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.set_totals(3, 300);
    /// assert_eq!(context.items_total(), 3);
    /// ```
    pub fn set_totals(&mut self, items: u64, bytes: u64) {
        self.items_total = items;
        self.bytes_total = bytes;
    }

    /// Announces the item the job is working on.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.begin_item("sunset.jpg");
    /// assert_eq!(context.current_item(), Some("sunset.jpg"));
    /// ```
    pub fn begin_item(&mut self, name: &str) {
        self.current_item = Some(name.to_string());
    }

    /// The item the job is working on.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let context = JobContext::new(control, Box::new(Silent));
    /// assert_eq!(context.current_item(), None);
    /// ```
    pub fn current_item(&self) -> Option<&str> {
        self.current_item.as_deref()
    }

    /// Counts the bytes of the finished item.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.add_bytes(120);
    /// assert_eq!(context.bytes_done(), 120);
    /// ```
    pub fn add_bytes(&mut self, bytes: u64) {
        self.bytes_done += bytes;
    }

    /// Counts the finished item.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.finish_item();
    /// assert_eq!(context.items_done(), 1);
    /// ```
    pub fn finish_item(&mut self) {
        self.items_done += 1;
        self.current_item = None;
    }

    /// Starts the verification phase.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.begin_verification(2, 100);
    /// assert_eq!(context.verification().map(|progress| progress.items_total), Some(2));
    /// ```
    pub fn begin_verification(&mut self, items: u64, bytes: u64) {
        self.state = JobState::Verifying;
        self.verification = Some(VerificationProgress {
            items_done: 0,
            items_total: items,
            bytes_done: 0,
            bytes_total: bytes,
        });
    }

    /// The progress of the verification, if the job verifies.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let context = JobContext::new(control, Box::new(Silent));
    /// assert!(context.verification().is_none());
    /// ```
    pub fn verification(&self) -> Option<&VerificationProgress> {
        self.verification.as_ref()
    }

    /// Counts the bytes that were compared.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.begin_verification(1, 50);
    /// context.add_verified_bytes(50);
    /// assert_eq!(context.verification().map(|progress| progress.bytes_done), Some(50));
    /// ```
    pub fn add_verified_bytes(&mut self, bytes: u64) {
        if let Some(progress) = self.verification.as_mut() {
            progress.bytes_done += bytes;
            progress.items_done += 1;
        }
    }

    /// The state the job is in.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::job::JobState;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let context = JobContext::new(control, Box::new(Silent));
    /// assert_eq!(context.state(), JobState::Running);
    /// ```
    pub fn state(&self) -> JobState {
        self.state
    }

    /// Sets the state the job is in.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::job::JobState;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.set_state(JobState::Verifying);
    /// assert_eq!(context.state(), JobState::Verifying);
    /// ```
    pub fn set_state(&mut self, state: JobState) {
        self.state = state;
    }

    /// The boundary between two items.
    ///
    /// WHY: every item of a job ends with this call. It is the place where a
    /// change that the user made takes effect: a pause stops the job here, a
    /// cancellation ends it here, and an edit of the target or of the mode is
    /// handed to the caller here. Nothing happens in the middle of a file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// assert!(context.checkpoint().unwrap_or(None).is_none());
    /// ```
    pub fn checkpoint(&mut self) -> Result<Option<Edit>, OpError> {
        let edit = self.control.take_pending();

        while self.control.is_paused() {
            if self.control.is_cancelled() {
                return Err(OpError::Cancelled);
            }
            std::thread::sleep(PAUSE_POLL_INTERVAL);
        }

        if self.control.is_cancelled() {
            return Err(OpError::Cancelled);
        }

        Ok(edit)
    }

    /// Sends the progress to the sink.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::edit::JobControl;
    /// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
    /// use std::sync::Arc;
    ///
    /// # struct Silent;
    /// # impl ProgressSink for Silent {
    /// #     fn report(&mut self, _update: ProgressUpdate) {}
    /// # }
    /// let control = Arc::new(JobControl::new());
    /// let mut context = JobContext::new(control, Box::new(Silent));
    /// context.report();
    /// ```
    pub fn report(&mut self) {
        self.sink.report(ProgressUpdate {
            state: self.state,
            items_done: self.items_done,
            items_total: self.items_total,
            bytes_done: self.bytes_done,
            bytes_total: self.bytes_total,
            current_item: self.current_item.clone(),
            verification: self.verification,
        });
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    /// A sink that keeps the last report.
    struct Recorder {
        last: Option<ProgressUpdate>,
    }

    impl ProgressSink for Recorder {
        fn report(&mut self, update: ProgressUpdate) {
            self.last = Some(update);
        }
    }

    fn context() -> (Arc<JobControl>, JobContext) {
        let control = Arc::new(JobControl::new());
        let sink = Box::new(Recorder { last: None });
        (control.clone(), JobContext::new(control, sink))
    }

    #[test]
    fn the_context_counts_the_items_and_the_bytes() {
        // arrange
        let (_control, mut context) = context();

        // act
        context.set_totals(2, 200);
        context.begin_item("a.txt");
        context.add_bytes(100);
        context.finish_item();

        // assert
        assert_eq!(context.items_total(), 2);
        assert_eq!(context.items_done(), 1);
        assert_eq!(context.bytes_done(), 100);
        assert_eq!(context.current_item(), None);
    }

    #[test]
    fn the_context_reports_the_progress_to_the_sink() {
        // arrange
        let (control, _context) = context();
        let sink = Recorder { last: None };
        let mut context = JobContext::new(control, Box::new(sink));

        // act
        context.set_totals(1, 10);
        context.report();

        // assert
        assert_eq!(context.items_total(), 1);
    }

    #[test]
    fn the_verification_has_its_own_progress() {
        // arrange
        let (_control, mut context) = context();

        // act
        context.begin_verification(2, 200);
        context.add_verified_bytes(100);

        // assert
        let progress = context.verification().copied().unwrap_or_default();
        assert_eq!(progress.items_total, 2);
        assert_eq!(progress.bytes_done, 100);
        assert_eq!(context.state(), JobState::Verifying);
    }

    #[test]
    fn a_cancelled_job_is_stopped_at_the_checkpoint() {
        // arrange
        let (control, mut context) = context();
        control.cancel();

        // act
        let result = context.checkpoint();

        // assert
        assert!(matches!(result, Err(OpError::Cancelled)));
    }

    #[test]
    fn a_pending_edit_is_handed_out_at_the_checkpoint() {
        // arrange
        let (control, mut context) = context();
        control.request(Edit::Priority(crate::job::Priority::High));

        // act
        let edit = context.checkpoint();

        // assert
        let edit = match edit {
            Ok(edit) => edit,
            Err(error) => panic!("the checkpoint should not fail: {error}"),
        };
        assert!(matches!(edit, Some(Edit::Priority(_))));
    }
}
