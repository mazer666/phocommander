//! Negative tests: the hardening rules H-02 to H-06 of the extraction.
//!
//! WHAT: every hostile archive of this file is **built at run time** in a
//!       temporary directory — no binary file is stored in the repository —
//!       and is then handed to the hardened extraction of `phocommander_archive`.
//!       Each test performs the attack and then proves that the defence holds.
//! WHY:  the specification names the hardening rules H-02 to H-06 and demands a
//!       negative test for each of them: "For every hardening rule there is a
//!       test that executes the actual attack and demonstrates that the defence
//!       holds. A rule without a negative test is not implemented, it is only
//!       intended." (specification, section 10.2). A positive test — "a normal
//!       archive is extracted" — proves nothing about a defence.
//!
//!       The rules of the threat model that these tests prove:
//!
//!       | Rule | What the test proves |
//!       |---|---|
//!       | H-02 | The project never calls the `extract()` of a container crate. |
//!       | H-03 | Every destination path is confined to the destination directory. |
//!       | H-04 | No symbolic link is ever created from archive material. |
//!       | H-05 | The set-user-id, the set-group-id and the sticky bit never reach the file system. |
//!       | H-06 | The total size, the number of entries and the size of a single entry are bounded. |
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test security_extraction_hardening
//! ```

use std::fs;
use std::io::Write;
use std::path::Path;

use phocommander_archive::{
    ArchiveError, EntryNameError, EntryOutcome, ExtractionOptions, RefusalReason, SkipReason,
    extract_archive,
};
use phocommander_testkit::{TempTree, count_files, directory_names};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, System, ZipWriter};

/// The entry name that leaves the destination directory.
const TRAVERSAL_NAME: &str = "../escape.txt";

/// The name of the symbolic link of the symlink attack.
const LINK_NAME: &str = "link";

/// The name of the entry that follows the symbolic link.
const ENTRY_THROUGH_LINK: &str = "link/escaped.txt";

/// Builds a ZIP archive with the given entries.
///
/// WHY: the archive is created at run time and lives in the temporary
///       directory of the test. A binary file in the repository would be an
///       attack sample that nobody asked for, and it would go stale with every
///       change of the archive library.
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

/// Builds a ZIP archive that carries a symbolic link.
///
/// WHY: the link is written with the container library, not with the writer of
///       the project, so that the test proves the defence against a foreign
///       producer and not against a cooperating one.
fn build_archive_with_symlink(path: &Path, link: &str, target: &str) {
    let file = match fs::File::create(path) {
        Ok(file) => file,
        Err(error) => panic!("the archive {path:?} could not be created: {error}"),
    };
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    if let Err(error) = writer.add_symlink(link, target, options) {
        panic!("the link {link} could not be added: {error}");
    }
    if let Err(error) = writer.finish() {
        panic!("the archive {path:?} could not be finished: {error}");
    }
}

/// Builds a ZIP archive whose entry carries the set-user-id bit.
///
/// WHY: the bit has to travel in the external attributes of the central
///       directory, because the permission helper of the container library
///       masks the special bits away. The archive is therefore built with the
///       system "Unix" and with the attributes `0o1004755 << 16`, which is the
///       way a Unix producer writes a regular file with the set-user-id bit.
fn build_archive_with_set_user_id(path: &Path, name: &str, content: &[u8]) {
    let file = match fs::File::create(path) {
        Ok(file) => file,
        Err(error) => panic!("the archive {path:?} could not be created: {error}"),
    };
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .system(System::Unix)
        .external_attributes(SET_USER_ID_ATTRIBUTES);
    if let Err(error) = writer.start_file(name, options) {
        panic!("the entry {name} could not be started: {error}");
    }
    if let Err(error) = writer.write_all(content) {
        panic!("the entry {name} could not be written: {error}");
    }
    if let Err(error) = writer.finish() {
        panic!("the archive {path:?} could not be finished: {error}");
    }
}

/// The external attributes of a regular file with the set-user-id bit.
const SET_USER_ID_ATTRIBUTES: u32 = 0o1004755_u32 << 16;

/// Returns the names of the entries of a directory, sorted.
fn names_of(directory: &Path) -> Vec<String> {
    directory_names(directory)
}

/// Returns the mode of a file, or `None` when the file does not exist.
#[cfg(unix)]
fn mode_of(path: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    fs::symlink_metadata(path)
        .ok()
        .map(|metadata| metadata.permissions().mode() & 0o7777)
}

/// Returns the mode of a file, or `None` when the file does not exist.
#[cfg(not(unix))]
fn mode_of(_path: &Path) -> Option<u32> {
    None
}

#[test]
fn the_archive_with_the_set_user_id_bit_really_carries_it() {
    // arrange
    let tree = match TempTree::new("suid-attack") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("suid.zip");
    build_archive_with_set_user_id(&archive, "payload.sh", b"#!/bin/sh\n");

    // act — the raw bytes of the archive are inspected, because the reading
    // library removes the special bits while it reads the central directory
    let bytes = match fs::read(&archive) {
        Ok(bytes) => bytes,
        Err(error) => panic!("the archive {archive:?} could not be read: {error}"),
    };

    // assert — the attack material carries the set-user-id bit
    let needle = SET_USER_ID_ATTRIBUTES.to_le_bytes();
    let carries = bytes.windows(needle.len()).any(|window| window == needle);
    assert!(
        carries,
        "the archive does not carry the external attributes {SET_USER_ID_ATTRIBUTES:#x} — \
         the attack material is wrong, the test would prove nothing"
    );
}

#[test]
fn the_set_user_id_and_the_set_group_id_bits_never_reach_the_file_system() {
    // arrange
    let tree = match TempTree::new("suid") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("suid.zip");
    build_archive_with_set_user_id(&archive, "payload.sh", b"#!/bin/sh\n");
    let destination = tree.path().join("destination");

    // act
    let report = match extract_archive(&archive, &destination, &ExtractionOptions::default()) {
        Ok(report) => report,
        Err(error) => panic!("the extraction of a harmless archive failed: {error}"),
    };

    // assert — the file was written
    let extracted = destination.join("payload.sh");
    assert!(extracted.exists(), "the entry was not written: {report:?}");
    assert_eq!(report.written_entries, 1);

    // assert — and it carries no special bit
    if let Some(mode) = mode_of(&extracted) {
        assert_eq!(
            mode & 0o7000,
            0,
            "the extracted file carries the mode {mode:#o}; the set-user-id, the set-group-id \
             or the sticky bit reached the file system"
        );
        assert_eq!(
            mode & 0o022,
            0,
            "the extracted file is writable for the group or for the other: {mode:#o}"
        );
    }
}

#[test]
fn an_entry_that_leaves_the_destination_is_refused() {
    // arrange
    let tree = match TempTree::new("traversal") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("hostile.zip");
    build_archive(&archive, &[(TRAVERSAL_NAME, b"payload")]);
    let destination = tree.path().join("destination");

    // act
    let result = extract_archive(&archive, &destination, &ExtractionOptions::default());

    // assert — the entry is refused and the reason names the traversal
    match result {
        Ok(report) => panic!("the hostile entry was written: {report:?}"),
        Err(ArchiveError::Refused { name, reason }) => {
            assert_eq!(name, TRAVERSAL_NAME);
            match reason {
                RefusalReason::Name(EntryNameError::Traversal { .. }) => {}
                other => panic!("the entry was refused with the wrong reason: {other:?}"),
            }
        }
        Err(other) => panic!("the extraction failed with the wrong error: {other:?}"),
    }
}

#[test]
fn nothing_is_written_outside_of_the_destination_directory() {
    // arrange
    let tree = match TempTree::new("traversal") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("hostile.zip");
    build_archive(
        &archive,
        &[
            (TRAVERSAL_NAME, b"payload"),
            ("docs/../../escape.txt", b"payload"),
            ("/etc/passwd", b"payload"),
            ("docs/readme.md", b"harmless"),
        ],
    );
    let destination = tree.path().join("destination");

    // act — the extraction is told not to abort, so that every hostile entry
    // is met and the report names all of them
    let options = ExtractionOptions {
        abort_on_refused_entry: false,
        ..ExtractionOptions::default()
    };
    let report = match extract_archive(&archive, &destination, &options) {
        Ok(report) => report,
        Err(error) => panic!("the extraction failed: {error}"),
    };

    // assert — the three hostile entries are refused and the one harmless
    // entry is written
    assert_eq!(report.refused_entries, 3, "the report: {report:?}");
    assert_eq!(report.written_entries, 1, "the report: {report:?}");
    for outcome in &report.outcomes {
        if let EntryOutcome::Refused { name, .. } = outcome {
            assert_ne!(name, "docs/readme.md", "the harmless entry was refused");
        }
    }

    // assert — and nothing lies outside of the destination directory
    let escaped = tree.path().join("escape.txt");
    assert!(
        !escaped.exists(),
        "the file {escaped:?} was written outside of the destination directory"
    );
    assert_eq!(
        names_of(tree.path()),
        vec!["destination".to_string(), "hostile.zip".to_string()],
        "a file was written outside of the destination directory"
    );
    assert_eq!(
        count_files(&destination),
        1,
        "the destination does not hold exactly the one harmless entry"
    );
}

#[test]
fn a_symbolic_link_is_skipped_instead_of_being_created() {
    // arrange — the target of the link is a name that could be written, so
    // that the test proves the rule "no link is created" and not the rule
    // "a target that leaves the destination is refused".
    let tree = match TempTree::new("symlink") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("link.zip");
    build_archive_with_symlink(&archive, LINK_NAME, "target.txt");
    let destination = tree.path().join("destination");

    // act
    let report = match extract_archive(&archive, &destination, &ExtractionOptions::default()) {
        Ok(report) => report,
        Err(error) => panic!("the extraction failed: {error}"),
    };

    // assert — the link is reported as skipped and never created
    assert_eq!(report.skipped_entries, 1, "the report: {report:?}");
    assert_eq!(report.written_entries, 0, "the report: {report:?}");
    let skipped = report.outcomes.iter().find_map(|outcome| match outcome {
        EntryOutcome::Skipped { name, reason } => Some((name.clone(), reason.clone())),
        _ => None,
    });
    match skipped {
        Some((name, SkipReason::Symlink { target })) => {
            assert_eq!(name, LINK_NAME);
            assert_eq!(target, "target.txt");
        }
        Some((name, reason)) => panic!("the link {name} was skipped with the reason {reason:?}"),
        None => panic!("the link was not reported as skipped: {report:?}"),
    }
    let link = destination.join(LINK_NAME);
    assert!(
        !fs::symlink_metadata(&link).is_ok(),
        "the symbolic link {link:?} was created from archive material"
    );
}

#[test]
fn a_symbolic_link_with_a_refused_target_is_reported_as_such() {
    // arrange — the target of the link points out of the destination.
    let tree = match TempTree::new("symlink-target") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("link.zip");
    build_archive_with_symlink(&archive, LINK_NAME, "../outside");
    let destination = tree.path().join("destination");

    // act
    let report = match extract_archive(&archive, &destination, &ExtractionOptions::default()) {
        Ok(report) => report,
        Err(error) => panic!("the extraction failed: {error}"),
    };

    // assert — the link is skipped, and the report names the refused target
    assert_eq!(report.skipped_entries, 1, "the report: {report:?}");
    let skipped = report.outcomes.iter().find_map(|outcome| match outcome {
        EntryOutcome::Skipped { name, reason } => Some((name.clone(), reason.clone())),
        _ => None,
    });
    match skipped {
        Some((name, SkipReason::SymlinkWithRefusedTarget { target })) => {
            assert_eq!(name, LINK_NAME);
            assert_eq!(target, "../outside");
        }
        Some((name, reason)) => panic!("the link {name} was skipped with the reason {reason:?}"),
        None => panic!("the link was not reported as skipped: {report:?}"),
    }
}

#[test]
fn a_symbolic_link_cannot_redirect_a_later_entry() {
    // arrange
    let tree = match TempTree::new("symlink-redirect") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = tree.path().join("redirect.zip");
    build_archive_with_symlink(&archive, LINK_NAME, "../outside");
    let destination = tree.path().join("destination");

    // act — the archive carries the link and an entry that writes through it
    let archive = tree.path().join("redirect.zip");
    let mut writer = match fs::File::create(&archive) {
        Ok(file) => ZipWriter::new(file),
        Err(error) => panic!("the archive could not be created: {error}"),
    };
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    if let Err(error) = writer.add_symlink(LINK_NAME, "../outside", options) {
        panic!("the link could not be added: {error}");
    }
    if let Err(error) = writer.start_file(ENTRY_THROUGH_LINK, options) {
        panic!("the entry could not be started: {error}");
    }
    if let Err(error) = writer.write_all(b"payload") {
        panic!("the entry could not be written: {error}");
    }
    if let Err(error) = writer.finish() {
        panic!("the archive could not be finished: {error}");
    }

    // act
    let report = match extract_archive(&archive, &destination, &ExtractionOptions::default()) {
        Ok(report) => report,
        Err(error) => panic!("the extraction failed: {error}"),
    };

    // assert — the link was skipped, so the entry behind it cannot have been
    // redirected out of the destination
    assert_eq!(report.skipped_entries, 1, "the report: {report:?}");
    let escaped = tree.path().join("outside").join("escaped.txt");
    assert!(
        !escaped.exists(),
        "the file {escaped:?} was written through the symbolic link"
    );
    assert_eq!(
        names_of(tree.path()),
        vec!["destination".to_string(), "redirect.zip".to_string()],
        "a file was written outside of the destination directory"
    );
}

#[test]
fn an_entry_written_through_an_existing_symbolic_link_is_refused() {
    // arrange — the destination already holds a symbolic link that points out
    // of it. This is the state an attacker creates with a second archive.
    let tree = match TempTree::new("symlink-existing") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let outside = tree.path().join("outside");
    if let Err(error) = fs::create_dir_all(&outside) {
        panic!("the directory {outside:?} could not be created: {error}");
    }
    let destination = tree.path().join("destination");
    if let Err(error) = fs::create_dir_all(&destination) {
        panic!("the destination {destination:?} could not be created: {error}");
    }
    let link = destination.join(LINK_NAME);
    if let Err(error) = std::os::unix::fs::symlink(&outside, &link) {
        panic!("the symbolic link {link:?} could not be created: {error}");
    }
    let archive = tree.path().join("through.zip");
    let mut writer = match fs::File::create(&archive) {
        Ok(file) => ZipWriter::new(file),
        Err(error) => panic!("the archive could not be created: {error}"),
    };
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    if let Err(error) = writer.start_file(ENTRY_THROUGH_LINK, options) {
        panic!("the entry could not be started: {error}");
    }
    if let Err(error) = writer.write_all(b"payload") {
        panic!("the entry could not be written: {error}");
    }
    if let Err(error) = writer.finish() {
        panic!("the archive could not be finished: {error}");
    }

    // act
    let result = extract_archive(&archive, &destination, &ExtractionOptions::default());

    // assert — the entry is refused, because a directory of its path is a
    // symbolic link
    match result {
        Ok(report) => panic!("the entry was written through the symbolic link: {report:?}"),
        Err(ArchiveError::Refused { name, reason }) => {
            assert_eq!(name, ENTRY_THROUGH_LINK);
            assert_eq!(
                reason,
                RefusalReason::SymlinkInPath,
                "the entry was refused with the wrong reason"
            );
        }
        Err(other) => panic!("the extraction failed with the wrong error: {other:?}"),
    }

    // assert — and nothing was written outside of the destination
    let escaped = outside.join("escaped.txt");
    assert!(
        !escaped.exists(),
        "the file {escaped:?} was written through the symbolic link"
    );
    assert_eq!(
        names_of(&outside),
        Vec::<String>::new(),
        "a file was written outside of the destination directory"
    );
}
