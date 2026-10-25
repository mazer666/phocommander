//! The tests of the adapter of the TAR format.
//!
//! WHAT: the round trip of a TAR container — with and without a compression
//!       — the directories, the symbolic links, the names that are not valid
//!       UTF-8 and the files that are no container at all.
//! WHY: the TAR format carries more information than the other two: the mode
//!       bits, the kind of the entry and the target of a link. An adapter
//!       that loses one of them breaks the listing of the panel and the
//!       extraction, and the loss stays invisible until a user opens an
//!       archive that carries a link.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-archive formats::tar_tests
//! ```

use std::io::Cursor;
use std::path::Path;

use crate::entry::EntryKind;
use crate::entry_name::EntryNameError;
use crate::error::{ArchiveError, RefusalReason};
use crate::formats::tar::{TarArchiveReader, TarArchiveWriter};
use crate::formats::test_support::{CollectedEntry, TempDir, collect};
use crate::formats::{ArchiveWriter, TarCompression};

/// The content that the tests write and read.
const CONTENT: &[u8] = b"the quick brown fox";

/// The name of the file that the tests write.
const FILE_NAME: &str = "docs/readme.md";

/// Writes a TAR container with one file.
fn write_container(path: &Path, compression: TarCompression) {
    let mut writer = TarArchiveWriter::create(path, compression)
        .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
    let mut content = Cursor::new(CONTENT.to_vec());
    writer
        .add_file(FILE_NAME, CONTENT.len() as u64, &mut content)
        .unwrap_or_else(|error| panic!("the entry {FILE_NAME:?} could not be written: {error}"));
    Box::new(writer)
        .finish()
        .unwrap_or_else(|error| panic!("the container {path:?} could not be finished: {error}"));
}

/// Reads a TAR container and returns its entries.
fn read_container(path: &Path, compression: TarCompression) -> Vec<CollectedEntry> {
    let mut reader = TarArchiveReader::open(path, compression)
        .unwrap_or_else(|error| panic!("the container {path:?} could not be opened: {error}"));
    collect(&mut reader)
        .unwrap_or_else(|error| panic!("the container {path:?} could not be read: {error}"))
}

/// Returns the entry with the given name, or fails with the whole list.
fn entry<'a>(entries: &'a [CollectedEntry], name: &str) -> &'a CollectedEntry {
    entries
        .iter()
        .find(|candidate| candidate.name == name)
        .unwrap_or_else(|| panic!("the entry {name:?} is missing in {entries:?}"))
}

#[test]
fn the_reader_returns_the_entries_that_were_written() {
    // arrange
    let directory = TempDir::new("tar-entries");
    let path = directory.join("archive.tar");
    write_container(&path, TarCompression::None);

    // act
    let entries = read_container(&path, TarCompression::None);

    // assert
    assert_eq!(entries.len(), 1, "{entries:?}");
    let file = entry(&entries, FILE_NAME);
    assert_eq!(file.kind, EntryKind::File);
    assert_eq!(file.size, CONTENT.len() as u64);
    assert_eq!(file.mode, Some(0o644));
    assert!(!file.name_is_lossy);
}

#[test]
fn the_reader_returns_the_content_that_were_written() {
    // arrange
    let directory = TempDir::new("tar-content");
    let path = directory.join("archive.tar");
    write_container(&path, TarCompression::None);

    // act
    let entries = read_container(&path, TarCompression::None);

    // assert
    let file = entry(&entries, FILE_NAME);
    assert_eq!(file.content.as_deref(), Some(CONTENT));
}

#[test]
fn the_container_keeps_the_structure_of_the_directories() {
    // arrange
    let directory = TempDir::new("tar-directories");
    let path = directory.join("archive.tar");
    let mut writer = TarArchiveWriter::create(&path, TarCompression::None)
        .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
    writer
        .add_directory("docs")
        .unwrap_or_else(|error| panic!("the directory could not be written: {error}"));
    let mut content = Cursor::new(CONTENT.to_vec());
    writer
        .add_file(FILE_NAME, CONTENT.len() as u64, &mut content)
        .unwrap_or_else(|error| panic!("the entry {FILE_NAME:?} could not be written: {error}"));
    Box::new(writer)
        .finish()
        .unwrap_or_else(|error| panic!("the container {path:?} could not be finished: {error}"));

    // act
    let entries = read_container(&path, TarCompression::None);

    // assert
    assert_eq!(entry(&entries, "docs").kind, EntryKind::Directory);
    assert_eq!(entry(&entries, FILE_NAME).kind, EntryKind::File);
}

#[test]
fn the_reader_reports_a_link_as_a_link_and_not_as_its_target() {
    // arrange
    let directory = TempDir::new("tar-link");
    let path = directory.join("archive.tar");
    let mut writer = TarArchiveWriter::create(&path, TarCompression::None)
        .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
    let mut content = Cursor::new(CONTENT.to_vec());
    writer
        .add_file(FILE_NAME, CONTENT.len() as u64, &mut content)
        .unwrap_or_else(|error| panic!("the entry {FILE_NAME:?} could not be written: {error}"));
    writer
        .add_symlink("link", FILE_NAME)
        .unwrap_or_else(|error| panic!("the link could not be written: {error}"));
    Box::new(writer)
        .finish()
        .unwrap_or_else(|error| panic!("the container {path:?} could not be finished: {error}"));

    // act
    let entries = read_container(&path, TarCompression::None);

    // assert
    let link = entry(&entries, "link");
    assert_eq!(link.kind, EntryKind::Symlink);
    assert_eq!(link.link_target.as_deref(), Some(FILE_NAME));
    assert_eq!(link.content, None);
}

#[test]
fn the_container_is_read_back_with_every_compression() {
    // arrange, act, assert
    for compression in [
        TarCompression::None,
        TarCompression::Gzip,
        TarCompression::BZip2,
        TarCompression::Xz,
        TarCompression::Zstd,
    ] {
        let directory = TempDir::new("tar-compression");
        let path = directory.join("archive.tar");
        write_container(&path, compression);

        let mut reader = TarArchiveReader::open(&path, compression)
            .unwrap_or_else(|error| panic!("the container could not be opened: {error}"));
        let entries = collect(&mut reader)
            .unwrap_or_else(|error| panic!("the container could not be read: {error}"));

        assert_eq!(entries.len(), 1, "{compression:?}");
        assert_eq!(
            entries[0].content.as_deref(),
            Some(CONTENT),
            "{compression:?}"
        );
    }
}

#[test]
fn a_file_that_is_no_container_is_an_error_and_not_a_panic() {
    // arrange
    let directory = TempDir::new("tar-error");
    let path = directory.write_file("notes.txt", b"this is not a container at all");

    // act
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut reader = match TarArchiveReader::open(&path, TarCompression::None) {
            Ok(reader) => reader,
            Err(error) => return Err(error),
        };
        collect(&mut reader)
    }));

    // assert
    let error = outcome
        .unwrap_or_else(|_| panic!("the reader panicked on a file that is no container"))
        .err()
        .unwrap_or_else(|| panic!("the file was read as a container"));
    assert!(matches!(&error, ArchiveError::Format { .. }), "{error:?}");
}

#[test]
fn the_writer_refuses_a_name_that_leaves_the_destination() {
    // arrange
    let directory = TempDir::new("tar-refused-name");
    let path = directory.join("archive.tar");
    let mut writer = TarArchiveWriter::create(&path, TarCompression::None)
        .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
    let mut content = Cursor::new(CONTENT.to_vec());

    // act
    let result = writer.add_file("../escape.txt", CONTENT.len() as u64, &mut content);

    // assert
    assert!(
        matches!(
            &result,
            Err(ArchiveError::Refused {
                reason: RefusalReason::Name(EntryNameError::Traversal { .. }),
                ..
            })
        ),
        "{result:?}"
    );
}

#[cfg(unix)]
#[test]
fn a_name_that_is_not_valid_utf8_is_reported_as_lossy_and_is_not_cut_short() {
    use std::ffi::OsStr;
    use std::fs::File;
    use std::os::unix::ffi::OsStrExt;
    use tar::{Builder, Header};

    // arrange: the container is written with the container crate itself and
    // not with the writer of the project, so that the test proves the
    // behaviour against a foreign producer. The name of a TAR entry is a
    // sequence of bytes, so a producer may write bytes that are no UTF-8.
    let directory = TempDir::new("tar-lossy-name");
    let path = directory.join("archive.tar");
    {
        let file = File::create(&path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
        let mut builder = Builder::new(file);
        let mut header = Header::new_gnu();
        header.set_size(CONTENT.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_cksum();
        let name = OsStr::from_bytes(b"broken\xffname.txt");
        builder
            .append_data(&mut header, Path::new(name), CONTENT)
            .unwrap_or_else(|error| panic!("the entry could not be written: {error}"));
        builder.finish().unwrap_or_else(|error| {
            panic!("the container {path:?} could not be finished: {error}")
        });
    }

    // act
    let entries = read_container(&path, TarCompression::None);

    // assert
    assert_eq!(entries.len(), 1, "{entries:?}");
    let file = &entries[0];
    assert!(file.name_is_lossy, "the name was not marked as lossy");
    assert_eq!(file.name, "broken\u{fffd}name.txt");
}
