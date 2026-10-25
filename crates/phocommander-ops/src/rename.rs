//! The atomic multiple rename.
//!
//! WHAT: one pair of the rename ([{RenamePair}]), the conflicts of a rename
//!       ([{RenameConflict}], [{RenameConflictKind}]), the planning of the
//!       rename ([{plan_renames}]) and the execution of the rename
//!       ([{execute_renames}]).
//! WHY: the rename of a selection is atomic: either every file of the
//!       selection is renamed or none of them is (FR-41). A half-renamed
//!       selection is the worst outcome a file manager can produce — the
//!       user sees names that belong to two different rules and does not know
//!       which of them to trust. The file system offers no transaction, so
//!       the guarantee is built in three steps: every conflict is looked for
//!       *before* the first rename, the renames run in an order in which no
//!       file overwrites another, and a rename that fails in the middle is
//!       taken back by renaming the files that were already renamed.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::rename::{RenamePair, plan_renames};
//! use std::path::PathBuf;
//!
//! # fn run(fs: &dyn phocommander_ops::port::FileSystem) {
//! let pairs = vec![
//!     RenamePair::new(PathBuf::from("/photos/a.txt"), PathBuf::from("/photos/b.txt")),
//! ];
//! let conflicts = plan_renames(fs, &pairs);
//! assert!(conflicts.is_empty());
//! # }
//! ```
//! BOUNDARY: this module works through the [{crate::port::FileSystem}] of the
//!       operations. It never touches a file system on its own.

use std::path::{Path, PathBuf};

use crate::error::OpError;
use crate::port::FileSystem;
use crate::progress::JobContext;
use crate::temporary::temporary_path;

/// One file of a multiple rename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePair {
    /// The path the file carries now.
    pub source: PathBuf,

    /// The path the file receives.
    pub target: PathBuf,
}

impl RenamePair {
    /// Builds a pair of a rename.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::rename::RenamePair;
    /// use std::path::PathBuf;
    ///
    /// let pair = RenamePair::new(PathBuf::from("/photos/a.txt"), PathBuf::from("/photos/b.txt"));
    /// assert_eq!(pair.source, PathBuf::from("/photos/a.txt"));
    /// ```
    pub fn new(source: impl Into<PathBuf>, target: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

/// The reason why a rename is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenameConflictKind {
    /// The target of the rename is occupied by another file.
    TargetExists,

    /// Two files of the selection would receive the same name.
    DuplicateTarget,

    /// The file that is to be renamed does not exist.
    MissingSource,
}

impl RenameConflictKind {
    /// The name of the reason.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::rename::RenameConflictKind;
    ///
    /// assert_eq!(RenameConflictKind::TargetExists.as_str(), "target-exists");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            RenameConflictKind::TargetExists => "target-exists",
            RenameConflictKind::DuplicateTarget => "duplicate-target",
            RenameConflictKind::MissingSource => "missing-source",
        }
    }
}

/// A conflict that stops a multiple rename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameConflict {
    /// The reason why the rename is refused.
    pub kind: RenameConflictKind,

    /// The path the conflict is about.
    pub path: PathBuf,
}

impl RenameConflict {
    /// Builds a conflict.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::rename::{RenameConflict, RenameConflictKind};
    /// use std::path::PathBuf;
    ///
    /// let conflict = RenameConflict::new(RenameConflictKind::TargetExists, PathBuf::from("/photos/b.txt"));
    /// assert_eq!(conflict.kind, RenameConflictKind::TargetExists);
    /// ```
    pub fn new(kind: RenameConflictKind, path: impl Into<PathBuf>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

/// Looks for the conflicts of a multiple rename.
///
/// WHY: the conflicts are looked for before the first rename. A selection
///       with a conflict is refused as a whole — that is what "either every
///       file is renamed or none" means.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::rename::{RenamePair, plan_renames};
/// use std::path::PathBuf;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) {
/// let pairs = vec![RenamePair::new(
///     PathBuf::from("/photos/a.txt"),
///     PathBuf::from("/photos/b.txt"),
/// )];
/// let conflicts = plan_renames(fs, &pairs);
/// assert!(conflicts.is_empty());
/// # }
/// ```
pub fn plan_renames(fs: &dyn FileSystem, pairs: &[RenamePair]) -> Vec<RenameConflict> {
    let mut conflicts: Vec<RenameConflict> = Vec::new();

    for pair in pairs {
        if fs.metadata(&pair.source).is_err() {
            conflicts.push(RenameConflict::new(
                RenameConflictKind::MissingSource,
                pair.source.clone(),
            ));
            continue;
        }

        // WHY: a file may be renamed onto a name that one of the *other*
        // files of the selection is leaving. Such a target is free, and the
        // comparison has to look at the whole selection to find that out.
        let leaving = pairs
            .iter()
            .any(|other| other.source == pair.target && other.target != pair.target);
        if !leaving && fs.metadata(&pair.target).is_ok() {
            conflicts.push(RenameConflict::new(
                RenameConflictKind::TargetExists,
                pair.target.clone(),
            ));
        }
    }

    // WHY: two files that want the same name are one conflict, not two. The
    //       conflict is about the name, and the jump list of FR-35 points at
    //       the name. A list that carried the same path twice would send the
    //       user to the same place twice.
    let mut contested: Vec<PathBuf> = Vec::new();
    for (index, pair) in pairs.iter().enumerate() {
        let duplicated = pairs
            .iter()
            .enumerate()
            .any(|(other_index, other)| other_index != index && other.target == pair.target);
        if duplicated && !contested.contains(&pair.target) {
            contested.push(pair.target.clone());
        }
    }
    for target in contested {
        conflicts.push(RenameConflict::new(
            RenameConflictKind::DuplicateTarget,
            target,
        ));
    }

    conflicts
}

/// Renames every file of a selection, or none of them.
///
/// WHY: the rename is executed in three steps. First the conflicts are
///       looked for; a conflict refuses the whole rename and nothing is
///       changed. Then the renames run in an order in which no file
///       overwrites another, and every rename that succeeded is remembered.
///       If a rename fails, the remembered renames are taken back — the files
///       get their old names again — and the failure is reported. The
///       selection therefore never ends in a half-renamed state.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::edit::JobControl;
/// use phocommander_ops::progress::{JobContext, ProgressSink, ProgressUpdate};
/// use phocommander_ops::rename::{RenamePair, execute_renames};
/// use std::path::PathBuf;
/// use std::sync::Arc;
///
/// struct Silent;
/// impl ProgressSink for Silent {
///     fn report(&mut self, _update: ProgressUpdate) {}
/// }
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let mut context = JobContext::new(Arc::new(JobControl::new()), Box::new(Silent));
/// let pairs = vec![RenamePair::new(
///     PathBuf::from("/photos/a.txt"),
///     PathBuf::from("/photos/b.txt"),
/// )];
/// let renamed = execute_renames(fs, &pairs, &mut context)?;
/// assert_eq!(renamed, 1);
/// # Ok(())
/// # }
/// ```
pub fn execute_renames(
    fs: &dyn FileSystem,
    pairs: &[RenamePair],
    context: &mut JobContext,
) -> Result<usize, OpError> {
    let conflicts = plan_renames(fs, pairs);
    if !conflicts.is_empty() {
        return Err(OpError::RenameConflict {
            conflicts: conflicts
                .iter()
                .map(|conflict| format!("{}: {}", conflict.kind.as_str(), conflict.path.display()))
                .collect(),
        });
    }

    let steps = order_renames(pairs);
    let mut done: Vec<(PathBuf, PathBuf)> = Vec::new();

    for (from, to) in &steps {
        match context.checkpoint() {
            Ok(_) => {}
            Err(error) => {
                take_back(fs, &done);
                return Err(error);
            }
        }

        let name = item_name(to);
        context.begin_item(&name);

        if let Err(error) = fs.rename(from, to) {
            take_back(fs, &done);
            context.finish_item();
            return Err(error);
        }

        done.push((from.clone(), to.clone()));
        context.finish_item();
        context.report();
    }

    Ok(pairs.len())
}

/// The order in which the renames run.
///
/// WHY: a rename may use the name that another file of the selection is
///       leaving — `a.txt` becomes `b.txt` while `b.txt` becomes `c.txt`.
///       The file that leaves the name has to move first, otherwise the
///       first rename would overwrite it. The pairs are therefore sorted so
///       that a pair runs after the pair that empties its target. A cycle —
///       `a.txt` becomes `b.txt` and `b.txt` becomes `a.txt` — cannot be
///       sorted; one file of the cycle is moved aside under a name of its
///       own, and it is moved to its place at the end.
fn order_renames(pairs: &[RenamePair]) -> Vec<(PathBuf, PathBuf)> {
    let mut pending: Vec<RenamePair> = pairs.to_vec();
    let mut steps: Vec<(PathBuf, PathBuf)> = Vec::new();

    while !pending.is_empty() {
        match ready_index(&pending) {
            Some(index) => {
                let pair = pending.remove(index);
                steps.push((pair.source, pair.target));
            }
            None => {
                // WHY: the pairs form a cycle. One file is moved aside under
                // a name that no user file carries, and the pair continues
                // with that name as its source.
                let pair = pending.remove(0);
                let temporary = temporary_path(&pair.target);
                steps.push((pair.source.clone(), temporary.clone()));
                pending.push(RenamePair {
                    source: temporary,
                    target: pair.target,
                });
            }
        }
    }

    steps
}

/// The index of the pair that can run now.
///
/// WHY: a pair can run when no other pair of the selection still carries the
///       name of its target as its own name — that other pair has to move
///       first, so that the name is free.
fn ready_index(pending: &[RenamePair]) -> Option<usize> {
    pending.iter().enumerate().position(|(index, pair)| {
        !pending
            .iter()
            .enumerate()
            .any(|(other_index, other)| other_index != index && other.source == pair.target)
    })
}

/// Takes the renames that were carried out back.
///
/// WHY: the file system has no transaction, so the guarantee of the atomic
///       rename is built by hand: every file that was renamed gets its old
///       name back. The renames are taken back in the reverse order, because
///       a name that was used twice in the row has to be freed again before
///       it can be filled.
fn take_back(fs: &dyn FileSystem, done: &[(PathBuf, PathBuf)]) {
    for (from, to) in done.iter().rev() {
        let _ = fs.rename(to, from);
    }
}

/// The name of a file of a rename, as the job cue shows it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::rename::item_name;
/// use std::path::Path;
///
/// assert_eq!(item_name(Path::new("/photos/sunset.jpg")), "sunset.jpg");
/// ```
pub fn item_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::edit::JobControl;
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
        JobContext::new(Arc::new(JobControl::new()), Box::new(Silent))
    }

    /// Writes a file with the given content into the fake file system.
    fn write_file(fs: &FakeFileSystem, path: &str, content: &[u8]) {
        fs.write_file(Path::new(path), content)
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
    }

    #[test]
    fn a_rename_without_a_conflict_renames_every_file() {
        // arrange
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"a");
        write_file(&fs, "/photos/b.txt", b"b");
        let pairs = vec![
            RenamePair::new("/photos/a.txt", "/photos/x.txt"),
            RenamePair::new("/photos/b.txt", "/photos/y.txt"),
        ];
        let mut context = context();

        // act
        let renamed = execute_renames(&fs, &pairs, &mut context);

        // assert
        assert_eq!(renamed.unwrap_or(0), 2);
        assert!(fs.exists(Path::new("/photos/x.txt")));
        assert!(fs.exists(Path::new("/photos/y.txt")));
        assert!(!fs.exists(Path::new("/photos/a.txt")));
    }

    #[test]
    fn a_conflict_refuses_the_whole_rename() {
        // arrange
        // WHY: the conflict is a target that is occupied and that no file of
        //       the selection is leaving. A file that is renamed onto the
        //       name of a file that is itself renamed away is no conflict —
        //       the order of the renames frees the name.
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"a");
        write_file(&fs, "/photos/b.txt", b"b");
        write_file(&fs, "/photos/taken.txt", b"taken");
        let pairs = vec![
            RenamePair::new("/photos/a.txt", "/photos/x.txt"),
            RenamePair::new("/photos/b.txt", "/photos/taken.txt"),
        ];
        let mut context = context();

        // act
        let result = execute_renames(&fs, &pairs, &mut context);

        // assert
        assert!(
            matches!(result, Err(OpError::RenameConflict { .. })),
            "{result:?}"
        );
        assert!(fs.exists(Path::new("/photos/a.txt")));
        assert!(fs.exists(Path::new("/photos/b.txt")));
        assert!(fs.exists(Path::new("/photos/taken.txt")));
        assert!(!fs.exists(Path::new("/photos/x.txt")));
    }

    #[test]
    fn two_files_that_would_receive_the_same_name_are_a_conflict() {
        // arrange
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"a");
        write_file(&fs, "/photos/b.txt", b"b");
        let pairs = vec![
            RenamePair::new("/photos/a.txt", "/photos/x.txt"),
            RenamePair::new("/photos/b.txt", "/photos/x.txt"),
        ];

        // act
        let conflicts = plan_renames(&fs, &pairs);

        // assert
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].kind, RenameConflictKind::DuplicateTarget);
    }

    #[test]
    fn a_file_that_is_missing_is_a_conflict() {
        // arrange
        let fs = FakeFileSystem::new();
        let pairs = vec![RenamePair::new("/photos/gone.txt", "/photos/x.txt")];

        // act
        let conflicts = plan_renames(&fs, &pairs);

        // assert
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].kind, RenameConflictKind::MissingSource);
    }

    #[test]
    fn a_name_that_another_file_of_the_selection_leaves_is_free() {
        // arrange
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"a");
        write_file(&fs, "/photos/b.txt", b"b");
        let pairs = vec![
            RenamePair::new("/photos/a.txt", "/photos/b.txt"),
            RenamePair::new("/photos/b.txt", "/photos/c.txt"),
        ];
        let mut context = context();

        // act
        let conflicts = plan_renames(&fs, &pairs);
        let renamed = execute_renames(&fs, &pairs, &mut context);

        // assert
        assert!(conflicts.is_empty(), "{conflicts:?}");
        assert_eq!(renamed.unwrap_or(0), 2);
        let at_b = fs
            .read_file(Path::new("/photos/b.txt"))
            .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
        let at_c = fs
            .read_file(Path::new("/photos/c.txt"))
            .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
        assert_eq!(at_b, b"a");
        assert_eq!(at_c, b"b");
    }

    #[test]
    fn two_files_that_exchange_their_names_keep_their_content() {
        // WHY: the exchange is a cycle, and a cycle cannot be sorted. One
        // file is moved aside under a name of its own.
        // arrange
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"a");
        write_file(&fs, "/photos/b.txt", b"b");
        let pairs = vec![
            RenamePair::new("/photos/a.txt", "/photos/b.txt"),
            RenamePair::new("/photos/b.txt", "/photos/a.txt"),
        ];
        let mut context = context();

        // act
        let renamed = execute_renames(&fs, &pairs, &mut context);

        // assert
        assert_eq!(renamed.unwrap_or(0), 2);
        let at_a = fs
            .read_file(Path::new("/photos/a.txt"))
            .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
        let at_b = fs
            .read_file(Path::new("/photos/b.txt"))
            .unwrap_or_else(|error| panic!("the file could not be read: {error}"));
        assert_eq!(at_a, b"b");
        assert_eq!(at_b, b"a");
    }
}
