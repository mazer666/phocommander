//! The hex dump of a file.
//!
//! WHAT: the hex dump of bytes ([`HexDump`]) — the offset, the bytes in
//! hexadecimal and the printable characters.
//! WHY: a file that is no text has to be shown somehow, and the hex dump
//! is the way that every developer knows. It shows what the file really
//! holds — the bytes and nothing else (FR-52).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_viewer::hex::HexDump;
//!
//! let dump = HexDump::new(b"hello", 0);
//! assert_eq!(dump.lines().len(), 1);
//! assert!(dump.lines()[0].hex.contains("68 65 6c 6c 6f"));
//! ```

/// The number of the bytes in one line of the dump.
pub const BYTES_PER_LINE: usize = 16;

/// One line of a hex dump.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexLine {
    /// The offset of the first byte of the line.
    pub offset: u64,

    /// The bytes of the line, in hexadecimal.
    pub hex: String,

    /// The printable characters of the line.
    pub text: String,
}

impl HexLine {
    /// The line as it is shown to the user.
    ///
    /// WHY: the three parts have their own column, so that the eye can
    ///       find the offset of a byte and the character behind it without
    ///       counting.
    pub fn to_line(&self) -> String {
        format!("{:08x}  {:<47}  |{}|", self.offset, self.hex, self.text)
    }
}

/// The hex dump of a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexDump {
    /// The lines of the dump.
    lines: Vec<HexLine>,

    /// The number of the bytes that the dump shows.
    shown: usize,

    /// The number of the bytes that the file holds.
    total: usize,
}

impl HexDump {
    /// Builds the dump of the bytes, starting at the given offset.
    ///
    /// WHY: the offset of the first line is the offset of the file, and not
    ///       always zero — the viewer shows a part of a big file, and the
    ///       offset tells the user where in the file he is.
    pub fn new(bytes: &[u8], start: u64) -> Self {
        let mut lines = Vec::with_capacity(bytes.len() / BYTES_PER_LINE + 1);
        let mut versatz = start;

        for stueck in bytes.chunks(BYTES_PER_LINE) {
            let mut hex = String::with_capacity(BYTES_PER_LINE * 3);
            let mut text = String::with_capacity(BYTES_PER_LINE);

            for (nummer, byte) in stueck.iter().enumerate() {
                if nummer == BYTES_PER_LINE / 2 {
                    // WHY: the middle of the line is separated, so that the
                    //       eye can count the eight bytes of a half.
                    hex.push(' ');
                }
                hex.push_str(&format!("{byte:02x} "));
                // WHY: the printable characters are the American ones, and
                //       every other byte becomes a point. A text that is
                //       not American would show nothing at all.
                text.push(if (0x20..0x7F).contains(byte) {
                    *byte as char
                } else {
                    '.'
                });
            }

            lines.push(HexLine {
                offset: versatz,
                hex: hex.trim_end().to_string(),
                text,
            });
            versatz += stueck.len() as u64;
        }

        Self {
            lines,
            shown: bytes.len(),
            total: bytes.len(),
        }
    }

    /// The lines of the dump.
    pub fn lines(&self) -> &[HexLine] {
        &self.lines
    }

    /// The number of the bytes that the dump shows.
    pub fn shown(&self) -> usize {
        self.shown
    }

    /// The number of the bytes that the file holds.
    pub fn total(&self) -> usize {
        self.total
    }

    /// The dump as the lines of text.
    pub fn to_lines(&self) -> Vec<String> {
        self.lines.iter().map(HexLine::to_line).collect()
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn a_short_text_takes_one_line() {
        // act
        let dump = HexDump::new(b"hello", 0);

        // assert
        assert_eq!(dump.lines().len(), 1);
        assert_eq!(dump.lines()[0].offset, 0);
        assert!(dump.lines()[0].hex.contains("68 65 6c 6c 6f"));
        assert_eq!(dump.lines()[0].text, "hello");
    }

    #[test]
    fn a_byte_that_is_not_printable_becomes_a_point() {
        // act
        let dump = HexDump::new(&[0x00, 0x41, 0xFF], 0);

        // assert
        // WHY: the printable characters are the American ones, and every
        //       other byte becomes a point.
        assert_eq!(dump.lines()[0].text, ".A.");
        assert!(dump.lines()[0].hex.contains("00 41 ff"));
    }

    #[test]
    fn sixteen_bytes_take_one_line_and_seventeen_take_two() {
        // act
        let sechzehn = HexDump::new(&[0x41; 16], 0);
        let siebzehn = HexDump::new(&[0x41; 17], 0);

        // assert
        assert_eq!(sechzehn.lines().len(), 1);
        assert_eq!(siebzehn.lines().len(), 2);
    }

    #[test]
    fn the_offset_of_the_second_line_follows_the_first() {
        // act
        let dump = HexDump::new(&[0x41; 32], 0x100);

        // assert
        assert_eq!(dump.lines()[0].offset, 0x100);
        assert_eq!(dump.lines()[1].offset, 0x110);
    }

    #[test]
    fn a_file_without_bytes_has_no_line() {
        // act
        let dump = HexDump::new(b"", 0);

        // assert
        assert!(dump.lines().is_empty());
        assert_eq!(dump.shown(), 0);
    }

    #[test]
    fn the_line_shows_the_offset_the_bytes_and_the_characters() {
        // act
        let dump = HexDump::new(b"AB", 0);

        // assert
        // WHY: the three parts have their own column, so that the eye can
        //       find the offset of a byte and the character behind it.
        let zeile = dump.to_lines()[0].clone();
        assert!(zeile.starts_with("00000000"), "{zeile}");
        assert!(zeile.contains("41 42"), "{zeile}");
        assert!(zeile.ends_with("|AB|"), "{zeile}");
    }
}
