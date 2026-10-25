//! The adapter of the 7z format.
//!
//! WHAT: reads the entries of a 7z container and writes a 7z container.
//! WHY: the 7z format is the third format of FR-67. It has no symbolic links
//!       and no Unix mode bits, so an entry of a 7z container is either a file
//!       or a directory — and the extraction gives it the permissions of the
//!       application, never the ones of the archive.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::formats::sevenz::SevenZArchiveReader;
//!
//! # fn main() -> Result<(), phocommander_archive::ArchiveError> {
//! let mut reader = SevenZArchiveReader::open(std::path::Path::new("holiday.7z"))?;
//! # Ok(())
//! # }
//! ```

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sevenz_rust2::ArchiveWriter as SevenZWriter;
use sevenz_rust2::{ArchiveEntry as SevenZEntry, EncoderConfiguration, EncoderMethod};

use crate::entry::{ArchiveEntry, EntryKind};
use crate::entry_name::resolve_entry_name;
use crate::error::{ArchiveError, RefusalReason};
use crate::formats::{ArchiveReader, ArchiveWriter, EntryVisitor, VisitDecision};

/// Reads the entries of a 7z container.
pub struct SevenZArchiveReader {
    reader: sevenz_rust2::ArchiveReader<File>,
    path: PathBuf,
}

impl SevenZArchiveReader {
    /// Opens a 7z container for reading.
    ///
    /// WHY: the container is opened with an empty password. An encrypted 7z
    ///       container needs the password of the user, and that is a decision
    ///       of the user interface, not of the adapter.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::sevenz::SevenZArchiveReader;
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let reader = SevenZArchiveReader::open(std::path::Path::new("holiday.7z"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open(path: &Path) -> Result<Self, ArchiveError> {
        let reader = sevenz_rust2::ArchiveReader::open(path, sevenz_rust2::Password::empty())
            .map_err(|error| ArchiveError::Format {
                reason: format!("the 7z container could not be opened: {error}"),
            })?;
        Ok(Self {
            reader,
            path: path.to_path_buf(),
        })
    }

    /// Returns the path of the archive.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::sevenz::SevenZArchiveReader;
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let reader = SevenZArchiveReader::open(std::path::Path::new("holiday.7z"))?;
    /// assert!(reader.path().ends_with("holiday.7z"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ArchiveReader for SevenZArchiveReader {
    fn for_each_entry(&mut self, visitor: &mut dyn EntryVisitor) -> Result<(), ArchiveError> {
        // The closure of the container crate cannot carry an error of this
        // crate, so the first failure is remembered and the iteration is
        // stopped at once.
        let mut failure: Option<ArchiveError> = None;

        self.reader
            .for_each_entries(|entry, content| {
                if failure.is_some() {
                    return Ok(false);
                }

                let archive_entry = convert_entry(entry);
                // WHY: a directory carries no content. The ZIP and the TAR
                //       adapter hand over nothing for a directory, and the
                //       three adapters behave the same way. The extraction
                //       checks the kind of the entry anyway, but a visitor
                //       that reads the content of a directory would read
                //       bytes that do not belong to it.
                let content: Option<&mut dyn Read> = match archive_entry.kind {
                    EntryKind::Directory => None,
                    _ => Some(content),
                };
                match visitor.visit(&archive_entry, content) {
                    Ok(VisitDecision::Continue) => Ok(true),
                    Ok(VisitDecision::Stop) => Ok(false),
                    Err(error) => {
                        failure = Some(error);
                        Ok(false)
                    }
                }
            })
            .map_err(|error| ArchiveError::Format {
                reason: format!("the 7z container could not be read: {error}"),
            })?;

        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

/// Translates an entry of the container crate into the model of this crate.
fn convert_entry(entry: &SevenZEntry) -> ArchiveEntry {
    let kind = if entry.is_directory {
        EntryKind::Directory
    } else {
        EntryKind::File
    };
    ArchiveEntry {
        name: entry.name.clone(),
        name_is_lossy: false,
        kind,
        size: entry.size,
        // The 7z format has no Unix mode bits. The extraction therefore uses
        // the permissions of the application for every entry.
        mode: None,
        link_target: None,
        // WHY: the 7z format stores the CRC-32 of the content of every entry
        // in its header. It is the value the integrity check compares the
        // content with.
        stored_checksum: entry.has_crc.then_some(entry.crc),
    }
}

/// Writes a 7z container.
pub struct SevenZArchiveWriter {
    writer: SevenZWriter<File>,
    path: PathBuf,
}

impl SevenZArchiveWriter {
    /// Creates a 7z container for writing.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::sevenz::SevenZArchiveWriter;
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let writer = SevenZArchiveWriter::create(std::path::Path::new("holiday.7z"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(path: &Path) -> Result<Self, ArchiveError> {
        let mut writer = SevenZWriter::create(path).map_err(|error| ArchiveError::Format {
            reason: format!("the 7z container could not be created: {error}"),
        })?;
        writer.set_content_methods(vec![EncoderConfiguration::new(EncoderMethod::LZMA2)]);
        Ok(Self {
            writer,
            path: path.to_path_buf(),
        })
    }

    /// Returns the path of the archive.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::sevenz::SevenZArchiveWriter;
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let writer = SevenZArchiveWriter::create(std::path::Path::new("holiday.7z"))?;
    /// assert!(writer.path().ends_with("holiday.7z"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ArchiveWriter for SevenZArchiveWriter {
    fn add_file(
        &mut self,
        name: &str,
        _size: u64,
        content: &mut dyn Read,
    ) -> Result<(), ArchiveError> {
        let relative = resolve_entry_name(name).map_err(|source| ArchiveError::Refused {
            name: name.to_string(),
            reason: RefusalReason::Name(source),
        })?;
        let entry = SevenZEntry::new_file(&relative.to_string_lossy());
        self.writer
            .push_archive_entry(entry, Some(content))
            .map_err(|error| ArchiveError::Format {
                reason: format!("the entry {name:?} could not be added: {error}"),
            })?;
        Ok(())
    }

    fn add_directory(&mut self, name: &str) -> Result<(), ArchiveError> {
        let relative = resolve_entry_name(name).map_err(|source| ArchiveError::Refused {
            name: name.to_string(),
            reason: RefusalReason::Name(source),
        })?;
        let entry = SevenZEntry::new_directory(&relative.to_string_lossy());
        self.writer
            .push_archive_entry(entry, None::<&[u8]>)
            .map_err(|error| ArchiveError::Format {
                reason: format!("the directory {name:?} could not be added: {error}"),
            })?;
        Ok(())
    }

    fn add_symlink(&mut self, name: &str, _target: &str) -> Result<(), ArchiveError> {
        // WHY: the 7z format of this crate stores no symbolic links. The
        //       writer says so instead of writing a file that pretends to be
        //       a link.
        Err(ArchiveError::Format {
            reason: format!("the 7z format cannot store the symbolic link {name:?}"),
        })
    }

    fn finish(self: Box<Self>) -> Result<(), ArchiveError> {
        let path = self.path.clone();
        self.writer
            .finish()
            .map_err(|source| ArchiveError::io(format!("the archive {path:?}"), source))?;
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
    use crate::formats::test_support::{CollectedEntry, TempDir, collect};
    use std::io::Cursor;

    /// The content that the tests write and read.
    const CONTENT: &[u8] = b"the quick brown fox";

    /// The name of the file that the tests write.
    const FILE_NAME: &str = "docs/readme.md";

    /// Writes a 7z container with one file.
    fn write_container(path: &Path) {
        let mut writer = SevenZArchiveWriter::create(path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
        let mut content = Cursor::new(CONTENT.to_vec());
        writer
            .add_file(FILE_NAME, CONTENT.len() as u64, &mut content)
            .unwrap_or_else(|error| {
                panic!("the entry {FILE_NAME:?} could not be written: {error}")
            });
        Box::new(writer).finish().unwrap_or_else(|error| {
            panic!("the container {path:?} could not be finished: {error}")
        });
    }

    /// Reads a 7z container and returns its entries.
    fn read_container(path: &Path) -> Vec<CollectedEntry> {
        let mut reader = SevenZArchiveReader::open(path)
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
        let directory = TempDir::new("sevenz-entries");
        let path = directory.join("archive.7z");
        write_container(&path);

        // act
        let entries = read_container(&path);

        // assert
        assert_eq!(entries.len(), 1, "{entries:?}");
        let file = entry(&entries, FILE_NAME);
        assert_eq!(file.kind, EntryKind::File);
        assert_eq!(file.size, CONTENT.len() as u64);
    }

    #[test]
    fn the_reader_returns_the_content_that_were_written() {
        // arrange
        let directory = TempDir::new("sevenz-content");
        let path = directory.join("archive.7z");
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
        let directory = TempDir::new("sevenz-directories");
        let path = directory.join("archive.7z");
        let mut writer = SevenZArchiveWriter::create(&path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));
        writer
            .add_directory("docs")
            .unwrap_or_else(|error| panic!("the directory could not be written: {error}"));
        let mut content = Cursor::new(CONTENT.to_vec());
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
        assert_eq!(entry(&entries, "docs").kind, EntryKind::Directory);
        assert_eq!(entry(&entries, FILE_NAME).kind, EntryKind::File);
    }

    #[test]
    fn the_reader_reports_the_checksum_that_the_container_stores() {
        // arrange
        let directory = TempDir::new("sevenz-checksum");
        let path = directory.join("archive.7z");
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
    fn the_name_of_an_entry_is_never_lossy_because_the_format_carries_it_as_utf16() {
        // WHY: the 7z format stores the name of an entry as UTF-16. A name
        //      that is not valid UTF-8 cannot be carried by the format at
        //      all, so the adapter never has to report a loss. The ZIP and
        //      the TAR format carry the name as bytes and can therefore
        //      carry such a name.
        // arrange
        let directory = TempDir::new("sevenz-lossy-name");
        let path = directory.join("archive.7z");
        write_container(&path);

        // act
        let entries = read_container(&path);

        // assert
        let file = entry(&entries, FILE_NAME);
        assert!(!file.name_is_lossy);
    }

    #[test]
    fn the_writer_refuses_to_store_a_symbolic_link() {
        // arrange
        let directory = TempDir::new("sevenz-link");
        let path = directory.join("archive.7z");
        let mut writer = SevenZArchiveWriter::create(&path)
            .unwrap_or_else(|error| panic!("the container {path:?} could not be created: {error}"));

        // act
        let result = writer.add_symlink("link", FILE_NAME);

        // assert
        assert!(
            matches!(&result, Err(ArchiveError::Format { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn a_file_that_is_no_container_is_an_error_and_not_a_panic() {
        // arrange
        let directory = TempDir::new("sevenz-error");
        let path = directory.write_file("notes.txt", b"this is not a container at all");

        // act
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            SevenZArchiveReader::open(&path)
        }));

        // assert
        let error = outcome
            .unwrap_or_else(|_| panic!("the reader panicked on a file that is no container"))
            .err()
            .unwrap_or_else(|| panic!("the file was opened as a container"));
        assert!(matches!(&error, ArchiveError::Format { .. }), "{error:?}");
    }
}
