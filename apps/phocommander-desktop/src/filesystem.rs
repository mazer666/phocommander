//! The file system of the host machine.
//!
//! WHAT: the file system that the operations work on ([`HostFileSystem`]).
//! WHY: the operations ask a source for the content of a directory and for
//!       the facts of a file, and they never touch a file system on their
//!       own. The adapter of the host machine is therefore the one place
//!       where the calls of the library meet the calls of the operating
//!       system. The composition root owns it, because it is the only crate
//!       that may see the crate of the operations and the crate of the
//!       platform at the same time.
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::filesystem::HostFileSystem;
//! use phocommander_ops::port::FileSystem;
//! use std::path::Path;
//!
//! let fs = HostFileSystem::new();
//! assert!(fs.metadata(Path::new("/a/path/that/does/not/exist")).is_err());
//! ```

use std::fs;
use std::path::Path;

use phocommander_ops::error::OpError;
use phocommander_ops::port::{CopyResult, DirEntry, EntryKind, FileInfo, FileSystem};

/// The file system of the host machine.
#[derive(Debug, Clone, Copy, Default)]
pub struct HostFileSystem;

impl HostFileSystem {
    /// Builds the file system of the host machine.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_desktop::filesystem::HostFileSystem;
    ///
    /// let fs = HostFileSystem::new();
    /// let _ = fs;
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl FileSystem for HostFileSystem {
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>, OpError> {
        let entries = fs::read_dir(path).map_err(|error| io_error(path, error))?;
        let mut listing = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| io_error(path, error))?;
            let entry_path = entry.path();
            let metadata = fs::symlink_metadata(&entry_path).ok();
            listing.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                kind: kind_of(&entry_path),
                size: metadata.as_ref().map(|metadata| metadata.len()),
                modified: metadata
                    .as_ref()
                    .and_then(|metadata| metadata.modified().ok()),
            });
        }
        Ok(listing)
    }

    fn metadata(&self, path: &Path) -> Result<FileInfo, OpError> {
        let metadata = fs::symlink_metadata(path).map_err(|error| io_error(path, error))?;
        Ok(FileInfo {
            kind: kind_of(path),
            size: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<CopyResult, OpError> {
        let bytes = fs::copy(from, to).map_err(|error| io_error(from, error))?;
        let checksum = self.checksum(from)?;
        Ok(CopyResult { bytes, checksum })
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), OpError> {
        fs::rename(from, to).map_err(|error| io_error(from, error))
    }

    fn remove_file(&self, path: &Path) -> Result<(), OpError> {
        fs::remove_file(path).map_err(|error| io_error(path, error))
    }

    fn remove_dir(&self, path: &Path) -> Result<(), OpError> {
        fs::remove_dir(path).map_err(|error| io_error(path, error))
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), OpError> {
        fs::create_dir_all(path).map_err(|error| io_error(path, error))
    }

    fn checksum(&self, path: &Path) -> Result<u64, OpError> {
        let content = fs::read(path).map_err(|error| io_error(path, error))?;
        Ok(checksum_of(&content))
    }

    fn copy_symlink(&self, from: &Path, to: &Path) -> Result<(), OpError> {
        let target = fs::read_link(from).map_err(|error| io_error(from, error))?;
        // WHY: on a system with symbolic links the copy of a link creates a
        //       link and never writes the target of the link as a regular
        //       file.
        #[cfg(unix)]
        let result = std::os::unix::fs::symlink(target, to).map_err(|error| io_error(to, error));
        // WHY: on a system without symbolic links the copy of a link is
        //       refused instead of writing the target of the link as a
        //       regular file.
        #[cfg(not(unix))]
        let result = {
            let _ = (target, to);
            Err(OpError::UnsupportedMode {
                mode: "copy_symlink".to_string(),
            })
        };
        result
    }
}

/// The kind of an entry of the host machine.
///
/// WHY: a link is a link and not the file it points to. The operations copy
///       links as links and never follow them, and the kind is what tells
///       them the difference.
fn kind_of(path: &Path) -> EntryKind {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                EntryKind::Symlink
            } else if file_type.is_dir() {
                EntryKind::Directory
            } else if file_type.is_file() {
                EntryKind::File
            } else {
                EntryKind::Other
            }
        }
        Err(_) => EntryKind::Other,
    }
}

/// The checksum of a content.
///
/// WHY: the verified copy compares the checksum of the source with the
///       checksum of the destination. The two values have to come from the
///       same rule, and the rule is the one of the product.
fn checksum_of(content: &[u8]) -> u64 {
    let mut sum: u64 = 0;
    for byte in content {
        sum = sum.wrapping_mul(31).wrapping_add(u64::from(*byte));
    }
    sum
}

/// The error of a call that the operating system refused.
fn io_error(path: &Path, error: std::io::Error) -> OpError {
    OpError::Io {
        path: path.to_path_buf(),
        source: error,
    }
}
