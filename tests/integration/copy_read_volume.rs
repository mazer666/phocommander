//! The read volume of a verified copy.
//!
//! WHAT: the test proves that the verification of a copy raises the read
//!       volume to at most twice the copied volume (NFR-09a).
//! WHY: the requirement is a limit and not a wish. A copy that reads the
//!       source three times — the copy, the checksum of the source and the
//!       checksum of the file that was written — would fulfil every test of
//!       the behaviour and still break the requirement. Only a file system
//!       that counts what it reads can tell the difference.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test copy_read_volume
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use phocommander_ops::copy::{CopyMode, CopyPolicy, copy_one_file};
use phocommander_ops::edit::JobControl;
use phocommander_ops::fake_fs::FakeFileSystem;
use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
use phocommander_testkit::counting::CountingFileSystem;

/// The size of the file that is copied, in bytes.
const FILE_SIZE: usize = 4 * 1024 * 1024;

/// The highest read volume that the requirement allows, as a multiple of the
/// copied volume.
const ALLOWED_MULTIPLE: u64 = 2;

/// The sink that discards the progress.
struct Silent;

impl ProgressSink for Silent {
    fn report(&mut self, _update: ProgressUpdate) {}
}

/// Copies the file and reports the bytes that were read and the bytes that
/// were written.
fn copy_and_count(mode: CopyMode, policy: &CopyPolicy) -> (u64, u64) {
    let fs = FakeFileSystem::new();
    let source = PathBuf::from("/photos/holiday.mkv");
    let target = PathBuf::from("/backup/holiday.mkv");
    fs.write_file(&source, &vec![7_u8; FILE_SIZE])
        .expect("the file should be written");
    let counting = CountingFileSystem::new(fs);

    let control = Arc::new(JobControl::new());
    let mut context = JobContext::new(control, Box::new(Silent));
    let geschrieben = copy_one_file(&counting, &source, &target, mode, policy, &mut context)
        .expect("the copy should have worked");

    (counting.read_bytes(), geschrieben)
}

#[test]
fn the_verification_raises_the_read_volume_to_at_most_twice_the_copied_volume() {
    // act
    let (gelesen, geschrieben) = copy_and_count(CopyMode::Verified, &CopyPolicy::strict());

    // assert
    // WHY: the copy reads the source once and the verification reads the
    //       file that was written once. The rule allows twice the copied
    //       volume, and no more.
    assert!(geschrieben > 0, "the copy should have written bytes");
    assert!(
        gelesen <= geschrieben * ALLOWED_MULTIPLE,
        "the verification read {gelesen} bytes for {geschrieben} bytes that were copied — \
         the rule of NFR-09a allows {}",
        geschrieben * ALLOWED_MULTIPLE
    );
}

#[test]
fn the_quick_copy_reads_the_source_only_once() {
    // act
    let (gelesen, geschrieben) = copy_and_count(CopyMode::Quick, &CopyPolicy::strict());

    // assert
    // WHY: the quick mode does not verify, and it therefore reads the
    //       source only once.
    assert!(geschrieben > 0, "the copy should have written bytes");
    assert!(
        gelesen <= geschrieben,
        "the quick copy read {gelesen} bytes for {geschrieben} bytes that were copied"
    );
}

#[test]
fn a_copy_without_the_verification_reads_nothing_more_than_the_copy() {
    // act
    let (gelesen, geschrieben) = copy_and_count(CopyMode::Verified, &CopyPolicy::new(false, false));

    // assert
    // WHY: the rule of the user can switch the comparison off. The read
    //       volume then falls back to the volume of the copy itself.
    assert!(geschrieben > 0, "the copy should have written bytes");
    assert!(
        gelesen <= geschrieben,
        "the copy read {gelesen} bytes for {geschrieben} bytes that were copied"
    );
}
