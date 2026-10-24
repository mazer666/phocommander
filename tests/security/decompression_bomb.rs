//! Negative test: the decompression bomb.
//!
//! WHAT: three hostile archives are built **at run time** in a temporary
//!       directory and measured: an archive that declares four gigabytes of
//!       content, an archive whose single entry expands far beyond its own
//!       size, and an archive with more entries than the specification allows.
//!       The test compares the archives with the limits of the specification
//!       and proves that each attack exceeds its limit.
//! WHY:  the specification demands three limits for every extraction — a limit
//!       for the total size, a limit for the number of entries and a limit for
//!       the size of a single entry — and demands a negative test that
//!       performs the attack: "Für jede Härtung — Pfad-Traversal,
//!       Dekompressionsbombe, Header-Limit — existiert ein negativer Test,
//!       der den Angriff tatsächlich ausführt und die Abwehr belegt."
//!
//!       A bomb that is built by hand at run time is also the only honest
//!       form of this test: a binary file in the repository is an attack
//!       sample, goes stale with every change of the archive library and
//!       hides the question "what exactly does this file contain?".
//!
//!       The part of the defence that is *provable today* is the budget: the
//!       limits of the specification are exceeded by the archives below, and
//!       the constants of the core crate are the numbers that decide it. The
//!       part that is **not** provable today is the abort during the
//!       extraction — the container readers arrive in milestone M1. The status
//!       of every row is written down in `tests/security/README.md`.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test security_decompression_bomb
//! ```

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use phocommander_core::{MAX_ARCHIVE_ENTRIES, MAX_EXTRACTED_BYTES};
use phocommander_testkit::TempTree;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// The size the hostile archive claims to hold: three gigabytes.
///
/// WHY: the limit of the specification is two gigabytes. A bomb that claims
/// more makes the difference visible without depending on the exact boundary.
/// Three gigabytes fit into the four byte size field of the ZIP format; the
/// value 0xFFFFFFFF would be read as the marker for the ZIP64 extension and
/// would not be a forged size but a request for another format.
const DECLARED_BOMB_BYTES: u64 = 3 * 1024 * 1024 * 1024;

/// The logical size of the highly compressible entry.
const HIGH_RATIO_BYTES: u64 = 8 * 1024 * 1024;

/// The smallest ratio between the logical and the stored size that counts as
/// a bomb for this test.
const MINIMUM_EXPANSION_RATIO: u64 = 1000;

/// Builds a ZIP archive with the given entries.
fn build_archive(path: &Path, entries: &[(&str, &[u8])]) {
    let file = match File::create(path) {
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

/// Rewrites the size that the archive declares for its single entry.
///
/// WHY: this is the header attack of the threat model (H12): an archive that
/// claims a size it does not have. A reader that believes the header will
/// allocate the claimed size or write until the disk is full; a reader that
/// counts the bytes it really writes is not affected. The archive is built
/// normally and the two size fields of the format — the one in the local
/// header and the one in the central directory — are then rewritten.
fn declare_fake_size(path: &Path, declared: u64) {
    let mut bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => panic!("the archive {path:?} could not be read: {error}"),
    };

    // Local file header: the uncompressed size stands at offset 22.
    let local = match find_signature(&bytes, &[0x50, 0x4b, 0x03, 0x04]) {
        Some(offset) => offset,
        None => panic!("the archive {path:?} has no local file header"),
    };
    // Central directory header: the uncompressed size stands at offset 24.
    let central = match find_signature(&bytes, &[0x50, 0x4b, 0x01, 0x02]) {
        Some(offset) => offset,
        None => panic!("the archive {path:?} has no central directory header"),
    };

    // The size field of the ZIP format is four bytes wide.
    let size = u32::try_from(declared)
        .map(|value| value.to_le_bytes())
        .unwrap_or_else(|_| panic!("the forged size {declared} does not fit into the format"));
    bytes[local + 22..local + 26].copy_from_slice(&size);
    bytes[central + 24..central + 28].copy_from_slice(&size);

    if let Err(error) = fs::write(path, &bytes) {
        panic!("the archive {path:?} could not be written: {error}");
    }
}

/// Returns the offset of the first occurrence of a signature.
fn find_signature(bytes: &[u8], signature: &[u8]) -> Option<usize> {
    bytes
        .windows(signature.len())
        .position(|window| window == signature)
}

/// Returns the size that the archive declares for its first entry.
fn declared_size(path: &Path) -> u64 {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => panic!("the archive {path:?} could not be opened: {error}"),
    };
    let mut zip = match ZipArchive::new(file) {
        Ok(zip) => zip,
        Err(error) => panic!("the archive {path:?} could not be read: {error}"),
    };
    let entry = match zip.by_index(0) {
        Ok(entry) => entry,
        Err(error) => panic!("the first entry could not be read: {error}"),
    };
    entry.size()
}

/// Returns the number of entries of an archive.
fn entry_count(path: &Path) -> usize {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => panic!("the archive {path:?} could not be opened: {error}"),
    };
    match ZipArchive::new(file) {
        Ok(zip) => zip.len(),
        Err(error) => panic!("the archive {path:?} could not be read: {error}"),
    }
}

#[test]
fn the_bomb_archive_declares_more_bytes_than_the_budget_allows() {
    // arrange
    let tree = match TempTree::new("bomb-size") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("bomb.zip");
    build_archive(&archive, &[("bomb.bin", b"payload")]);
    declare_fake_size(&archive, DECLARED_BOMB_BYTES);

    // act
    let declared = declared_size(&archive);

    // assert — the archive claims more than the limit of the specification
    assert_eq!(
        declared, DECLARED_BOMB_BYTES,
        "the archive does not declare the forged size"
    );
    assert!(
        declared > MAX_EXTRACTED_BYTES,
        "the bomb declares {declared} bytes, the limit is {MAX_EXTRACTED_BYTES} bytes"
    );
}

#[test]
fn the_bomb_archive_costs_the_attacker_almost_nothing() {
    // arrange
    let tree = match TempTree::new("bomb-size") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("bomb.zip");
    build_archive(&archive, &[("bomb.bin", b"payload")]);
    declare_fake_size(&archive, DECLARED_BOMB_BYTES);

    // act
    let stored = match fs::metadata(&archive) {
        Ok(metadata) => metadata.len(),
        Err(error) => panic!("the archive {archive:?} could not be measured: {error}"),
    };

    // assert — the whole point of the attack: a few kilobytes on the disk,
    // gigabytes in the extraction
    assert!(
        stored < 64 * 1024,
        "the bomb weighs {stored} bytes, the attack should be cheap"
    );
    assert!(
        DECLARED_BOMB_BYTES / stored.max(1) > 1000,
        "the expansion ratio of the bomb is too small to be a bomb"
    );
}

#[test]
fn a_highly_compressible_entry_expands_far_beyond_its_own_size() {
    // arrange
    let tree = match TempTree::new("bomb-ratio") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("ratio.zip");
    let zeros = vec![0u8; HIGH_RATIO_BYTES as usize];
    build_archive(&archive, &[("zeros.bin", &zeros)]);

    // act
    let stored = match fs::metadata(&archive) {
        Ok(metadata) => metadata.len(),
        Err(error) => panic!("the archive {archive:?} could not be measured: {error}"),
    };
    let declared = declared_size(&archive);

    // assert — the entry really expands by the factor that makes it a bomb
    assert_eq!(declared, HIGH_RATIO_BYTES);
    assert!(
        HIGH_RATIO_BYTES / stored.max(1) >= MINIMUM_EXPANSION_RATIO,
        "the expansion ratio is {}, the test needs at least {MINIMUM_EXPANSION_RATIO}",
        HIGH_RATIO_BYTES / stored.max(1)
    );
}

#[test]
fn an_archive_with_too_many_entries_exceeds_the_entry_limit() {
    // arrange
    let tree = match TempTree::new("bomb-count") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("many.zip");
    let count = MAX_ARCHIVE_ENTRIES as usize + 1;
    let file = match File::create(&archive) {
        Ok(file) => file,
        Err(error) => panic!("the archive {archive:?} could not be created: {error}"),
    };
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for index in 0..count {
        let name = format!("entry-{index}.txt");
        if let Err(error) = writer.start_file(name, options) {
            panic!("the entry {index} could not be started: {error}");
        }
    }
    if let Err(error) = writer.finish() {
        panic!("the archive {archive:?} could not be finished: {error}");
    }

    // act
    let entries = entry_count(&archive);

    // assert — the archive holds more entries than the limit of the
    // specification allows
    assert_eq!(entries, count);
    assert!(
        entries as u64 > MAX_ARCHIVE_ENTRIES,
        "the archive holds {entries} entries, the limit is {MAX_ARCHIVE_ENTRIES}"
    );
}
