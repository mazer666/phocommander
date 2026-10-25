//! One entry of a directory listing.
//!
//! WHAT: the two types that describe what a directory contains — [{Entry}]
//!       and [{EntryKind}].
//! WHY: the core asks a source for the content of a directory and the source
//!       answers with a list of entries (ADR-0002). The answer has to survive
//!       sources that cannot answer every question: a remote server may know
//!       the name of an entry but not its size, and a directory the user
//!       released on iPadOS may know neither. The two optional fields carry
//!       that "unknown" without a second type — and a source that does not
//!       know something must say so instead of guessing.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::entry::{Entry, EntryKind};
//!
//! let entry = Entry::new("notes.txt", EntryKind::File).with_size(42);
//! assert_eq!(entry.name, "notes.txt");
//! assert_eq!(entry.size, Some(42));
//! assert!(entry.kind.is_file());
//! ```

use std::time::SystemTime;

/// The kind of an entry of a directory listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A regular file.
    File,

    /// A directory.
    Directory,

    /// A symbolic link.
    ///
    /// WHY: a link is reported as a link and never as the file it points at.
    /// A listing that follows the link would show the target twice and would
    /// hide the fact that the target may live outside the directory.
    Symlink,

    /// Anything else — a socket, a device node or a named pipe.
    Other,
}

impl EntryKind {
    /// Whether the entry is a regular file.
    ///
    /// WHY: the operations ask this question about every entry they walk,
    /// and the answer decides whether the entry is copied byte by byte.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::entry::EntryKind;
    ///
    /// assert!(EntryKind::File.is_file());
    /// assert!(!EntryKind::Directory.is_file());
    /// ```
    pub fn is_file(&self) -> bool {
        matches!(self, EntryKind::File)
    }

    /// Whether the entry is a directory.
    ///
    /// WHY: a directory is walked, a file is copied. The two decisions are
    /// made in different places and must not be confused.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::entry::EntryKind;
    ///
    /// assert!(EntryKind::Directory.is_directory());
    /// assert!(!EntryKind::Symlink.is_directory());
    /// ```
    pub fn is_directory(&self) -> bool {
        matches!(self, EntryKind::Directory)
    }

    /// Whether the entry is a symbolic link.
    ///
    /// WHY: the copy operations treat a link as a link — they recreate the
    /// link instead of copying the file it points at, and they never follow
    /// it while deleting.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::entry::EntryKind;
    ///
    /// assert!(EntryKind::Symlink.is_symlink());
    /// assert!(!EntryKind::File.is_symlink());
    /// ```
    pub fn is_symlink(&self) -> bool {
        matches!(self, EntryKind::Symlink)
    }
}

/// One entry of a directory listing, as the core sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The name of the entry, without the path of its directory.
    ///
    /// WHY: the name is the only part that every source can deliver. The
    /// path of the entry belongs to the source and may be a bookmark, an
    /// index or a handle.
    pub name: String,

    /// The kind of the entry.
    pub kind: EntryKind,

    /// The size of the entry in bytes, if the source knows it.
    ///
    /// WHY: a remote source may not be able to answer the question, and a
    /// wrong answer would be worse than no answer.
    pub size: Option<u64>,

    /// The time of the last change of the entry, if the source knows it.
    pub modified: Option<SystemTime>,
}

impl Entry {
    /// Builds an entry with a name and a kind.
    ///
    /// WHY: the size and the time are optional, so a source that knows
    /// neither can still deliver a correct entry without inventing values.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::entry::{Entry, EntryKind};
    ///
    /// let entry = Entry::new("pictures", EntryKind::Directory);
    /// assert_eq!(entry.size, None);
    /// assert_eq!(entry.modified, None);
    /// ```
    pub fn new(name: impl Into<String>, kind: EntryKind) -> Self {
        Self {
            name: name.into(),
            kind,
            size: None,
            modified: None,
        }
    }

    /// Builds an entry that carries a size.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::entry::{Entry, EntryKind};
    ///
    /// let entry = Entry::new("movie.mkv", EntryKind::File).with_size(1024);
    /// assert_eq!(entry.size, Some(1024));
    /// ```
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Builds an entry that carries a modification time.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::entry::{Entry, EntryKind};
    /// use std::time::{Duration, SystemTime};
    ///
    /// let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    /// let entry = Entry::new("movie.mkv", EntryKind::File).with_modified(time);
    /// assert_eq!(entry.modified, Some(time));
    /// ```
    pub fn with_modified(mut self, modified: SystemTime) -> Self {
        self.modified = Some(modified);
        self
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn a_new_entry_knows_only_its_name_and_its_kind() {
        // act
        let entry = Entry::new("notes.txt", EntryKind::File);

        // assert
        assert_eq!(entry.name, "notes.txt");
        assert_eq!(entry.kind, EntryKind::File);
        assert_eq!(entry.size, None);
        assert_eq!(entry.modified, None);
    }

    #[test]
    fn the_optional_fields_carry_the_answer_of_the_source() {
        // arrange
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(42);

        // act
        let entry = Entry::new("movie.mkv", EntryKind::File)
            .with_size(2048)
            .with_modified(time);

        // assert
        assert_eq!(entry.size, Some(2048));
        assert_eq!(entry.modified, Some(time));
    }

    #[test]
    fn the_kind_answers_the_questions_the_operations_ask() {
        // act
        let kinds = [EntryKind::File, EntryKind::Directory, EntryKind::Symlink];

        // assert
        assert!(kinds[0].is_file());
        assert!(!kinds[0].is_directory());
        assert!(!kinds[0].is_symlink());

        assert!(kinds[1].is_directory());
        assert!(!kinds[1].is_file());

        assert!(kinds[2].is_symlink());
        assert!(!kinds[2].is_directory());
        assert!(!kinds[2].is_file());
    }

    #[test]
    fn the_other_kind_is_neither_a_file_nor_a_directory() {
        // act
        let kind = EntryKind::Other;

        // assert
        assert!(!kind.is_file());
        assert!(!kind.is_directory());
        assert!(!kind.is_symlink());
    }
}
