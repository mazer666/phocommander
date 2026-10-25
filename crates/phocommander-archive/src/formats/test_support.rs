//! The helpers that the tests of the container adapters share.
//!
//! WHAT: a temporary directory that removes itself when it is dropped, the
//!       visitor that collects the entries of a container and the entry that
//!       it collects.
//! WHY: the three adapters are tested in the same way — a container is
//!       written, opened again and compared with the entries that were
//!       written. The helpers hold that comparison in one place, so that
//!       every adapter is compared in the same terms and a change of the
//!       model of an entry does not have to be made in three test files.
//!       The helpers live in the crate and not in the test kit of the
//!       workspace, because the adapters are the only user of them and a
//!       crate of the workspace may not depend on a sibling crate.
//! EXAMPLE:
//! ```text
//! let directory = TempDir::new("demo");
//! let path = directory.write_file("a.txt", b"hello");
//! ```

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::entry::{ArchiveEntry, EntryKind};
use crate::error::ArchiveError;
use crate::formats::{ArchiveReader, EntryVisitor, VisitDecision};

/// Counter that makes the name of a temporary directory unique.
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A temporary directory that removes itself when it is dropped.
///
/// WHY: a test that leaves its files behind makes the next run depend on the
///       previous one. The name of the directory carries the process, the
///       time and a counter, so that two tests that run at the same time
///       never share one directory.
#[derive(Debug)]
pub struct TempDir {
    root: PathBuf,
}

impl TempDir {
    /// Creates a temporary directory whose name starts with `label`.
    ///
    /// WHY: the label makes the directory recognisable in a list of files
    ///       while a test is running.
    pub fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "phocommander-archive-{label}-{}-{nanos}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap_or_else(|error| {
            panic!("the temporary directory {root:?} could not be created: {error}")
        });
        Self { root }
    }

    /// Returns the path of a file in the temporary directory.
    ///
    /// WHY: the file does not have to exist. A test that wants to write a
    ///       container needs its path first.
    pub fn join(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    /// Writes a file with the given content and returns its path.
    ///
    /// WHY: the directories above the file are created as well, so that a
    ///       test can describe a tree and not repeat the creation of the
    ///       directories for every file.
    pub fn write_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.root.join(name);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, content)
            .unwrap_or_else(|error| panic!("the file {path:?} could not be written: {error}"));
        path
    }
}

impl Drop for TempDir {
    /// Removes the temporary directory and everything in it.
    ///
    /// WHY: the removal is best effort. A file that cannot be removed — a
    ///       file that is still open, for example — must not turn a green
    ///       test into a red one.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// One entry of a container, as a test wants to see it.
///
/// WHY: the entry of the crate does not carry the content, because the
///       content is handed over as a reader. A test has to have the content
///       next to the other fields, to compare a container with what was
///       written into it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectedEntry {
    /// The name of the entry, as the container carries it.
    pub name: String,

    /// Whether the name could not be decoded without a loss.
    pub name_is_lossy: bool,

    /// What the entry is.
    pub kind: EntryKind,

    /// The number of bytes the entry announces.
    pub size: u64,

    /// The mode bits the entry carries, if the format has them.
    pub mode: Option<u32>,

    /// The target of the link, if the entry is a link.
    pub link_target: Option<String>,

    /// The checksum the container stores for the content of the entry.
    pub stored_checksum: Option<u64>,

    /// The content of the entry, if the container gave one.
    pub content: Option<Vec<u8>>,
}

/// The visitor that collects the entries of a container.
///
/// WHY: the visitor owns the rules of the extraction — the limits, the path
///       resolution, the permissions. A test wants to see the entries and
///       nothing else, so it brings a visitor that keeps them.
#[derive(Debug, Default)]
pub struct CollectingVisitor {
    entries: Vec<CollectedEntry>,
}

impl CollectingVisitor {
    /// Builds a visitor that has not seen an entry yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// The entries the visitor has seen, in the order of the container.
    pub fn entries(&self) -> &[CollectedEntry] {
        &self.entries
    }
}

impl EntryVisitor for CollectingVisitor {
    fn visit(
        &mut self,
        entry: &ArchiveEntry,
        content: Option<&mut dyn Read>,
    ) -> Result<VisitDecision, ArchiveError> {
        let content = match content {
            Some(reader) => {
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).map_err(|source| {
                    ArchiveError::io(format!("the entry {:?}", entry.name), source)
                })?;
                Some(buffer)
            }
            None => None,
        };

        self.entries.push(CollectedEntry {
            name: entry.name.clone(),
            name_is_lossy: entry.name_is_lossy,
            kind: entry.kind,
            size: entry.size,
            mode: entry.mode,
            link_target: entry.link_target.clone(),
            stored_checksum: entry.stored_checksum,
            content,
        });

        Ok(VisitDecision::Continue)
    }
}

/// Reads every entry of a container and returns them.
///
/// WHY: the three adapters are compared in the same terms. The helper reads
///       the entries with one visitor, so that a difference between two
///       adapters is a difference of the adapters and not of the test.
pub fn collect(reader: &mut dyn ArchiveReader) -> Result<Vec<CollectedEntry>, ArchiveError> {
    let mut visitor = CollectingVisitor::new();
    reader.for_each_entry(&mut visitor)?;
    Ok(visitor.entries().to_vec())
}
