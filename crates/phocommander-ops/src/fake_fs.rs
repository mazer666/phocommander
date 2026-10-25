//! A file system that lives in the memory, for the tests of the crate.
//!
//! WHAT: [{FakeFileSystem}], an implementation of
//!       [{crate::port::FileSystem}] that keeps its files in a map.
//! WHY: the operations have to be testable without a file system — a test
//!       that writes into a temporary directory proves that the copy works,
//!       a test that works on a map proves that the *decisions* of the copy
//!       are right: which file is skipped, which entry is deleted, which
//!       failure is collected. The fake answers with a checksum of its own,
//!       because the checksum belongs to the source and a source may use any
//!       algorithm.
//! SIZE: the fake holds the whole file system in the memory and therefore
//!       needs the writing, the reading, the linking, the deleting and the
//!       failing in one place. The alternative — a fake per test — would
//!       repeat the same hundred lines in every test file, and the tests
//!       would differ from each other in the details that matter.
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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
    checksums: AtomicU64,
    corrupt_next: AtomicBool,
    failing: Mutex<std::collections::HashSet<PathBuf>>,
}

impl FakeFileSystem {
    /// Builds a file system that is empty.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use phocommander_ops::port::FileSystem;
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
            checksums: AtomicU64::new(0),
            corrupt_next: AtomicBool::new(false),
            failing: Mutex::new(std::collections::HashSet::new()),
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

    /// Creates a symbolic link that points at {target}.
    ///
    /// WHY: the fake has to know about links, because the rules of the
    /// specification say that a link is copied as a link. The target is stored
    /// as the content of the entry, which is what a link is on a file system.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.write_file(Path::new("/a.txt"), b"payload").unwrap_or_default();
    /// fs.write_symlink(Path::new("/link.txt"), Path::new("/a.txt")).unwrap_or_default();
    /// assert_eq!(fs.link_target(Path::new("/link.txt")).as_deref(), Some("/a.txt"));
    /// ```
    pub fn write_symlink(&self, path: &Path, target: &Path) -> Result<(), OpError> {
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
                kind: EntryKind::Symlink,
                content: target.to_string_lossy().as_bytes().to_vec(),
                modified: UNIX_EPOCH + Duration::from_secs(1),
            },
        );
        Ok(())
    }

    /// The target of a symbolic link.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.write_symlink(Path::new("/link.txt"), Path::new("/a.txt")).unwrap_or_default();
    /// assert_eq!(fs.link_target(Path::new("/link.txt")).as_deref(), Some("/a.txt"));
    /// ```
    pub fn link_target(&self, path: &Path) -> Option<String> {
        let entries = self.lock();
        let entry = entries.get(path)?;
        if entry.kind != EntryKind::Symlink {
            return None;
        }
        Some(String::from_utf8_lossy(&entry.content).to_string())
    }

    /// The kind of an entry.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use phocommander_ops::port::EntryKind;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.write_symlink(Path::new("/link.txt"), Path::new("/a.txt")).unwrap_or_default();
    /// assert_eq!(fs.kind_of(Path::new("/link.txt")), Some(EntryKind::Symlink));
    /// ```
    pub fn kind_of(&self, path: &Path) -> Option<EntryKind> {
        self.lock().get(path).map(|entry| entry.kind)
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

    /// The entries of a directory.
    ///
    /// WHY: a test has to be able to see what a directory holds, to prove
    ///       that a failed operation leaves nothing behind. The names of the
    ///       direct children are enough for that, and the order is the order
    ///       of the paths, so that the result of a test does not depend on the
    ///       order of a hash table.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.write_file(Path::new("/backup/a.txt"), b"a").unwrap_or_default();
    /// assert_eq!(fs.read_dir(Path::new("/backup")).unwrap_or_default().len(), 1);
    /// ```
    pub fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>, OpError> {
        let entries = self.lock();
        if !entries.contains_key(path) {
            return Err(OpError::NotFound {
                path: path.to_path_buf(),
            });
        }

        let mut listing: Vec<DirEntry> = entries
            .iter()
            .filter_map(|(candidate, entry)| {
                let rest = candidate.strip_prefix(path).ok()?;
                if rest.as_os_str().is_empty() {
                    return None;
                }
                // WHY: only the direct children are listed. A file deeper in
                //       the tree belongs to a directory that is itself a
                //       child of the listed directory.
                if rest.components().count() != 1 {
                    return None;
                }
                Some(DirEntry {
                    name: rest.to_string_lossy().to_string(),
                    kind: entry.kind,
                    size: match entry.kind {
                        EntryKind::Directory => None,
                        _ => Some(entry.content.len() as u64),
                    },
                    modified: Some(entry.modified),
                })
            })
            .collect();
        listing.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(listing)
    }

    /// Makes the copies of the given path fail.
    ///
    /// WHY: a job that copies a tree collects the failures and carries on
    ///       with the next entry (FR-25). A test therefore has to be able to
    ///       make exactly one entry fail, to prove that the other entries
    ///       are copied and that the retry touches only the failed one
    ///       (FR-22k).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.fail_copy_of(Path::new("/photos/a.txt"));
    /// ```
    pub fn fail_copy_of(&self, path: &Path) {
        self.lock_failing().insert(path.to_path_buf());
    }

    /// Lets the copies of the given path succeed again.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.fail_copy_of(Path::new("/photos/a.txt"));
    /// fs.stop_failing(Path::new("/photos/a.txt"));
    /// ```
    pub fn stop_failing(&self, path: &Path) {
        self.lock_failing().remove(path);
    }

    /// Whether the copies of the given path fail.
    pub fn fails_to_copy(&self, path: &Path) -> bool {
        self.lock_failing().contains(path)
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

    /// The number of times a checksum was computed.
    ///
    /// WHY: the rule "checksum_on_copy" decides whether the verified copy
    /// compares the checksums of source and destination. A test has to see that
    /// the comparison did not happen, and the counter is the only way to see it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    /// use phocommander_ops::port::FileSystem;
    /// use std::path::Path;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.write_file(Path::new("/a.txt"), b"payload").unwrap_or_default();
    /// let _ = fs.checksum(Path::new("/a.txt"));
    /// assert_eq!(fs.checksum_calls(), 1);
    /// ```
    pub fn checksum_calls(&self) -> u64 {
        self.checksums.load(Ordering::Relaxed)
    }

    /// Makes the next copy write content that differs from the source.
    ///
    /// WHY: the verified and the atomic copy compare the checksum of the
    ///       source with the checksum of the file that was written. A test
    ///       has to be able to produce a file that differs — a medium that
    ///       writes the wrong bytes is exactly the case the modes exist for.
    ///       The copy reports the checksum of the source, as a file system
    ///       does, and writes content with a different checksum.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::fake_fs::FakeFileSystem;
    ///
    /// let fs = FakeFileSystem::new();
    /// fs.corrupt_next_copy();
    /// ```
    pub fn corrupt_next_copy(&self) {
        self.corrupt_next.store(true, Ordering::SeqCst);
    }

    /// Locks the entries of the file system.
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<PathBuf, FakeEntry>> {
        self.entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    /// The set of the paths whose copies fail.
    fn lock_failing(&self) -> std::sync::MutexGuard<'_, std::collections::HashSet<PathBuf>> {
        self.failing
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
                kind: entry.kind,
                size: entry.content.len() as u64,
                modified: Some(entry.modified),
            }),
            None => Err(OpError::NotFound {
                path: path.to_path_buf(),
            }),
        }
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<crate::port::CopyResult, OpError> {
        // WHY: the failure of one entry is asked for by the test, so that the
        //       job can collect it and carry on with the next entry.
        if self.fails_to_copy(from) {
            return Err(OpError::Io {
                path: from.to_path_buf(),
                source: std::io::Error::other("the copy of this path was made to fail"),
            });
        }

        let mut entries = self.lock();
        let entry = entries
            .get(from)
            .cloned()
            .ok_or_else(|| OpError::NotFound {
                path: from.to_path_buf(),
            })?;

        // WHY: a copy through the ordinary call dereferences a link, exactly
        // as the file system does. The rule that a link is copied as a link
        // uses the separate call.
        let content = if entry.kind == EntryKind::Symlink {
            let target = PathBuf::from(String::from_utf8_lossy(&entry.content).to_string());
            let target_entry = entries
                .get(&target)
                .cloned()
                .ok_or_else(|| OpError::NotFound {
                    path: target.clone(),
                })?;
            target_entry.content
        } else {
            entry.content
        };

        let bytes = content.len() as u64;
        let checksum = fake_checksum(&content);

        // WHY: the corruption is applied after the checksum of the source is
        // computed. The copy therefore reports the checksum of the source and
        // writes content with a different checksum — the situation the
        // verified and the atomic copy have to notice.
        let written = if self.corrupt_next.swap(false, Ordering::SeqCst) {
            let mut damaged = content.clone();
            if let Some(first) = damaged.first_mut() {
                *first = first.wrapping_add(1);
            } else {
                damaged.push(1);
            }
            damaged
        } else {
            content
        };

        if let Some(parent) = to.parent() {
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
            to.to_path_buf(),
            FakeEntry {
                kind: EntryKind::File,
                content: written,
                modified: entry.modified,
            },
        );
        Ok(crate::port::CopyResult { bytes, checksum })
    }

    fn copy_symlink(&self, from: &Path, to: &Path) -> Result<(), OpError> {
        let mut entries = self.lock();
        let entry = entries
            .get(from)
            .cloned()
            .ok_or_else(|| OpError::NotFound {
                path: from.to_path_buf(),
            })?;

        if entry.kind != EntryKind::Symlink {
            return Err(OpError::Io {
                path: from.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "the source is not a symbolic link",
                ),
            });
        }

        entries.insert(
            to.to_path_buf(),
            FakeEntry {
                kind: EntryKind::Symlink,
                content: entry.content,
                modified: entry.modified,
            },
        );
        Ok(())
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
        self.checksums.fetch_add(1, Ordering::Relaxed);
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
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
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
        assert!(
            matches!(result, Err(OpError::NotFound { .. })),
            "{result:?}"
        );
    }
}
