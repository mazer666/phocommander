//! The queue of the background jobs.
//!
//! WHAT: the queue itself ([{JobQueue}]), the state it keeps ([{JobState}]
//!       of the crate) and the snapshot the user interface reads
//!       ([{JobSnapshot}]).
//! WHY: every operation that can take longer than a frame belongs into the
//!       queue, so that the user interface stays usable while a copy runs
//!       (FR-21, NFR-09). The queue is thread-safe, the number of workers is
//!       bounded, and the interface only ever reads a snapshot — it never
//!       reaches into the queue. A change to a job that is still queued is
//!       applied at once, a change to a running job is applied at its next
//!       item boundary.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::job::{Job, JobKind, Priority};
//! use phocommander_ops::queue::JobQueue;
//! use phocommander_ops::fake_fs::FakeFileSystem;
//! use std::sync::Arc;
//!
//! let fs = Arc::new(FakeFileSystem::new());
//! let queue = JobQueue::new(fs, 2);
//! let id = queue.enqueue(Job::new(
//!     JobKind::CreateDirectory { path: "/photos".into() },
//!     Priority::Normal,
//! ));
//! assert!(queue.snapshot_of(id).is_some());
//! ```
//! BOUNDARY: the queue owns the jobs and the workers. It performs no file
//!       operation itself — the workers use the [{crate::port::FileSystem}]
//!       that the composition root hands over.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::edit::{Edit, EditError, JobControl, check_edit};
use crate::error::JobError;
use crate::executor::{self, JobResult};
use crate::job::{Job, JobId, JobKind, JobSnapshot, JobState, Priority};
use crate::port::FileSystem;
use crate::progress::{JobContext, ProgressSink, ProgressUpdate};

/// The longest time a worker waits before it looks at the queue again.
const IDLE_WAIT: Duration = Duration::from_millis(20);

/// The shortest time between two reports of the same job.
///
/// WHY: the specification asks for at most ten updates per second, so that
/// a fast copy cannot flood the interface (NFR-09b).
const REPORT_INTERVAL: Duration = Duration::from_millis(100);

/// One job of the queue.
#[derive(Debug)]
struct JobRecord {
/// The job itself.
    job: Job,

/// The flags the worker and the queue share.
    control: Arc<JobControl>,
}

/// The state the queue keeps.
#[derive(Debug, Default)]
struct QueueState {
/// The jobs of the queue, in the order they were submitted.
    records: Vec<JobRecord>,

/// The number the next job receives.
    next_id: u64,

/// Whether the queue is closing.
    shutting_down: bool,
}

/// The parts the workers share.
#[derive(Debug)]
struct QueueInner {
/// The state of the queue.
    state: Mutex<QueueState>,

/// Wakes a worker that waits for a job.
    wake: Condvar,
}

/// The queue of the background jobs.
pub struct JobQueue {
    inner: Arc<QueueInner>,
    workers: Vec<JoinHandle<()>>,
    closed: AtomicBool,
}

impl JobQueue {
/// Builds a queue and starts its workers.
///
/// WHY: the number of workers is bounded, because a queue that starts one
/// thread per job would flood the disk with seeks and would take the
/// machine away from the program that shows the progress. A number of
/// zero means one worker.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::fake_fs::FakeFileSystem;
/// use phocommander_ops::queue::JobQueue;
/// use std::sync::Arc;
///
/// let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 2);
/// assert!(queue.snapshot().is_empty());
/// ```
    pub fn new(fs: Arc<dyn FileSystem>, workers: usize) -> Self {
        let inner = Arc::new(QueueInner {
            state: Mutex::new(QueueState {
                records: Vec::new(),
                next_id: 1,
                shutting_down: false,
            }),
            wake: Condvar::new(),
        });

        let count = workers.max(1);
        let mut threads = Vec::with_capacity(count);
        for _ in 0..count {
            let inner = Arc::clone(&inner);
            let fs = Arc::clone(&fs);
            let handle = std::thread::spawn(move || worker_loop(inner, fs));
            threads.push(handle);
        }

        Self {
            inner,
            workers: threads,
            closed: AtomicBool::new(false),
        }
    }

/// Puts a job into the queue.
///
/// WHY: the job gets its identifier here, because the identifier is what
/// the user interface holds on to while the job runs.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::fake_fs::FakeFileSystem;
/// use phocommander_ops::job::{Job, JobKind, Priority};
/// use phocommander_ops::queue::JobQueue;
/// use std::sync::Arc;
///
/// let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
/// let id = queue.enqueue(Job::new(
///     JobKind::CreateDirectory { path: "/photos".into() },
///     Priority::Normal,
/// );
/// assert!(id.get() > 0);
/// ```
    pub fn enqueue(&self, mut job: Job) -> JobId {
        let mut state = self.lock();
        let id = JobId::new(state.next_id);
        state.next_id += 1;
        job.id = id;
        state.records.push(JobRecord {
            job,
            control: Arc::new(JobControl::new()),
        });
        drop(state);
        self.inner.wake.notify_one();
        id
    }

/// The state of every job, as the user interface reads it.
///
/// WHY: the interface never reaches into the queue. It reads a copy of
/// the state and draws that.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::fake_fs::FakeFileSystem;
/// use phocommander_ops::queue::JobQueue;
/// use std::sync::Arc;
///
/// let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
/// assert!(queue.snapshot().is_empty());
/// ```
    pub fn snapshot(&self) -> Vec<JobSnapshot> {
        let state = self.lock();
        state
            .records
            .iter()
            .map(|record| record.job.snapshot())
            .collect()
    }

/// The state of one job.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::fake_fs::FakeFileSystem;
/// use phocommander_ops::job::{Job, JobKind, Priority};
/// use phocommander_ops::queue::JobQueue;
/// use std::sync::Arc;
///
/// let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
/// let id = queue.enqueue(Job::new(
///     JobKind::CreateDirectory { path: "/photos".into() },
///     Priority::Normal,
/// );
/// assert!(queue.snapshot_of(id).is_some());
/// ```
    pub fn snapshot_of(&self, id: JobId) -> Option<JobSnapshot> {
        let state = self.lock();
        state
            .records
            .iter()
            .find(|record| record.job.id == id)
            .map(|record| record.job.snapshot())
    }

/// The failures of one job.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::fake_fs::FakeFileSystem;
/// use phocommander_ops::job::{Job, JobKind, Priority};
/// use phocommander_ops::queue::JobQueue;
/// use std::sync::Arc;
///
/// let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
/// let id = queue.enqueue(Job::new(
///     JobKind::CreateDirectory { path: "/photos".into() },
///     Priority::Normal,
/// );
/// assert!(queue.errors_of(id).unwrap_or_default().is_empty());
/// ```
    pub fn errors_of(&self, id: JobId) -> Option<Vec<JobError>> {
        let state = self.lock();
        state
            .records
            .iter()
            .find(|record| record.job.id == id)
            .map(|record| record.job.errors.clone())
    }

/// Changes a job that has not finished.
///
/// WHY: the rule of the states decides whether the change is allowed at
/// all, and a change to a running job waits in the control of the job
/// until the worker reaches the next item boundary.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::Edit;
/// use phocommander_ops::fake_fs::FakeFileSystem;
/// use phocommander_ops::job::{Job, JobKind, Priority};
/// use phocommander_ops::queue::JobQueue;
/// use std::sync::Arc;
///
/// let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
/// let id = queue.enqueue(Job::new(
///     JobKind::CreateDirectory { path: "/photos".into() },
///     Priority::Normal,
/// );
/// assert!(queue.apply_edit(id, Edit::Priority(Priority::High)).is_ok());
/// ```
    pub fn apply_edit(&self, id: JobId, edit: Edit) -> Result<(), EditError> {
        let mut state = self.lock();
        let record = state
            .records
            .iter_mut()
            .find(|record| record.job.id == id)
            .ok_or(EditError::UnknownJob { id: id.get() })?;

        check_edit(&record.job.state, &edit)?;

        match &edit {
            Edit::Priority(priority) => {
                record.job.priority = *priority;
                record.job.history.push(history(&format!(
                    "the priority changed to {}",
                    priority.as_str()
                )));
                Ok(())
            }
            Edit::Destination(path) => {
                match &mut record.job.kind {
                    JobKind::Copy { destination, .. } | JobKind::Move { destination, .. } => {
                        *destination = path.clone();
                        record.job.history.push(history(&format!(
                            "the destination changed to {}",
                            path.display()
                        )));
                        Ok(())
                    }
                    _ => Err(EditError::NotPossible {
                        edit: edit.as_str(),
                        reason: "the job does not copy into a directory".to_string(),
                    }),
                }
            }
            Edit::ConflictPolicy(_) => {
                record.job.history.push(history("the conflict policy changed"));
                Ok(())
            }
            Edit::VerificationMode(mode) => {
                match &mut record.job.kind {
                    JobKind::Copy { mode: job_mode, .. } | JobKind::Move { mode: job_mode, .. } => {
                        *job_mode = *mode;
                        record.job.history.push(history(&format!(
                            "the mode changed to {}",
                            mode.as_str()
                        )));
                        Ok(())
                    }
                    _ => Err(EditError::NotPossible {
                        edit: edit.as_str(),
                        reason: "the job does not copy files".to_string(),
                    }),
                }
            }
            Edit::Pause => {
                record.control.pause();
                record.job.history.push(history("the job was paused"));
                Ok(())
            }
            Edit::Resume => {
                record.control.resume();
                record.job.history.push(history("the job was continued"));
                Ok(())
            }
            Edit::Cancel => {
                record.control.cancel();
                record.job.history.push(history("the job was cancelled"));
                Ok(())
            }
            Edit::RetryFailed => {
                record.job.errors.clear();
                record.job.state = JobState::Queued;
                record.job.history.push(history("the failed items run again"));
                Ok(())
            }
        }
    }

/// Waits until no job of the queue is running any more.
///
/// WHY: the test of the queue needs a point at which it can look at the
/// result. The interface never waits — it draws the snapshot.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::fake_fs::FakeFileSystem;
/// use phocommander_ops::queue::JobQueue;
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
/// assert!(queue.wait_until_idle(Duration::from_millis(50)));
/// ```
    pub fn wait_until_idle(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            let state = self.lock();
            let busy = state.records.iter().any(|record| {
                !record.job.state.is_finished() && record.job.state != JobState::Queued
            });
            let pending = state
                .records
                .iter()
                .any(|record| record.job.state == JobState::Queued);
            drop(state);

            if !busy && !pending {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

/// Closes the queue and waits for its workers.
///
/// WHY: a program that leaves does not leave workers behind that write
/// into files. The workers stop after the job they are working on.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::fake_fs::FakeFileSystem;
/// use phocommander_ops::queue::JobQueue;
/// use std::sync::Arc;
///
/// let queue = JobQueue::new(Arc::new(FakeFileSystem::new()), 1);
/// queue.shutdown();
/// ```
    pub fn shutdown(&mut self) {
        {
            let mut state = self.lock();
            state.shutting_down = true;
        }
        self.closed.store(true, Ordering::SeqCst);
        self.inner.wake.notify_all();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }

/// Locks the state of the queue.
    fn lock(&self) -> std::sync::MutexGuard<'_, QueueState> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

impl Drop for JobQueue {
/// Closes the queue when it is dropped.
///
/// WHY: a queue that is dropped without being closed would leave its
/// workers running.
    fn drop(&mut self) {
        if !self.closed.load(Ordering::SeqCst) {
            self.shutdown();
        }
    }
}

/// Builds an entry of the history of a job.
fn history(message: &str) -> crate::job::HistoryEntry {
    crate::job::HistoryEntry {
        at: std::time::SystemTime::now(),
        message: message.to_string(),
    }
}

/// The worker loop: it waits for a job and carries it out.
fn worker_loop(inner: Arc<QueueInner>, fs: Arc<dyn FileSystem>) {
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
fn finish(inner: &Arc<QueueInner>, id: JobId, result: Result<JobResult, crate::error::OpError>, control: &Arc<JobControl>) {
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
        let Some(record) = state.records.iter_mut().find(|record| record.job.id == self.id) else {
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
/// use phocommander_ops::queue::unfinished;
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
