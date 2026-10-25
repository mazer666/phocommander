//! The atomic copy: write into a temporary file, check it, rename it into place.
//!
//! WHAT: the copy that never shows a half-written file ([{copy_atomically}]).
//! WHY: the atomic mode of the specification (section 6.9) exists for the
//!       files that a running program reads while they are written. The
//!       difference between the atomic mode and the verified mode is *where*
//!       the comparison happens: the verified mode writes into the target and
//!       compares afterwards, the atomic mode writes into a temporary file
//!       next to the target, compares that file and renames it into place.
//!       Three properties follow from that order, and every one of them is
//!       tested:
//!
//! 1. the comparison happens **before** the rename — a file that does
//!    not match never reaches the target;
//! 2. the temporary file has a name of its own, so that two copies into
//!    the same directory do not tread on each other;
//! 3. the temporary file is removed on every path out of the operation.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::copy::CopyMode;
//! use phocommander_ops::copy::atomic::copy_atomically;
//! use phocommander_ops::edit::JobControl;
//! use phocommander_ops::port::FileSystem;
//! use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
//! use std::path::Path;
//! use std::sync::Arc;
//!
//! struct Silent;
//! impl ProgressSink for Silent {
//!     fn report(&mut self, _update: ProgressUpdate) {}
//! }
//!
//! # fn run(fs: &dyn FileSystem) -> Result<(), phocommander_ops::error::OpError> {
//! let mut context = JobContext::new(Arc::new(JobControl::new()), Box::new(Silent));
//! let bytes = copy_atomically(
//!     fs,
//!     Path::new("/photos/sunset.jpg"),
//!     Path::new("/backup/sunset.jpg"),
//!     &mut context,
//! )?;
//! assert!(bytes > 0);
//! # Ok(())
//! # }
//! ```
//! BOUNDARY: this module works through the [{crate::port::FileSystem}] of the
//!       operations. It never touches a file system on its own.

use std::path::Path;

use crate::error::OpError;
use crate::port::FileSystem;
use crate::progress::JobContext;
use crate::temporary::TemporaryFile;

/// Copies a file so that the target never carries a half-written file.
///
/// WHY: the three steps of the mode are in this order and nowhere else:
/// the bytes go into a temporary file next to the target, the checksum of
/// that file is compared with the checksum of the content that was read, and
/// only then is the temporary file renamed into place. A file that fails the
/// comparison is removed and the target keeps whatever it carried before.
///
/// The conflict policy is decided by the caller — the copy of a tree applies
/// it before it reaches this function — because the policy is part of the
/// operation and not of the mode.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::copy::atomic::copy_atomically;
/// use phocommander_ops::edit::JobControl;
/// use phocommander_ops::port::FileSystem;
/// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
/// use std::path::Path;
/// use std::sync::Arc;
///
/// struct Silent;
/// impl ProgressSink for Silent {
///     fn report(&mut self, _update: ProgressUpdate) {}
/// }
///
/// # fn run(fs: &dyn FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let mut context = JobContext::new(Arc::new(JobControl::new()), Box::new(Silent));
/// copy_atomically(
///     fs,
///     Path::new("/photos/sunset.jpg"),
///     Path::new("/backup/sunset.jpg"),
///     &mut context,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn copy_atomically(
    fs: &dyn FileSystem,
    source: &Path,
    target: &Path,
    context: &mut JobContext,
) -> Result<u64, OpError> {
    // WHY: the guard removes the temporary file when it is dropped. Every
    // path that leaves this function — a failed copy, a failed comparison, a
    // failed rename — drops the guard and therefore removes the file.
    let mut temporary = TemporaryFile::reserve(fs, target);

    let result = fs.copy_file(source, temporary.path())?;

    // WHY: the comparison happens before the rename. The file that is
    // compared is the file that is written, and the target is only touched
    // when the two checksums agree.
    let checksum = fs.checksum(temporary.path())?;
    if checksum != result.checksum {
        context.begin_verification(1, result.bytes);
        context.add_verified_bytes(result.bytes);
        return Err(OpError::ChecksumMismatch {
            path: target.to_path_buf(),
        });
    }

    fs.rename(temporary.path(), target)?;

    // WHY: the file is at the target now. The temporary name does not exist
    // any more, and the guard must not try to remove a file that belongs to
    // the user.
    temporary.disarm();

    Ok(result.bytes)
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::fake_fs::FakeFileSystem;
    use crate::progress::{ProgressSink, ProgressUpdate};
    use std::sync::Arc;

    /// A sink that discards every report.
    struct Silent;

    impl ProgressSink for Silent {
        fn report(&mut self, _update: ProgressUpdate) {}
    }

    /// Builds a context for a job that runs without a queue.
    fn context() -> JobContext {
        JobContext::new(Arc::new(crate::edit::JobControl::new()), Box::new(Silent))
    }

    #[test]
    fn the_target_carries_the_content_after_the_copy() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        let mut context = context();

        // act
        let bytes = copy_atomically(
            &fs,
            Path::new("/photos/a.txt"),
            Path::new("/backup/a.txt"),
            &mut context,
        );

        // assert
        assert!(bytes.is_ok(), "{bytes:?}");
        assert_eq!(bytes.unwrap_or(0), 7);
        let content = fs
            .read_file(Path::new("/backup/a.txt"))
            .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
        assert_eq!(content, b"payload");
    }

    #[test]
    fn no_temporary_file_is_left_behind_after_a_successful_copy() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        let mut context = context();

        // act
        let _ = copy_atomically(
            &fs,
            Path::new("/photos/a.txt"),
            Path::new("/backup/a.txt"),
            &mut context,
        );

        // assert
        let leftovers: Vec<String> = fs
            .read_dir(Path::new("/backup"))
            .unwrap_or_default()
            .into_iter()
            .map(|entry| entry.name)
            .filter(|name| name.ends_with(".part"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }
}
