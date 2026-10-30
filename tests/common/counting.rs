//! The file system that counts the bytes it reads.
//!
//! WHAT: a file system that wraps another one and counts the bytes that
//!       pass through it ([`CountingFileSystem`]).
//! WHY: the specification demands in NFR-09a that the verification of a copy
//!       raises the read volume to at most twice the copied volume. A
//!       statement like that cannot be checked by reading the code — the code
//!       says what it *would* do. The only proof is a file system that counts
//!       what it really reads, and a test that compares the count with the
//!       volume that was copied.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_testkit::counting::CountingFileSystem;
//! use phocommander_ops::fake_fs::FakeFileSystem;
//!
//! let counting = CountingFileSystem::new(FakeFileSystem::new());
//! assert_eq!(counting.read_bytes(), 0);
//! ```

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use phocommander_ops::error::OpError;
use phocommander_ops::port::{CopyResult, DirEntry, FileInfo, FileSystem};

/// The bytes that the file system has read.
#[derive(Debug, Clone, Default)]
pub struct ReadVolume {
    /// The number of the bytes that were read.
    bytes: Arc<AtomicU64>,
}

impl ReadVolume {
    /// The number of the bytes that were read so far.
    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    /// Adds the given number of bytes.
    fn add(&self, bytes: u64) {
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }
}

/// The file system that counts the bytes it reads.
///
/// WHY: the count lives in a cell that both the file system and the test
///       hold, so that the test can read the count after the copy has run.
pub struct CountingFileSystem<F: FileSystem> {
    /// The file system that does the work.
    inner: F,
    /// The bytes that were read.
    volume: ReadVolume,
}

impl<F: FileSystem> CountingFileSystem<F> {
    /// Wraps the file system.
    pub fn new(inner: F) -> Self {
        Self {
            inner,
            volume: ReadVolume::default(),
        }
    }

    /// The bytes that were read.
    pub fn read_bytes(&self) -> u64 {
        self.volume.bytes()
    }

    /// The bytes that were read, as a handle of their own.
    ///
    /// WHY: the test wants to hold the count while the file system is
    ///       borrowed, and a handle of its own makes that possible.
    pub fn volume(&self) -> ReadVolume {
        self.volume.clone()
    }
}

impl<F: FileSystem> FileSystem for CountingFileSystem<F> {
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>, OpError> {
        self.inner.read_dir(path)
    }

    fn metadata(&self, path: &Path) -> Result<FileInfo, OpError> {
        self.inner.metadata(path)
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<CopyResult, OpError> {
        // WHY: the copy reads the whole file, and the volume of the read is
        //       what the requirement speaks about.
        let bytes = self.inner.metadata(from).map(|info| info.size).unwrap_or(0);
        let ergebnis = self.inner.copy_file(from, to)?;
        self.volume.add(bytes.max(ergebnis.bytes));
        Ok(ergebnis)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), OpError> {
        self.inner.rename(from, to)
    }

    fn move_to_trash(&self, path: &Path) -> Result<(), OpError> {
        self.inner.move_to_trash(path)
    }

    fn remove_file(&self, path: &Path) -> Result<(), OpError> {
        self.inner.remove_file(path)
    }

    fn remove_dir(&self, path: &Path) -> Result<(), OpError> {
        self.inner.remove_dir(path)
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), OpError> {
        self.inner.create_dir_all(path)
    }

    fn checksum(&self, path: &Path) -> Result<u64, OpError> {
        // WHY: the checksum reads the file again. This is the second read
        //       that the requirement of NFR-09a speaks about.
        let bytes = self.inner.metadata(path).map(|info| info.size).unwrap_or(0);
        self.volume.add(bytes);
        self.inner.checksum(path)
    }

    fn copy_symlink(&self, from: &Path, to: &Path) -> Result<(), OpError> {
        self.inner.copy_symlink(from, to)
    }
}
