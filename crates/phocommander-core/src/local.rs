//! The local file system as a source of directory content.
//!
//! WHAT: [{LocalSource}], the implementation of [{Source}] and
//!       [{MutableSource}] that works on the file system of the machine the
//!       program runs on.
//! WHY: the local file system is the source that every other source is
//!       measured against, and it is the only one that can be written to
//!       without a permission the user has to grant. It treats a symbolic
//!       link as a link: the link is listed as a link, copied as a link and
//!       unlinked as a link. A source that follows the link would copy a file
//!       the user never selected, and a delete that follows it would remove
//!       a file outside the directory the user pointed at.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::local::LocalSource;
//! use phocommander_core::source::Source;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), phocommander_core::Error> {
//! let source = LocalSource::new();
//! for entry in source.list(Path::new("/tmp"))? {
//!     println!("{:?} {}", entry.kind, entry.name);
//! }
//! # Ok(())
//! # }
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::checksum::crc32_file;
use crate::entry::{Entry, EntryKind};
use crate::error::Error;
use crate::path::{CaseSensitivity, resolve_within};
use crate::source::{MutableSource, Source};

/// The local file system as a source of directory content.
#[derive(Debug, Clone, Default)]
pub struct LocalSource {
    root: Option<PathBuf>,
    case: CaseSensitivity,
}

impl LocalSource {
    /// Builds a source that uses the paths it is given as they are.
    ///
    /// WHY: the local file system is the only source whose entries are
    /// addressed by a path, so it needs no root to make a path meaningful.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::local::LocalSource;
    ///
    /// let source = LocalSource::new();
    /// assert!(source.root().is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            root: None,
            case: CaseSensitivity::Sensitive,
        }
    }

    /// Builds a source whose paths are relative to {root}.
    ///
    /// WHY: a source that stands for a directory the user released — on
    /// iPadOS, on Android, in a container — must not be able to leave that
    /// directory. The root makes the boundary part of the type instead of a
    /// convention.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::local::LocalSource;
    /// use std::path::Path;
    ///
    /// let source = LocalSource::rooted("/tmp/photos");
    /// assert_eq!(source.root(), Some(Path::new("/tmp/photos")));
    /// ```
    pub fn rooted(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
            case: CaseSensitivity::Sensitive,
        }
    }

    /// Sets the way the source compares names that differ only in their case.
    ///
    /// WHY: the core cannot know whether the file system of the machine is
    /// case-sensitive. The caller knows, and the source uses the answer for
    /// every check that depends on it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::local::LocalSource;
    /// use phocommander_core::path::CaseSensitivity;
    ///
    /// let source = LocalSource::new().with_case_sensitivity(CaseSensitivity::Insensitive);
    /// assert!(source.is_case_sensitive() == false);
    /// ```
    pub fn with_case_sensitivity(mut self, case: CaseSensitivity) -> Self {
        self.case = case;
        self
    }

    /// The directory the source is rooted at, if it is rooted.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::local::LocalSource;
    ///
    /// assert_eq!(LocalSource::new().root(), None);
    /// ```
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Whether the source compares names case-sensitively.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::local::LocalSource;
    ///
    /// assert!(LocalSource::new().is_case_sensitive());
    /// ```
    pub fn is_case_sensitive(&self) -> bool {
        self.case == CaseSensitivity::Sensitive
    }

    /// Resolves a path of the source to the path of the file system.
    ///
    /// WHY: a rooted source refuses a path that leaves its root. The check
    /// is the same one the archive extraction uses, so that a released
    /// directory behaves like a destination directory.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::local::LocalSource;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), phocommander_core::Error> {
    /// let source = LocalSource::rooted("/tmp/photos");
    /// let path = source.resolve(Path::new("2026/sunset.jpg"))?;
    /// assert!(path.starts_with("/tmp/photos"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolve(&self, path: &Path) -> Result<PathBuf, Error> {
        match &self.root {
            Some(root) => Ok(resolve_within(root, path, self.case)?),
            None => Ok(path.to_path_buf()),
        }
    }

    /// The checksum of a file of the local file system.
    ///
    /// WHY: the verified copy compares the checksum of the source and of the
    /// destination. The calculation belongs to the file system that holds the
    /// file, not to the operation that copies it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::local::LocalSource;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), phocommander_core::Error> {
    /// let source = LocalSource::new();
    /// let checksum = source.checksum(Path::new("notes.txt"))?;
    /// println!("{checksum:08x}");
    /// # Ok(())
    /// # }
    /// ```
    pub fn checksum(&self, path: &Path) -> Result<u32, Error> {
        crc32_file(path).map_err(|source| Error::io(path, source))
    }
}

impl Source for LocalSource {
    type Error = Error;

    fn list(&self, path: &Path) -> Result<Vec<Entry>, Self::Error> {
        let directory = self.resolve(path)?;
        let reader = fs::read_dir(&directory).map_err(|source| Error::io(&directory, source))?;

        let mut entries: Vec<Entry> = Vec::new();
        for item in reader {
            let item = item.map_err(|source| Error::io(&directory, source))?;
            let name = item.file_name().to_string_lossy().to_string();
            let metadata = item
                .path()
                .symlink_metadata()
                .map_err(|source| Error::io(&directory, source))?;

            let file_type = metadata.file_type();
            let kind = if file_type.is_symlink() {
                EntryKind::Symlink
            } else if file_type.is_dir() {
                EntryKind::Directory
            } else if file_type.is_file() {
                EntryKind::File
            } else {
                EntryKind::Other
            };

            // The size of a link is the length of the text it points at, not
            // the size of the file it points at. Reporting it would be a lie,
            // so the size of a link stays unknown.
            let size = match kind {
                EntryKind::File => Some(metadata.len()),
                _ => None,
            };

            let modified = metadata.modified().ok();
            let mut entry = Entry::new(name, kind);
            entry.size = size;
            entry.modified = modified;
            entries.push(entry);
        }

        // The order of the listing is the order of the names, so that two
        // listings of the same directory are the same list.
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }
}

impl MutableSource for LocalSource {
    fn copy(&self, from: &Path, to: &Path) -> Result<(), Self::Error> {
        let source = self.resolve(from)?;
        let destination = self.resolve(to)?;

        let metadata = fs::symlink_metadata(&source).map_err(|error| Error::io(&source, error))?;
        if metadata.file_type().is_symlink() {
            return copy_link(&source, &destination);
        }

        fs::copy(&source, &destination).map_err(|error| Error::write(&destination, error))?;
        Ok(())
    }

    fn move_entry(&self, from: &Path, to: &Path) -> Result<(), Self::Error> {
        let source = self.resolve(from)?;
        let destination = self.resolve(to)?;

        match fs::rename(&source, &destination) {
            Ok(()) => Ok(()),
            // A move across two file systems cannot be a rename. The copy and
            // the delete are two steps then, and a failure of the second one
            // leaves the copy behind — which the job reports.
            Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
                fs::copy(&source, &destination)
                    .map_err(|error| Error::write(&destination, error))?;
                fs::remove_file(&source).map_err(|error| Error::write(&source, error))?;
                Ok(())
            }
            Err(error) => Err(Error::write(&destination, error)),
        }
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), Self::Error> {
        let source = self.resolve(from)?;
        let destination = self.resolve(to)?;
        fs::rename(&source, &destination).map_err(|error| Error::write(&destination, error))
    }

    fn remove(&self, path: &Path) -> Result<(), Self::Error> {
        let target = self.resolve(path)?;
        let metadata = fs::symlink_metadata(&target).map_err(|error| Error::io(&target, error))?;

        if metadata.file_type().is_symlink() || metadata.is_file() {
            return fs::remove_file(&target).map_err(|error| Error::write(&target, error));
        }

        // The removal of a directory does not follow the links inside it: a
        // link is removed, the file it points at stays.
        fs::remove_dir_all(&target).map_err(|error| Error::write(&target, error))
    }

    fn create_directory(&self, path: &Path) -> Result<(), Self::Error> {
        let directory = self.resolve(path)?;
        fs::create_dir_all(&directory).map_err(|error| Error::write(&directory, error))
    }
}

/// Copies a symbolic link as a symbolic link.
///
/// WHY: a copy that dereferences the link would copy a file the user never
/// selected and would lose the link. On a platform that cannot create a
/// link, the copy fails with a message that says so.
#[cfg(unix)]
fn copy_link(from: &Path, to: &Path) -> Result<(), Error> {
    let target = fs::read_link(from).map_err(|error| Error::io(from, error))?;
    std::os::unix::fs::symlink(&target, to).map_err(|error| Error::write(to, error))
}

/// Copies a symbolic link as a symbolic link.
#[cfg(not(unix))]
fn copy_link(_from: &Path, _to: &Path) -> Result<(), Error> {
    Err(Error::write(
        Path::new(""),
        io::Error::new(
            io::ErrorKind::Unsupported,
            "this platform cannot create a symbolic link",
        ),
    ))
}

#[cfg(test)]
mod tests;
