//! The checksum of the integrity check of an archive.
//!
//! WHAT: the CRC-32 of the container formats ([{Crc32}]).
//! WHY: the ZIP format and the 7z format store a CRC-32 with every entry,
//!       and the integrity check of FR-71 compares the checksum of the entry
//!       with the value the container stores. The algorithm is the one of the
//!       IEEE 802.3 — the same one that the container crates use — and it is
//!       written down here, so that the check does not depend on a crate
//!       that only exists as a dependency of a dependency.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::crc32::Crc32;
//!
//! let mut checksum = Crc32::new();
//! checksum.update(b"123456789");
//! assert_eq!(checksum.finish(), 0xcbf4_3926);
//! ```
//! BOUNDARY: this module computes a checksum of bytes. It reads no file and
//!       writes no file.

/// The polynomial of the CRC-32, in its reversed form.
const POLYNOMIAL: u32 = 0xedb8_8320;

/// The checksum of a stream of bytes.
///
/// WHY: the checksum is computed while the content is read, so that the
///       integrity check never has to hold a whole entry in the memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc32 {
    state: u32,
}

impl Crc32 {
    /// Builds an empty checksum.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::crc32::Crc32;
    ///
    /// let checksum = Crc32::new();
    /// assert_eq!(checksum.finish(), 0);
    /// ```
    pub const fn new() -> Self {
        Self { state: 0xffff_ffff }
    }

    /// Adds bytes to the checksum.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::crc32::Crc32;
    ///
    /// let mut checksum = Crc32::new();
    /// checksum.update(b"1234");
    /// checksum.update(b"56789");
    /// assert_eq!(checksum.finish(), 0xcbf4_3926);
    /// ```
    pub fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state ^= u32::from(*byte);
            for _ in 0..8 {
                // WHY: the shift and the polynomial are chosen by the lowest
                // bit of the state. The mask carries the choice into the
                // polynomial without a branch.
                let mask = 0u32.wrapping_sub(self.state & 1);
                self.state = (self.state >> 1) ^ (POLYNOMIAL & mask);
            }
        }
    }

    /// The checksum of the bytes that were added.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::crc32::Crc32;
    ///
    /// let mut checksum = Crc32::new();
    /// checksum.update(b"");
    /// assert_eq!(checksum.finish(), 0);
    /// ```
    pub const fn finish(&self) -> u32 {
        !self.state
    }
}

impl Default for Crc32 {
    /// Builds an empty checksum.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_archive::crc32::Crc32;
    ///
    /// assert_eq!(Crc32::default().finish(), 0);
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_checksum_of_the_test_vector_is_the_one_of_the_standard() {
        // act
        let mut checksum = Crc32::new();
        checksum.update(b"123456789");

        // assert
        assert_eq!(checksum.finish(), 0xcbf4_3926);
    }

    #[test]
    fn the_checksum_of_an_empty_input_is_zero() {
        // act, assert
        assert_eq!(Crc32::new().finish(), 0);
    }

    #[test]
    fn the_checksum_does_not_depend_on_the_chunks_it_is_given_in() {
        // act
        let mut whole = Crc32::new();
        whole.update(b"123456789");
        let mut parts = Crc32::new();
        parts.update(b"123");
        parts.update(b"456");
        parts.update(b"789");

        // assert
        assert_eq!(whole.finish(), parts.finish());
    }

    #[test]
    fn a_changed_byte_changes_the_checksum() {
        // act
        let mut first = Crc32::new();
        first.update(b"123456789");
        let mut second = Crc32::new();
        second.update(b"123456780");

        // assert
        assert_ne!(first.finish(), second.finish());
    }
}
