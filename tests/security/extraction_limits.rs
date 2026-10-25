//! Negative tests: the limits of an extraction (the hardening rule H-06).
//!
//! WHAT: three hostile archives are **built at run time** in a temporary
//!       directory and handed to the hardened extraction of
//!       `phocommander_archive` with limits that the archives exceed.
//! WHY:  an archive is untrusted input. Without the limits a small archive
//!       fills the disk of the machine it is unpacked on (a decompression
//!       bomb), an archive can declare millions of entries without occupying
//!       much space, and a single entry that is larger than the whole budget
//!       fills the disk before the total is noticed. The rule H-06 of the
//!       threat model therefore names three limits, and each of them needs a
//!       negative test that performs the attack (specification, section 10.2).
//!
//!       The limits of the test are lowered, because the limits of the
//!       specification — two gigabytes and a hundred thousand entries — would
//!       fill the disk of the machine that runs the test. The mechanism that
//!       is proven is the same: the extraction aborts with a readable message
//!       and writes nothing.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test security_extraction_limits
//! ```

use std::fs;
use std::io::Write;
use std::path::Path;

use phocommander_archive::{
    ArchiveError, ExtractionLimits, ExtractionOptions, LimitKind, extract_archive,
};
use phocommander_testkit::TempTree;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Builds a ZIP archive with the given entries.
///
/// WHY: the archive is created at run time and lives in the temporary
///       directory of the test. A binary file in the repository would be an
///       attack sample that nobody asked for.
fn build_archive(path: &Path, entries: &[(&str, &[u8])]) {
    let file = match fs::File::create(path) {
        Ok(file) => file,
        Err(error) => panic!("the archive {path:?} could not be created: {error}"),
    };
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, content) in entries {
        if let Err(error) = writer.start_file(*name, options) {
            panic!("the entry {name} could not be started: {error}");
        }
        if let Err(error) = writer.write_all(content) {
            panic!("the entry {name} could not be written: {error}");
        }
    }
    if let Err(error) = writer.finish() {
        panic!("the archive {path:?} could not be finished: {error}");
    }
}

/// Builds the options of an extraction with the given limits.
fn options_with(limits: ExtractionLimits) -> ExtractionOptions {
    ExtractionOptions {
        limits,
        ..ExtractionOptions::default()
    }
}

#[test]
fn the_extraction_aborts_when_the_total_size_limit_is_exceeded() {
    // arrange
    let tree = match TempTree::new("bomb-total") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("bomb.zip");
    let payload = [0_u8; 4096];
    build_archive(&archive, &[("payload.bin", &payload)]);
    let destination = tree.path().join("destination");
    let options = options_with(ExtractionLimits::new(1024, 1000, 1_000_000));

    // act
    let result = extract_archive(&archive, &destination, &options);

    // assert — the extraction aborts with the limit of the total size
    match result {
        Ok(report) => panic!("the bomb was extracted: {report:?}"),
        Err(ArchiveError::LimitExceeded {
            kind,
            actual,
            limit,
        }) => {
            assert_eq!(kind, LimitKind::TotalBytes);
            assert!(
                actual > limit,
                "the value {actual} is not more than {limit}"
            );
            assert_eq!(limit, 1024);
        }
        Err(other) => panic!("the extraction failed with the wrong error: {other:?}"),
    }

    // assert — and the payload was not written
    let written = fs::metadata(destination.join("payload.bin"))
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    assert!(
        written < 4096,
        "the payload was written completely: {written} bytes"
    );
}

#[test]
fn the_extraction_aborts_when_the_number_of_entries_limit_is_exceeded() {
    // arrange
    let tree = match TempTree::new("bomb-entries") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("many.zip");
    build_archive(
        &archive,
        &[
            ("a.txt", b"a"),
            ("b.txt", b"b"),
            ("c.txt", b"c"),
            ("d.txt", b"d"),
        ],
    );
    let destination = tree.path().join("destination");
    let options = options_with(ExtractionLimits::new(1_000_000, 2, 1_000_000));

    // act
    let result = extract_archive(&archive, &destination, &options);

    // assert — the extraction aborts with the limit of the number of entries
    match result {
        Ok(report) => panic!("the archive with too many entries was extracted: {report:?}"),
        Err(ArchiveError::LimitExceeded {
            kind,
            actual,
            limit,
        }) => {
            assert_eq!(kind, LimitKind::EntryCount);
            assert_eq!(actual, 3);
            assert_eq!(limit, 2);
        }
        Err(other) => panic!("the extraction failed with the wrong error: {other:?}"),
    }
}

#[test]
fn the_extraction_aborts_when_the_size_of_a_single_entry_limit_is_exceeded() {
    // arrange
    let tree = match TempTree::new("bomb-entry") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("one.zip");
    let payload = [0_u8; 4096];
    build_archive(&archive, &[("payload.bin", &payload)]);
    let destination = tree.path().join("destination");
    let options = options_with(ExtractionLimits::new(1_000_000, 1000, 1024));

    // act
    let result = extract_archive(&archive, &destination, &options);

    // assert — the extraction aborts with the limit of a single entry
    match result {
        Ok(report) => panic!("the oversized entry was extracted: {report:?}"),
        Err(ArchiveError::LimitExceeded {
            kind,
            actual,
            limit,
        }) => {
            assert_eq!(kind, LimitKind::SingleEntryBytes);
            assert_eq!(actual, 4096);
            assert_eq!(limit, 1024);
        }
        Err(other) => panic!("the extraction failed with the wrong error: {other:?}"),
    }

    // assert — and the entry was not written
    assert!(
        !destination.join("payload.bin").exists(),
        "the oversized entry was written"
    );
}

#[test]
fn an_archive_within_the_limits_is_extracted_completely() {
    // WHY: a gate that is red for every archive proves nothing. This test is
    // the positive counterpart of the three tests above: the same extraction
    // with limits that the archive respects writes every entry.
    let tree = match TempTree::new("limits-positive") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("small.zip");
    build_archive(
        &archive,
        &[
            ("a.txt", b"first"),
            ("b.txt", b"second"),
            ("c.txt", b"third"),
        ],
    );
    let destination = tree.path().join("destination");
    let options = options_with(ExtractionLimits::new(1_000_000, 10, 1_000_000));

    // act
    let report = match extract_archive(&archive, &destination, &options) {
        Ok(report) => report,
        Err(error) => panic!("the extraction of a harmless archive failed: {error}"),
    };

    // assert
    assert_eq!(report.written_entries, 3, "the report: {report:?}");
    assert_eq!(report.written_bytes, 16, "the report: {report:?}");
    assert_eq!(
        fs::read_to_string(destination.join("b.txt")).unwrap_or_default(),
        "second"
    );
}
