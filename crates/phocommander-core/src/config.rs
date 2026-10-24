//! Canonical configuration defaults of Phocommander.
//!
//! WHAT: every limit that more than one crate has to agree on, each exactly
//!       once, with a name that explains what it limits.
//! WHY: a limit that exists twice drifts. The specification fixes the numbers
//!       in {docs/REQUIREMENTS.md} under "Konfigurationsdatei"; this module is
//!       the single place where they are written down, so that the
//!       configuration file, the viewer, the archive adapter and the test
//!       suite cannot disagree about them. A limit that only one crate
//!       enforces does not belong here — it lives in that crate.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::config::MAX_ARCHIVE_ENTRIES;
//!
//! # let entry_count = 12_u64;
//! if entry_count > MAX_ARCHIVE_ENTRIES {
//!     return Err("archive has too many entries");
//! }
//! # Ok::<(), &str>(())
//! ```

/// Largest total number of bytes that a single archive extraction may write.
///
/// WHY: an archive is untrusted input. Without a total limit a small archive
/// can fill the disk of the machine it is unpacked on (decompression bomb).
/// The value is the one the specification fixes in the {[archives]} section of
/// the configuration file.
pub const MAX_EXTRACTED_BYTES: u64 = 2_147_483_648;

/// Largest number of entries that a single archive may contain.
///
/// WHY: an archive can declare millions of entries without occupying much
/// space. The limit keeps the listing of such an archive from exhausting the
/// memory of the process.
pub const MAX_ARCHIVE_ENTRIES: u64 = 100_000;

/// Largest number of bytes that the viewer loads from a file into memory.
///
/// WHY: the viewer runs inside the user interface thread. A file that is
/// loaded completely would block the interface and could exhaust the memory of
/// the machine.
pub const MAX_VIEWER_FILE_SIZE: u64 = 10_485_760;

/// Largest number of bytes that the viewer reads from a file as text.
///
/// WHY: text decoding needs several times the memory of the file itself
/// (character conversion, line splitting). The limit is therefore lower than
/// [{MAX_VIEWER_FILE_SIZE}].
pub const MAX_VIEWER_TEXT_SIZE: u64 = 4_194_304;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limits_are_the_ones_the_specification_fixes() {
        // WHY: the specification writes these four numbers down. A change to
        // one of them is a decision that belongs into the specification
        // first, not into the code.
        assert_eq!(MAX_EXTRACTED_BYTES, 2_147_483_648);
        assert_eq!(MAX_ARCHIVE_ENTRIES, 100_000);
        assert_eq!(MAX_VIEWER_FILE_SIZE, 10_485_760);
        assert_eq!(MAX_VIEWER_TEXT_SIZE, 4_194_304);
    }

    #[test]
    fn text_limit_is_smaller_than_the_file_limit() {
        // WHY: text decoding multiplies the memory need of a file. A text
        // limit above the file limit would make the text limit unreachable.
        let limits = [MAX_VIEWER_TEXT_SIZE, MAX_VIEWER_FILE_SIZE];
        assert!(limits[0] < limits[1]);
    }

    #[test]
    fn every_limit_is_positive() {
        // WHY: a limit of zero would turn the feature it protects into a
        // permanent refusal. Zero is never a valid limit.
        let limits = [
            MAX_EXTRACTED_BYTES,
            MAX_ARCHIVE_ENTRIES,
            MAX_VIEWER_FILE_SIZE,
            MAX_VIEWER_TEXT_SIZE,
        ];
        for limit in limits {
            assert!(limit > 0);
        }
    }

    #[test]
    fn archive_limits_leave_room_for_a_normal_archive() {
        // WHY: the limits exist to stop attacks, not to inconvenience users.
        // A two gigabyte extraction and a hundred thousand entries are far
        // above what an ordinary photo or music archive needs.
        let limits = [MAX_EXTRACTED_BYTES, MAX_ARCHIVE_ENTRIES];
        assert!(limits[0] >= 2 * 1024 * 1024 * 1024);
        assert!(limits[1] >= 10_000);
    }
}
