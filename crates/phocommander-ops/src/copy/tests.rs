// The tests of the copy.
//
// WHY: the tests live in a file of their own, so that the copy itself
//       stays readable. They may fail loudly when their arrangement is
//       wrong — the rule that forbids a panic applies to the production
//       code, and it is enforced there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::edit::JobControl;
use crate::fake_fs::FakeFileSystem;
use crate::port::{EntryKind, FileSystem};
use crate::progress::{JobContext, ProgressSink, ProgressUpdate};
use crate::temporary::temporary_path;
use std::sync::Arc;

/// A sink that discards every report.
struct Silent;

impl ProgressSink for Silent {
    fn report(&mut self, _update: ProgressUpdate) {}
}

/// Builds a context for a job that runs without a queue.
fn context() -> JobContext {
    JobContext::new(Arc::new(JobControl::new()), Box::new(Silent))
}

/// Writes a file with the given content into the fake file system.
fn write_file(fs: &FakeFileSystem, path: &str, content: &[u8]) {
    fs.write_file(Path::new(path), content)
        .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
}

#[test]
fn the_default_of_a_copy_is_the_verified_mode() {
    // act
    let mode = CopyMode::default();

    // assert
    assert_eq!(mode, CopyMode::Verified);
    assert!(mode.verifies());
}

#[test]
fn the_default_policy_of_a_copy_verifies_and_never_follows_a_link() {
    // act
    let policy = CopyPolicy::strict();

    // assert
    assert!(policy.verifies_checksums());
    assert!(!policy.follows_symlinks());
}

#[test]
fn the_verified_copy_compares_the_checksums_when_the_rule_is_in_force() {
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    let mut context = context();

    // act
    let result = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Verified,
        &CopyPolicy::strict(),
        &mut context,
    );

    // assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(fs.checksum_calls(), 1);
}

#[test]
fn the_verified_copy_does_not_compare_the_checksums_when_the_rule_is_off() {
    // WHY: the rule "checksum_on_copy" is switched off, so the
    // comparison is not executed and the destination is not read a
    // second time.
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    let mut context = context();

    // act
    let result = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Verified,
        &CopyPolicy::new(false, false),
        &mut context,
    );

    // assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(fs.checksum_calls(), 0);
}

#[test]
fn the_quick_copy_never_compares_the_checksums() {
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    let mut context = context();

    // act
    let result = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Quick,
        &CopyPolicy::strict(),
        &mut context,
    );

    // assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(fs.checksum_calls(), 0);
}

#[test]
fn the_atomic_copy_always_verifies_the_temporary_file() {
    // WHY: the promise of the atomic mode is that a program never reads
    // a half-written file. That promise does not belong to the switch of
    // the verified mode.
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    let mut context = context();

    // act
    let result = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Atomic,
        &CopyPolicy::new(false, false),
        &mut context,
    );

    // assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(fs.checksum_calls(), 1);
}

#[test]
fn a_symbolic_link_is_copied_as_a_link_when_the_rule_is_in_force() {
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    fs.write_symlink(Path::new("/photos/link.txt"), Path::new("/photos/a.txt"))
        .unwrap_or_else(|error| panic!("the link could not be written: {error}"));
    let mut context = context();
    let request = CopyRequest::new("/photos", "/backup")
        .with_mode(CopyMode::Quick)
        .with_rules(CopyPolicy::strict());

    // act
    let outcome = execute_copy(&fs, &request, &mut context)
        .unwrap_or_else(|error| panic!("the copy failed: {error}"));

    // assert
    assert_eq!(outcome.errors.len(), 0, "{:?}", outcome.errors);
    assert_eq!(
        fs.kind_of(Path::new("/backup/link.txt")),
        Some(EntryKind::Symlink)
    );
    assert_eq!(
        fs.link_target(Path::new("/backup/link.txt")).as_deref(),
        Some("/photos/a.txt")
    );
}

#[test]
fn a_symbolic_link_is_dereferenced_when_the_rule_allows_it() {
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    fs.write_symlink(Path::new("/photos/link.txt"), Path::new("/photos/a.txt"))
        .unwrap_or_else(|error| panic!("the link could not be written: {error}"));
    let mut context = context();
    let request = CopyRequest::new("/photos", "/backup")
        .with_mode(CopyMode::Quick)
        .with_rules(CopyPolicy::new(true, true));

    // act
    let outcome = execute_copy(&fs, &request, &mut context)
        .unwrap_or_else(|error| panic!("the copy failed: {error}"));

    // assert
    assert_eq!(outcome.errors.len(), 0, "{:?}", outcome.errors);
    assert_eq!(
        fs.kind_of(Path::new("/backup/link.txt")),
        Some(EntryKind::File)
    );
    let content = fs
        .read_file(Path::new("/backup/link.txt"))
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    assert_eq!(content, b"payload");
}

#[test]
fn a_copied_link_carries_no_bytes_of_its_own() {
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    fs.write_symlink(Path::new("/photos/link.txt"), Path::new("/photos/a.txt"))
        .unwrap_or_else(|error| panic!("the link could not be written: {error}"));
    let mut context = context();
    let request = CopyRequest::new("/photos", "/backup")
        .with_mode(CopyMode::Quick)
        .with_rules(CopyPolicy::strict());

    // act
    let outcome = execute_copy(&fs, &request, &mut context)
        .unwrap_or_else(|error| panic!("the copy failed: {error}"));

    // assert
    assert_eq!(outcome.files_copied, 2);
    assert_eq!(outcome.bytes_copied, 7);
}

#[test]
fn only_the_mirror_mode_deletes() {
    // act, assert
    assert!(CopyMode::SyncMirror.deletes());
    assert!(!CopyMode::SyncUpdate.deletes());
    assert!(!CopyMode::SyncTwoWay.deletes());
    assert!(!CopyMode::SyncPreview.deletes());
    assert!(!CopyMode::Quick.deletes());
}

#[test]
fn the_modes_know_which_of_them_are_synchronisations() {
    // act, assert
    assert!(CopyMode::SyncMirror.is_sync());
    assert!(CopyMode::SyncUpdate.is_sync());
    assert!(CopyMode::SyncTwoWay.is_sync());
    assert!(CopyMode::SyncPreview.is_sync());
    assert!(!CopyMode::Quick.is_sync());
    assert!(!CopyMode::Verified.is_sync());
    assert!(!CopyMode::Atomic.is_sync());
}

#[test]
fn the_modes_have_the_names_the_configuration_uses() {
    // act, assert
    assert_eq!(CopyMode::Quick.as_str(), "quick");
    assert_eq!(CopyMode::Verified.as_str(), "verified");
    assert_eq!(CopyMode::Atomic.as_str(), "atomic");
    assert_eq!(CopyMode::SyncMirror.as_str(), "sync-mirror");
    assert_eq!(CopyMode::SyncUpdate.as_str(), "sync-update");
    assert_eq!(CopyMode::SyncTwoWay.as_str(), "sync-two-way");
    assert_eq!(CopyMode::SyncPreview.as_str(), "sync-preview");
}

#[test]
fn the_temporary_file_of_the_atomic_mode_sits_next_to_the_target() {
    // arrange
    let target = Path::new("/backup/photos/sunset.jpg");

    // act
    let temporary = temporary_path(target);

    // assert
    assert_eq!(temporary.parent(), target.parent());
    let name = temporary
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    assert!(name.starts_with(".sunset.jpg."), "{name}");
    assert!(name.ends_with(".part"), "{name}");
}

#[test]
fn the_atomic_copy_leaves_no_temporary_file_behind_when_the_comparison_fails() {
    // WHY: the temporary file is removed on every path out of the
    // operation — a failed comparison is one of them.
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    fs.corrupt_next_copy();
    let mut context = context();

    // act
    let result = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Atomic,
        &CopyPolicy::strict(),
        &mut context,
    );

    // assert
    assert!(
        matches!(result, Err(OpError::ChecksumMismatch { .. })),
        "{result:?}"
    );
    let leftovers = fs
        .read_dir(Path::new("/backup"))
        .unwrap_or_else(|error| panic!("the directory could not be listed: {error}"));
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn the_atomic_copy_never_writes_the_target_before_the_comparison() {
    // WHY: the target may not carry a file that was not compared. The
    // comparison fails, and the target does not exist.
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    fs.corrupt_next_copy();
    let mut context = context();

    // act
    let _ = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Atomic,
        &CopyPolicy::strict(),
        &mut context,
    );

    // assert
    assert!(!fs.exists(Path::new("/backup/a.txt")));
}

#[test]
fn the_atomic_copy_replaces_the_target_only_after_the_comparison() {
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    write_file(&fs, "/backup/a.txt", b"old");
    let mut context = context();

    // act
    let result = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Atomic,
        &CopyPolicy::strict(),
        &mut context,
    );

    // assert
    assert!(result.is_ok(), "{result:?}");
    let content = fs
        .read_file(Path::new("/backup/a.txt"))
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    assert_eq!(content, b"payload");
}

#[test]
fn the_policy_that_never_overwrites_leaves_an_existing_target_alone() {
    // WHY: the conflict policy is the same for every mode. The atomic mode
    // does not get a name of its own.
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    write_file(&fs, "/backup/a.txt", b"old");
    let mut context = context();
    let request = CopyRequest::new("/photos", "/backup")
        .with_mode(CopyMode::Atomic)
        .with_policy(ConflictPolicy::Never);

    // act
    let outcome = execute_copy(&fs, &request, &mut context)
        .unwrap_or_else(|error| panic!("the copy failed: {error}"));

    // assert
    assert_eq!(outcome.errors.len(), 0, "{:?}", outcome.errors);
    assert_eq!(outcome.files_skipped, 1);
    let content = fs
        .read_file(Path::new("/backup/a.txt"))
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    assert_eq!(content, b"old");
}

#[test]
fn the_copy_of_a_tree_can_be_limited_to_some_items() {
    // WHY: the retry of the failed items (FR-22k) copies the items that
    // failed and nothing else.
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"a");
    write_file(&fs, "/photos/b.txt", b"b");
    let mut context = context();
    let request = CopyRequest::new("/photos", "/backup")
        .with_mode(CopyMode::Quick)
        .with_restricted_items(vec![Path::new("/photos/a.txt").to_path_buf()]);

    // act
    let outcome = execute_copy(&fs, &request, &mut context)
        .unwrap_or_else(|error| panic!("the copy failed: {error}"));

    // assert
    assert_eq!(outcome.files_copied, 1);
    assert!(fs.exists(Path::new("/backup/a.txt")));
    assert!(!fs.exists(Path::new("/backup/b.txt")));
}

#[test]
fn a_verified_copy_whose_destination_differs_ends_in_the_error_of_the_checksum() {
    // arrange
    // WHY: the verified mode compares the checksum of the source with the
    //       checksum of the file that was written. A file system that
    //       writes the wrong bytes is exactly the case the mode exists for,
    //       and the job has to end with the error of the checksum — not
    //       with a success and a damaged file.
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    fs.corrupt_next_copy();
    let mut context = context();

    // act
    let result = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Verified,
        &CopyPolicy::strict(),
        &mut context,
    );

    // assert
    let error = match result {
        Err(error) => error,
        Ok(bytes) => panic!("the copy should have failed, but it wrote {bytes} bytes"),
    };
    assert!(
        matches!(error, OpError::ChecksumMismatch { .. }),
        "the error should name the checksum: {error:?}"
    );
}

#[test]
fn a_quick_copy_whose_destination_differs_writes_the_file_all_the_same() {
    // arrange
    // WHY: the quick mode does not verify, and it therefore writes what the
    //       file system gave it. The mode is the one that trades the check
    //       for the speed.
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    fs.corrupt_next_copy();
    let mut context = context();

    // act
    let result = copy_one_file(
        &fs,
        Path::new("/photos/a.txt"),
        Path::new("/backup/a.txt"),
        CopyMode::Quick,
        &CopyPolicy::strict(),
        &mut context,
    );

    // assert
    // WHY: the quick mode does not compare, and it therefore reports no
    //       error — even though the medium wrote something else. That is
    //       the trade of the mode: the speed instead of the proof.
    assert!(result.is_ok(), "{result:?}");
    let geschrieben = fs
        .checksum(Path::new("/backup/a.txt"))
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    let quelle = fs
        .checksum(Path::new("/photos/a.txt"))
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    // WHY: the file that was written differs from the source, and nobody
    //       said so — the quick mode is the mode that does not say.
    assert_ne!(geschrieben, quelle);
}

#[test]
fn the_policy_that_always_overwrites_replaces_an_existing_target() {
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    write_file(&fs, "/backup/a.txt", b"old");
    let mut context = context();
    let request = CopyRequest::new("/photos", "/backup")
        .with_mode(CopyMode::Quick)
        .with_policy(ConflictPolicy::Always);

    // act
    let outcome = execute_copy(&fs, &request, &mut context)
        .unwrap_or_else(|error| panic!("the copy failed: {error}"));

    // assert
    assert_eq!(outcome.errors.len(), 0, "{:?}", outcome.errors);
    assert_eq!(outcome.files_copied, 1);
    assert_eq!(outcome.files_skipped, 0);
    let content = fs
        .read_file(Path::new("/backup/a.txt"))
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    assert_eq!(content, b"payload");
}

#[test]
fn the_policy_that_keeps_both_writes_a_name_that_does_not_exist_yet() {
    // arrange
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    write_file(&fs, "/backup/a.txt", b"old");
    let mut context = context();
    let request = CopyRequest::new("/photos", "/backup")
        .with_mode(CopyMode::Quick)
        .with_policy(ConflictPolicy::KeepBoth);

    // act
    let outcome = execute_copy(&fs, &request, &mut context)
        .unwrap_or_else(|error| panic!("the copy failed: {error}"));

    // assert
    assert_eq!(outcome.errors.len(), 0, "{:?}", outcome.errors);
    assert_eq!(outcome.files_copied, 1);
    // WHY: the file that exists already keeps its content — the copy went
    //       somewhere else.
    let content = fs
        .read_file(Path::new("/backup/a.txt"))
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    assert_eq!(content, b"old");
    // WHY: and the copy carries the content of the source under a name that
    //       does not exist yet.
    let namen = fs
        .read_dir(Path::new("/backup"))
        .unwrap_or_else(|error| panic!("the directory could not be read: {error}"));
    let zweiter = namen
        .iter()
        .find(|eintrag| eintrag.name != "a.txt")
        .unwrap_or_else(|| panic!("the copy should have a name of its own: {namen:?}"));
    let pfad = Path::new("/backup").join(&zweiter.name);
    let content = fs
        .read_file(&pfad)
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    assert_eq!(content, b"payload");
}

#[test]
fn the_policy_that_asks_leaves_the_existing_target_alone_until_somebody_answers() {
    // arrange
    // WHY: the queue of the specification has no room for a question in the
    //       middle of a copy — the job runs to the end and reports. A file
    //       that exists already is therefore left alone, and the job counts
    //       it as skipped. The user decides afterwards, by copying again
    //       with another policy.
    let fs = FakeFileSystem::new();
    write_file(&fs, "/photos/a.txt", b"payload");
    write_file(&fs, "/backup/a.txt", b"old");
    let mut context = context();
    let request = CopyRequest::new("/photos", "/backup")
        .with_mode(CopyMode::Quick)
        .with_policy(ConflictPolicy::Ask);

    // act
    let outcome = execute_copy(&fs, &request, &mut context)
        .unwrap_or_else(|error| panic!("the copy failed: {error}"));

    // assert
    assert_eq!(outcome.errors.len(), 0, "{:?}", outcome.errors);
    assert_eq!(outcome.files_skipped, 1);
    assert_eq!(outcome.files_copied, 0);
    let content = fs
        .read_file(Path::new("/backup/a.txt"))
        .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
    assert_eq!(content, b"old");
}
