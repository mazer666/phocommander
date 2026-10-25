//! The workers of the queue.
//!
//! WHAT: the loop of a worker ([`worker_loop`]), the choice of the job that
//!       runs next ([`pick_next`]), the writing of the result of a job back
//!       into the queue ([`finish`]) and the sink that carries the progress of
//!       a running job into the queue ([`QueueSink`]).
//! WHY: the queue owns the jobs and the workers; the worker owns the loop
//!       that carries a job out. The two live in two files, because the file
//!       that does the work carries the work and not the bookkeeping.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::queue::JobQueue;
//! use phocommander_ops::fake_fs::FakeFileSystem;
//! use std::sync::Arc;
//!
//! let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
//! assert!(queue.snapshot().is_empty());
//! ```
//! BOUNDARY: the worker performs the file operation of a job through the
//!       [{crate::port::FileSystem}] that the composition root hands over.

use std::sync::Arc;
use std::time::Instant;

use super::{IDLE_WAIT, QueueInner, QueueState, REPORT_INTERVAL};
use crate::edit::JobControl;
use crate::executor::{self, JobResult};
use crate::job::{JobId, JobSnapshot, JobState, Priority};
use crate::port::FileSystem;
use crate::progress::{JobContext, ProgressSink, ProgressUpdate};

/// The worker loop: it waits for a job and carries it out.
pub(super) fn worker_loop(inner: Arc<QueueInner>, fs: Arc<dyn FileSystem>) {
    loop {
        let next = {
            let mut state = inner
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            loop {
                if state.shutting_down {
                    return;
                }
                if let Some(index) = pick_next(&state) {
                    break Some(index);
                }
                let (guard, _timeout) = inner
                    .wake
                    .wait_timeout(state, IDLE_WAIT)
                    .unwrap_or_else(|error| error.into_inner());
                state = guard;
            }
        };

        let Some(index) = next else {
            continue;
        };

        let (job, control) = {
            let mut state = inner
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let record = &mut state.records[index];
            record.job.state = JobState::Running;
            (record.job.clone(), Arc::clone(&record.control))
        };

        let sink = QueueSink {
            inner: Arc::clone(&inner),
            id: job.id,
            last_report: Instant::now(),
        };
        let mut context = JobContext::new(control.clone(), Box::new(sink));

        let result = executor::execute(fs.as_ref(), &job, &mut context);
        finish(&inner, job.id, result, &control);
    }
}

/// Picks the job that runs next.
///
/// WHY: the queue runs the job with the highest priority first, and within
/// one priority the job that was submitted first.
fn pick_next(state: &QueueState) -> Option<usize> {
    let mut best: Option<(usize, Priority)> = None;
    for (index, record) in state.records.iter().enumerate() {
        if record.job.state != JobState::Queued {
            continue;
        }
        let better = match best {
            None => true,
            Some((_, priority)) => record.job.priority > priority,
        };
        if better {
            best = Some((index, record.job.priority));
        }
    }
    best.map(|(index, _)| index)
}

/// Writes the result of a job back into the queue.
fn finish(
    inner: &Arc<QueueInner>,
    id: JobId,
    result: Result<JobResult, crate::error::OpError>,
    control: &Arc<JobControl>,
) {
    let mut state = inner
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner());

    let Some(record) = state.records.iter_mut().find(|record| record.job.id == id) else {
        return;
    };

    match result {
        Ok(result) => {
            record.job.state = result.state;
            record.job.errors.extend(result.errors);
            record.job.progress.items_done = result.items_done;
            record.job.progress.bytes_done = result.bytes_done;
            // WHY: the job remembers the items that failed, so that the
            // retry of the failed items (FR-22k) knows what it has to run
            // again. The items are taken from the failures, and a failure
            // without an item is left out.
            record.job.failed_items = record
                .job
                .errors
                .iter()
                .map(|error| error.item.clone())
                .filter(|item| !item.as_os_str().is_empty())
                .collect();
        }
        Err(crate::error::OpError::Cancelled) => {
            record.job.state = JobState::Cancelled;
        }
        Err(error) => {
            record.job.state = JobState::Failed;
            record.job.errors.push(crate::error::JobError::new(
                std::path::PathBuf::new(),
                error.code(),
                error.to_string(),
                false,
            ));
        }
    }

    if control.is_cancelled() && record.job.state != JobState::Cancelled {
        record.job.state = JobState::Cancelled;
    }
}

/// The sink that writes the progress of a running job into the queue.
struct QueueSink {
    inner: Arc<QueueInner>,
    id: JobId,
    last_report: Instant,
}

impl ProgressSink for QueueSink {
    fn report(&mut self, update: ProgressUpdate) {
        let now = Instant::now();
        if now.duration_since(self.last_report) < REPORT_INTERVAL {
            return;
        }
        self.last_report = now;

        let mut state = self
            .inner
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(record) = state
            .records
            .iter_mut()
            .find(|record| record.job.id == self.id)
        else {
            return;
        };
        record.job.progress.items_done = update.items_done;
        record.job.progress.items_total = update.items_total;
        record.job.progress.bytes_done = update.bytes_done;
        record.job.progress.bytes_total = update.bytes_total;
        record.job.progress.current_item = update.current_item;
        record.job.progress.verification = update.verification;
        if update.state != JobState::Running {
            record.job.state = update.state;
        }
    }
}

/// The order in which the jobs of the queue are shown.
///
/// WHY: the job list can be reordered, filtered by state and inspected. The
/// queue answers with the jobs in the order of their priority, so that the
/// list shows what is going to happen next.
pub fn ordered_snapshots(snapshots: &[JobSnapshot]) -> Vec<JobSnapshot> {
    let mut ordered: Vec<JobSnapshot> = snapshots.to_vec();
    ordered.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.id.cmp(&right.id))
    });
    ordered
}

/// The jobs of a snapshot that have not finished.
///
/// WHY: the job list filters by state, and the filter that matters most is
/// the one that shows what is still to come.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::queue::worker::unfinished;
///
/// let snapshots: Vec<phocommander_ops::job::JobSnapshot> = Vec::new();
/// assert!(unfinished(&snapshots).is_empty());
/// ```
pub fn unfinished(snapshots: &[JobSnapshot]) -> Vec<&JobSnapshot> {
    snapshots
        .iter()
        .filter(|snapshot| !snapshot.state.is_finished())
        .collect()
}
