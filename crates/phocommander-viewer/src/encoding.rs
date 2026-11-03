//! The character set of a text.
//!
//! WHAT: the character set of a text ([`Encoding`]) and the detection of it
//! ([`Encoding::detect`]).
//! WHY: a file has no character set of its own. The bytes of a text file
//! are the same, whether they were written on a machine that writes UTF-8
//! or on one that writes UTF-16. The viewer therefore has to recognise the
//! character set before it shows the text — otherwise the German umlauts of
//! a text from another machine appear as nonsense (FR-51).
//! The recognition uses three signs, in this order: the byte order mark,
//! which the writer of the file has put there on purpose; the null bytes,
//! because a text of ASCII characters in UTF-16 is also valid UTF-8; and
//! the validity of the bytes as UTF-8, because the format of UTF-8 is
//! strict.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_viewer::encoding::Encoding;
//!
//! let encoding = Encoding::detect("Grüße".as_bytes());
//! assert_eq!(encoding, Encoding::Utf8);
//! ```

/// The character set of a text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Encoding {
    /// The text is UTF-8.
    #[default]
    Utf8,

    /// The text is UTF-16 in the little-endian order.
    Utf16Le,

    /// The text is UTF-16 in the big-endian order.
    Utf16Be,

    /// The bytes are no text.
    Binary,
}

impl Encoding {
    /// The byte order mark of the character set, if it has one.
    ///
    /// WHY: the mark tells the reader of the file which character set the
    ///       file has. The viewer strips it, because it is not a character
    ///       of the text.
    pub const fn byte_order_mark(&self) -> &'static [u8] {
        match self {
            Self::Utf8 => &[0xEF, 0xBB, 0xBF],
            Self::Utf16Le => &[0xFF, 0xFE],
            Self::Utf16Be => &[0xFE, 0xFF],
            Self::Binary => &[],
        }
    }

    /// The name of the character set, as it is shown to the user.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf16Le => "UTF-16 LE",
            Self::Utf16Be => "UTF-16 BE",
            Self::Binary => "binary",
        }
    }

    /// Recognises the character set of the bytes.
    ///
    /// WHY: the byte order mark comes first, because it is the only sign
    ///       that the writer of the file has put there on purpose. After
    ///       that come the null bytes, because a text of ASCII characters in
    ///       UTF-16 is also valid UTF-8 and the check for UTF-8 would
    ///       therefore claim it. What is left is the check for UTF-8, because
    ///       the format of UTF-8 is strict: a byte sequence that is not
    ///       valid UTF-8 is no text in UTF-8. The recognition errs on the
    ///       side of the text: a file that is shown as nonsense is better
    ///       than a text that is refused.
    pub fn detect(bytes: &[u8]) -> Encoding {
        if bytes.starts_with(Self::Utf8.byte_order_mark()) {
            return Encoding::Utf8;
        }
        if bytes.starts_with(Self::Utf16Le.byte_order_mark()) {
            return Encoding::Utf16Le;
        }
        if bytes.starts_with(Self::Utf16Be.byte_order_mark()) {
            return Encoding::Utf16Be;
        }
        // WHY: the null bytes come before the check for UTF-8. A text of
        //       ASCII characters in UTF-16 is also valid UTF-8 — the null
        //       bytes are valid characters there — and the check for UTF-8
        //       would therefore claim every such text. The null bytes are
        //       the stronger sign.
        if bytes.len() >= 2 {
            let paare = bytes.as_chunks::<2>().0;
            let ungerade_nullen = paare.iter().filter(|paar| paar[1] == 0).count();
            let gerade_nullen = paare.iter().filter(|paar| paar[0] == 0).count();
            let paare = paare.len();
            // WHY: a UTF-16 text in the little-endian order has a null byte
            //       behind every byte of an ASCII character. The text in the
            //       big-endian order has it in front. A text that carries
            //       null bytes in only a few places is a UTF-8 text with
            //       null bytes in it, and the check does not claim it.
            if paare > 0 && ungerade_nullen * 2 >= paare {
                return Encoding::Utf16Le;
            }
            if paare > 0 && gerade_nullen * 2 >= paare {
                return Encoding::Utf16Be;
            }
        }
        if std::str::from_utf8(bytes).is_ok() {
            return Encoding::Utf8;
        }
        Encoding::Binary
    }

    /// The text of the bytes, if the bytes carry a text.
    ///
    /// WHY: the byte order mark is not a character of the text, and it is
    ///       therefore taken away before the text is read.
    pub fn decode(&self, bytes: &[u8]) -> Option<String> {
        let inhalt =
            if bytes.starts_with(self.byte_order_mark()) && !self.byte_order_mark().is_empty() {
                &bytes[self.byte_order_mark().len()..]
            } else {
                bytes
            };
        match self {
            Self::Utf8 => std::str::from_utf8(inhalt).ok().map(str::to_string),
            Self::Utf16Le => {
                let paare: Vec<u16> = inhalt
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|paar| u16::from_le_bytes([paar[0], paar[1]]))
                    .collect();
                String::from_utf16(&paare).ok()
            }
            Self::Utf16Be => {
                let paare: Vec<u16> = inhalt
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|paar| u16::from_be_bytes([paar[0], paar[1]]))
                    .collect();
                String::from_utf16(&paare).ok()
            }
            Self::Binary => None,
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn a_text_with_the_mark_of_utf_8_is_utf_8() {
        // act, assert
        let bytes = [
            Encoding::Utf8.byte_order_mark(),
            b"Gr",
            b"\xc3\xbc",
            b"\xc3\x9f",
            b"e",
        ]
        .concat();
        assert_eq!(Encoding::detect(&bytes), Encoding::Utf8);
    }

    #[test]
    fn a_text_with_the_mark_of_utf_16_is_utf_16() {
        // act, assert
        let bytes = [Encoding::Utf16Le.byte_order_mark(), b"h\0i\0"].concat();
        assert_eq!(Encoding::detect(&bytes), Encoding::Utf16Le);
        let bytes = [Encoding::Utf16Be.byte_order_mark(), b"\0h\0i"].concat();
        assert_eq!(Encoding::detect(&bytes), Encoding::Utf16Be);
    }

    #[test]
    fn a_text_without_a_mark_is_utf_8_when_it_can_be_one() {
        // act, assert
        // WHY: the German text with the umlauts — the case the
        //       requirement names.
        assert_eq!(Encoding::detect("Grüße".as_bytes()), Encoding::Utf8);
    }

    #[test]
    fn a_text_of_utf_16_without_a_mark_is_recognised_by_its_null_bytes() {
        // act, assert
        let little: Vec<u8> = "hallo"
            .encode_utf16()
            .flat_map(|w| w.to_le_bytes())
            .collect();
        assert_eq!(Encoding::detect(&little), Encoding::Utf16Le);
        let gross: Vec<u8> = "hallo"
            .encode_utf16()
            .flat_map(|w| w.to_be_bytes())
            .collect();
        assert_eq!(Encoding::detect(&gross), Encoding::Utf16Be);
    }

    #[test]
    fn bytes_that_are_no_text_are_binary() {
        // act, assert
        // WHY: the bytes of an image or of a sound file are no text, and
        //       the viewer shows them as a hex dump.
        assert_eq!(
            Encoding::detect(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            Encoding::Binary
        );
    }

    #[test]
    fn the_text_of_utf_8_is_read_without_the_mark() {
        // arrange
        let bytes = [Encoding::Utf8.byte_order_mark(), "Grüße".as_bytes()].concat();

        // act
        let text = Encoding::Utf8.decode(&bytes);

        // assert
        // WHY: the mark is not a character of the text.
        assert_eq!(text.as_deref(), Some("Grüße"));
    }

    #[test]
    fn the_text_of_utf_16_is_read_in_both_orders() {
        // arrange
        let little: Vec<u8> = "Grüße"
            .encode_utf16()
            .flat_map(|w| w.to_le_bytes())
            .collect();
        let gross: Vec<u8> = "Grüße"
            .encode_utf16()
            .flat_map(|w| w.to_be_bytes())
            .collect();

        // act, assert
        assert_eq!(Encoding::Utf16Le.decode(&little).as_deref(), Some("Grüße"));
        assert_eq!(Encoding::Utf16Be.decode(&gross).as_deref(), Some("Grüße"));
    }

    #[test]
    fn the_bytes_that_are_no_text_have_no_text() {
        // act, assert
        assert_eq!(Encoding::Binary.decode(&[0x00, 0x01]), None);
    }
}
