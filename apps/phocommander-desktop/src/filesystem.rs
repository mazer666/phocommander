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

use phocommander_core::path::length::{PathLengthLimit, check_path_length};
use phocommander_ops::error::OpError;
use phocommander_ops::port::{CopyResult, DirEntry, EntryKind, FileInfo, FileSystem};

use crate::windows;

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

    /// The limit of the path length on this machine.
    ///
    /// WHY: the limit belongs to the platform, and only the platform knows
    ///       it. Windows counts the characters of a path and allows 260 of
    ///       them — or 32 767, when the system has the long paths switched
    ///       on. Linux counts the bytes and has no limit of its own.
    pub fn path_length_limit(&self) -> Option<PathLengthLimit> {
        match phocommander_platform::host_platform() {
            phocommander_platform::Platform::Windows => {
                Some(windows::path_length_limit(long_paths_of_the_system()))
            }
            phocommander_platform::Platform::Linux
            | phocommander_platform::Platform::Bsd
            | phocommander_platform::Platform::MacOs => Some(PathLengthLimit::linux()),
            _ => None,
        }
    }

    /// Refuses a path that is longer than the platform allows.
    fn check_length(&self, path: &Path) -> Result<(), OpError> {
        let Some(limit) = self.path_length_limit() else {
            return Ok(());
        };
        check_path_length(path, limit).map_err(|refused| OpError::PathTooLong {
            path: refused.path().to_path_buf(),
            amount: refused.amount(),
            unit: unit_of(refused.limit().unit()),
            limit: refused.limit().amount(),
        })
    }
}

/// The name of the unit of a length.
///
/// WHY: the unit is a code and not a text. The catalogue of the interface
///       translates the code, so that the message of a refused path speaks
///       the language of the interface.
fn unit_of(unit: phocommander_core::path::length::LengthUnit) -> String {
    use phocommander_core::path::length::LengthUnit;
    match unit {
        LengthUnit::Characters => "characters".to_string(),
        LengthUnit::Bytes => "bytes".to_string(),
    }
}

/// Whether the system of this machine has the long paths of Windows switched on.
///
/// WHY: Windows allows 32 767 characters in a path when the system has the
///       long paths switched on, and 260 otherwise. The application cannot
///       read the setting of the system, but the manifest it embeds declares
///       the wish — and the honest limit is the smaller one, so that a
///       program that runs on a system without the setting still works.
fn long_paths_of_the_system() -> bool {
    false
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
        // WHY: a path that is longer than the platform allows cannot be
        //       written. The operation refuses it before it starts, and the
        //       message says which path it is and what the limit is.
        self.check_length(to)?;
        let bytes = fs::copy(from, to).map_err(|error| io_error(from, error))?;
        let checksum = self.checksum(from)?;
        Ok(CopyResult { bytes, checksum })
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), OpError> {
        self.check_length(to)?;
        fs::rename(from, to).map_err(|error| io_error(from, error))
    }

    fn remove_file(&self, path: &Path) -> Result<(), OpError> {
        fs::remove_file(path).map_err(|error| io_error(path, error))
    }

    fn remove_dir(&self, path: &Path) -> Result<(), OpError> {
        fs::remove_dir(path).map_err(|error| io_error(path, error))
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), OpError> {
        self.check_length(path)?;
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

#[cfg(test)]
// WHY: the tests of the module live in the module itself, because the rule of
//       the path length belongs to it and nowhere else.
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use phocommander_ops::port::FileSystem;
    use std::path::PathBuf;

    #[test]
    fn a_path_that_is_longer_than_the_limit_of_windows_is_refused() {
        // arrange
        // WHY: the limit of Windows is 260 characters, and a path that is
        //       longer cannot be written. The operation has to refuse it
        //       before it starts, and the message has to name the path.
        let fs = HostFileSystem::new();
        let limit = fs
            .path_length_limit()
            .expect("the machine has a limit of the path length");
        let mut long = PathBuf::from("/photos");
        while long.to_string_lossy().len() < limit.amount() + 100 {
            long.push("a-directory-with-a-long-name");
        }
        let source = PathBuf::from("/photos/a.txt");

        // act
        let result = fs.copy_file(&source, &long);

        // assert
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("the copy of a path that is too long should be refused"),
        };
        // WHY: the key is what the interface reads the text from, and the
        //       code is what the tables of ADR-0003 name. Both have to point
        //       at the same case.
        assert_eq!(error.message_key(), "path-too-long");
        assert_eq!(
            error.code(),
            phocommander_ops::error::JobErrorCode::PathTooLong
        );
        let arguments = error.message_arguments();
        assert_eq!(
            arguments.len(),
            4,
            "the message names the path, the length, the unit and the limit"
        );
        assert!(arguments[0].1.contains("a-directory-with-a-long-name"));
        assert_eq!(
            arguments[3].1,
            limit.amount().to_string(),
            "the message names the limit of the platform"
        );
    }

    #[test]
    fn a_path_that_fits_is_written() {
        // arrange
        let fs = HostFileSystem::new();
        let target = PathBuf::from("/photos/a-name-that-is-short.txt");

        // act
        let result = fs.create_dir_all(&target);

        // assert
        // WHY: the check must not refuse a path that fits. A rule that
        //       refuses everything is not a rule.
        assert!(result.is_ok(), "{result:?}");
    }
}
