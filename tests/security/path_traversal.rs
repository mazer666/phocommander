//! Negative test: the path traversal attacks against the extraction of an
//! archive.
//!
//! WHAT: a hostile ZIP archive is built **at run time** in a temporary
//!       directory — no binary file is stored in the repository — and every
//!       entry name of that archive is handed to the hardened name resolution
//!       of `phocommander_archive`. The test proves that every hostile name
//!       is refused and that every accepted name stays inside the destination
//!       directory.
//! WHY:  the specification lists the path traversal as the first attack
//!       surface ("Pfad-Traversal bei der Extraktion (Zip-Slip)") and demands
//!       a negative test that performs the attack: "Für jede Härtung — Pfad-
//!       Traversal, Dekompressionsbombe, Header-Limit — existiert ein
//!       negativer Test, der den Angriff tatsächlich ausführt und die Abwehr
//!       belegt." A positive test — "a normal archive is extracted" — proves
//!       nothing about the defence.
//!
//!       The archive is built from the entry names of the real attack
//!       catalogue of the threat model: `../../escape.txt` is the classic zip
//!       slip, `/etc/passwd` the absolute path, `C:\Windows\evil.dll` the
//!       drive letter of the Windows variant, `docs/../../escape.txt` the
//!       variant that hides the traversal in the middle of the name.
//!
//!       The container readers — the part that writes the files — arrive in
//!       milestone M1. What is provable today is the layer that decides
//!       *where* a file may be written: `resolve_entry_name`. The test
//!       therefore asserts the invariant of that layer for every name of the
//!       hostile archive.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test security_path_traversal
//! ```

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use phocommander_archive::{EntryNameError, resolve_entry_name};
use phocommander_testkit::{TempTree, count_files, directory_names};
use proptest::prelude::*;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// The hostile entry names of the attack catalogue of the threat model.
const HOSTILE_NAMES: &[&str] = &[
    "../escape.txt",
    "../../escape.txt",
    "docs/../../escape.txt",
    "..\\escape.txt",
    "docs\\..\\..\\escape.txt",
    "/etc/passwd",
    "\\windows\\system32\\evil.dll",
    "C:\\Windows\\evil.dll",
    "d:evil.txt",
];

/// The one harmless entry, so that the archive is not only hostile.
const BENIGN_NAME: &str = "docs/readme.md";

/// Builds a ZIP archive with the given entries.
///
/// WHY: the archive is created at run time and lives in the temporary
/// directory of the test. A binary file in the repository would be an attack
/// sample that nobody asked for — and it would go stale with every change of
/// the archive format.
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

/// Builds the hostile archive of the test and returns its path.
fn hostile_archive(tree: &TempTree) -> PathBuf {
    let path = tree.path().join("hostile.zip");
    let mut entries: Vec<(&str, &[u8])> = Vec::new();
    for name in HOSTILE_NAMES {
        entries.push((name, b"payload"));
    }
    entries.push((BENIGN_NAME, b"harmless"));
    build_archive(&path, &entries);
    path
}

/// Returns the entry names of an archive, in the order of the archive.
fn entry_names(archive: &Path) -> Vec<String> {
    let file = match File::open(archive) {
        Ok(file) => file,
        Err(error) => panic!("the archive {archive:?} could not be opened: {error}"),
    };
    let mut zip = match ZipArchive::new(file) {
        Ok(zip) => zip,
        Err(error) => panic!("the archive {archive:?} could not be read: {error}"),
    };
    let mut names = Vec::new();
    for index in 0..zip.len() {
        let entry = match zip.by_index(index) {
            Ok(entry) => entry,
            Err(error) => panic!("the entry {index} could not be read: {error}"),
        };
        let name = match entry.name() {
            Ok(name) => name.to_string(),
            Err(error) => panic!("the name of the entry {index} could not be read: {error}"),
        };
        names.push(name);
    }
    names
}

#[test]
fn the_hostile_archive_really_carries_the_hostile_names() {
    // arrange
    let tree = match TempTree::new("zip-slip") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = hostile_archive(&tree);

    // act
    let names = entry_names(&archive);

    // assert — the attack material exists; without this assertion the rest of
    // the file would prove nothing about an archive that was never built.
    assert_eq!(names.len(), HOSTILE_NAMES.len() + 1);
    for hostile in HOSTILE_NAMES {
        assert!(
            names.iter().any(|name| name == hostile),
            "the archive does not contain the hostile name {hostile}: {names:?}"
        );
    }
    assert!(names.iter().any(|name| name == BENIGN_NAME));
}

#[test]
fn every_hostile_entry_name_is_refused() {
    // arrange
    let tree = match TempTree::new("zip-slip") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = hostile_archive(&tree);

    // act
    let names = entry_names(&archive);

    // assert — every hostile name is refused with the reason that names it
    for name in &names {
        if name == BENIGN_NAME {
            continue;
        }
        let result = resolve_entry_name(name);
        match result {
            Ok(path) => panic!("the hostile name {name} was accepted as {path:?}"),
            Err(error) => {
                assert!(
                    matches!(
                        error,
                        EntryNameError::Traversal { .. }
                            | EntryNameError::Absolute { .. }
                            | EntryNameError::ReservedName { .. }
                    ),
                    "the hostile name {name} was refused with the wrong reason: {error:?}"
                );
            }
        }
    }
}

#[test]
fn the_harmless_entry_name_is_accepted() {
    // act
    let result = resolve_entry_name(BENIGN_NAME);

    // assert
    match result {
        Ok(path) => assert_eq!(path.to_string_lossy(), BENIGN_NAME),
        Err(error) => panic!("the harmless name {BENIGN_NAME} was refused: {error}"),
    }
}

#[test]
fn every_accepted_name_stays_inside_the_destination_directory() {
    // arrange
    let tree = match TempTree::new("zip-slip") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = hostile_archive(&tree);
    let destination = tree.path().join("destination");
    if let Err(error) = std::fs::create_dir_all(&destination) {
        panic!("the destination {destination:?} could not be created: {error}");
    }

    // act
    let names = entry_names(&archive);
    let mut accepted = 0;
    for name in &names {
        if let Ok(relative) = resolve_entry_name(name) {
            accepted += 1;
            let target = destination.join(&relative);

            // assert — the resolved path is below the destination and carries
            // no `..` segment, so it cannot leave the destination
            assert!(
                target.starts_with(&destination),
                "the entry {name} would be written to {target:?}, outside of {destination:?}"
            );
            assert!(
                !relative
                    .components()
                    .any(|component| component.as_os_str() == ".."),
                "the entry {name} resolved to a path with a `..` segment: {relative:?}"
            );
        }
    }

    // assert — exactly one entry of the archive is harmless, the other nine
    // are refused
    assert_eq!(
        accepted, 1,
        "the archive {names:?} was not refused as expected"
    );
}

#[test]
fn a_name_that_is_too_long_is_refused() {
    // WHY: a name that is longer than any file system accepts is either an
    // attack or a mistake. On Windows the limit of the path is 260 characters
    // without the opt-in, and the name of an archive entry is added to the
    // path of the destination — the sum of both is what the operating system
    // sees. The resolver therefore refuses a name that is too long before it
    // reaches the file system.
    use phocommander_archive::MAX_ENTRY_NAME_BYTES;

    // arrange
    let too_long = "a".repeat(MAX_ENTRY_NAME_BYTES + 1);
    let at_the_limit = "a".repeat(MAX_ENTRY_NAME_BYTES);

    // act
    let refused = resolve_entry_name(&too_long);
    let accepted = resolve_entry_name(&at_the_limit);

    // assert
    match refused {
        Ok(path) => panic!("a name of {MAX_ENTRY_NAME_BYTES} bytes was accepted as {path:?}"),
        Err(error) => assert!(
            matches!(
                error,
                EntryNameError::TooLong {
                    length,
                    limit: MAX_ENTRY_NAME_BYTES
                } if length == MAX_ENTRY_NAME_BYTES + 1
            ),
            "the too long name was refused with the wrong reason: {error:?}"
        ),
    }
    assert!(accepted.is_ok(), "a name at the limit was refused");
}

#[test]
fn the_attack_really_writes_outside_of_the_destination_without_the_hardening() {
    // WHY: a hardening that is never attacked is a comment. This test runs the
    // attack against a deliberately unhardened reference extraction — the one
    // every archive library offers, the one that simply joins the name of the
    // entry to the destination — and shows that the file lands **outside** of
    // the destination. The attack is therefore real, not hypothetical, and the
    // test that follows proves that the hardened resolver stops it.
    let tree = match TempTree::new("zip-slip-attack") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = hostile_archive(&tree);
    let destination = tree.path().join("destination");
    if let Err(error) = std::fs::create_dir_all(&destination) {
        panic!("the destination {destination:?} could not be created: {error}");
    }

    // act — the unhardened extraction: join the name, create the parents, write
    for name in entry_names(&archive) {
        let target = destination.join(&name);
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(error) = std::fs::write(&target, b"payload") {
            panic!("the file {target:?} could not be written: {error}");
        }
    }

    // assert — the file landed next to the destination, not inside of it
    let escaped = tree.path().join("escape.txt");
    assert!(
        escaped.exists(),
        "the unhardened extraction did not escape the destination — the attack does not work, the test proves nothing"
    );
    assert!(
        !destination.join("escape.txt").exists(),
        "the file was written inside the destination — the attack did not work as described"
    );
}

#[test]
fn no_file_is_written_outside_of_the_destination_directory() {
    // arrange
    let tree = match TempTree::new("zip-slip") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let archive = hostile_archive(&tree);
    let destination = tree.path().join("destination");
    if let Err(error) = std::fs::create_dir_all(&destination) {
        panic!("the destination {destination:?} could not be created: {error}");
    }

    // act — the hostile names are resolved, exactly as a reader would do
    // before it writes a file
    for name in entry_names(&archive) {
        if let Ok(relative) = resolve_entry_name(&name) {
            let target = destination.join(relative);
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(error) = std::fs::write(&target, b"payload") {
                panic!("the file {target:?} could not be written: {error}");
            }
        }
    }

    // assert — the only file below the temporary directory is the archive
    // itself; nothing escaped
    let entries = directory_names(tree.path());
    assert_eq!(
        entries,
        vec!["destination".to_string(), "hostile.zip".to_string()],
        "a file was written outside of the destination directory: {entries:?}"
    );
    assert_eq!(
        count_files(&destination),
        1,
        "the destination does not hold exactly the one harmless entry"
    );
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(256))]

    /// WHY: the catalogue of hostile names is finite, the names an archive can
    /// carry are not. The property test therefore checks the invariant for
    /// names that nobody thought of: whatever the name is, the resolved path is
    /// relative, free of `..` and therefore inside the destination.
    #[test]
    fn the_resolved_path_stays_inside(name in proptest::collection::vec("[a-zA-Z0-9._\\\\/-]{0,64}", 0..6)) {
        let joined = name.concat();
        if let Ok(relative) = resolve_entry_name(&joined) {
            prop_assert!(relative.is_relative());
            prop_assert!(
                !relative.components().any(|component| component.as_os_str() == ".."),
                "the name {:?} resolved to {:?}, which contains a `..` segment",
                joined,
                relative
            );
            let destination = Path::new("/tmp/destination");
            let target = destination.join(&relative);
            prop_assert!(target.starts_with(destination));
        }
    }
}
