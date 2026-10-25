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
                let content: &mut dyn Read = content;
                match visitor.visit(&archive_entry, Some(content)) {
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
