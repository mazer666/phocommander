//! The temporary file of the atomic copy.
//!
//! WHAT: the name of a temporary file ([{temporary_path}]) and the guard that
//!       removes it again ([{TemporaryFile}]).
//! WHY: the atomic mode writes into a file next to the target, checks that
//!       file and renames it into place (FR-22d). Two properties of that
//!       temporary file matter: its name has to be unique, so that two copies
//!       into the same directory do not tread on each other, and it has to be
//!       removed on *every* path out of the operation — a failed comparison, a
//!       failed rename, a cancellation. The guard makes the second property a
//!       property of the type instead of a property of the discipline of the
//!       programmer: the file is removed when the guard is dropped, whatever
//!       the reason the operation ended.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::port::FileSystem;
//! use phocommander_ops::temporary::temporary_path;
//! use std::path::Path;
//!
//! # fn run(fs: &dyn FileSystem) {
//! let first = temporary_path(Path::new("/backup/sunset.jpg"));
//! let second = temporary_path(Path::new("/backup/sunset.jpg"));
//! assert_ne!(first, second);
//! assert_eq!(first.parent(), second.parent());
//! # }
//! ```
//! BOUNDARY: this module knows the file system only through the
//!       [{crate::port::FileSystem}] trait of the operations.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::port::FileSystem;

/// The counter that separates two temporary files of the same process.
static TEMPORARY_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The seed of the names of the temporary files of this process.
///
/// WHY: the seed is drawn once per process from the hasher of the standard
///       library, whose keys are randomised when the process starts. Two
///       processes therefore never build the same name, even when they write
///       into the same directory at the same time.
fn process_seed() -> u64 {
    static SEED: AtomicU64 = AtomicU64::new(0);
    let existing = SEED.load(Ordering::Relaxed);
    if existing != 0 {
        return existing;
    }

    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(0x9e37_79b9_7f4a_7c15);
    let seed = hasher.finish() | 1;
    // WHY: the exchange keeps the value that another thread has written in
    // the meantime. The name of a temporary file does not depend on the seed
    // alone — the counter is part of it as well — so a lost race costs
    // nothing.
    match SEED.compare_exchange(0, seed, Ordering::Relaxed, Ordering::Relaxed) {
        Ok(_) => seed,
        Err(current) => current,
    }
}

/// The name of the temporary file of a target.
///
/// WHY: the temporary file has to sit in the same directory as the target,
///       because a rename cannot cross a file system. The name starts with a
///       dot and ends with a suffix that no user file carries, so that the
///       temporary file is not mistaken for a result. The middle of the name
///       carries the seed of the process and a counter, so that two copies
///       into the same directory never use the same name.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::temporary::temporary_path;
/// use std::path::Path;
///
/// let temporary = temporary_path(Path::new("/backup/photos/sunset.jpg"));
/// let name = temporary
///     .file_name()
///     .map(|name| name.to_string_lossy().to_string())
///     .unwrap_or_default();
/// assert!(name.starts_with(".sunset.jpg."), "{name}");
/// assert!(name.ends_with(".part"), "{name}");
/// ```
pub fn temporary_path(target: &Path) -> PathBuf {
    let counter = TEMPORARY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let seed = process_seed();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);

    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let parent = target.parent().map(Path::to_path_buf).unwrap_or_default();
    parent.join(format!(".{name}.{seed:x}-{nanos:x}-{counter:x}.part"))
}

/// The temporary file of an operation, and the promise that it is removed.
///
/// WHY: the guard removes the file when it is dropped. The operation therefore
///       cannot leave a temporary file behind, not when the comparison fails,
///       not when the rename fails and not when the job is cancelled. The
///       guard is disarmed only after the rename succeeded — at that moment
///       the file no longer exists under the temporary name.
// WHY: the derive of Debug cannot be used here: the file system is a trait
//       object, and the trait does not demand Debug. The two fields that
//       carry the state are written by hand instead, so that the guard can be
//       printed in a log without the file system.
pub struct TemporaryFile<'a> {
    file_system: &'a dyn FileSystem,
    path: PathBuf,
    armed: bool,
}

impl std::fmt::Debug for TemporaryFile<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TemporaryFile")
            .field("path", &self.path)
            .field("armed", &self.armed)
            .finish()
    }
}

impl<'a> TemporaryFile<'a> {
    /// Reserves the name of the temporary file of a target.
    ///
    /// WHY: the name is reserved before the first byte is written, so that
    ///       two operations that write into the same directory cannot choose
    ///       the same name.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::port::FileSystem;
    /// use phocommander_ops::temporary::TemporaryFile;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn FileSystem) {
    /// let temporary = TemporaryFile::reserve(fs, Path::new("/backup/a.txt"));
    /// assert!(temporary.path().file_name().is_some());
    /// # }
    /// ```
    pub fn reserve(file_system: &'a dyn FileSystem, target: &Path) -> Self {
        Self {
            file_system,
            path: temporary_path(target),
            armed: true,
        }
    }

    /// The path of the temporary file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::port::FileSystem;
    /// use phocommander_ops::temporary::TemporaryFile;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn FileSystem) {
    /// let temporary = TemporaryFile::reserve(fs, Path::new("/backup/a.txt"));
    /// assert_eq!(temporary.path().parent(), Some(Path::new("/backup")));
    /// # }
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Gives up the promise to remove the file.
    ///
    /// WHY: the method is called after the rename succeeded. The file is then
    ///       at the target and the temporary name does not exist any more.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::port::FileSystem;
    /// use phocommander_ops::temporary::TemporaryFile;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn FileSystem) {
    /// let mut temporary = TemporaryFile::reserve(fs, Path::new("/backup/a.txt"));
    /// temporary.disarm();
    /// # }
    /// ```
    pub fn disarm(&mut self) {
        self.armed = false;
    }

    /// Whether the file is still going to be removed.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::port::FileSystem;
    /// use phocommander_ops::temporary::TemporaryFile;
    /// use std::path::Path;
    ///
    /// # fn run(fs: &dyn FileSystem) {
    /// let mut temporary = TemporaryFile::reserve(fs, Path::new("/backup/a.txt"));
    /// assert!(temporary.is_armed());
    /// temporary.disarm();
    /// assert!(!temporary.is_armed());
    /// # }
    /// ```
    pub fn is_armed(&self) -> bool {
        self.armed
    }
}

impl Drop for TemporaryFile<'_> {
    /// Removes the temporary file, whatever the reason the operation ended.
    ///
    /// WHY: the removal is best effort. A file that cannot be removed is a
    ///       problem of the file system, and the operation that failed has
    ///       already failed — the error of the operation is the one the user
    ///       sees.
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let _ = self.file_system.remove_file(&self.path);
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::fake_fs::FakeFileSystem;

    #[test]
    fn two_temporary_files_of_the_same_target_have_different_names() {
        // act
        let first = temporary_path(Path::new("/backup/a.txt"));
        let second = temporary_path(Path::new("/backup/a.txt"));

        // assert
        assert_ne!(first, second);
    }

    #[test]
    fn the_temporary_file_sits_next_to_the_target() {
        // act
        let temporary = temporary_path(Path::new("/backup/photos/sunset.jpg"));

        // assert
        assert_eq!(temporary.parent(), Some(Path::new("/backup/photos")));
    }

    #[test]
    fn the_guard_removes_the_file_when_it_is_dropped() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/backup/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        let path = {
            let temporary = TemporaryFile::reserve(&fs, Path::new("/backup/a.txt"));
            fs.write_file(temporary.path(), b"payload")
                .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
            temporary.path().to_path_buf()
        };

        // assert
        assert!(!fs.exists(&path), "the temporary file was left behind");
    }

    #[test]
    fn the_guard_keeps_the_file_when_it_is_disarmed() {
        // arrange
        let fs = FakeFileSystem::new();
        let path = {
            let mut temporary = TemporaryFile::reserve(&fs, Path::new("/backup/a.txt"));
            fs.write_file(temporary.path(), b"payload")
                .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
            temporary.disarm();
            assert!(!temporary.is_armed());
            temporary.path().to_path_buf()
        };

        // assert
        assert!(fs.exists(&path));
    }

    #[test]
    fn the_seed_of_the_process_stays_the_same() {
        // act
        let first = process_seed();
        let second = process_seed();

        // assert
        assert_eq!(first, second);
        assert_ne!(first, 0);
    }
}
