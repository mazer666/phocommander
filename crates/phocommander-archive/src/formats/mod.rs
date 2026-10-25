//! The container formats of Phocommander.
//!
//! WHAT: the traits that a container format implements, the detection of the
//!       format of a file and the three adapters — ZIP, TAR and 7z.
//! WHY: the container crates parse a container and nothing else. Every
//!       decision about *where* an entry is written, whether it is written at
//!       all and with which permissions is made by the extraction of this
//!       crate. That is what makes the hardening of the specification a
//!       property of the project and not of a dependency.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::formats::{ContainerFormat, detect_format};
//!
//! # fn main() -> Result<(), phocommander_archive::ArchiveError> {
//! let format = detect_format(std::path::Path::new("holiday.zip"))?;
//! assert_eq!(format, ContainerFormat::Zip);
//! # Ok(())
//! # }
//! ```

pub mod sevenz;
pub mod tar;
pub mod zip;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::entry::ArchiveEntry;
use crate::error::ArchiveError;

/// The container formats that the crate supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFormat {
    /// The ZIP format.
    Zip,

    /// The TAR format, with or without a compression.
    Tar,

    /// The 7z format.
    SevenZ,
}

/// The compression that a TAR container uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TarCompression {
    /// No compression at all.
    None,

    /// GZip.
    Gzip,

    /// BZip2.
    BZip2,

    /// XZ.
    Xz,

    /// Zstd.
    Zstd,
}

/// The decision of the visitor about the entry it has just seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitDecision {
    /// The extraction continues with the next entry.
    Continue,

    /// The extraction stops; the remaining entries are not read.
    Stop,
}

/// The visitor that the extraction is.
///
/// WHY: the visitor owns the rules — the limits, the path resolution, the
///       permissions — and the adapter only owns the container. The content
///       of an entry is handed over as a reader, so that a large entry never
///       has to be held in memory.
pub trait EntryVisitor {
    /// Handles one entry of the archive.
    ///
    /// The content is {None} for an entry that has none: a directory or a
    /// link.
    fn visit(
        &mut self,
        entry: &ArchiveEntry,
        content: Option<&mut dyn Read>,
    ) -> Result<VisitDecision, ArchiveError>;
}

/// A reader of one container format.
pub trait ArchiveReader {
    /// Hands every entry of the archive to the visitor.
    ///
    /// WHY: the reader never writes anything itself. The extraction decides
    ///       what happens with an entry, and the reader only reports what the
    ///       container carries.
    fn for_each_entry(&mut self, visitor: &mut dyn EntryVisitor) -> Result<(), ArchiveError>;
}

/// A writer of one container format.
pub trait ArchiveWriter {
    /// Adds a file with the given content to the archive.
    ///
    /// The size is the number of bytes the content holds. The TAR format
    /// writes it into the header of the entry; the ZIP format computes it
    /// itself.
    fn add_file(
        &mut self,
        name: &str,
        size: u64,
        content: &mut dyn Read,
    ) -> Result<(), ArchiveError>;

    /// Adds a directory to the archive.
    fn add_directory(&mut self, name: &str) -> Result<(), ArchiveError>;

    /// Adds a symbolic link to the archive.
    ///
    /// WHY: writing a link is allowed — a user may want to store one — but
    ///       reading one never creates a link on the disk.
    fn add_symlink(&mut self, name: &str, target: &str) -> Result<(), ArchiveError>;

    /// Finishes the archive and writes everything that is left.
    fn finish(self: Box<Self>) -> Result<(), ArchiveError>;
}

/// The number of bytes that are read for the detection of the format.
const SIGNATURE_LENGTH: usize = 512;

/// The magic bytes of the formats that the crate supports.
mod signature {
    /// The magic bytes of a GZip stream.
    pub const GZIP: [u8; 2] = [0x1f, 0x8b];

    /// The magic bytes of a BZip2 stream.
    pub const BZIP2: [u8; 3] = *b"BZh";

    /// The magic bytes of an XZ stream.
    pub const XZ: [u8; 6] = [0xfd, b'7', b'z', b'X', b'Z', 0x00];

    /// The magic bytes of a Zstd stream.
    pub const ZSTD: [u8; 4] = [0x28, 0xb5, 0x2f, 0xfd];

    /// The magic bytes of a ZIP archive.
    pub const ZIP: [u8; 4] = [b'P', b'K', 0x03, 0x04];

    /// The magic bytes of an empty ZIP archive.
    pub const ZIP_EMPTY: [u8; 4] = [b'P', b'K', 0x05, 0x06];

    /// The magic bytes of a 7z archive.
    pub const SEVENZ: [u8; 6] = [b'7', b'z', 0xbc, 0xaf, 0x27, 0x1c];

    /// The marker of a TAR archive, at the offset 257.
    pub const TAR: [u8; 5] = *b"ustar";
}

/// Returns the first bytes of a file.
///
/// WHY: the detection of a format must not read the whole file, and it must
///       not depend on the name of the file — the name of an archive is
///       chosen by its sender as well.
fn read_signature(path: &Path) -> Result<Vec<u8>, ArchiveError> {
    let mut file = File::open(path).map_err(|source| ArchiveError::io(path, source))?;
    let mut buffer = vec![0u8; SIGNATURE_LENGTH];
    let read = read_at_most(&mut file, &mut buffer)?;
    buffer.truncate(read);
    Ok(buffer)
}

/// Reads as many bytes as the buffer holds, without failing at the end of the
/// file.
fn read_at_most(file: &mut File, buffer: &mut [u8]) -> Result<usize, ArchiveError> {
    let mut read = 0;
    while read < buffer.len() {
        match file.read(&mut buffer[read..]) {
            Ok(0) => break,
            Ok(count) => read += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(source) => return Err(ArchiveError::io("the archive", source)),
        }
    }
    Ok(read)
}

/// Detects the format of an archive.
///
/// WHY: the extension of a file is a hint, not a fact. The magic bytes of the
///       container decide what the file really is.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::formats::{ContainerFormat, detect_format};
///
/// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
/// let format = detect_format(std::path::Path::new("holiday.7z"))?;
/// assert!(matches!(format, ContainerFormat::Zip | ContainerFormat::Tar | ContainerFormat::SevenZ));
/// # Ok(())
/// # }
/// ```
pub fn detect_format(path: &Path) -> Result<ContainerFormat, ArchiveError> {
    let signature = read_signature(path)?;
    if signature.starts_with(&signature::ZIP) || signature.starts_with(&signature::ZIP_EMPTY) {
        return Ok(ContainerFormat::Zip);
    }
    if signature.starts_with(&signature::SEVENZ) {
        return Ok(ContainerFormat::SevenZ);
    }
    if is_tar(&signature) {
        return Ok(ContainerFormat::Tar);
    }
    Err(ArchiveError::Format {
        reason: format!("the format of {path:?} is not one of ZIP, TAR or 7z"),
    })
}

/// Detects the compression of a TAR container.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::formats::{TarCompression, detect_tar_compression};
///
/// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
/// let compression = detect_tar_compression(std::path::Path::new("holiday.tar.gz"))?;
/// assert!(matches!(
///     compression,
///     TarCompression::None
///         | TarCompression::Gzip
///         | TarCompression::BZip2
///         | TarCompression::Xz
///         | TarCompression::Zstd
/// ));
/// # Ok(())
/// # }
/// ```
pub fn detect_tar_compression(path: &Path) -> Result<TarCompression, ArchiveError> {
    let signature = read_signature(path)?;
    if signature.starts_with(&signature::GZIP) {
        return Ok(TarCompression::Gzip);
    }
    if signature.starts_with(&signature::BZIP2) {
        return Ok(TarCompression::BZip2);
    }
    if signature.starts_with(&signature::XZ) {
        return Ok(TarCompression::Xz);
    }
    if signature.starts_with(&signature::ZSTD) {
        return Ok(TarCompression::Zstd);
    }
    Ok(TarCompression::None)
}

/// Whether the signature belongs to a TAR container.
fn is_tar(signature: &[u8]) -> bool {
    // The marker of the TAR format stands at the offset 257 of the first
    // block of the archive.
    signature.len() > 262 && signature[257..262] == signature::TAR
}

/// Opens an archive for reading and detects its format.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::formats::open_archive;
///
/// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
/// let reader = open_archive(std::path::Path::new("holiday.zip"))?;
/// # Ok(())
/// # }
/// ```
pub fn open_archive(path: &Path) -> Result<Box<dyn ArchiveReader>, ArchiveError> {
    match detect_format(path)? {
        ContainerFormat::Zip => Ok(Box::new(zip::ZipArchiveReader::open(path)?)),
        ContainerFormat::Tar => {
            let compression = detect_tar_compression(path)?;
            Ok(Box::new(tar::TarArchiveReader::open(path, compression)?))
        }
        ContainerFormat::SevenZ => Ok(Box::new(sevenz::SevenZArchiveReader::open(path)?)),
    }
}
