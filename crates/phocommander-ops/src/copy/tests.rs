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
use crate::port::EntryKind;
use crate::progress::{JobContext, ProgressSink, ProgressUpdate};
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
