// The tests of the queue.
//
// WHAT: the tests of the waiting room of the jobs — the order of the jobs,
//       the collecting of the failures, the repeating of the failed entries
//       and the changing of a job that has not finished yet (FR-22k).
// WHY: the queue is the part of the program that the user waits for. A job
//       that hangs, a job that is forgotten and a job that copies the same
//       file twice are all faults that the user notices at once.
// EXAMPLE:
// ```no_run
// use phocommander_ops::queue::JobQueue;
// use std::sync::Arc;
// ```
//
// WHY: a test may fail loudly when its arrangement is wrong. The rule that
// forbids a panic applies to the production code, and it is enforced there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::copy::CopyMode;
use crate::edit::Edit;
use crate::fake_fs::FakeFileSystem;
use crate::job::{Job, JobKind, JobSnapshot, JobState, Priority};

/// The time the tests wait for the queue to become quiet.
const WAIT: Duration = Duration::from_secs(5);

/// Builds a file system with two files in a directory.
fn file_system() -> Arc<FakeFileSystem> {
    let fs = Arc::new(FakeFileSystem::new());
    fs.write_file(Path::new("/photos/a.txt"), b"a")
        .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
    fs.write_file(Path::new("/photos/b.txt"), b"b")
        .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
    fs
}

/// Builds a job that copies a directory.
fn copy_job(source: &str, destination: &str) -> Job {
    Job::new(
        JobKind::Copy {
            source: PathBuf::from(source),
            destination: PathBuf::from(destination),
            mode: CopyMode::Quick,
        },
        Priority::Normal,
    )
}

/// The snapshot of a job, or a panic with the reason.
fn snapshot_of(queue: &JobQueue, id: JobId) -> JobSnapshot {
    queue
        .snapshot_of(id)
        .unwrap_or_else(|| panic!("the job should be in the queue"))
}

#[test]
fn a_job_that_was_given_to_the_queue_is_carried_out() {
    // arrange
    let fs = file_system();
    let queue = JobQueue::new(fs.clone(), 1);
    let id = queue.enqueue(copy_job("/photos", "/backup"));

    // act
    let idle = queue.wait_until_idle(WAIT);

    // assert
    assert!(idle, "the queue should become quiet");
    let snapshot = snapshot_of(&queue, id);
    assert_eq!(snapshot.state, JobState::Done);
    assert!(fs.exists(Path::new("/backup/a.txt")));
    assert!(fs.exists(Path::new("/backup/b.txt")));
}

#[test]
fn a_job_that_fails_collects_the_failed_entries_and_carries_on() {
    // arrange
    let fs = file_system();
    fs.fail_copy_of(Path::new("/photos/a.txt"));
    let queue = JobQueue::new(fs.clone(), 1);
    let id = queue.enqueue(copy_job("/photos", "/backup"));

    // act
    let idle = queue.wait_until_idle(WAIT);

    // assert
    assert!(idle, "the queue should become quiet");
    let snapshot = snapshot_of(&queue, id);
    assert_eq!(snapshot.state, JobState::Failed);
    assert_eq!(snapshot.failed_items, vec![PathBuf::from("/photos/a.txt")]);
    // WHY: the other entry was copied anyway. A job that stops at the first
    //       failure would leave the user with a half copied directory and no
    //       way to know which half is missing.
    assert!(fs.exists(Path::new("/backup/b.txt")));
    assert!(!fs.exists(Path::new("/backup/a.txt")));
}

#[test]
fn the_retry_of_a_failed_job_copies_only_the_failed_entries() {
    // arrange
    let fs = file_system();
    fs.fail_copy_of(Path::new("/photos/a.txt"));
    let queue = JobQueue::new(fs.clone(), 1);
    let id = queue.enqueue(copy_job("/photos", "/backup"));
    assert!(queue.wait_until_idle(WAIT));
    fs.stop_failing(Path::new("/photos/a.txt"));

    // act
    let retry = queue
        .apply_edit(id, Edit::RetryFailed)
        .unwrap_or_else(|error| panic!("the retry should be possible: {error}"));
    let retry_id = retry.unwrap_or_else(|| panic!("the retry should produce a job"));
    let idle = queue.wait_until_idle(WAIT);

    // assert
    assert!(idle, "the queue should become quiet");
    let snapshot = snapshot_of(&queue, retry_id);
    assert_eq!(snapshot.state, JobState::Done);
    // WHY: the retry touches only the entry that failed. The files that were
    //       copied already are not copied a second time.
    assert_eq!(
        snapshot.restricted_items.map(|items| items.len()),
        Some(1),
        "the retry should be limited to the failed entry"
    );
    assert!(fs.exists(Path::new("/backup/a.txt")));
}

#[test]
fn a_job_that_has_nothing_to_repeat_refuses_the_retry() {
    // arrange
    let fs = file_system();
    let queue = JobQueue::new(fs.clone(), 1);
    let id = queue.enqueue(copy_job("/photos", "/backup"));
    assert!(queue.wait_until_idle(WAIT));

    // act
    let result = queue.apply_edit(id, Edit::RetryFailed);

    // assert
    // WHY: the job is finished and it has no failed entry. The retry is
    //       therefore not possible, and the error names the state of the job
    //       so that the interface can say what is wrong.
    assert!(
        matches!(result, Err(EditError::WrongState { .. })),
        "{result:?}"
    );
}

#[test]
fn the_priority_of_a_job_can_be_changed_while_it_waits() {
    // arrange
    let fs = file_system();
    let queue = JobQueue::new(fs.clone(), 1);
    let id = queue.enqueue(copy_job("/photos", "/backup"));

    // act
    let result = queue.apply_edit(id, Edit::Priority(Priority::High));

    // assert
    assert!(result.is_ok(), "{result:?}");
    let snapshot = snapshot_of(&queue, id);
    assert_eq!(snapshot.priority, Priority::High);
}

#[test]
fn the_destination_of_a_job_can_be_changed_while_it_waits() {
    // arrange
    let fs = file_system();
    let queue = JobQueue::new(fs.clone(), 1);
    let id = queue.enqueue(copy_job("/photos", "/backup"));

    // act
    let result = queue.apply_edit(id, Edit::Destination(PathBuf::from("/elsewhere")));

    // assert
    assert!(result.is_ok(), "{result:?}");
    match snapshot_of(&queue, id).kind {
        JobKind::Copy { destination, .. } => {
            assert_eq!(destination, PathBuf::from("/elsewhere"));
        }
        other => panic!("the job should still be a copy, but it is {other:?}"),
    }
}

#[test]
fn an_edit_of_a_job_that_does_not_exist_is_refused() {
    // arrange
    let fs = file_system();
    let queue = JobQueue::new(fs.clone(), 1);

    // act
    let result = queue.apply_edit(JobId::new(9999), Edit::Pause);

    // assert
    assert!(
        matches!(result, Err(EditError::UnknownJob { .. })),
        "{result:?}"
    );
}

#[test]
fn the_jobs_of_the_queue_are_listed_with_their_state() {
    // arrange
    let fs = file_system();
    let queue = JobQueue::new(fs.clone(), 1);
    let id = queue.enqueue(copy_job("/photos", "/backup"));

    // act
    assert!(queue.wait_until_idle(WAIT));
    let snapshots = queue.snapshot();

    // assert
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].id, id);
    assert_eq!(snapshots[0].state, JobState::Done);
}

#[test]
fn the_unfinished_jobs_are_the_ones_that_are_not_finished() {
    // arrange
    let fs = file_system();
    let queue = JobQueue::new(fs.clone(), 1);
    let id = queue.enqueue(copy_job("/photos", "/backup"));

    // act
    assert!(queue.wait_until_idle(WAIT));
    let snapshots = queue.snapshot();

    // assert
    assert!(
        super::worker::unfinished(&snapshots).is_empty(),
        "a finished job is not unfinished"
    );
    assert_eq!(super::worker::unfinished(&snapshots).len(), 0);
    assert_eq!(id, snapshots[0].id);
}
