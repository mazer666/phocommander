//! The document of a synchronisation plan.
//!
//! WHAT: the document of the plan ([{write_document}]) and the fingerprint
//!       that names it ([{fingerprint}]).
//! WHY: the plan is the document the acceptance of a synchronisation refers
//!       to, and an acceptance has to be able to name the plan it talks
//!       about. The document is therefore a stable text with a fixed grammar:
//!       the same plan always produces the same document, and the document
//!       carries a fingerprint that changes as soon as one entry of the plan
//!       changes. The fingerprint is the FNV-1a hash of the body of the
//!       document — a short, well-known digest that needs no dependency and
//!       that is easy to reproduce by hand.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::sync::document::write_document;
//! use phocommander_ops::sync::{SyncPlan, build_plan};
//! use phocommander_ops::copy::CopyMode;
//! use std::path::Path;
//!
//! # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
//! let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncPreview)?;
//! let document = write_document(&plan);
//! assert!(document.starts_with("phocommander-sync-plan v1"));
//! # Ok(())
//! # }
//! ```
//! BOUNDARY: this module reads a plan and writes text. It performs no file
//!       operation and knows no file system.

use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::sync::SyncPlan;

/// The offset of the FNV-1a hash.
pub const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// The prime of the FNV-1a hash.
pub const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The first line of the document.
const HEADER: &str = "phocommander-sync-plan v1";

/// Writes the document of a plan.
///
/// WHY: the document is the plan in a form that can be quoted, stored and
///       compared. Every value of the plan is in the document, and the
///       document holds nothing else — two documents are equal exactly when
///       the plans are equal.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::sync::document::write_document;
/// use phocommander_ops::sync::{SyncPlan, build_plan};
/// use phocommander_ops::copy::CopyMode;
/// use std::path::Path;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncMirror)?;
/// let document = write_document(&plan);
/// assert!(document.contains("mode sync-mirror"));
/// assert!(document.contains("fingerprint "));
/// # Ok(())
/// # }
/// ```
pub fn write_document(plan: &SyncPlan) -> String {
    let body = body_lines(plan);
    let fingerprint = digest(&body);

    let mut document = String::new();
    let _ = writeln!(document, "{HEADER}");
    for line in &body {
        let _ = writeln!(document, "{line}");
    }
    let _ = writeln!(document, "fingerprint {fingerprint:016x}");
    document
}

/// The fingerprint of a plan.
///
/// WHY: the fingerprint is the digest of the body of the document. Two plans
///       with the same fingerprint hold the same entries, and a plan whose
///       fingerprint changed is a plan that changed.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::sync::document::fingerprint;
/// use phocommander_ops::sync::{SyncPlan, build_plan};
/// use phocommander_ops::copy::CopyMode;
/// use std::path::Path;
///
/// # fn run(fs: &dyn phocommander_ops::port::FileSystem) -> Result<(), phocommander_ops::error::OpError> {
/// let plan = build_plan(fs, Path::new("/photos"), Path::new("/backup"), CopyMode::SyncPreview)?;
/// assert_eq!(fingerprint(&plan), fingerprint(&plan));
/// # Ok(())
/// # }
/// ```
pub fn fingerprint(plan: &SyncPlan) -> u64 {
    digest(&body_lines(plan))
}

/// The lines of the body of the document.
fn body_lines(plan: &SyncPlan) -> Vec<String> {
    let mut lines = Vec::with_capacity(plan.entries.len() + 4);
    lines.push(format!("mode {}", plan.mode.as_str()));
    lines.push(format!("source {}", quote(&plan.source.to_string_lossy())));
    lines.push(format!(
        "destination {}",
        quote(&plan.destination.to_string_lossy())
    ));
    lines.push(format!("entries {}", plan.entries.len()));

    for entry in &plan.entries {
        lines.push(format!(
            "entry {} {} {} {} {} {} {} {} {} {}",
            quote(&entry.relative_path.to_string_lossy()),
            entry.kind.as_str(),
            entry.state.as_str(),
            entry.action.as_str(),
            optional_number(entry.source_size),
            optional_number(entry.destination_size),
            format_time(entry.source_modified),
            format_time(entry.destination_modified),
            optional_number(entry.checksum),
            optional_number(entry.destination_checksum),
        ));
    }

    lines
}

/// The FNV-1a hash of the lines of the body.
fn digest(lines: &[String]) -> u64 {
    let mut hash = FNV_OFFSET;
    for line in lines {
        for byte in line.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash ^= u64::from(b'\n');
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Writes a number, or a dash when the number is unknown.
fn optional_number(value: Option<u64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "-".to_string(),
    }
}

/// Writes a time as the seconds and the nanoseconds since the epoch.
///
/// WHY: the document has to be the same on every machine, so the time is
///       written as a number and not as a formatted date. A time that lies
///       before the epoch is written as a dash — the value cannot be
///       expressed as a duration since the epoch.
fn format_time(time: Option<SystemTime>) -> String {
    match time {
        None => "-".to_string(),
        Some(time) => match time.duration_since(UNIX_EPOCH) {
            Ok(duration) => format!("{}.{:09}", duration.as_secs(), duration.subsec_nanos()),
            Err(_) => "-".to_string(),
        },
    }
}

/// Writes a text in quotes, so that a space or a separator in a path cannot
/// break the grammar of the document.
fn quote(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for character in text.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::copy::CopyMode;
    use crate::port::EntryKind;
    use crate::sync::{SyncAction, SyncPlanEntry, SyncState};
    use std::path::PathBuf;

    /// Builds a plan with two entries.
    fn plan() -> SyncPlan {
        SyncPlan {
            mode: CopyMode::SyncMirror,
            source: PathBuf::from("/photos"),
            destination: PathBuf::from("/backup"),
            entries: vec![
                SyncPlanEntry {
                    relative_path: PathBuf::from("a.txt"),
                    kind: EntryKind::File,
                    state: SyncState::Different,
                    source_size: Some(7),
                    destination_size: Some(7),
                    source_modified: None,
                    destination_modified: None,
                    checksum: Some(11),
                    destination_checksum: Some(22),
                    action: SyncAction::Overwrite,
                },
                SyncPlanEntry {
                    relative_path: PathBuf::from("old.txt"),
                    kind: EntryKind::File,
                    state: SyncState::ExtraInDestination,
                    source_size: None,
                    destination_size: Some(3),
                    source_modified: None,
                    destination_modified: None,
                    checksum: None,
                    destination_checksum: None,
                    action: SyncAction::Delete,
                },
            ],
        }
    }

    #[test]
    fn the_document_names_the_plan_and_its_entries() {
        // act
        let document = write_document(&plan());

        // assert
        assert!(
            document.starts_with("phocommander-sync-plan v1\n"),
            "{document}"
        );
        assert!(document.contains("mode sync-mirror"), "{document}");
        assert!(document.contains("source \"/photos\""), "{document}");
        assert!(document.contains("destination \"/backup\""), "{document}");
        assert!(document.contains("entries 2"), "{document}");
        assert!(document.contains("fingerprint "), "{document}");
    }

    #[test]
    fn the_same_plan_always_produces_the_same_document() {
        // act
        let first = write_document(&plan());
        let second = write_document(&plan());

        // assert
        assert_eq!(first, second);
        assert_eq!(fingerprint(&plan()), fingerprint(&plan()));
    }

    #[test]
    fn a_changed_entry_changes_the_fingerprint() {
        // arrange
        let before = plan();
        let mut after = plan();
        after.entries[0].source_size = Some(8);

        // act, assert
        assert_ne!(fingerprint(&before), fingerprint(&after));
    }

    #[test]
    fn a_path_with_a_space_does_not_break_the_document() {
        // arrange
        let mut changed = plan();
        changed.entries[0].relative_path = PathBuf::from("a file.txt");

        // act
        let document = write_document(&changed);

        // assert
        assert!(document.contains("entry \"a file.txt\""), "{document}");
    }

    #[test]
    fn the_hash_of_an_empty_body_is_the_offset_of_the_hash() {
        // act, assert
        assert_eq!(digest(&[]), FNV_OFFSET);
    }
}
