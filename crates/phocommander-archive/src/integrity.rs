//! The integrity check of an archive.
//!
//! WHAT: the state of one entry of the check ([{IntegrityState}]), one entry
//!       of the report ([{IntegrityEntry}]), the report of the check
//!       ([{IntegrityReport}]) and the check itself ([{verify_archive}]).
//! WHY: an archive can be tested for integrity (FR-71): the checksum of
//!       every entry is compared with the value the container stores, and
//!       nothing is written to the disk. The check is read-only on purpose —
//!       a user asks whether an archive is intact *before* they extract it,
//!       and an extraction that repairs an archive would hide the damage.
//!       The TAR format stores no checksum of the content; the entries of a
//!       TAR container are therefore reported as {IntegrityState::NoChecksum},
//!       and the report says so in words.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::integrity::verify_archive;
//!
//! # fn main() -> Result<(), phocommander_archive::ArchiveError> {
//! let report = verify_archive(std::path::Path::new("holiday.zip"))?;
//! if report.is_intact() {
//!     println!("the archive is intact");
//! }
//! # Ok(())
//! # }
//! ```
//! BOUNDARY: this module reads an archive and writes a report. It never
//!       writes a file of the archive to the disk.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::crc32::Crc32;
use crate::entry::{ArchiveEntry, EntryKind};
use crate::error::ArchiveError;
use crate::formats::{EntryVisitor, VisitDecision, open_archive};

/// The number of bytes that are read at once.
const READ_BUFFER_BYTES: usize = 64 * 1024;

/// The state of one entry of the integrity check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityState {
    /// The checksum of the entry is the value the container stores.
    Verified,

    /// The checksum of the entry differs from the value the container stores.
    Mismatch {
        /// The value the container stores.
        expected: u64,

        /// The value the content of the entry has.
        found: u64,
    },

    /// The container stores no checksum for this entry.
    ///
    /// WHY: the TAR format keeps the checksum of the header of an entry and
    ///       not the checksum of its content. Such an entry cannot be
    ///       compared, and the report says that instead of pretending that
    ///       the entry was checked.
    NoChecksum,

    /// The entry could not be read.
    Unreadable {
        /// The reason why the entry could not be read.
        reason: String,
    },
}

impl IntegrityState {
    /// The name of the state.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::integrity::IntegrityState;
    ///
    /// assert_eq!(IntegrityState::Verified.as_str(), "verified");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            IntegrityState::Verified => "verified",
            IntegrityState::Mismatch { .. } => "mismatch",
            IntegrityState::NoChecksum => "no-checksum",
            IntegrityState::Unreadable { .. } => "unreadable",
        }
    }

    /// Whether the entry is intact.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::integrity::IntegrityState;
    ///
    /// assert!(IntegrityState::Verified.is_intact());
    /// assert!(!IntegrityState::NoChecksum.is_intact());
    /// ```
    pub const fn is_intact(&self) -> bool {
        matches!(self, IntegrityState::Verified)
    }
}

/// One entry of the report of the integrity check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityEntry {
    /// The name of the entry, as the container carries it.
    pub name: String,

    /// The kind of the entry.
    pub kind: EntryKind,

    /// The number of bytes the content of the entry holds.
    pub size: u64,

    /// The state of the entry.
    pub state: IntegrityState,
}

/// The report of the integrity check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityReport {
    /// The archive that was checked.
    pub path: PathBuf,

    /// The entries of the archive and their states.
    pub entries: Vec<IntegrityEntry>,
}

impl IntegrityReport {
    /// The entries of the report.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::integrity::IntegrityReport;
    ///
    /// let report = IntegrityReport { path: "/tmp/a.zip".into(), entries: Vec::new() };
    /// assert!(report.entries().is_empty());
    /// ```
    pub fn entries(&self) -> &[IntegrityEntry] {
        &self.entries
    }

    /// Whether every entry of the archive is intact.
    ///
    /// WHY: an entry without a checksum is not intact — the archive could not
    ///       be checked, and a report that called such an archive intact
    ///       would promise more than it knows.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::integrity::IntegrityReport;
    ///
    /// let report = IntegrityReport { path: "/tmp/a.zip".into(), entries: Vec::new() };
    /// assert!(report.is_intact());
    /// ```
    pub fn is_intact(&self) -> bool {
        self.entries.iter().all(|entry| entry.state.is_intact())
    }

    /// The entries that are not intact.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::integrity::IntegrityReport;
    ///
    /// let report = IntegrityReport { path: "/tmp/a.zip".into(), entries: Vec::new() };
    /// assert!(report.failed().is_empty());
    /// ```
    pub fn failed(&self) -> Vec<&IntegrityEntry> {
        self.entries
            .iter()
            .filter(|entry| !entry.state.is_intact())
            .collect()
    }

    /// The number of entries that were checked against a stored checksum.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::integrity::IntegrityReport;
    ///
    /// let report = IntegrityReport { path: "/tmp/a.zip".into(), entries: Vec::new() };
    /// assert_eq!(report.verified(), 0);
    /// ```
    pub fn verified(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.state.is_intact())
            .count()
    }
}

/// Tests an archive for integrity without extracting anything.
///
/// WHY: the check reads every entry and compares its checksum with the value
///       the container stores. Nothing is written to the disk, so the user can
///       test an archive that came from somewhere else before they trust it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::integrity::verify_archive;
///
/// # fn main() -> Result<(), phocommander_archive::ArchiveError> {
/// let report = verify_archive(std::path::Path::new("holiday.zip"))?;
/// println!("{} entries, {} intact", report.entries().len(), report.verified());
/// # Ok(())
/// # }
/// ```
pub fn verify_archive(path: &Path) -> Result<IntegrityReport, ArchiveError> {
    let mut reader = open_archive(path)?;
    let mut visitor = IntegrityVisitor::default();
    reader.for_each_entry(&mut visitor)?;

    Ok(IntegrityReport {
        path: path.to_path_buf(),
        entries: visitor.entries,
    })
}

/// The visitor that checks the entries of an archive.
#[derive(Debug, Default)]
struct IntegrityVisitor {
    entries: Vec<IntegrityEntry>,
}

impl EntryVisitor for IntegrityVisitor {
    fn visit(
        &mut self,
        entry: &ArchiveEntry,
        content: Option<&mut dyn Read>,
    ) -> Result<VisitDecision, ArchiveError> {
        // WHY: a directory and a link carry no content whose checksum could
        //       be compared. The container stores a value for them anyway —
        //       the ZIP format stores the checksum of the target of a link —
        //       but that value describes the target and not the entry. Such
        //       an entry therefore reports "no checksum" instead of being
        //       compared against zero, which would report a mismatch that
        //       does not exist.
        let Some(content) = content else {
            self.entries.push(IntegrityEntry {
                name: entry.name.clone(),
                kind: entry.kind,
                size: entry.size,
                state: IntegrityState::NoChecksum,
            });
            return Ok(VisitDecision::Continue);
        };

        let (checksum, size, failure) = read_checksum(Some(content));
        let state = match (failure, entry.stored_checksum) {
            (Some(reason), _) => IntegrityState::Unreadable { reason },
            (None, Some(expected)) => {
                let found = u64::from(checksum);
                if found == expected {
                    IntegrityState::Verified
                } else {
                    IntegrityState::Mismatch { expected, found }
                }
            }
            (None, None) => IntegrityState::NoChecksum,
        };

        self.entries.push(IntegrityEntry {
            name: entry.name.clone(),
            kind: entry.kind,
            size,
            state,
        });

        Ok(VisitDecision::Continue)
    }

    fn skips_container_checksum(&self) -> bool {
        // WHY: the check of this project compares the checksum itself, so
        //       that the report can name the entry that differs and the value
        //       it found. The container crate would only report "invalid
        //       checksum" without a name.
        true
    }
}

/// Reads the content of an entry and computes its checksum.
///
/// WHY: the content is read in pieces, so that an entry of any size can be
///       checked without holding it in the memory.
fn read_checksum(content: Option<&mut dyn Read>) -> (u32, u64, Option<String>) {
    let Some(content) = content else {
        return (0, 0, None);
    };

    let mut checksum = Crc32::new();
    let mut buffer = [0u8; READ_BUFFER_BYTES];
    let mut size = 0u64;

    loop {
        match content.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                checksum.update(&buffer[..count]);
                size += count as u64;
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return (checksum.finish(), size, Some(error.to_string())),
        }
    }

    (checksum.finish(), size, None)
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::entry::ArchiveEntry;
    use crate::formats::zip::ZipArchiveWriter;
    // WHY: the writing of an entry lives in the trait, not in the type of the
    //       container. Without the trait in scope the methods cannot be
    //       called.
    use crate::formats::ArchiveWriter;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Builds an entry with the given checksum.
    fn entry(name: &str, checksum: Option<u64>) -> ArchiveEntry {
        ArchiveEntry {
            name: name.to_string(),
            name_is_lossy: false,
            kind: EntryKind::File,
            size: 9,
            mode: None,
            link_target: None,
            stored_checksum: checksum,
        }
    }

    /// Builds a directory of its own for a test and removes it again.
    ///
    /// WHY: the test writes a real archive, because the integrity check
    ///       compares the checksum that the container really stores. The
    ///       directory carries the name of the process and the time, so that
    ///       two tests never share one.
    fn temporary_directory(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "phocommander-integrity-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|error| panic!("the directory could not be created: {error}"));
        path
    }

    /// Writes a ZIP container with one file.
    ///
    /// WHY: the writing lives in the trait `ArchiveWriter`, not in the type
    ///       of the container. The trait therefore has to be in scope, and the
    ///       container has to be handed over in a box, because `finish` takes
    ///       the writer by value.
    fn write_zip(path: &Path, content: &[u8]) {
        let mut writer = ZipArchiveWriter::create(path)
            .unwrap_or_else(|error| panic!("the archive could not be created: {error}"));
        let mut reader = std::io::Cursor::new(content.to_vec());
        writer
            .add_file("a.txt", content.len() as u64, &mut reader)
            .unwrap_or_else(|error| panic!("the entry could not be written: {error}"));
        Box::new(writer)
            .finish()
            .unwrap_or_else(|error| panic!("the archive could not be finished: {error}"));
    }

    /// Replaces the checksum that the container stores with a wrong one.
    ///
    /// WHY: a damaged archive is the case the check exists for. The
    ///       checksum of the content is replaced by its complement in the
    ///       header and in the directory of the container, so that the
    ///       archive stays readable and only the checksum is wrong.
    fn corrupt_stored_checksum(path: &Path, content: &[u8]) {
        let mut data = std::fs::read(path)
            .unwrap_or_else(|error| panic!("the archive could not be read: {error}"));
        let mut checksum = Crc32::new();
        checksum.update(content);
        let stored = checksum.finish().to_le_bytes();
        let wrong = (!checksum.finish()).to_le_bytes();

        let mut index = 0;
        while index + stored.len() <= data.len() {
            if data[index..index + stored.len()] == stored {
                data[index..index + stored.len()].copy_from_slice(&wrong);
                index += stored.len();
                continue;
            }
            index += 1;
        }

        std::fs::write(path, &data)
            .unwrap_or_else(|error| panic!("the archive could not be written: {error}"));
    }

    #[test]
    fn an_archive_that_was_just_written_is_intact() {
        // arrange
        let directory = temporary_directory("intact");
        let path = directory.join("archive.zip");
        write_zip(&path, b"payload");

        // act
        let report = verify_archive(&path);

        // assert
        let report = report.unwrap_or_else(|error| panic!("the check failed: {error}"));
        assert_eq!(report.entries().len(), 1);
        assert!(report.is_intact(), "{report:?}");
        assert_eq!(report.verified(), 1);
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn an_archive_with_a_wrong_checksum_is_reported_as_a_mismatch() {
        // arrange
        let directory = temporary_directory("damaged");
        let path = directory.join("archive.zip");
        write_zip(&path, b"payload");
        corrupt_stored_checksum(&path, b"payload");

        // act
        let report = verify_archive(&path);

        // assert
        let report = report.unwrap_or_else(|error| panic!("the check failed: {error}"));
        assert!(!report.is_intact(), "{report:?}");
        assert_eq!(report.failed().len(), 1);
        assert!(matches!(
            report.entries()[0].state,
            IntegrityState::Mismatch { .. }
        ));
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn an_entry_whose_checksum_matches_is_verified() {
        // arrange
        let mut visitor = IntegrityVisitor::default();
        let mut checksum = Crc32::new();
        checksum.update(b"123456789");
        let mut content = std::io::Cursor::new(b"123456789".to_vec());

        // act
        let _ = visitor.visit(
            &entry("a.txt", Some(u64::from(checksum.finish()))),
            Some(&mut content),
        );

        // assert
        assert_eq!(visitor.entries.len(), 1);
        assert_eq!(visitor.entries[0].state, IntegrityState::Verified);
        assert!(visitor.entries[0].state.is_intact());
    }

    #[test]
    fn an_entry_whose_checksum_differs_is_a_mismatch() {
        // arrange
        let mut visitor = IntegrityVisitor::default();
        let mut content = std::io::Cursor::new(b"123456789".to_vec());

        // act
        let _ = visitor.visit(&entry("a.txt", Some(1)), Some(&mut content));

        // assert
        assert_eq!(
            visitor.entries[0].state,
            IntegrityState::Mismatch {
                expected: 1,
                found: 0xcbf4_3926
            }
        );
        assert!(!visitor.entries[0].state.is_intact());
    }

    #[test]
    fn an_entry_without_content_carries_no_checksum_to_compare() {
        // arrange
        let mut visitor = IntegrityVisitor::default();
        let mut entry = entry("link", Some(1234));
        entry.kind = EntryKind::Symlink;

        // act
        let _ = visitor.visit(&entry, None);

        // assert
        // WHY: the container stores a value for a link, but that value
        //       describes the target of the link and not the entry itself.
        //       The entry therefore reports that it carries no checksum,
        //       instead of a mismatch that would be wrong.
        assert_eq!(visitor.entries[0].state, IntegrityState::NoChecksum);
        assert!(!visitor.entries[0].state.is_intact());
    }

    #[test]
    fn an_entry_without_a_stored_checksum_is_reported_as_such() {
        // arrange
        let mut visitor = IntegrityVisitor::default();

        // act
        let _ = visitor.visit(&entry("a.txt", None), None);

        // assert
        assert_eq!(visitor.entries[0].state, IntegrityState::NoChecksum);
        assert!(!visitor.entries[0].state.is_intact());
    }

    #[test]
    fn the_report_names_the_entries_that_are_not_intact() {
        // arrange
        let report = IntegrityReport {
            path: PathBuf::from("/tmp/a.zip"),
            entries: vec![
                IntegrityEntry {
                    name: "a.txt".to_string(),
                    kind: EntryKind::File,
                    size: 9,
                    state: IntegrityState::Verified,
                },
                IntegrityEntry {
                    name: "b.txt".to_string(),
                    kind: EntryKind::File,
                    size: 9,
                    state: IntegrityState::NoChecksum,
                },
            ],
        };

        // act, assert
        assert!(!report.is_intact());
        assert_eq!(report.failed().len(), 1);
        assert_eq!(report.verified(), 1);
    }
}
