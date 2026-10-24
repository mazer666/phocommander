//! The two interfaces a source of directory content implements.
//!
//! WHAT: [{Source}] — the read side, a directory listing — and
//!       [{MutableSource}] — the write side, copy, move, rename, delete and
//!       create-directory.
//! WHY: the whole product addresses one concept — "what is in this
//!       directory" — and the concept has to behave the same on a local
//!       disk, in an archive, on an SFTP server and in a directory the user
//!       released on iPadOS. The interface is deliberately small: everything
//!       that distinguishes the sources — the protocol, the permissions, the
//!       timeouts — belongs behind it and not in the core (ADR-0002). The
//!       write side is a second, narrower interface, because an archive can
//!       be read but not modified and a released directory on iPadOS is
//!       read-only; a source that cannot be written to simply does not
//!       implement it.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::entry::Entry;
//! use phocommander_core::source::Source;
//! use std::path::Path;
//!
//! // Every source answers the same question in the same way.
//! fn count_entries<S: Source<Error = std::io::Error>>(
//!     source: &S,
//!     path: &Path,
//! ) -> Result<usize, std::io::Error> {
//!     let entries: Vec<Entry> = source.list(path)?;
//!     Ok(entries.len())
//! }
//! ```

use std::path::Path;

use crate::entry::Entry;

/// A source of directory content.
///
/// WHY: the trait is generic over its error type, so that the SFTP source
/// reports an authentication failure and the source of a released directory
/// on iPadOS reports a revoked permission. The core does not have to know
/// either of them; it turns them into its own error at the boundary
/// (ADR-0003).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::entry::Entry;
/// use phocommander_core::source::Source;
/// use std::path::Path;
///
/// struct Empty;
///
/// impl Source for Empty {
///     type Error = std::io::Error;
///
/// fn list(&self, _path: &Path) -> Result<Vec<Entry>, Self::Error> {
///         Ok(Vec::new())
///     }
/// }
///
/// let entries = Empty.list(Path::new("/tmp"))?;
/// assert!(entries.is_empty());
/// # Ok::<(), std::io::Error>(())
/// ```
pub trait Source {
/// The failure type of this source.
///
/// WHY: every source reports its own failures — a permission that was
/// revoked is not the same failure as a server that did not answer, and
/// the core must not flatten them into one string.
    type Error;

/// Lists the entries of the directory at {path}.
///
/// WHY: the listing is the one operation that every source supports.
/// It is also the operation the user interface needs most often, and it
/// is the only one that has to work on a source that cannot be written
/// to.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::entry::Entry;
/// use phocommander_core::source::Source;
/// use std::path::Path;
///
/// # fn list<S: Source<Error = std::io::Error>>(source: &S) -> Result<(), std::io::Error> {
/// let entries = source.list(Path::new("/tmp"))?;
/// for entry in entries {
///     println!("{}", entry.name);
/// }
/// # Ok(())
/// # }
/// ```
    fn list(&self, path: &Path) -> Result<Vec<Entry>, Self::Error>;
}

/// A source whose content can be changed.
///
/// WHY: reading a directory is universal, writing is not. An archive can be
/// listed but not modified, and a directory that the user released on iPadOS
/// is read-only. A separate interface puts that difference into the type
/// system instead of into a runtime check that can be forgotten.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::source::MutableSource;
/// use std::path::Path;
///
/// fn duplicate<S: MutableSource<Error = std::io::Error>>(
///     source: &S,
///     from: &Path,
///     to: &Path,
/// ) -> Result<(), std::io::Error> {
///     source.copy(from, to)
/// }
/// ```
pub trait MutableSource: Source {
/// Copies the file at {from} to {to}.
///
/// WHY: a symbolic link is copied as a link and never as the file it
/// points at. A copy that dereferences the link would copy a file the
/// user never selected and would lose the link itself.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::source::MutableSource;
/// use std::path::Path;
///
/// # fn copy<S: MutableSource<Error = std::io::Error>>(source: &S) -> Result<(), std::io::Error> {
/// source.copy(Path::new("a.txt"), Path::new("b.txt"))?;
/// # Ok(())
/// # }
/// ```
    fn copy(&self, from: &Path, to: &Path) -> Result<(), Self::Error>;

/// Moves the file or the directory at {from} to {to}.
///
/// WHY: the move is a separate operation and not a copy followed by a
/// delete, because the two fail differently: a move across two file
/// systems cannot be atomic, and a source that cannot move can still
/// copy.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::source::MutableSource;
/// use std::path::Path;
///
/// # fn move_it<S: MutableSource<Error = std::io::Error>>(source: &S) -> Result<(), std::io::Error> {
/// source.move_entry(Path::new("a.txt"), Path::new("b.txt"))?;
/// # Ok(())
/// # }
/// ```
    fn move_entry(&self, from: &Path, to: &Path) -> Result<(), Self::Error>;

/// Renames the file or the directory at {from} to {to}.
///
/// WHY: a rename stays inside one directory. The operations that need a
/// different directory use the move instead, because the two have
/// different failure modes on the different platforms.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::source::MutableSource;
/// use std::path::Path;
///
/// # fn rename_it<S: MutableSource<Error = std::io::Error>>(source: &S) -> Result<(), std::io::Error> {
/// source.rename(Path::new("a.txt"), Path::new("c.txt"))?;
/// # Ok(())
/// # }
/// ```
    fn rename(&self, from: &Path, to: &Path) -> Result<(), Self::Error>;

/// Deletes the file or the directory at {path}.
///
/// WHY: a symbolic link is unlinked and never followed. A delete that
/// follows the link would remove a file outside the directory the user
/// pointed at.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::source::MutableSource;
/// use std::path::Path;
///
/// # fn delete<S: MutableSource<Error = std::io::Error>>(source: &S) -> Result<(), std::io::Error> {
/// source.remove(Path::new("a.txt"))?;
/// # Ok(())
/// # }
/// ```
    fn remove(&self, path: &Path) -> Result<(), Self::Error>;

/// Creates the directory at {path} and every missing directory above it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::source::MutableSource;
/// use std::path::Path;
///
/// # fn create<S: MutableSource<Error = std::io::Error>>(source: &S) -> Result<(), std::io::Error> {
/// source.create_directory(Path::new("photos/2026"))?;
/// # Ok(())
/// # }
/// ```
    fn create_directory(&self, path: &Path) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{Entry, EntryKind};
    use std::cell::RefCell;

/// A source that answers from a list that the test prepares.
    struct FakeSource {
        entries: RefCell<Vec<Entry>>,
        copied: RefCell<Vec<(std::path::PathBuf, std::path::PathBuf)>>,
    }

    impl FakeSource {
        fn new(entries: Vec<Entry>) -> Self {
            Self {
                entries: RefCell::new(entries),
                copied: RefCell::new(Vec::new()),
            }
        }
    }

    impl Source for FakeSource {
        type Error = std::io::Error;

        fn list(&self, _path: &Path) -> Result<Vec<Entry>, Self::Error> {
            Ok(self.entries.borrow().clone())
        }
    }

    impl MutableSource for FakeSource {
        fn copy(&self, from: &Path, to: &Path) -> Result<(), Self::Error> {
            self.copied
                .borrow_mut()
                .push((from.to_path_buf(), to.to_path_buf()));
            Ok(())
        }

        fn move_entry(&self, from: &Path, to: &Path) -> Result<(), Self::Error> {
            self.copy(from, to)
        }

        fn rename(&self, from: &Path, to: &Path) -> Result<(), Self::Error> {
            self.copy(from, to)
        }

        fn remove(&self, _path: &Path) -> Result<(), Self::Error> {
            Ok(())
        }

        fn create_directory(&self, _path: &Path) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn a_source_answers_a_listing_with_its_entries() {
        // arrange
        let source = FakeSource::new(vec![
            Entry::new("a.txt", EntryKind::File),
            Entry::new("b.txt", EntryKind::File),
        ]);

        // act
        let entries = source.list(Path::new("/tmp")).unwrap_or_default();

        // assert
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "a.txt");
    }

    #[test]
    fn a_mutable_source_copies_the_file_it_is_given() {
        // arrange
        let source = FakeSource::new(Vec::new());

        // act
        let result = source.copy(Path::new("a.txt"), Path::new("b.txt"));

        // assert
        assert!(result.is_ok());
        let copied = source.copied.borrow();
        assert_eq!(copied.len(), 1);
        assert_eq!(copied[0].0, Path::new("a.txt"));
        assert_eq!(copied[0].1, Path::new("b.txt"));
    }

    #[test]
    fn a_mutable_source_moves_and_renames_as_well() {
        // arrange
        let source = FakeSource::new(Vec::new());

        // act
        let moved = source.move_entry(Path::new("a.txt"), Path::new("b.txt"));
        let renamed = source.rename(Path::new("b.txt"), Path::new("c.txt"));

        // assert
        assert!(moved.is_ok());
        assert!(renamed.is_ok());
        assert_eq!(source.copied.borrow().len(), 2);
    }

    #[test]
    fn a_mutable_source_removes_and_creates_directories() {
        // arrange
        let source = FakeSource::new(Vec::new());

        // act
        let removed = source.remove(Path::new("a.txt"));
        let created = source.create_directory(Path::new("photos"));

        // assert
        assert!(removed.is_ok());
        assert!(created.is_ok());
    }
}
