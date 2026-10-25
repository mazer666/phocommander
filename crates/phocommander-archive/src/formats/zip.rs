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
use zip::{CompressionMethod, ZipArchive, ZipWriter};

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
            let mut file = self
                .archive
                .by_index(index)
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
