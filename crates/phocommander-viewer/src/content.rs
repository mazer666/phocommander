//! The content of a file the viewer opened.
//!
//! WHAT: the file the user opened and the bytes that were read from it
//!       ([`OpenedFile`]).
//! WHY: the viewer is read-only. It reads a file, it hands the bytes to a
//!       format adapter, and it never writes anything back. The type carries
//!       the path, the bytes and the moment of the last change, so that a
//!       format adapter does not have to ask the file system a second time.
//! EXAMPLE:
//! ```no_run
//! use phocommander_viewer::content::OpenedFile;
//! use std::path::PathBuf;
//!
//! let file = OpenedFile::new(PathBuf::from("/notes.txt"), b"hello".to_vec());
//! assert_eq!(file.size(), 5);
//! assert!(!file.is_empty());
//! ```

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The content of a file that the user opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedFile {
    path: PathBuf,
    bytes: Vec<u8>,
    size: u64,
    modified: Option<SystemTime>,
}

impl OpenedFile {
    /// Builds the content of a file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::content::OpenedFile;
    /// use std::path::{Path, PathBuf};
    ///
    /// let file = OpenedFile::new(PathBuf::from("/notes.txt"), b"hello".to_vec());
    /// assert_eq!(file.bytes(), b"hello");
    /// ```
    pub fn new(path: PathBuf, bytes: Vec<u8>) -> Self {
        let size = bytes.len() as u64;
        Self {
            path,
            bytes,
            size,
            modified: None,
        }
    }

    /// Adds the time of the last change of the file.
    ///
    /// WHY: the moment of the last change belongs to the file, and the
    /// reader knows it. A format adapter that shows a date needs it and
    /// must not read the file again for it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::content::OpenedFile;
    /// use std::path::PathBuf;
    /// use std::time::{Duration, UNIX_EPOCH};
    ///
    /// let file = OpenedFile::new(PathBuf::from("/notes.txt"), Vec::new())
    ///     .with_modified(UNIX_EPOCH + Duration::from_secs(1));
    /// assert!(file.modified().is_some());
    /// ```
    pub fn with_modified(mut self, modified: SystemTime) -> Self {
        self.modified = Some(modified);
        self
    }

    /// The path of the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::content::OpenedFile;
    /// use std::path::{Path, PathBuf};
    ///
    /// let file = OpenedFile::new(PathBuf::from("/notes.txt"), Vec::new());
    /// assert_eq!(file.path(), Path::new("/notes.txt"));
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The bytes that were read.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::content::OpenedFile;
    /// use std::path::PathBuf;
    ///
    /// let file = OpenedFile::new(PathBuf::from("/notes.txt"), b"hi".to_vec());
    /// assert_eq!(file.bytes().len(), 2);
    /// ```
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The size of the file in bytes.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::content::OpenedFile;
    /// use std::path::PathBuf;
    ///
    /// let file = OpenedFile::new(PathBuf::from("/notes.txt"), b"hello".to_vec());
    /// assert_eq!(file.size(), 5);
    /// ```
    pub fn size(&self) -> u64 {
        self.size
    }

    /// The time of the last change of the file, if the reader knows it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::content::OpenedFile;
    /// use std::path::PathBuf;
    ///
    /// let file = OpenedFile::new(PathBuf::from("/notes.txt"), Vec::new());
    /// assert!(file.modified().is_none());
    /// ```
    pub fn modified(&self) -> Option<SystemTime> {
        self.modified
    }

    /// Whether the file has no content at all.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::content::OpenedFile;
    /// use std::path::PathBuf;
    ///
    /// let file = OpenedFile::new(PathBuf::from("/notes.txt"), Vec::new());
    /// assert!(file.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn an_opened_file_carries_its_path_and_its_bytes() {
        // act
        let file = OpenedFile::new(PathBuf::from("/notes.txt"), b"hello".to_vec());

        // assert
        assert_eq!(file.path(), Path::new("/notes.txt"));
        assert_eq!(file.bytes(), b"hello");
        assert_eq!(file.size(), 5);
        assert!(!file.is_empty());
    }

    #[test]
    fn an_opened_file_without_content_is_empty() {
        // act
        let file = OpenedFile::new(PathBuf::from("/notes.txt"), Vec::new());

        // assert
        assert!(file.is_empty());
        assert_eq!(file.size(), 0);
    }

    #[test]
    fn the_time_of_the_last_change_is_carried_when_it_is_known() {
        // arrange
        let moment = UNIX_EPOCH + Duration::from_secs(42);

        // act
        let file = OpenedFile::new(PathBuf::from("/notes.txt"), Vec::new()).with_modified(moment);

        // assert
        assert_eq!(file.modified(), Some(moment));
    }
}
