//! Criterion benchmark: the read volume of a verified copy.
//!
//! WHAT: the benchmark measures how many bytes the verification of a copy
//!       reads in addition to the copy itself (NFR-09a).
//! WHY:  the specification makes the target values of the performance section
//!       binding gates: "Die folgenden Zielwerte sind verbindliche Gates der
//!       CI, keine Absichtserklärung." A benchmark that only prints a number
//!       does not hold a gate; a benchmark that fails when the number is too
//!       big does.
//!
//!       The copy itself reads the source once. The verification reads the
//!       file that was written a second time and compares the checksums. The
//!       read volume is therefore twice the copied volume — and the
//!       requirement names twice as the highest that is allowed. A third read
//!       would be a defect that no test of the behaviour would ever see.
//!
//!       The measurement runs against the file system of the operations,
//!       wrapped in the file system that counts the bytes. The count is the
//!       proof; the time is the information beside it.
//!
//! EXAMPLE:
//! ```text
//! cargo bench -p phocommander-integration-tests --bench copy_verification_read_volume
//! ```

use std::hint::black_box;
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
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
    // arrange
    let fs = FakeFileSystem::new();
    let source = PathBuf::from("/photos/holiday.mkv");
    let target = PathBuf::from("/backup/holiday.mkv");
    fs.write_file(&source, &vec![7_u8; FILE_SIZE])
        .unwrap_or_else(|error| panic!("the file should have been written: {error}"));
    let counting = CountingFileSystem::new(fs);

    let control = std::sync::Arc::new(JobControl::new());
    let mut context = JobContext::new(control, Box::new(Silent));

    // act
    let geschrieben =
        copy_one_file(&counting, &source, &target, mode, policy, &mut context).unwrap_or_default();

    // assert
    (counting.read_bytes(), geschrieben)
}

/// The benchmark of the verified copy.
///
/// WHY: the check stands before the measurement and not in it. A benchmark
///       that runs with `harness = false` carries no test harness, and a
///       check inside the measured loop would be measured as well. The check
///       therefore runs once, before the measurement, and a violation ends
///       the benchmark with a failure — the gate of NFR-09a.
fn the_verification_reads_at_most_twice_the_copied_volume(c: &mut Criterion) {
    let (gelesen, geschrieben) = copy_and_count(CopyMode::Verified, &CopyPolicy::strict());
    assert!(
        geschrieben > 0,
        "the copy should have written bytes, but it wrote {geschrieben}"
    );
    assert!(
        gelesen <= geschrieben * ALLOWED_MULTIPLE,
        "the verification read {gelesen} bytes for {geschrieben} bytes that were copied — \
         the rule of NFR-09a allows {}",
        geschrieben * ALLOWED_MULTIPLE
    );

    // WHY: the numbers are printed, so that the measurement of the volume is
    //       in the log of the benchmark and not only in the head of the
    //       person who ran it.
    println!(
        "the verified copy of {FILE_SIZE} bytes read {gelesen} bytes — the rule of NFR-09a \
         allows {}",
        geschrieben * ALLOWED_MULTIPLE
    );

    c.bench_function("the_verified_copy_reads_at_most_twice_the_volume", |b| {
        b.iter(|| {
            let (gelesen, geschrieben) = copy_and_count(CopyMode::Verified, &CopyPolicy::strict());
            black_box((gelesen, geschrieben));
        });
    });
}

/// The check that holds the gate of NFR-09a.
#[test]
fn the_verification_raises_the_read_volume_to_at_most_twice_the_copied_volume() {
    // arrange
    let (gelesen, geschrieben) = copy_and_count(CopyMode::Verified, &CopyPolicy::strict());

    // act, assert
    // WHY: the copy reads the source once and the verification reads the
    //       file that was written once. The requirement allows twice the
    //       copied volume, and no more.
    assert!(geschrieben > 0, "the copy should have written bytes");
    assert!(
        gelesen <= geschrieben * ALLOWED_MULTIPLE,
        "the verification read {gelesen} bytes for {geschrieben} bytes that \
         were copied — the rule allows {}",
        geschrieben * ALLOWED_MULTIPLE
    );
}

/// The check that the quick copy reads the source only once.
#[test]
fn the_quick_copy_reads_the_source_only_once() {
    // arrange
    let (gelesen, geschrieben) = copy_and_count(CopyMode::Quick, &CopyPolicy::strict());

    // act, assert
    // WHY: the quick mode does not verify, and it therefore reads the
    //       source only once. A quick copy that reads twice would be a
    //       defect that the rule of NFR-09a would hide.
    assert!(geschrieben > 0, "the copy should have written bytes");
    assert!(
        gelesen <= geschrieben,
        "the quick copy read {gelesen} bytes for {geschrieben} bytes that were copied"
    );
}

criterion_group!(
    benches,
    the_verification_reads_at_most_twice_the_copied_volume
);
criterion_main!(benches);
