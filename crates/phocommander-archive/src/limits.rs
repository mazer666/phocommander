//! The limits of an extraction.
//!
//! WHAT: the three limits that bound an extraction — the total number of
//!       bytes, the number of entries and the size of a single entry — and the
//!       permissions that the extraction gives to the files it writes.
//! WHY: an archive is untrusted input. Without the limits a small archive
//!       fills the disk of the machine it is unpacked on (a decompression
//!       bomb), and an archive that carries mode bits can hand a file with the
//!       set-user-id bit to a privileged process, which then runs it with the
//!       rights of the archive sender (RUSTSEC-2026-0367).
//!
//! The limits live here and nowhere else. The crate 'phocommander-core'
//!       cannot be a dependency of this crate, because no crate below
//!       {crates/} may depend on a sibling. The two numbers that the core
//!       crate also names are therefore written down a second time. The
//!       integration test of the package {tests} holds both writings against
//!       each other, so the duplication cannot drift apart unnoticed.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::ExtractionLimits;
//!
//! let limits = ExtractionLimits::default();
//! assert!(limits.allows_entry_size(1024));
//! assert!(!limits.allows_entry_size(limits.max_entry_bytes + 1));
//! ```

/// The largest total number of bytes that a single extraction may write.
///
/// WHY: without a total limit a small archive can fill the disk of the
///       machine it is unpacked on. The value is the one the specification
///       fixes in the {[archives]} section of the configuration file.
pub const MAX_EXTRACTED_BYTES: u64 = 2_147_483_648;

/// The largest number of entries that a single archive may contain.
///
/// WHY: an archive can declare millions of entries without occupying much
///       space. The limit keeps the extraction — and the listing of such an
///       archive — from exhausting the memory of the process.
pub const MAX_ARCHIVE_ENTRIES: u64 = 100_000;

/// The largest number of bytes that a single entry of an archive may have.
///
/// WHY: the total limit bounds the disk, but a single entry that is larger
///       than the total limit would be written in one piece and would fill
///       the disk before the total is noticed. The value is half of the total
///       budget: a full-length video in an archive can still be extracted,
///       and no single entry can ever occupy more than half of the space the
///       extraction may use at all.
pub const MAX_SINGLE_ENTRY_BYTES: u64 = 1_073_741_824;

/// The mode of a file that the extraction writes: 'rw-r--r--'.
///
/// WHY: the mode of an extracted file is decided by the application and not
///       by the archive. The set-user-id, the set-group-id and the sticky bit
///       are therefore never carried over, and the group and the other do not
///       get a write bit.
pub const EXTRACTED_FILE_MODE: u32 = 0o644;

/// The mode of a directory that the extraction creates: 'rwxr-xr-x'.
///
/// WHY: the same rule as for a file. The owner may enter the directory, the
///       group and the other may read it, nobody but the owner may write in
///       it.
pub const EXTRACTED_DIRECTORY_MODE: u32 = 0o755;

/// The limits of one extraction.
///
/// WHY: the limits are values and not constants, so that a caller can lower
///       them — the configuration file does exactly that — and so that a test
///       can exercise the mechanism with a budget that fits into a test run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractionLimits {
    /// The largest total number of bytes that the extraction may write.
    pub max_total_bytes: u64,

    /// The largest number of entries that the extraction may write.
    pub max_entries: u64,

    /// The largest number of bytes that a single entry may have.
    pub max_entry_bytes: u64,
}

impl Default for ExtractionLimits {
    /// Returns the limits of the specification.
    fn default() -> Self {
        Self {
            max_total_bytes: MAX_EXTRACTED_BYTES,
            max_entries: MAX_ARCHIVE_ENTRIES,
            max_entry_bytes: MAX_SINGLE_ENTRY_BYTES,
        }
    }
}

impl ExtractionLimits {
    /// Builds a set of limits from the three values.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::ExtractionLimits;
    ///
    /// let limits = ExtractionLimits::new(1024, 10, 512);
    /// assert_eq!(limits.max_total_bytes, 1024);
    /// ```
    pub const fn new(max_total_bytes: u64, max_entries: u64, max_entry_bytes: u64) -> Self {
        Self {
            max_total_bytes,
            max_entries,
            max_entry_bytes,
        }
    }

    /// Whether an entry of the given size may be written.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::ExtractionLimits;
    ///
    /// let limits = ExtractionLimits::new(1024, 10, 512);
    /// assert!(limits.allows_entry_size(512));
    /// assert!(!limits.allows_entry_size(513));
    /// ```
    pub const fn allows_entry_size(&self, size: u64) -> bool {
        size <= self.max_entry_bytes
    }

    /// Whether the given number of entries may be written.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::ExtractionLimits;
    ///
    /// let limits = ExtractionLimits::new(1024, 10, 512);
    /// assert!(limits.allows_entry_count(10));
    /// assert!(!limits.allows_entry_count(11));
    /// ```
    pub const fn allows_entry_count(&self, count: u64) -> bool {
        count <= self.max_entries
    }

    /// Whether the given number of bytes may be written in total.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::ExtractionLimits;
    ///
    /// let limits = ExtractionLimits::new(1024, 10, 512);
    /// assert!(limits.allows_total_size(1024));
    /// assert!(!limits.allows_total_size(1025));
    /// ```
    pub const fn allows_total_size(&self, bytes: u64) -> bool {
        bytes <= self.max_total_bytes
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn the_default_limits_are_the_ones_the_specification_fixes() {
        // act
        let limits = ExtractionLimits::default();

        // assert
        assert_eq!(limits.max_total_bytes, 2_147_483_648);
        assert_eq!(limits.max_entries, 100_000);
        assert_eq!(limits.max_entry_bytes, 1_073_741_824);
    }

    #[test]
    fn the_single_entry_limit_is_half_of_the_total_limit() {
        // WHY: the relation is the reason for the value. A single entry may
        // never occupy more than half of the space that the extraction may
        // use at all.
        // act
        let limits = ExtractionLimits::default();

        // assert
        assert_eq!(limits.max_entry_bytes * 2, limits.max_total_bytes);
    }

    #[test]
    fn the_permissions_carry_no_special_bit() {
        // WHY: the set-user-id, the set-group-id and the sticky bit are the
        // bits that RUSTSEC-2026-0367 is about.
        // assert
        assert_eq!(EXTRACTED_FILE_MODE & 0o7000, 0);
        assert_eq!(EXTRACTED_DIRECTORY_MODE & 0o7000, 0);
        assert_eq!(EXTRACTED_FILE_MODE & 0o022, 0);
        assert_eq!(EXTRACTED_DIRECTORY_MODE & 0o022, 0);
    }
}
