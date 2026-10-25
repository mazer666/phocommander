//! The comparison of two directory trees.
//!
//! WHAT: the depth of a comparison ([{ComparisonDepth}]), the comparison
//!       itself ([{compare_trees}]) and the comparison of two directories as
//!       the plan of a synchronisation ([{compare_directories}]).
//! WHY: the comparison behind every synchronisation is the same and produces
//!       the same plan — per entry the state is identical, different,
//!       missing-in-destination or extra-in-destination, together with the
//!       size, the time and, where it was computed, the checksum of both
//!       sides. The directory comparison of FR-79 uses that very plan, so
//!       there is one comparison and two uses of it: a synchronisation and a
//!       comparison of two directories. A second implementation would be a
//!       second answer to the same question, and the two would drift apart.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::compare::{ComparisonDepth, compare_directories};
//! use phocommander_ops::copy::CopyMode;
//! use std::path::Path;
//!
//! # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
//! let plan = compare_directories(
//!     fs,
//!     Path::new("/photos"),
//!     Path::new("/backup"),
//!     CopyMode::SyncPreview,
//!     ComparisonDepth::Content,
//! )?;
//! for entry in plan.entries() {
//!     println!("{:?} {:?}", entry.state, entry.relative_path);
//! }
//! # Ok(())
//! # }
//! ```
//! BOUNDARY: this module works through the [{crate::port::FileSystem}] of the
//!       operations. It never touches a file system on its own.

use std::path::Path;

use crate::copy::{CopyMode, collect_items};
use crate::error::OpError;
use crate::port::FileSystem;
use crate::sync::{SyncAction, SyncPlan, SyncPlanEntry, SyncState};

/// How deeply a comparison looks at an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ComparisonDepth {
    /// The comparison looks at the size and at the time of the entry.
    ///
    /// WHY: the depth is the cheap one. It answers the question "has
    ///       anything changed?" well enough for a synchronisation that copies
    ///       everything that differs, and it reads every file at most once.
    #[default]
    Metadata,

    /// The comparison looks at the content of the entries as well.
    ///
    /// WHY: the depth is the one of FR-79 — two directories are compared for
    ///       content, date and size. Two files of the same size and the same
    ///       time can still differ, and only the checksum of the content
    ///       tells them apart.
    Content,
}

impl ComparisonDepth {
    /// Whether the comparison computes the checksum of the content.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::compare::ComparisonDepth;
    ///
    /// assert!(ComparisonDepth::Content.computes_checksums());
    /// assert!(!ComparisonDepth::Metadata.computes_checksums());
    /// ```
    pub const fn computes_checksums(&self) -> bool {
        matches!(self, ComparisonDepth::Content)
    }
}

/// Compares two directory trees and returns the entries of the comparison.
///
/// WHY: the result is a list of plan entries and nothing else. The caller
///       decides what the entries mean — a synchronisation executes them, a
///       comparison shows them. The list is sorted by the path of the entry,
///       so that two comparisons of the same trees produce the same list in
///       the same order.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::compare::compare_trees;
/// use phocommander_ops::copy::CopyMode;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let entries = compare_trees(
///     fs,
///     std::path::Path::new("/photos"),
///     std::path::Path::new("/backup"),
///     CopyMode::SyncMirror,
/// )?;
/// assert!(!entries.is_empty());
/// # Ok(())
/// # }
/// ```
pub fn compare_trees(
    fs: &dyn FileSystem,
    source: &Path,
    destination: &Path,
    mode: CopyMode,
) -> Result<Vec<SyncPlanEntry>, OpError> {
    compare_trees_with(fs, source, destination, mode, ComparisonDepth::Metadata)
}

/// Compares two directory trees with the given depth.
///
/// WHY: the depth decides whether the content of the entries is compared as
///       well. The checksum is only computed when it is needed — a file that
///       differs in its size needs no checksum, and a file that is missing on
///       the other side cannot be compared at all.
pub fn compare_trees_with(
    fs: &dyn FileSystem,
    source: &Path,
    destination: &Path,
    mode: CopyMode,
    depth: ComparisonDepth,
) -> Result<Vec<SyncPlanEntry>, OpError> {
    let source_items = collect_items(fs, source)?;
    // WHY: a destination that does not exist yet is not an error. Every
    // entry of the source is then missing in the destination, and that is
    // the answer the plan has to give.
    let destination_items = collect_items(fs, destination).unwrap_or_default();

    let mut entries: Vec<SyncPlanEntry> = Vec::new();

    for item in &source_items {
        let counterpart = destination_items
            .iter()
            .find(|other| other.relative_path == item.relative_path);

        let Some(other) = counterpart else {
            entries.push(SyncPlanEntry {
                relative_path: item.relative_path.clone(),
                kind: item.kind,
                state: SyncState::MissingInDestination,
                source_size: Some(item.size),
                destination_size: None,
                source_modified: item.modified,
                destination_modified: None,
                checksum: None,
                destination_checksum: None,
                action: SyncAction::Copy,
            });
            continue;
        };

        let (state, source_checksum, destination_checksum) =
            compare_entry(fs, item, other, destination, depth)?;

        let action = action_for(state, mode, item.modified, other.modified);
        entries.push(SyncPlanEntry {
            relative_path: item.relative_path.clone(),
            kind: item.kind,
            state,
            source_size: Some(item.size),
            destination_size: Some(other.size),
            source_modified: item.modified,
            destination_modified: other.modified,
            checksum: source_checksum,
            destination_checksum,
            action,
        });
    }

    for item in &destination_items {
        let known = source_items
            .iter()
            .any(|other| other.relative_path == item.relative_path);
        if known {
            continue;
        }
        // WHY: only the mirror mode deletes. Every other mode leaves the
        // extra entries alone, and the plan says so.
        let action = if mode.deletes() {
            SyncAction::Delete
        } else {
            SyncAction::Keep
        };
        entries.push(SyncPlanEntry {
            relative_path: item.relative_path.clone(),
            kind: item.kind,
            state: SyncState::ExtraInDestination,
            source_size: None,
            destination_size: Some(item.size),
            source_modified: None,
            destination_modified: item.modified,
            checksum: None,
            destination_checksum: None,
            action,
        });
    }

    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(entries)
}

/// Compares two directories and returns the plan of the comparison.
///
/// WHY: this is the directory comparison of FR-79, and it returns the very
///       plan that a synchronisation confirms. One comparison, two uses: the
///       user compares two directories, and the synchronisation moves what
///       the comparison found.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::compare::{ComparisonDepth, compare_directories};
/// use phocommander_ops::copy::CopyMode;
/// use std::path::Path;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let plan = compare_directories(
///     fs,
///     Path::new("/photos"),
///     Path::new("/backup"),
///     CopyMode::SyncPreview,
///     ComparisonDepth::Content,
/// )?;
/// assert_eq!(plan.source, Path::new("/photos"));
/// # Ok(())
/// # }
/// ```
pub fn compare_directories(
    fs: &dyn FileSystem,
    left: &Path,
    right: &Path,
    mode: CopyMode,
    depth: ComparisonDepth,
) -> Result<SyncPlan, OpError> {
    let entries = compare_trees_with(fs, left, right, mode, depth)?;
    Ok(SyncPlan {
        mode,
        source: left.to_path_buf(),
        destination: right.to_path_buf(),
        entries,
    })
}

/// Compares one entry of the source with its counterpart in the destination.
///
/// WHY: the state is decided in three steps. The size and the time have to be
///       equal, otherwise the entries differ. Only then is the content
///       compared — and only when the depth asks for it, because the checksum
///       of a file costs a read of the whole file.
fn compare_entry(
    fs: &dyn FileSystem,
    source: &crate::copy::TreeItem,
    destination: &crate::copy::TreeItem,
    destination_root: &Path,
    depth: ComparisonDepth,
) -> Result<(SyncState, Option<u64>, Option<u64>), OpError> {
    if source.is_directory || destination.is_directory {
        // WHY: a directory has no content to compare. Two directories are
        // the same when they carry the same time; a file and a directory are
        // never the same.
        if source.is_directory && destination.is_directory {
            let identical = source.modified.is_some() && source.modified == destination.modified;
            let state = if identical {
                SyncState::Identical
            } else {
                SyncState::Different
            };
            return Ok((state, None, None));
        }
        return Ok((SyncState::Different, None, None));
    }

    let same_size = source.size == destination.size;
    let same_time = source.modified.is_some() && source.modified == destination.modified;

    if !same_size || !same_time {
        return Ok((SyncState::Different, None, None));
    }

    if !depth.computes_checksums() {
        return Ok((SyncState::Identical, None, None));
    }

    let source_checksum = fs.checksum(&source.path)?;
    let destination_checksum = fs.checksum(&destination_root.join(&destination.relative_path))?;
    let state = if source_checksum == destination_checksum {
        SyncState::Identical
    } else {
        SyncState::Different
    };
    Ok((state, Some(source_checksum), Some(destination_checksum)))
}

/// The action that the mode derives from the state of an entry.
///
/// WHY: the mirror mode and the update mode bring the destination to the
///       state of the source. The two-way mode lets the newer version win,
///       and the timestamp decides that. When the timestamps are equal, the
///       source wins — the source is the side the user pointed at, and a
///       synchronisation that does nothing would hide a difference.
fn action_for(
    state: SyncState,
    mode: CopyMode,
    source_modified: Option<std::time::SystemTime>,
    destination_modified: Option<std::time::SystemTime>,
) -> SyncAction {
    if state == SyncState::Identical {
        return SyncAction::Keep;
    }

    match mode {
        CopyMode::SyncTwoWay => {
            if newer(source_modified, destination_modified) {
                SyncAction::CopyToSource
            } else {
                SyncAction::Overwrite
            }
        }
        _ => SyncAction::Overwrite,
    }
}

/// Whether the first time is the newer one.
fn newer(first: Option<std::time::SystemTime>, second: Option<std::time::SystemTime>) -> bool {
    match (first, second) {
        (Some(first), Some(second)) => first > second,
        (Some(_), None) => true,
        _ => false,
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::fake_fs::FakeFileSystem;

    /// Writes a file with the given content into the fake file system.
    fn write_file(fs: &FakeFileSystem, path: &str, content: &[u8]) {
        fs.write_file(Path::new(path), content)
            .unwrap_or_else(|error| panic!("the file could not be written: {error}"));
    }

    /// Creates a directory of the fake file system.
    ///
    /// WHY: the comparison reads both sides, and a side that does not exist
    ///       is an error and not an empty directory. A test that compares two
    ///       trees therefore has to create both of them.
    fn make_directory(fs: &FakeFileSystem, path: &str) {
        fs.create_dir_all(Path::new(path))
            .unwrap_or_else(|error| panic!("the directory could not be created: {error}"));
    }

    #[test]
    fn a_file_that_is_missing_in_the_destination_is_reported_as_missing() {
        // arrange
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"payload");

        // act
        let entries = compare_trees(
            &fs,
            Path::new("/photos"),
            Path::new("/backup"),
            CopyMode::SyncPreview,
        );

        // assert
        let entries = entries.unwrap_or_default();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].state, SyncState::MissingInDestination);
        assert_eq!(entries[0].action, SyncAction::Copy);
        assert_eq!(entries[0].source_size, Some(7));
    }

    #[test]
    fn a_file_that_is_only_in_the_destination_is_reported_as_extra() {
        // arrange
        let fs = FakeFileSystem::new();
        make_directory(&fs, "/photos");
        write_file(&fs, "/backup/old.txt", b"old");

        // act
        let entries = compare_trees(
            &fs,
            Path::new("/photos"),
            Path::new("/backup"),
            CopyMode::SyncPreview,
        );

        // assert
        let entries = entries.unwrap_or_default();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].state, SyncState::ExtraInDestination);
        assert_eq!(entries[0].action, SyncAction::Keep);
    }

    #[test]
    fn only_the_mirror_mode_deletes_the_extra_entries() {
        // arrange
        let fs = FakeFileSystem::new();
        make_directory(&fs, "/photos");
        write_file(&fs, "/backup/old.txt", b"old");

        // act
        let entries = compare_trees(
            &fs,
            Path::new("/photos"),
            Path::new("/backup"),
            CopyMode::SyncMirror,
        );

        // assert
        let entries = entries.unwrap_or_default();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, SyncAction::Delete);
    }

    #[test]
    fn the_content_comparison_finds_two_files_with_the_same_size_and_time() {
        // WHY: the fake file system gives every file the same time, so two
        // files of the same size look the same to the metadata comparison.
        // The checksum is what tells them apart.
        // arrange
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"aaaa");
        write_file(&fs, "/backup/a.txt", b"bbbb");

        // act
        let entries = compare_trees_with(
            &fs,
            Path::new("/photos"),
            Path::new("/backup"),
            CopyMode::SyncPreview,
            ComparisonDepth::Content,
        );

        // assert
        let entries = entries.unwrap_or_default();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].state, SyncState::Different);
        assert!(entries[0].checksum.is_some());
        assert!(entries[0].destination_checksum.is_some());
    }

    #[test]
    fn the_metadata_comparison_calls_two_files_with_the_same_size_the_same() {
        // arrange
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"aaaa");
        write_file(&fs, "/backup/a.txt", b"bbbb");

        // act
        let entries = compare_trees(
            &fs,
            Path::new("/photos"),
            Path::new("/backup"),
            CopyMode::SyncPreview,
        );

        // assert
        let entries = entries.unwrap_or_default();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].state, SyncState::Identical);
        assert!(entries[0].checksum.is_none());
    }

    #[test]
    fn the_comparison_of_two_directories_returns_the_plan_of_the_synchronisation() {
        // arrange
        let fs = FakeFileSystem::new();
        write_file(&fs, "/photos/a.txt", b"payload");

        // act
        let plan = compare_directories(
            &fs,
            Path::new("/photos"),
            Path::new("/backup"),
            CopyMode::SyncPreview,
            ComparisonDepth::Content,
        );

        // assert
        let plan = plan.unwrap_or_default();
        assert_eq!(plan.mode, CopyMode::SyncPreview);
        assert_eq!(plan.source, Path::new("/photos"));
        assert_eq!(plan.destination, Path::new("/backup"));
        assert_eq!(plan.entries().len(), 1);
    }
}
