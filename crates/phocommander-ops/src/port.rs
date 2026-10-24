//! The interface the operations use to reach a file system.
//!
//! WHAT: the trait [{FileSystem}] and the three types it speaks —
//!       [{DirEntry}], [{EntryKind}] and [{FileInfo}].
//! WHY: the operations must not know the file system directly (the module
//!       catalogue of the specification): they ask a source for the content
//!       of a directory and they work on what the source answers. The
//!       interface lives here, in the crate that consumes it, because the
//!       workspace is flat: no crate below {crates/} may depend on a
//!       sibling, so the model crate cannot hand its own interfaces to the
//!       operations crate. The composition root adapts the one to the other
//!       and hands the result to the queue — the arrangement ADR-0001
//!       describes as Option B.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::port::{DirEntry, EntryKind, FileSystem};
//! use std::path::Path;
//!
//! # fn count<S: FileSystem>(source: &S) -> Result<usize, phocommander_ops::error::OpError> {
//! let entries: Vec<DirEntry> = source.read_dir(Path::new("/tmp"))?;
//! assert_eq!(entries[0].kind, EntryKind::File);
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::time::SystemTime;

use crate::error::OpError;

/// The kind of an entry of a directory listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryKind {
/// A regular file.
    File,

/// A directory.
    Directory,

/// A symbolic link.
    Symlink,

/// Anything else — a socket, a device node or a named pipe.
    Other,
}

impl EntryKind {
/// Whether the entry is a regular file.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::port::EntryKind;
///
/// assert!(EntryKind::File.is_file());
/// ```
    pub fn is_file(&self) -> bool {
        matches!(self, EntryKind::File)
    }

/// Whether the entry is a directory.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::port::EntryKind;
///
/// assert!(EntryKind::Directory.is_directory());
/// ```
    pub fn is_directory(&self) -> bool {
        matches!(self, EntryKind::Directory)
    }

/// Whether the entry is a symbolic link.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::port::EntryKind;
///
/// assert!(EntryKind::Symlink.is_symlink());
/// ```
    pub fn is_symlink(&self) -> bool {
        matches!(self, EntryKind::Symlink)
    }
}

/// One entry of a directory listing, as the operations need it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
/// The name of the entry, without the path of its directory.
    pub name: String,

/// The kind of the entry.
    pub kind: EntryKind,

/// The size of the entry in bytes, if the source knows it.
    pub size: Option<u64>,

/// The time of the last change of the entry, if the source knows it.
    pub modified: Option<SystemTime>,
}

impl DirEntry {
/// Builds an entry with a name and a kind.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::port::{DirEntry, EntryKind};
///
/// let entry = DirEntry::new("notes.txt", EntryKind::File);
/// assert_eq!(entry.name, "notes.txt");
/// ```
    pub fn new(name: impl Into<String>, kind: EntryKind) -> Self {
        Self {
            name: name.into(),
            kind,
            size: None,
            modified: None,
        }
    }
}

/// What the operations need to know about a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
/// The kind of the file.
    pub kind: EntryKind,

/// The size of the file in bytes.
    pub size: u64,

/// The time of the last change of the file, if the source knows it.
    pub modified: Option<SystemTime>,
}

/// What a copy wrote.
///
/// WHY: the copy reports the number of bytes, because the progress of the
/// job counts them, and the checksum of the content that was read, because
/// the verification compares it with the checksum of the destination. The
/// checksum of the source is computed *while* the file is copied, so that
/// the verification does not read the source a second time and the read
/// volume stays at twice the copied volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyResult {
/// The number of bytes that were written.
    pub bytes: u64,

/// The checksum of the content that was copied.
    pub checksum: u64,
}

/// The file system operations that a source has to provide.
///
/// WHY: the operations need exactly these seven calls and nothing more. A
/// source that cannot provide one of them — an archive that cannot be
/// written, a released directory on iPadOS that cannot be deleted — does not
/// implement the trait at all, and the difference between a read-only source
/// and a writable one is visible in the type.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::port::{CopyResult, DirEntry, FileSystem, FileInfo};
/// use phocommander_ops::error::OpError;
/// use std::path::Path;
///
/// struct Nothing;
///
/// impl FileSystem for Nothing {
///     fn read_dir(&self, _path: &Path) -> Result<Vec<DirEntry>, OpError> {
///         Ok(Vec::new())
///     }
///     fn metadata(&self, _path: &Path) -> Result<FileInfo, OpError> {
///         Err(OpError::NotFound { path: Path::new("").to_path_buf() })
///     }
///     fn copy_file(&self, _from: &Path, _to: &Path) -> Result<CopyResult, OpError> {
///         Err(OpError::Cancelled)
///     }
///     fn rename(&self, _from: &Path, _to: &Path) -> Result<(), OpError> {
///         Err(OpError::Cancelled)
///     }
///     fn remove_file(&self, _path: &Path) -> Result<(), OpError> {
///         Err(OpError::Cancelled)
///     }
///     fn remove_dir(&self, _path: &Path) -> Result<(), OpError> {
///         Err(OpError::Cancelled)
///     }
///     fn create_dir_all(&self, _path: &Path) -> Result<(), OpError> {
///         Err(OpError::Cancelled)
///     }
///     fn checksum(&self, _path: &Path) -> Result<u64, OpError> {
///         Err(OpError::Cancelled)
///     }
/// }
/// ```
pub trait FileSystem: Send + Sync {
/// Lists the entries of the directory at {path}.
///
/// WHY: the comparison of two directories needs the content of both
/// sides, and the listing is the only call that provides it.
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>, OpError>;

/// Reports the size and the modification time of the file at {path}.
    fn metadata(&self, path: &Path) -> Result<FileInfo, OpError>;

/// Copies the file at {from} to {to} and reports what was written.
///
/// WHY: the number of bytes is what the progress of the job counts, and
/// the checksum of the copied content is what the verification compares
/// with the destination. A copy that cannot report them would make the
/// progress a guess and the verification impossible.
    fn copy_file(&self, from: &Path, to: &Path) -> Result<CopyResult, OpError>;

/// Moves or renames the file or the directory at {from} to {to}.
    fn rename(&self, from: &Path, to: &Path) -> Result<(), OpError>;

/// Deletes the file at {path}.
    fn remove_file(&self, path: &Path) -> Result<(), OpError>;

/// Deletes the empty directory at {path}.
    fn remove_dir(&self, path: &Path) -> Result<(), OpError>;

/// Creates the directory at {path} and every missing directory above it.
    fn create_dir_all(&self, path: &Path) -> Result<(), OpError>;

/// Computes the checksum of the file at {path}.
///
/// WHY: the verified copy compares the checksum of the source and of the
/// destination. The calculation belongs to the source, because only the
/// source knows how the file is stored.
    fn checksum(&self, path: &Path) -> Result<u64, OpError>;
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn the_kind_answers_the_questions_the_operations_ask() {
        // act
        let file = EntryKind::File;
        let directory = EntryKind::Directory;
        let link = EntryKind::Symlink;

        // assert
        assert!(file.is_file());
        assert!(!file.is_directory());
        assert!(directory.is_directory());
        assert!(link.is_symlink());
        assert!(!link.is_file());
    }

    #[test]
    fn a_new_entry_knows_only_its_name_and_its_kind() {
        // act
        let entry = DirEntry::new("notes.txt", EntryKind::File);

        // assert
        assert_eq!(entry.name, "notes.txt");
        assert_eq!(entry.kind, EntryKind::File);
        assert_eq!(entry.size, None);
        assert_eq!(entry.modified, None);
    }
}
