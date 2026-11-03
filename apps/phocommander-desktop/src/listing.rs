//! The reading of the directories of the desktop build.
//!
//! WHAT: the adapter that reads a directory and hands the entries to the
//! surface ([`read_entries`]).
//! WHY: the surface knows the panels and not the file system, and the file
//! system knows the disk and not the panels. The adapter is the one place
//! where the two meet, so that neither crate has to know the other.
//! The command of the surface that asks for a directory names the panel
//! and the path. The adapter reads the path and fills the panel — and a
//! directory that cannot be read leaves the panel as it was, with the
//! reason in the log of the application.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::listing::read_entries;
//! use phocommander_ops::fake_fs::FakeFileSystem;
//!
//! let fs = FakeFileSystem::new();
//! let eintraege = read_entries(&fs, std::path::Path::new("/photos"));
//! assert!(eintraege.is_ok());
//! ```

use std::path::Path;

use phocommander_ops::port::{DirEntry, EntryKind as OpsEntryKind, FileSystem};
use phocommander_ui::listing::{Entry, EntryKind};

/// Reads the entries of the directory at the given path.
///
/// WHY: a directory that cannot be read is not an error that stops the
///       application — the panel keeps what it shows, and the reason is in
///       the log. A file manager that shows nothing at all because one
///       directory is unreadable would be worse than useless.
pub fn read_entries(
    fs: &dyn FileSystem,
    path: &Path,
) -> Result<Vec<Entry>, phocommander_ops::error::OpError> {
    let eintraege = fs.read_dir(path)?;
    Ok(eintraege.iter().map(entry_of).collect())
}

/// The entry of the surface, out of the entry of the file system.
///
/// WHY: the two crates have their own entry, because the interface crate
///       may not depend on the operations crate. The conversion is here,
///       in the one place that knows both.
pub fn entry_of(eintrag: &DirEntry) -> Entry {
    Entry {
        name: eintrag.name.clone(),
        kind: kind_of(eintrag.kind),
        size: eintrag.size,
        modified: eintrag.modified,
        // WHY: an entry of the directory that is shown has the distance
        //       zero. The filter uses the distance to decide whether it
        //       reaches into the directories below (FR-74d).
        depth: 0,
    }
}

/// The kind of the surface, out of the kind of the file system.
fn kind_of(kind: OpsEntryKind) -> EntryKind {
    match kind {
        OpsEntryKind::File => EntryKind::File,
        OpsEntryKind::Directory => EntryKind::Directory,
        OpsEntryKind::Symlink => EntryKind::Symlink,
        OpsEntryKind::Other => EntryKind::Other,
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use phocommander_ops::fake_fs::FakeFileSystem;

    #[test]
    fn the_entries_of_the_directory_are_read() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file should have been written: {error}"));

        // act
        let eintraege = read_entries(&fs, Path::new("/photos"))
            .unwrap_or_else(|error| panic!("the directory should have been read: {error}"));

        // assert
        // WHY: the name of the entry and its kind are what the panel shows.
        assert!(eintraege.iter().any(|eintrag| eintrag.name == "a.txt"));
        assert!(eintraege.iter().all(|eintrag| eintrag.depth == 0));
    }

    #[test]
    fn a_directory_that_does_not_exist_gives_an_error() {
        // arrange
        let fs = FakeFileSystem::new();

        // act
        let ergebnis = read_entries(&fs, Path::new("/gibt-es-nicht"));

        // assert
        // WHY: the panel keeps what it shows, and the reason is in the log.
        assert!(ergebnis.is_err());
    }
}
