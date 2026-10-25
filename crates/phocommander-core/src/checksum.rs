//! The checksum that the verified copy compares.
//!
//! WHAT: a CRC-32 checksum ([{Crc32}]) that is computed over a stream of
//!       bytes ([{crc32}]), over a slice ([{crc32_bytes}]) or over a file
//!       ([{crc32_file}]).
//! WHY: the copy modes of the specification compare the source and the
//!       destination after a copy, and the comparison has to be cheap enough
//!       to run on every file and strong enough to notice a truncated or a
//!       garbled write. A CRC-32 is not a cryptographic hash and is not
//!       meant to be one: it protects against the accident of a bad write,
//!       not against an attacker who chooses the content. The checksum is
//!       computed over a *stream*, so that the memory use does not depend on
//!       the size of the file.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::checksum::{Crc32, crc32_bytes};
//!
//! let mut checksum = Crc32::new();
//! checksum.update(b"phocommander");
//! assert_eq!(checksum.finish(), crc32_bytes(b"phocommander"));
//! ```

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::OnceLock;

/// The size of the block that is read from a file.
const READ_BLOCK_SIZE: usize = 64 * 1024;

/// The table of the CRC-32 polynomial.
fn table() -> &'static [u32; 256] {
    static TABLE: OnceLock<[u32; 256]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u32; 256];
        for (index, entry) in table.iter_mut().enumerate() {
            let mut value = index as u32;
            for _ in 0..8 {
                value = if value & 1 == 1 {
                    0xEDB8_8320 ^ (value >> 1)
                } else {
                    value >> 1
                };
            }
            *entry = value;
        }
        table
    })
}

/// A running CRC-32 checksum.
///
/// WHY: the checksum is fed block by block, because a file may be larger
/// than the memory the application may use. The state is kept between the
/// blocks, so that the result is the same as if the whole file had been
/// read at once.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::checksum::Crc32;
///
/// let mut first = Crc32::new();
/// first.update(b"one ");
/// first.update(b"two");
///
/// let mut second = Crc32::new();
/// second.update(b"one two");
///
/// assert_eq!(first.finish(), second.finish());
/// ```
#[derive(Debug, Clone)]
pub struct Crc32 {
    state: u32,
}

impl Crc32 {
    /// Starts a new checksum.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::checksum::Crc32;
    ///
    /// let checksum = Crc32::new();
    /// assert_ne!(checksum.finish(), 0);
    /// ```
    pub fn new() -> Self {
        Self { state: 0xFFFF_FFFF }
    }

    /// Adds a block of bytes to the checksum.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::checksum::Crc32;
    ///
    /// let mut checksum = Crc32::new();
    /// checksum.update(b"data");
    /// ```
    pub fn update(&mut self, bytes: &[u8]) {
        let table = table();
        let mut state = self.state;
        for byte in bytes {
            let index = ((state ^ u32::from(*byte)) & 0xFF) as usize;
            state = table[index] ^ (state >> 8);
        }
        self.state = state;
    }

    /// The checksum of everything that was added so far.
    ///
    /// WHY: the value can be read without consuming the checksum, so that
    /// the caller can compare an intermediate state and continue afterwards.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::checksum::Crc32;
    ///
    /// let mut checksum = Crc32::new();
    /// checksum.update(b"data");
    /// let first = checksum.finish();
    /// assert_eq!(first, checksum.finish());
    /// ```
    pub fn finish(&self) -> u32 {
        self.state ^ 0xFFFF_FFFF
    }

    /// Starts over with an empty checksum.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::checksum::Crc32;
    ///
    /// let mut checksum = Crc32::new();
    /// checksum.update(b"data");
    /// checksum.reset();
    /// assert_eq!(checksum.finish(), Crc32::new().finish());
    /// ```
    pub fn reset(&mut self) {
        self.state = 0xFFFF_FFFF;
    }
}

impl Default for Crc32 {
    /// Starts a new checksum.
    ///
    /// WHY: the type has a natural zero value — the checksum of no bytes at
    /// all — so that a caller does not have to know about the initial state
    /// of the algorithm.
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the checksum of a stream of bytes.
///
/// WHY: the function reads the stream in blocks, so that the memory use does
/// not depend on the size of the stream. This is what the copy operations
/// use to verify a file they have just written.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::checksum::crc32;
///
/// let mut data = b"phocommander".as_slice();
/// let checksum = crc32(&mut data)?;
/// assert_ne!(checksum, 0);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn crc32(reader: &mut impl Read) -> io::Result<u32> {
    let mut checksum = Crc32::new();
    let mut buffer = [0u8; READ_BLOCK_SIZE];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        checksum.update(&buffer[..read]);
    }
    Ok(checksum.finish())
}

/// Computes the checksum of a slice of bytes.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::checksum::crc32_bytes;
///
/// assert_eq!(crc32_bytes(b""), crc32_bytes(b""));
/// assert_ne!(crc32_bytes(b"a"), crc32_bytes(b"b"));
/// ```
pub fn crc32_bytes(bytes: &[u8]) -> u32 {
    let mut checksum = Crc32::new();
    checksum.update(bytes);
    checksum.finish()
}

/// Computes the checksum of a file.
///
/// WHY: the file is read in blocks, so that the checksum of a large file
/// costs no more memory than the checksum of a small one.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::checksum::crc32_file;
/// use std::path::Path;
///
/// # fn main() -> Result<(), std::io::Error> {
/// let checksum = crc32_file(Path::new("notes.txt"))?;
/// println!("{checksum:08x}");
/// # Ok(())
/// # }
/// ```
pub fn crc32_file(path: &Path) -> io::Result<u32> {
    let mut file = File::open(path)?;
    crc32(&mut file)
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn the_checksum_of_an_empty_input_is_known() {
        // act
        let checksum = crc32_bytes(b"");

        // assert
        // The CRC-32 of no bytes at all is the well known value 0x00000000.
        assert_eq!(checksum, 0x0000_0000);
    }

    #[test]
    fn the_checksum_of_the_reference_text_is_known() {
        // act
        let checksum = crc32_bytes(b"123456789");

        // assert
        // The check value of the CRC-32 (IEEE 802.3) for the text
        // "123456789" is 0xCBF43926.
        assert_eq!(checksum, 0xCBF4_3926);
    }

    #[test]
    fn the_checksum_does_not_depend_on_the_size_of_the_blocks() {
        // arrange
        let data: Vec<u8> = (0..=255u8).cycle().take(5000).collect();

        // act
        let mut whole = Crc32::new();
        whole.update(&data);
        let mut piecewise = Crc32::new();
        for block in data.chunks(7) {
            piecewise.update(block);
        }

        // assert
        assert_eq!(whole.finish(), piecewise.finish());
    }

    #[test]
    fn a_changed_byte_changes_the_checksum() {
        // act
        let original = crc32_bytes(b"phocommander");
        let changed = crc32_bytes(b"phocommandeR");

        // assert
        assert_ne!(original, changed);
    }

    #[test]
    fn the_checksum_of_a_file_is_the_checksum_of_its_content() {
        // arrange
        let path = std::env::temp_dir().join("phocommander-checksum.txt");
        fs::write(&path, b"123456789").unwrap_or_else(|error| {
            panic!("the file {path:?} could not be written: {error}");
        });

        // act
        let checksum = crc32_file(&path);

        // assert
        let checksum = match checksum {
            Ok(checksum) => checksum,
            Err(error) => panic!("the checksum could not be computed: {error}"),
        };
        assert_eq!(checksum, 0xCBF4_3926);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn the_checksum_of_a_missing_file_is_an_error() {
        // act
        let result = crc32_file(&PathBuf::from("/does/not/exist/at/all"));

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn a_reset_checksum_forgets_what_was_added() {
        // act
        let mut checksum = Crc32::new();
        checksum.update(b"data");
        checksum.reset();

        // assert
        assert_eq!(checksum.finish(), Crc32::new().finish());
    }

    #[test]
    fn the_default_checksum_is_a_new_checksum() {
        // act
        let checksum = Crc32::default();

        // assert
        assert_eq!(checksum.finish(), Crc32::new().finish());
    }
}
