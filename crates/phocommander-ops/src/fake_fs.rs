//! A file system that lives in the memory, for the tests of the crate.
//!
//! WHAT: [`FakeFileSystem`], an implementation of
//!       [`crate::port::FileSystem`] that keeps its files in a map.
//! WHY: the operations have to be testable without a file system — a test
//!       that writes into a temporary directory proves that the copy works,
//!       a test that works on a map proves that the *decisions* of the copy
//!       are right: which file is skipped, which entry is deleted, which
//!       failure is collected. The fake answers with a checksum of its own,
//!       because the checksum belongs to the source and a source may use any
//!       algorithm.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::fake_fs::FakeFileSystem;
//! use phocommander_ops::port::FileSystem;
//! use std::path::Path;
//!
//! let fs = FakeFileSystem::new();
//! fs.write_file(Path::new("/photos/a.txt"), b"payload").unwrap_or_default();
//! let entries = fs.read_dir(Path::new("/photos")).unwrap_or_default();
//! assert_eq!(entries.len(), 1);
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::OpError;
use crate::port::{DirEntry, EntryKind, FileInfo, FileSystem};

/// One entry of the fake file system.
#[derive(Debug, Clone)]
struct FakeEntry {
    kind: EntryKind,
    content: Vec<u8>,
    modified: SystemTime,
}

/// A file system that lives in the memory.
#[derive(Debug, Default)]
pub struct FakeFileSystem {
    entries: Mutex<HashMap<PathBuf, FakeEntry>>,
}

impl FakeFileSystem {
    /// Builds a file system that is empty.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    ///
    /// let fs = FakeFileSystem::new();
    /// assert!(fs.read_dir(std::path::Path::new("/")).unwrap_or_default().is_empty());
    /// ```
    pub fn new() -> Self {
        let mut entries = HashMap::new();
        entries.insert(
            PathBuf::from("/"),
            FakeEntry {
                kind: EntryKind::Directory,
                content: Vec::new(),
                modified: UNIX_EPOCH,
            },
        );
        Self {
            entries: Mutex::new(entries),
        }
    }

    /// Writes a file and creates the directories above it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.write_file(Path::new("/photos/a.txt"), b"payload").unwrap_or_default();
    /// assert_eq!(fs.read_file(Path::new("/photos/a.txt")).unwrap_or_default(), b"payload");
    /// ```
    pub fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), OpError> {
        let mut entries = self.lock();
        if let Some(parent) = path.parent() {
            entries.insert(
                parent.to_path_buf(),
                FakeEntry {
                    kind: EntryKind::Directory,
                    content: Vec::new(),
                    modified: UNIX_EPOCH,
                },
            );
        }
        entries.insert(
            path.to_path_buf(),
            FakeEntry {
                kind: EntryKind::File,
                content: content.to_vec(),
                modified: UNIX_EPOCH + Duration::from_secs(1),
            },
        );
        Ok(())
    }

    /// The content of a file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.write_file(Path::new("/a.txt"), b"payload").unwrap_or_default();
    /// assert_eq!(fs.read_file(Path::new("/a.txt")).unwrap_or_default(), b"payload");
    /// ```
    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>, OpError> {
        let entries = self.lock();
        match entries.get(path) {
            Some(entry) => Ok(entry.content.clone()),
            None => Err(OpError::NotFound {
                path: path.to_path_buf(),
            }),
        }
    }

    /// Whether a path exists.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// assert!(!fs.exists(Path::new("/a.txt")));
    /// ```
    pub fn exists(&self, path: &Path) -> bool {
        self.lock().contains_key(path)
    }

    /// The number of entries the file system holds.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    ///
    /// let fs = FakeFileSystem::new();
    /// assert_eq!(fs.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// Whether the file system is empty.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    ///
    /// let fs = FakeFileSystem::new();
    /// assert!(!fs.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Locks the entries of the file system.
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<PathBuf, FakeEntry>> {
        self.entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

/// The checksum of the content of the fake file system.
///
/// WHY: the fake needs a checksum of its own, because the checksum belongs
/// to the source. The algorithm is a simple sum: it has to be stable and it
/// has to notice a changed byte — nothing more.
fn fake_checksum(content: &[u8]) -> u64 {
    let mut sum: u64 = 0;
    for byte in content {
        sum = sum.wrapping_mul(31).wrapping_add(u64::from(*byte));
    }
    sum
}

impl FileSystem for FakeFileSystem {
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>, OpError> {
        let entries = self.lock();
        if !entries.contains_key(path) {
            return Err(OpError::NotFound {
                path: path.to_path_buf(),
            });
        }

        let mut result: Vec<DirEntry> = Vec::new();
        for (entry_path, entry) in entries.iter() {
            if entry_path.parent() != Some(path) {
                continue;
            }
            let name = entry_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            let size = if entry.kind == EntryKind::File {
                Some(entry.content.len() as u64)
            } else {
                None
            };
            result.push(DirEntry {
                name,
                kind: entry.kind,
                size,
                modified: Some(entry.modified),
            });
        }
        result.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(result)
    }

    fn metadata(&self, path: &Path) -> Result<FileInfo, OpError> {
        let entries = self.lock();
        match entries.get(path) {
            Some(entry) => Ok(FileInfo {
                size: entry.content.len() as u64,
                modified: Some(entry.modified),
            }),
            None => Err(OpError::NotFound {
                path: path.to_path_buf(),
            }),
        }
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<crate::port::CopyResult, OpError> {
        let mut entries = self.lock();
        let entry = entries.get(from).cloned().ok_or_else(|| OpError::NotFound {
            path: from.to_path_buf(),
        })?;
        let bytes = entry.content.len() as u64;
        let checksum = fake_checksum(&entry.content);
        entries.insert(
            to.to_path_buf(),
            FakeEntry {
                kind: entry.kind,
                content: entry.content,
                modified: entry.modified,
            },
        );
        Ok(crate::port::CopyResult { bytes, checksum })
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), OpError> {
        let mut entries = self.lock();
        let entry = entries.remove(from).ok_or_else(|| OpError::NotFound {
            path: from.to_path_buf(),
        })?;
        entries.insert(to.to_path_buf(), entry);
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), OpError> {
        let mut entries = self.lock();
        entries.remove(path).ok_or_else(|| OpError::NotFound {
            path: path.to_path_buf(),
        })?;
        Ok(())
    }

    fn remove_dir(&self, path: &Path) -> Result<(), OpError> {
        let mut entries = self.lock();
        entries.remove(path).ok_or_else(|| OpError::NotFound {
            path: path.to_path_buf(),
        })?;
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), OpError> {
        let mut entries = self.lock();
        entries.insert(
            path.to_path_buf(),
            FakeEntry {
                kind: EntryKind::Directory,
                content: Vec::new(),
                modified: UNIX_EPOCH,
            },
        );
        Ok(())
    }

    fn checksum(&self, path: &Path) -> Result<u64, OpError> {
        let entries = self.lock();
        match entries.get(path) {
            Some(entry) => Ok(fake_checksum(&entry.content)),
            None => Err(OpError::NotFound {
                path: path.to_path_buf(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_written_file_is_read_back() {
        // arrange
        let fs = FakeFileSystem::new();

        // act
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));

        // assert
        let content = fs
            .read_file(Path::new("/photos/a.txt"))
            .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
        assert_eq!(content, b"payload");
        assert!(fs.exists(Path::new("/photos/a.txt")));
    }

    #[test]
    fn a_directory_lists_its_entries() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"a")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));

        // act
        let entries = fs
            .read_dir(Path::new("/photos"))
            .unwrap_or_else(|error| panic!("the directory could not be listed: {error}"));

        // assert
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "a.txt");
        assert_eq!(entries[0].size, Some(1));
    }

    #[test]
    fn a_missing_path_is_an_error() {
        // arrange
        let fs = FakeFileSystem::new();

        // act
        let result = fs.read_file(Path::new("/missing.txt"));

        // assert
        assert!(matches!(result, Err(OpError::NotFound { .. })), "{result:?}");
    }
}
