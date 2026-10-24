//! The error type of the core layer.
//!
//! WHAT: the failures that the core layer reports, and nothing else.
//! WHY: every fallible operation in this workspace returns a `Result`. The
//!       error has to name what failed and carry the cause, otherwise the
//!       user interface can only show "something went wrong". The type lives
//!       in the core crate because the platform adapters report their failures
//!       in it, and the composition root turns it into a message.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::Error;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Error> {
//! let path = Path::new("/etc/hosts");
//! let metadata = std::fs::metadata(path).map_err(|source| Error::io(path, source))?;
//! println!("{} bytes", metadata.len());
//! # Ok(())
//! # }
//! ```

use std::io;
use std::path::{Path, PathBuf};

/// A failure of the core layer.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operating system did not report a directory for user configuration.
    #[error("the platform did not report a configuration directory")]
    NoConfigDirectory,

    /// A file or directory was larger than the limit that applies to it.
    #[error("{path} is {size} bytes, the limit is {limit} bytes")]
    TooLarge {
        /// The path of the file that was too large.
        path: PathBuf,
        /// The size that was measured.
        size: u64,
        /// The limit that was exceeded.
        limit: u64,
    },

    /// A filesystem operation failed.
    #[error("filesystem operation failed for {path}: {source}")]
    Io {
        /// The path the operation was working on.
        path: PathBuf,
        /// The error the operating system reported.
        #[source]
        source: io::Error,
    },
}

impl Error {
    /// Builds an [`Error::Io`] for a path and the error the operating system
    /// reported.
    ///
    /// WHY: the path is usually lost when an [`io::Error`] travels upwards.
    /// The path is what the user needs in order to understand the failure, so
    /// it is carried along.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::Error;
    /// use std::path::Path;
    ///
    /// let path = Path::new("/does/not/exist");
    /// let error = Error::io(path, std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
    /// assert!(error.to_string().contains("missing"));
    /// ```
    pub fn io(path: impl AsRef<Path>, source: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    /// Builds an [`Error::TooLarge`] for a path, the measured size and the
    /// limit that was exceeded.
    ///
    /// WHY: the viewer and the archive adapter have to tell the user why they
    /// refused a file. Naming the size and the limit makes the refusal
    /// explainable instead of mysterious.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::Error;
    /// use std::path::Path;
    ///
    /// let error = Error::too_large(Path::new("movie.mkv"), 20_000_000, 10_485_760);
    /// assert!(error.to_string().contains("movie.mkv"));
    /// ```
    pub fn too_large(path: impl AsRef<Path>, size: u64, limit: u64) -> Self {
        Self::TooLarge {
            path: path.as_ref().to_path_buf(),
            size,
            limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_names_the_path_and_keeps_the_source() {
        // arrange
        let path = Path::new("/tmp/missing.txt");
        let source = io::Error::new(io::ErrorKind::NotFound, "no such file");

        // act
        let error = Error::io(path, source);

        // assert
        let message = error.to_string();
        assert!(message.contains("/tmp/missing.txt"), "{message}");
        assert!(message.contains("no such file"), "{message}");
    }

    #[test]
    fn too_large_error_names_size_and_limit() {
        // arrange
        let path = Path::new("movie.mkv");

        // act
        let error = Error::too_large(path, 20_000_000, 10_485_760);

        // assert
        let message = error.to_string();
        assert!(message.contains("movie.mkv"), "{message}");
        assert!(message.contains("20000000"), "{message}");
        assert!(message.contains("10485760"), "{message}");
    }

    #[test]
    fn io_error_reports_the_original_error_as_source() {
        // arrange
        let source = io::Error::new(io::ErrorKind::PermissionDenied, "denied");

        // act
        let error = Error::io(Path::new("/root"), source);

        // assert
        assert_eq!(
            std::error::Error::source(&error).map(io::Error::kind),
            Some(io::ErrorKind::PermissionDenied)
        );
    }
}
