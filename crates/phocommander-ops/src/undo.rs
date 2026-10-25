//! The undo of a copy.
//!
//! WHAT: what the undo can do ([{UndoScope}]), one entry of the ledger
//!       ([{UndoEntry}]), the ledger itself ([{UndoLedger}]) and the result of
//!       an undo ([{UndoOutcome}]).
//! WHY: the undo of the specification is deliberately incomplete (FR-26): the
//!       undo of a copy means "delete the copy", not "restore the state that
//!       was there before". A file that the copy overwrote is gone, and the
//!       undo cannot bring it back — the copy did not keep a copy of it. The
//!       limitation is therefore part of the type and not a remark in the
//!       documentation: {UndoScope::RemovesCopiesOnly} is the only scope the
//!       undo has today, and the outcome of an undo says in words that
//!       nothing was restored. The interface shows the same limitation — the
//!       notice of the undo area of the work surface carries it.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::undo::{UndoLedger, UndoScope};
//! use std::path::PathBuf;
//!
//! let mut ledger = UndoLedger::new();
//! ledger.record_copy(
//!     PathBuf::from("/photos/a.txt"),
//!     PathBuf::from("/backup/a.txt"),
//!     vec![PathBuf::from("/backup/a.txt")],
//! );
//! assert_eq!(ledger.scope(), UndoScope::RemovesCopiesOnly);
//! assert_eq!(ledger.len(), 1);
//! ```
//! BOUNDARY: this module works through the [{crate::port::FileSystem}] of the
//!       operations. It never touches a file system on its own.

use std::path::PathBuf;

use crate::error::{JobError, OpError};
use crate::port::FileSystem;

/// What the undo of the product can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UndoScope {
    /// The undo removes the copies and restores nothing.
    ///
    /// WHY: a copy does not keep a copy of the file it overwrote, so the
    ///       undo cannot restore it. The scope names this limitation, so
    ///       that the interface can show it and the user is not surprised.
    RemovesCopiesOnly,
}

impl UndoScope {
    /// The name of the scope.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::undo::UndoScope;
    ///
    /// assert_eq!(UndoScope::RemovesCopiesOnly.as_str(), "removes-copies-only");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            UndoScope::RemovesCopiesOnly => "removes-copies-only",
        }
    }

    /// Whether the undo restores the state that was there before.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::undo::UndoScope;
    ///
    /// assert!(!UndoScope::RemovesCopiesOnly.restores_previous_state());
    /// ```
    pub const fn restores_previous_state(&self) -> bool {
        false
    }
}

/// One operation that the ledger can take back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoEntry {
    /// The file the operation copied.
    pub source: PathBuf,

    /// The file the operation wrote.
    pub destination: PathBuf,

    /// The files the operation created.
    ///
    /// WHY: a copy of a tree creates more than one file. The undo removes
    ///       every file the operation created, and nothing else.
    pub created: Vec<PathBuf>,
}

/// The result of an undo.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UndoOutcome {
    /// The files the undo removed.
    pub removed: Vec<PathBuf>,

    /// The files the undo restored.
    ///
    /// WHY: the list is always empty today, because the undo of a copy does
    ///       not restore anything. The list exists so that the interface can
    ///       say what was restored — and that the answer is "nothing".
    pub restored: Vec<PathBuf>,

    /// The failures the undo collected.
    pub errors: Vec<JobError>,
}

/// The ledger of the operations that can be taken back.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoLedger {
    entries: Vec<UndoEntry>,
}

impl UndoLedger {
    /// Builds an empty ledger.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::undo::UndoLedger;
    ///
    /// let ledger = UndoLedger::new();
    /// assert!(ledger.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Remembers a copy.
    ///
    /// WHY: the ledger remembers what the copy created and not what it
    ///       overwrote — an overwritten file cannot be brought back, and a
    ///       ledger that promised it would lie.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::undo::UndoLedger;
    /// use std::path::PathBuf;
    ///
    /// let mut ledger = UndoLedger::new();
    /// ledger.record_copy(
    ///     PathBuf::from("/photos/a.txt"),
    ///     PathBuf::from("/backup/a.txt"),
    ///     vec![PathBuf::from("/backup/a.txt")],
    /// );
    /// assert_eq!(ledger.len(), 1);
    /// ```
    pub fn record_copy(
        &mut self,
        source: impl Into<PathBuf>,
        destination: impl Into<PathBuf>,
        created: Vec<PathBuf>,
    ) {
        self.entries.push(UndoEntry {
            source: source.into(),
            destination: destination.into(),
            created,
        });
    }

    /// The number of operations the ledger holds.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::undo::UndoLedger;
    ///
    /// assert_eq!(UndoLedger::new().len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the ledger holds nothing.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::undo::UndoLedger;
    ///
    /// assert!(UndoLedger::new().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The scope of the undo.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::undo::{UndoLedger, UndoScope};
    ///
    /// assert_eq!(UndoLedger::new().scope(), UndoScope::RemovesCopiesOnly);
    /// ```
    pub const fn scope(&self) -> UndoScope {
        UndoScope::RemovesCopiesOnly
    }

    /// The operation that the next undo takes back.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::undo::UndoLedger;
    ///
    /// assert!(UndoLedger::new().last().is_none());
    /// ```
    pub fn last(&self) -> Option<&UndoEntry> {
        self.entries.last()
    }

    /// Takes back the last operation.
    ///
    /// WHY: the undo removes the files the copy created. It restores nothing,
    ///       and the result says so — the list of the restored files is empty
    ///       and the scope of the undo names the limitation.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::port::FileSystem;
    /// use phocommander_ops::undo::UndoLedger;
    /// use std::path::PathBuf;
    ///
    /// # fn run(fs: &dyn FileSystem) -> Result<(), phocommander_ops::error::OpError> {
    /// let mut ledger = UndoLedger::new();
    /// ledger.record_copy(
    ///     PathBuf::from("/photos/a.txt"),
    ///     PathBuf::from("/backup/a.txt"),
    ///     vec![PathBuf::from("/backup/a.txt")],
    /// );
    /// let outcome = ledger.undo_last(fs)?;
    /// assert_eq!(outcome.removed.len(), 1);
    /// assert!(outcome.restored.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn undo_last(&mut self, fs: &dyn FileSystem) -> Result<UndoOutcome, OpError> {
        let Some(entry) = self.entries.pop() else {
            return Ok(UndoOutcome::default());
        };

        let mut outcome = UndoOutcome::default();
        for path in &entry.created {
            match fs.remove_file(path) {
                Ok(()) => outcome.removed.push(path.clone()),
                // WHY: a file that is not there any more is not a failure of
                //       the undo. The copy is taken back as far as the files
                //       that are still there allow it.
                Err(OpError::NotFound { .. }) => {}
                Err(error) => outcome.errors.push(JobError::new(
                    path.clone(),
                    error.code(),
                    error.to_string(),
                    true,
                )),
            }
        }

        Ok(outcome)
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::fake_fs::FakeFileSystem;
    use std::path::Path;

    #[test]
    fn the_undo_of_a_copy_removes_the_copy_and_restores_nothing() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/photos/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        fs.write_file(Path::new("/backup/a.txt"), b"payload")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        let mut ledger = UndoLedger::new();
        ledger.record_copy(
            PathBuf::from("/photos/a.txt"),
            PathBuf::from("/backup/a.txt"),
            vec![PathBuf::from("/backup/a.txt")],
        );

        // act
        let outcome = ledger.undo_last(&fs);

        // assert
        let outcome = outcome.unwrap_or_default();
        assert_eq!(outcome.removed.len(), 1);
        assert!(outcome.restored.is_empty());
        assert!(!fs.exists(Path::new("/backup/a.txt")));
        assert!(fs.exists(Path::new("/photos/a.txt")));
    }

    #[test]
    fn the_undo_names_its_limitation() {
        // act
        let ledger = UndoLedger::new();

        // assert
        assert_eq!(ledger.scope(), UndoScope::RemovesCopiesOnly);
        assert!(!ledger.scope().restores_previous_state());
        assert_eq!(ledger.scope().as_str(), "removes-copies-only");
    }

    #[test]
    fn an_empty_ledger_undoes_nothing() {
        // arrange
        let fs = FakeFileSystem::new();
        let mut ledger = UndoLedger::new();

        // act
        let outcome = ledger.undo_last(&fs);

        // assert
        let outcome = outcome.unwrap_or_default();
        assert!(outcome.removed.is_empty());
        assert!(ledger.is_empty());
    }

    #[test]
    fn the_undo_removes_every_file_the_copy_created() {
        // arrange
        let fs = FakeFileSystem::new();
        fs.write_file(Path::new("/backup/tree/a.txt"), b"a")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        fs.write_file(Path::new("/backup/tree/b.txt"), b"b")
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
        let mut ledger = UndoLedger::new();
        ledger.record_copy(
            PathBuf::from("/photos"),
            PathBuf::from("/backup/tree"),
            vec![
                PathBuf::from("/backup/tree/a.txt"),
                PathBuf::from("/backup/tree/b.txt"),
            ],
        );

        // act
        let outcome = ledger.undo_last(&fs);

        // assert
        let outcome = outcome.unwrap_or_default();
        assert_eq!(outcome.removed.len(), 2);
        assert!(!fs.exists(Path::new("/backup/tree/a.txt")));
        assert!(!fs.exists(Path::new("/backup/tree/b.txt")));
    }

    #[test]
    fn a_file_that_is_gone_does_not_fail_the_undo() {
        // arrange
        let fs = FakeFileSystem::new();
        let mut ledger = UndoLedger::new();
        ledger.record_copy(
            PathBuf::from("/photos/a.txt"),
            PathBuf::from("/backup/a.txt"),
            vec![PathBuf::from("/backup/a.txt")],
        );

        // act
        let outcome = ledger.undo_last(&fs);

        // assert
        let outcome = outcome.unwrap_or_default();
        assert!(outcome.errors.is_empty());
        assert!(outcome.removed.is_empty());
    }
}
