//! The adapter of the ZIP format.
//!
//! WHAT: reads the entries of a ZIP container and writes a ZIP container.
//! WHY: the ZIP format is the format that the "zip slip" made famous. It
//!       carries the name of an entry, the mode bits in the external
//!       attributes and the target of a link as the content of the entry.
//!       All three are chosen by whoever built the archive, so the adapter
//!       reports them and decides nothing.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::formats::zip::ZipArchiveReader;
//!
//! # fn main() -> Result<(), phocommander_archive::ArchiveError> {
//! let mut reader = ZipArchiveReader::open(std::path::Path::new("holiday.zip"))?;
//! # Ok(())
//! # }
//! ```

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipReadOptions, ZipWriter};

use crate::entry::{ArchiveEntry, EntryKind};
use crate::entry_name::resolve_entry_name;
use crate::error::ArchiveError;
use crate::formats::{ArchiveReader, ArchiveWriter, EntryVisitor, VisitDecision};

/// The largest number of bytes that are read from the target of a link.
///
/// WHY: the target of a link is a path, and a path that a user cannot read
///       is of no use. The limit keeps a hostile entry from making the
///       process allocate a huge buffer.
pub const MAX_LINK_TARGET_BYTES: u64 = 4096;

/// Reads the entries of a ZIP container.
pub struct ZipArchiveReader {
    archive: ZipArchive<File>,
    path: PathBuf,
}

impl ZipArchiveReader {
    /// Opens a ZIP container for reading.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::zip::ZipArchiveReader;
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let reader = ZipArchiveReader::open(std::path::Path::new("holiday.zip"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open(path: &Path) -> Result<Self, ArchiveError> {
        let file = File::open(path).map_err(|source| ArchiveError::io(path, source))?;
        let archive = ZipArchive::new(file).map_err(|error| ArchiveError::Format {
            reason: format!("the ZIP container could not be opened: {error}"),
        })?;
        Ok(Self {
            archive,
            path: path.to_path_buf(),
        })
    }

    /// Returns the path of the archive.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::zip::ZipArchiveReader;
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let reader = ZipArchiveReader::open(std::path::Path::new("holiday.zip"))?;
    /// assert!(reader.path().ends_with("holiday.zip"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ArchiveReader for ZipArchiveReader {
    fn for_each_entry(&mut self, visitor: &mut dyn EntryVisitor) -> Result<(), ArchiveError> {
        // The number of entries is read before the loop, because the file of
        // an entry borrows the archive while the visitor works with it.
        let count = self.archive.len();

        for index in 0..count {
            // WHY: the check of the container crate is switched off only when
            // the visitor asks for it. The integrity check compares the
            // checksum itself, so that its report can name the entry that
            // differs.
            let options =
                ZipReadOptions::default().ignore_crc32(visitor.skips_container_checksum());
            let mut file = self
                .archive
                .by_index_with_options(index, options)
                .map_err(|error| ArchiveError::Format {
                    reason: format!("the entry {index} could not be read: {error}"),
                })?;

            let (name, name_is_lossy) = decode_name(file.name_raw());
            let mode = file.unix_mode();
            let kind = if file.is_symlink() {
                EntryKind::Symlink
            } else if file.is_dir() {
                EntryKind::Directory
            } else {
                EntryKind::File
            };
            let size = file.size();
            // WHY: the ZIP container stores the CRC-32 of the content of
            // every entry in its directory. It is the value the integrity
            // check compares the content with.
            let stored_checksum = Some(u64::from(file.crc32()));

            // The target of a link is the content of the entry. It is read
            // here, because the extraction never creates a link and therefore
            // never looks at the content of one.
            let link_target = if kind == EntryKind::Symlink {
                Some(read_link_target(&mut file)?)
            } else {
                None
            };

            let entry = ArchiveEntry {
                name,
                name_is_lossy,
                kind,
                size,
                mode,
                link_target,
                stored_checksum,
            };

            let content = if kind == EntryKind::File {
                Some(&mut file as &mut dyn Read)
            } else {
                None
            };

            if visitor.visit(&entry, content)? == VisitDecision::Stop {
                break;
            }
        }
        Ok(())
    }
}

/// Decodes the name of an entry.
///
/// WHY: a ZIP container can carry a name that is not valid UTF-8. Such a name
///       is reported with the replacement character, and the entry is marked
///       as lossy so that the extraction refuses it instead of writing it
///       under a name that the archive does not carry.
fn decode_name(raw: &[u8]) -> (String, bool) {
    match std::str::from_utf8(raw) {
        Ok(name) => (name.to_string(), false),
        Err(_) => (String::from_utf8_lossy(raw).into_owned(), true),
    }
}

/// Reads the target of a link from the content of the entry.
fn read_link_target(file: &mut impl Read) -> Result<String, ArchiveError> {
    let mut target = String::new();
    let mut limited = file.take(MAX_LINK_TARGET_BYTES);
    limited
        .read_to_string(&mut target)
        .map_err(|source| ArchiveError::io("the target of a link", source))?;
    Ok(target)
}

/// Writes a ZIP container.
pub struct ZipArchiveWriter {
    writer: ZipWriter<File>,
    path: PathBuf,
}

impl ZipArchiveWriter {
    /// Creates a ZIP container for writing.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::zip::ZipArchiveWriter;
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let writer = ZipArchiveWriter::create(std::path::Path::new("holiday.zip"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(path: &Path) -> Result<Self, ArchiveError> {
        let file = File::create(path).map_err(|source| ArchiveError::io(path, source))?;
        Ok(Self {
            writer: ZipWriter::new(file),
            path: path.to_path_buf(),
        })
    }

    /// Returns the path of the archive.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::zip::ZipArchiveWriter;
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let writer = ZipArchiveWriter::create(std::path::Path::new("holiday.zip"))?;
    /// assert!(writer.path().ends_with("holiday.zip"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ArchiveWriter for ZipArchiveWriter {
    fn add_file(
        &mut self,
        name: &str,
        _size: u64,
        content: &mut dyn Read,
    ) -> Result<(), ArchiveError> {
        let relative = resolve_entry_name(name).map_err(|source| ArchiveError::Refused {
            name: name.to_string(),
            reason: crate::error::RefusalReason::Name(source),
        })?;
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        self.writer
            .start_file(relative.to_string_lossy().to_string(), options)
            .map_err(|error| ArchiveError::Format {
                reason: format!("the entry {name:?} could not be started: {error}"),
            })?;
        io::copy(content, &mut self.writer).map_err(|source| {
            ArchiveError::io(format!("the entry {name:?} of the archive"), source)
        })?;
        Ok(())
    }

    fn add_directory(&mut self, name: &str) -> Result<(), ArchiveError> {
        let relative = resolve_entry_name(name).map_err(|source| ArchiveError::Refused {
            name: name.to_string(),
            reason: crate::error::RefusalReason::Name(source),
        })?;
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        self.writer
            .add_directory(relative.to_string_lossy().to_string(), options)
            .map_err(|error| ArchiveError::Format {
                reason: format!("the directory {name:?} could not be added: {error}"),
            })?;
        Ok(())
    }

    fn add_symlink(&mut self, name: &str, target: &str) -> Result<(), ArchiveError> {
        let relative = resolve_entry_name(name).map_err(|source| ArchiveError::Refused {
            name: name.to_string(),
            reason: crate::error::RefusalReason::Name(source),
        })?;
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        self.writer
            .add_symlink(relative.to_string_lossy().to_string(), target, options)
            .map_err(|error| ArchiveError::Format {
                reason: format!("the link {name:?} could not be added: {error}"),
            })?;
        Ok(())
    }

    fn finish(self: Box<Self>) -> Result<(), ArchiveError> {
        self.writer.finish().map_err(|error| ArchiveError::Format {
            reason: format!("the archive {:?} could not be finished: {error}", self.path),
        })?;
        Ok(())
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::crc32::Crc32;
    use crate::entry_name::EntryNameError;
    use crate::error::RefusalReason;
    use crate::formats::test_support::{CollectedEntry, TempDir, collect};
    use std::fs;

    /// The content that the tests write and read.
    const CONTENT: &[u8] = b"the quick brown fox";

    /// The name of the file that the tests write.
    const FILE_NAME: &str = "docs/readme.md";

    /// Writes a ZIP container with one file.
    fn write_container(path: &Path) {
        let mut writer = ZipArchiveWriter::create(path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
        let mut content = io::Cursor::new(CONTENT.to_vec());
        writer
            .add_file(FILE_NAME, CONTENT.len() as u64, &mut content)
            .unwrap_or_else(|error| {
                panic!("the entry {FILE_NAME:?} could not be written: {error}")
            });
        Box::new(writer).finish().unwrap_or_else(|error| {
            panic!("the container {path:?} could not be finished: {error}")
        });
    }

    /// Reads a ZIP container and returns its entries.
    fn read_container(path: &Path) -> Vec<CollectedEntry> {
        let mut reader = ZipArchiveReader::open(path)
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

    /// Replaces every occurrence of `from` with `to` and counts them.
    ///
    /// WHY: the name of an entry stands in the header of the entry and in the
    ///       directory of the container. Every occurrence has to be replaced,
    ///       or the reader refuses the container as inconsistent.
    fn replace_bytes(data: &mut [u8], from: &[u8], to: &[u8]) -> usize {
        assert_eq!(
            from.len(),
            to.len(),
            "the replacement has to keep the length of the name"
        );
        let mut count = 0;
        let mut index = 0;
        while index + from.len() <= data.len() {
            if &data[index..index + from.len()] == from {
                data[index..index + from.len()].copy_from_slice(to);
                count += 1;
                index += from.len();
                continue;
            }
            index += 1;
        }
        count
    }

    #[test]
    fn the_reader_returns_the_entries_that_were_written() {
        // arrange
        let directory = TempDir::new("zip-entries");
        let path = directory.join("archive.zip");
        write_container(&path);

        // act
        let entries = read_container(&path);

        // assert
        assert_eq!(entries.len(), 1, "{entries:?}");
        let file = entry(&entries, FILE_NAME);
        assert_eq!(file.kind, EntryKind::File);
        assert_eq!(file.size, CONTENT.len() as u64);
        assert!(!file.name_is_lossy);
    }

    #[test]
    fn the_reader_returns_the_content_that_were_written() {
        // arrange
        let directory = TempDir::new("zip-content");
        let path = directory.join("archive.zip");
        write_container(&path);

        // act
        let entries = read_container(&path);

        // assert
        let file = entry(&entries, FILE_NAME);
        assert_eq!(file.content.as_deref(), Some(CONTENT));
    }

    #[test]
    fn the_container_keeps_the_structure_of_the_directories() {
        // arrange
        let directory = TempDir::new("zip-directories");
        let path = directory.join("archive.zip");
        let mut writer = ZipArchiveWriter::create(&path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
        writer
            .add_directory("docs")
            .unwrap_or_else(|error| panic!("the directory could not be written: {error}"));
        let mut content = io::Cursor::new(CONTENT.to_vec());
        writer
            .add_file(FILE_NAME, CONTENT.len() as u64, &mut content)
            .unwrap_or_else(|error| {
                panic!("the entry {FILE_NAME:?} could not be written: {error}")
            });
        Box::new(writer).finish().unwrap_or_else(|error| {
            panic!("the container {path:?} could not be finished: {error}")
        });

        // act
        let entries = read_container(&path);

        // assert
        // WHY: the ZIP format marks a directory with a slash at the end of
        //      the name, and the writer of the container crate adds it.
        assert_eq!(entry(&entries, "docs/").kind, EntryKind::Directory);
        assert_eq!(entry(&entries, FILE_NAME).kind, EntryKind::File);
    }

    #[test]
    fn the_reader_reports_a_link_as_a_link_and_not_as_its_target() {
        // arrange
        let directory = TempDir::new("zip-link");
        let path = directory.join("archive.zip");
        let mut writer = ZipArchiveWriter::create(&path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
        let mut content = io::Cursor::new(CONTENT.to_vec());
        writer
            .add_file(FILE_NAME, CONTENT.len() as u64, &mut content)
            .unwrap_or_else(|error| {
                panic!("the entry {FILE_NAME:?} could not be written: {error}")
            });
        writer
            .add_symlink("link", FILE_NAME)
            .unwrap_or_else(|error| panic!("the link could not be written: {error}"));
        Box::new(writer).finish().unwrap_or_else(|error| {
            panic!("the container {path:?} could not be finished: {error}")
        });

        // act
        let entries = read_container(&path);

        // assert
        let link = entry(&entries, "link");
        assert_eq!(link.kind, EntryKind::Symlink);
        assert_eq!(link.link_target.as_deref(), Some(FILE_NAME));
        assert_eq!(link.content, None);
    }

    #[test]
    fn the_reader_reports_the_checksum_that_the_container_stores() {
        // arrange
        let directory = TempDir::new("zip-checksum");
        let path = directory.join("archive.zip");
        write_container(&path);
        let mut checksum = Crc32::new();
        checksum.update(CONTENT);

        // act
        let entries = read_container(&path);

        // assert
        let file = entry(&entries, FILE_NAME);
        assert_eq!(file.stored_checksum, Some(u64::from(checksum.finish())));
    }

    #[test]
    fn a_file_that_is_no_container_is_an_error_and_not_a_panic() {
        // arrange
        let directory = TempDir::new("zip-error");
        let path = directory.write_file("notes.txt", b"this is not a container at all");

        // act
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ZipArchiveReader::open(&path)
        }));

        // assert
        let error = outcome
            .unwrap_or_else(|_| panic!("the reader panicked on a file that is no container"))
            .err()
            .unwrap_or_else(|| panic!("the file was opened as a container"));
        assert!(matches!(&error, ArchiveError::Format { .. }), "{error:?}");
    }

    #[test]
    fn the_writer_refuses_a_name_that_leaves_the_destination() {
        // arrange
        let directory = TempDir::new("zip-refused-name");
        let path = directory.join("archive.zip");
        let mut writer = ZipArchiveWriter::create(&path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
        let mut content = io::Cursor::new(CONTENT.to_vec());

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

    #[test]
    fn a_name_that_is_not_valid_utf8_is_reported_as_lossy_and_is_not_cut_short() {
        // arrange: the name of the entry is patched after the container was
        // written, because the writer of the container crate takes the name
        // as a string and cannot carry a name that is not valid UTF-8. A
        // patched container is what a foreign producer writes.
        let directory = TempDir::new("zip-lossy-name");
        let path = directory.join("archive.zip");
        let mut writer = ZipArchiveWriter::create(&path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
        let mut content = io::Cursor::new(CONTENT.to_vec());
        writer
            .add_file("notes.txt", CONTENT.len() as u64, &mut content)
            .unwrap_or_else(|error| panic!("the entry could not be written: {error}"));
        Box::new(writer).finish().unwrap_or_else(|error| {
            panic!("the container {path:?} could not be finished: {error}")
        });

        let mut data = fs::read(&path)
            .unwrap_or_else(|error| panic!("the container could not be read: {error}"));
        let occurrences = replace_bytes(&mut data, b"notes.txt", b"no\xffes.txt");
        assert!(occurrences > 0, "the name was not found in the container");
        fs::write(&path, &data)
            .unwrap_or_else(|error| panic!("the container could not be written: {error}"));

        // act
        let entries = read_container(&path);

        // assert
        assert_eq!(entries.len(), 1, "{entries:?}");
        let file = &entries[0];
        assert!(file.name_is_lossy, "the name was not marked as lossy");
        assert_eq!(file.name, "no\u{fffd}es.txt");
    }
}
