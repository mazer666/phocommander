//! The adapter of the TAR format.
//!
//! WHAT: reads the entries of a TAR container — plain or compressed with
//!       GZip, BZip2, XZ or Zstd — and writes a TAR container in the same
//!       four compressions.
//! WHY: the TAR format carries the name of an entry, the mode bits, the
//!       kind of the entry and, for a link, its target. The RustSec database
//!       knows two advisories against the extraction of the {tar} crate
//!       (RUSTSEC-2026-0067 and RUSTSEC-2026-0068), so the container is
//!       read and written here and never unpacked by the crate itself.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::formats::{TarCompression, tar::TarArchiveReader};
//!
//! # fn main() -> Result<(), phocommander_archive::ArchiveError> {
//! let reader = TarArchiveReader::open(
//!     std::path::Path::new("holiday.tar.gz"),
//!     TarCompression::Gzip,
//! )?;
//! # Ok(())
//! # }
//! ```

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::entry::{ArchiveEntry, EntryKind};
use crate::entry_name::resolve_entry_name;
use crate::error::{ArchiveError, RefusalReason};
use crate::formats::{ArchiveReader, ArchiveWriter, EntryVisitor, TarCompression, VisitDecision};

/// The largest number of bytes that are read from the target of a link.
///
/// WHY: the target of a link is a path. A path that no user can read is of
///       no use, and the limit keeps a hostile entry from making the process
///       allocate a huge buffer.
pub const MAX_LINK_TARGET_BYTES: u64 = 4096;

/// Reads the entries of a TAR container.
pub struct TarArchiveReader {
    archive: tar::Archive<Box<dyn Read>>,
    path: PathBuf,
}

impl TarArchiveReader {
    /// Opens a TAR container for reading.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::{TarCompression, tar::TarArchiveReader};
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let reader = TarArchiveReader::open(
    ///     std::path::Path::new("holiday.tar.xz"),
    ///     TarCompression::Xz,
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open(path: &Path, compression: TarCompression) -> Result<Self, ArchiveError> {
        let file = File::open(path).map_err(|source| ArchiveError::io(path, source))?;
        let reader: Box<dyn Read> = match compression {
            TarCompression::None => Box::new(file),
            TarCompression::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
            TarCompression::BZip2 => Box::new(bzip2::read::BzDecoder::new(file)),
            // Only a single XZ stream is accepted. A container that
            // concatenates several streams can hide content behind the first
            // one, and the extraction has no reason to accept that.
            TarCompression::Xz => Box::new(lzma_rust2::XzReader::new(file, false)),
            TarCompression::Zstd => {
                let decoder = zstd::stream::read::Decoder::new(file)
                    .map_err(|source| ArchiveError::io(path, source))?;
                Box::new(decoder)
            }
        };
        Ok(Self {
            archive: tar::Archive::new(reader),
            path: path.to_path_buf(),
        })
    }

    /// Returns the path of the archive.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::{TarCompression, tar::TarArchiveReader};
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let reader = TarArchiveReader::open(
    ///     std::path::Path::new("holiday.tar"),
    ///     TarCompression::None,
    /// )?;
    /// assert!(reader.path().ends_with("holiday.tar"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ArchiveReader for TarArchiveReader {
    fn for_each_entry(&mut self, visitor: &mut dyn EntryVisitor) -> Result<(), ArchiveError> {
        let entries = self
            .archive
            .entries()
            .map_err(|source| ArchiveError::io(&self.path, source))?;
        for entry in entries {
            let mut entry = entry.map_err(|error| ArchiveError::Format {
                reason: format!("an entry of the TAR container could not be read: {error}"),
            })?;

            let header = entry.header();
            let entry_type = header.entry_type();
            let kind = if entry_type.is_dir() {
                EntryKind::Directory
            } else if entry_type.is_symlink() {
                EntryKind::Symlink
            } else if entry_type.is_file() || entry_type.is_contiguous() {
                EntryKind::File
            } else {
                EntryKind::Other
            };
            let mode = header.mode().ok();
            let size = header.size().unwrap_or(0);
            let (name, name_is_lossy) = decode_name(&entry.path_bytes());

            let link_target = if kind == EntryKind::Symlink {
                Some(read_link_target(&mut entry)?)
            } else {
                None
            };

            let archive_entry = ArchiveEntry {
                name,
                name_is_lossy,
                kind,
                size,
                mode,
                link_target,
            };

            let content = if kind == EntryKind::File {
                Some(&mut entry as &mut dyn Read)
            } else {
                None
            };

            if visitor.visit(&archive_entry, content)? == VisitDecision::Stop {
                break;
            }
        }
        Ok(())
    }
}

/// Decodes the name of an entry.
///
/// WHY: a TAR container carries the name as raw bytes. A name that is not
///       valid UTF-8 is reported with the replacement character and marked as
///       lossy, so that the extraction refuses it instead of writing it under
///       a name that the archive does not carry.
fn decode_name(raw: &[u8]) -> (String, bool) {
    match std::str::from_utf8(raw) {
        Ok(name) => (name.to_string(), false),
        Err(_) => (String::from_utf8_lossy(raw).into_owned(), true),
    }
}

/// Reads the target of a link from the header of the entry.
fn read_link_target(entry: &mut tar::Entry<'_, Box<dyn Read>>) -> Result<String, ArchiveError> {
    let target = entry
        .link_name()
        .map_err(|source| ArchiveError::io("the target of a link", source))?;
    Ok(match target {
        Some(path) => path.to_string_lossy().into_owned(),
        None => String::new(),
    })
}

/// The sink of a TAR container: the file and, for a compressed container, the
/// compressor in front of it.
///
/// WHY: the type of the compressor differs, and the builder of the {tar}
///       crate needs a writer of a concrete type. The enum keeps the four
///       compressions in one place and gives them one {Write} implementation.
enum TarSink {
    /// The container is not compressed.
    Plain(File),

    /// The container is compressed with GZip.
    Gzip(flate2::write::GzEncoder<File>),

    /// The container is compressed with BZip2.
    BZip2(bzip2::write::BzEncoder<File>),

    /// The container is compressed with XZ.
    Xz(lzma_rust2::XzWriter<File>),

    /// The container is compressed with Zstd.
    Zstd(Box<zstd::stream::write::Encoder<'static, File>>),
}

impl Write for TarSink {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(file) => file.write(buffer),
            Self::Gzip(encoder) => encoder.write(buffer),
            Self::BZip2(encoder) => encoder.write(buffer),
            Self::Xz(encoder) => encoder.write(buffer),
            Self::Zstd(encoder) => encoder.write(buffer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(file) => file.flush(),
            Self::Gzip(encoder) => encoder.flush(),
            Self::BZip2(encoder) => encoder.flush(),
            Self::Xz(encoder) => encoder.flush(),
            Self::Zstd(encoder) => encoder.flush(),
        }
    }
}

/// Writes a TAR container.
pub struct TarArchiveWriter {
    builder: tar::Builder<TarSink>,
    path: PathBuf,
}

impl TarArchiveWriter {
    /// Creates a TAR container for writing.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::{TarCompression, tar::TarArchiveWriter};
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let writer = TarArchiveWriter::create(
    ///     std::path::Path::new("holiday.tar.zst"),
    ///     TarCompression::Zstd,
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(path: &Path, compression: TarCompression) -> Result<Self, ArchiveError> {
        let file = File::create(path).map_err(|source| ArchiveError::io(path, source))?;
        let sink = match compression {
            TarCompression::None => TarSink::Plain(file),
            TarCompression::Gzip => TarSink::Gzip(flate2::write::GzEncoder::new(
                file,
                flate2::Compression::default(),
            )),
            TarCompression::BZip2 => TarSink::BZip2(bzip2::write::BzEncoder::new(
                file,
                bzip2::Compression::default(),
            )),
            TarCompression::Xz => {
                let options = lzma_rust2::XzOptions::with_preset(6);
                let writer = lzma_rust2::XzWriter::new(file, options)
                    .map_err(|source| ArchiveError::io(path, source))?;
                TarSink::Xz(writer)
            }
            TarCompression::Zstd => {
                let encoder = zstd::stream::write::Encoder::new(file, 3)
                    .map_err(|source| ArchiveError::io(path, source))?;
                TarSink::Zstd(Box::new(encoder))
            }
        };
        Ok(Self {
            builder: tar::Builder::new(sink),
            path: path.to_path_buf(),
        })
    }

    /// Returns the path of the archive.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::formats::{TarCompression, tar::TarArchiveWriter};
    ///
    /// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
    /// let writer = TarArchiveWriter::create(
    ///     std::path::Path::new("holiday.tar"),
    ///     TarCompression::None,
    /// )?;
    /// assert!(writer.path().ends_with("holiday.tar"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl ArchiveWriter for TarArchiveWriter {
    fn add_file(
        &mut self,
        name: &str,
        size: u64,
        content: &mut dyn Read,
    ) -> Result<(), ArchiveError> {
        let relative = resolve_entry_name(name).map_err(|source| ArchiveError::Refused {
            name: name.to_string(),
            reason: RefusalReason::Name(source),
        })?;
        let mut header = tar::Header::new_gnu();
        header.set_size(size);
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_cksum();
        self.builder
            .append_data(&mut header, relative, content)
            .map_err(|source| {
                ArchiveError::io(format!("the entry {name:?} of the archive"), source)
            })
    }

    fn add_directory(&mut self, name: &str) -> Result<(), ArchiveError> {
        let relative = resolve_entry_name(name).map_err(|source| ArchiveError::Refused {
            name: name.to_string(),
            reason: RefusalReason::Name(source),
        })?;
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o755);
        header.set_entry_type(tar::EntryType::Directory);
        header.set_mtime(0);
        header.set_cksum();
        self.builder
            .append_data(&mut header, relative, io::empty())
            .map_err(|source| {
                ArchiveError::io(format!("the directory {name:?} of the archive"), source)
            })
    }

    fn add_symlink(&mut self, name: &str, target: &str) -> Result<(), ArchiveError> {
        let relative = resolve_entry_name(name).map_err(|source| ArchiveError::Refused {
            name: name.to_string(),
            reason: RefusalReason::Name(source),
        })?;
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o777);
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_mtime(0);
        header.set_cksum();
        self.builder
            .append_link(&mut header, relative, target)
            .map_err(|source| ArchiveError::io(format!("the link {name:?} of the archive"), source))
    }

    fn finish(self: Box<Self>) -> Result<(), ArchiveError> {
        let path = self.path.clone();
        let sink = self
            .builder
            .into_inner()
            .map_err(|source| ArchiveError::io(format!("the archive {path:?}"), source))?;
        finish_sink(sink, &path)
    }
}

/// Finishes the compressor of a TAR container.
///
/// WHY: a compressor writes its footer when it is finished. A container whose
///       footer is missing cannot be read by other tools, so the footer is
///       written here and not left to the drop of the compressor.
fn finish_sink(sink: TarSink, path: &Path) -> Result<(), ArchiveError> {
    match sink {
        TarSink::Plain(mut file) => file
            .flush()
            .map_err(|source| ArchiveError::io(path, source)),
        TarSink::Gzip(encoder) => encoder
            .finish()
            .map(|_| ())
            .map_err(|source| ArchiveError::io(path, source)),
        TarSink::BZip2(encoder) => encoder
            .finish()
            .map(|_| ())
            .map_err(|source| ArchiveError::io(path, source)),
        TarSink::Xz(encoder) => encoder
            .finish()
            .map(|_| ())
            .map_err(|source| ArchiveError::io(path, source)),
        TarSink::Zstd(encoder) => encoder
            .finish()
            .map(|_| ())
            .map_err(|source| ArchiveError::io(path, source)),
    }
}
