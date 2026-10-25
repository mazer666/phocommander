//! The reader of the viewer.
//!
//! WHAT: the reader that opens the files the viewer shows ([`ViewerReader`])
//!       and the failures it reports ([`ViewerError`]).
//! WHY: the reader is the place where the promise of the specification — "the
//!       application reads only what the user asks for" — becomes code. The
//!       reader has two doors: one for the file the user opened and one for
//!       the preview of a marked entry. The first is always open, the second
//!       depends on the rule "read_only_on_demand", and the door for a
//!       background scan does not exist at all. The size of the file is
//!       checked before the file is read, so that a file that is too large is
//!       refused without being loaded.
//! EXAMPLE:
//! ```no_run
//! use phocommander_viewer::policy::ReadPolicy;
//! use phocommander_viewer::reader::ViewerReader;
//! use std::path::Path;
//!
//! let reader = ViewerReader::new(ReadPolicy::strict(), 1024 * 1024);
//! let result = reader.open(Path::new("/does-not-exist"));
//! assert!(result.is_err());
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::content::OpenedFile;
use crate::policy::{ReadPolicy, ReadTrigger};

/// The failures of the viewer.
#[derive(Debug, thiserror::Error)]
pub enum ViewerError {
    /// The file could not be read.
    #[error("the file {path} could not be read: {source}")]
    Io {
        /// The path of the file that could not be read.
        path: PathBuf,
        /// The error the operating system reported.
        #[source]
        source: io::Error,
    },

    /// The read was refused by the rules.
    ///
    /// WHY: the refusal names the trigger, so that the log says *why* the
    /// file was not read and not only that it was not read.
    #[error("the file {path} was not read: {reason}")]
    Refused {
        /// The path of the file that was not read.
        path: PathBuf,
        /// The reason of the refusal.
        reason: &'static str,
    },

    /// The file is larger than the viewer is allowed to load.
    #[error("the file {path} has {size} bytes, the viewer reads at most {limit} bytes")]
    TooLarge {
        /// The path of the file that is too large.
        path: PathBuf,
        /// The size of the file in bytes.
        size: u64,
        /// The limit in bytes.
        limit: u64,
    },
}

/// Reads the files that the viewer shows.
#[derive(Debug, Clone)]
pub struct ViewerReader {
    policy: ReadPolicy,
    max_file_size: u64,
}

impl ViewerReader {
    /// Builds a reader.
    ///
    /// WHY: the limit is a parameter and not a constant of this crate. The
    /// workspace is flat, so the canonical limit lives in the core crate and
    /// the composition root hands it over; a second number in a second crate
    /// would drift.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::ReadPolicy;
    /// use phocommander_viewer::reader::ViewerReader;
    ///
    /// let reader = ViewerReader::new(ReadPolicy::strict(), 10_485_760);
    /// assert_eq!(reader.max_file_size(), 10_485_760);
    /// ```
    pub fn new(policy: ReadPolicy, max_file_size: u64) -> Self {
        Self {
            policy,
            max_file_size,
        }
    }

    /// The rules that decide what the reader may read.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::ReadPolicy;
    /// use phocommander_viewer::reader::ViewerReader;
    ///
    /// let reader = ViewerReader::new(ReadPolicy::strict(), 1024);
    /// assert!(reader.policy().reads_only_on_demand());
    /// ```
    pub fn policy(&self) -> &ReadPolicy {
        &self.policy
    }

    /// The largest file the reader loads, in bytes.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::ReadPolicy;
    /// use phocommander_viewer::reader::ViewerReader;
    ///
    /// let reader = ViewerReader::new(ReadPolicy::strict(), 4096);
    /// assert_eq!(reader.max_file_size(), 4096);
    /// ```
    pub const fn max_file_size(&self) -> u64 {
        self.max_file_size
    }

    /// Reads the file at {path} because the user opened it.
    ///
    /// WHY: this is the command of the user and it is always carried out. The
    /// file that the user opened is the one file the application is allowed
    /// to read without a further question.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::ReadPolicy;
    /// use phocommander_viewer::reader::ViewerReader;
    /// use std::path::Path;
    ///
    /// let reader = ViewerReader::new(ReadPolicy::strict(), 1024);
    /// let result = reader.open(Path::new("/notes.txt"));
    /// assert!(result.is_err() || result.is_ok());
    /// ```
    pub fn open(&self, path: &Path) -> Result<OpenedFile, ViewerError> {
        self.read(path, ReadTrigger::UserCommand)
    }

    /// Reads the file at {path} for the preview of the marked entry.
    ///
    /// WHY: a preview reads a file that the user has not opened. The rule
    /// "read_only_on_demand" decides whether that may happen; when the rule
    /// is in force the reader refuses and names the reason.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_viewer::policy::ReadPolicy;
    /// use phocommander_viewer::reader::ViewerReader;
    /// use std::path::Path;
    ///
    /// let reader = ViewerReader::new(ReadPolicy::strict(), 1024);
    /// let result = reader.preview(Path::new("/notes.txt"));
    /// assert!(result.is_err());
    /// ```
    pub fn preview(&self, path: &Path) -> Result<OpenedFile, ViewerError> {
        self.read(path, ReadTrigger::SelectionPreview)
    }

    /// Reads the file at {path} for the trigger.
    fn read(&self, path: &Path, trigger: ReadTrigger) -> Result<OpenedFile, ViewerError> {
        if !self.policy.allows(trigger) {
            return Err(ViewerError::Refused {
                path: path.to_path_buf(),
                reason: reason_for(trigger),
            });
        }

        let metadata = fs::metadata(path).map_err(|source| ViewerError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        if metadata.len() > self.max_file_size {
            return Err(ViewerError::TooLarge {
                path: path.to_path_buf(),
                size: metadata.len(),
                limit: self.max_file_size,
            });
        }

        let bytes = fs::read(path).map_err(|source| ViewerError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let modified: Option<SystemTime> = metadata.modified().ok();
        let file = OpenedFile::new(path.to_path_buf(), bytes);
        Ok(match modified {
            Some(modified) => file.with_modified(modified),
            None => file,
        })
    }
}

/// The reason why a trigger is refused.
const fn reason_for(trigger: ReadTrigger) -> &'static str {
    match trigger {
        ReadTrigger::UserCommand => "the user did not ask for this file",
        ReadTrigger::SelectionPreview => {
            "the rule read_only_on_demand forbids the preview of a file that was not opened"
        }
        ReadTrigger::BackgroundScan => {
            "the application keeps no index and reads no file that the user has not opened"
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// The counter that makes the name of a temporary file unique.
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Creates a directory that the test may write into.
    fn temporary_directory(label: &str) -> PathBuf {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "phocommander-viewer-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|error| panic!("the directory could not be created: {error}"));
        path
    }

    /// Writes a file with the given content into the directory.
    fn write_file(directory: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = directory.join(name);
        fs::write(&path, content)
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        path
    }

    #[test]
    fn the_reader_opens_the_file_the_user_asked_for() {
        // arrange
        let directory = temporary_directory("open");
        let path = write_file(&directory, "notes.txt", b"hello");
        let reader = ViewerReader::new(ReadPolicy::strict(), 1024);

        // act
        let file = reader.open(&path);

        // assert
        match file {
            Ok(file) => {
                assert_eq!(file.bytes(), b"hello");
                assert_eq!(file.size(), 5);
            }
            Err(error) => panic!("the file should have been opened: {error}"),
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_reader_refuses_the_preview_when_the_rule_is_in_force() {
        // arrange
        let directory = temporary_directory("preview-refused");
        let path = write_file(&directory, "notes.txt", b"hello");
        let reader = ViewerReader::new(ReadPolicy::strict(), 1024);

        // act
        let result = reader.preview(&path);

        // assert
        match result {
            Ok(_) => panic!("the preview should have been refused"),
            Err(ViewerError::Refused { reason, .. }) => {
                assert!(reason.contains("read_only_on_demand"), "{reason}");
            }
            Err(error) => panic!("the refusal should have been named: {error}"),
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_reader_reads_the_preview_when_the_rule_is_switched_off() {
        // arrange
        let directory = temporary_directory("preview-allowed");
        let path = write_file(&directory, "notes.txt", b"hello");
        let reader = ViewerReader::new(ReadPolicy::new(false), 1024);

        // act
        let file = reader.preview(&path);

        // assert
        match file {
            Ok(file) => assert_eq!(file.bytes(), b"hello"),
            Err(error) => panic!("the preview should have been read: {error}"),
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_reader_refuses_a_file_that_is_too_large() {
        // arrange
        let directory = temporary_directory("too-large");
        let path = write_file(&directory, "big.bin", &[0_u8; 64]);
        let reader = ViewerReader::new(ReadPolicy::strict(), 16);

        // act
        let result = reader.open(&path);

        // assert
        match result {
            Ok(_) => panic!("the file should have been refused"),
            Err(ViewerError::TooLarge { size, limit, .. }) => {
                assert_eq!(size, 64);
                assert_eq!(limit, 16);
            }
            Err(error) => panic!("the limit should have been named: {error}"),
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_reader_reports_a_file_that_does_not_exist() {
        // arrange
        let reader = ViewerReader::new(ReadPolicy::strict(), 1024);

        // act
        let result = reader.open(Path::new("/does-not-exist-phocommander.txt"));

        // assert
        match result {
            Ok(_) => panic!("the missing file should have been reported"),
            Err(ViewerError::Io { .. }) => {}
            Err(error) => panic!("the failure should have been named: {error}"),
        }
    }

    #[test]
    fn the_reader_reports_the_rules_it_works_with() {
        // arrange
        let reader = ViewerReader::new(ReadPolicy::strict(), 4096);

        // act, assert
        assert!(reader.policy().reads_only_on_demand());
        assert_eq!(reader.max_file_size(), 4096);
    }
}
