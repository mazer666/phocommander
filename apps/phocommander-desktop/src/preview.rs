//! The quick view of the desktop build.
//!
//! WHAT: the source that produces the content of the quick view
//! ([`ViewerPreviewSource`]).
//! WHY: the surface knows the quick view and not the reader of the viewer,
//! and the reader knows the file system and not the surface. The adapter
//! is the one place where the two meet: it opens the file with the reader,
//! recognises the character set of the text, and hands the lines to the
//! surface (FR-51, FR-52).
//! A file that is no text is shown as a hex dump — the offset, the bytes in
//! hexadecimal and the printable characters. That is the only way to show
//! what such a file really holds.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::preview::ViewerPreviewSource;
//! use phocommander_ui::preview::PreviewSource;
//! use phocommander_viewer::policy::ReadPolicy;
//! use phocommander_viewer::reader::ViewerReader;
//!
//! let source = ViewerPreviewSource::new(ViewerReader::new(ReadPolicy::strict(), 10_485_760));
//! assert!(source.preview(std::path::Path::new("/notes.txt")).is_none());
//! ```

use std::path::Path;

use phocommander_ui::preview::{Preview, PreviewSource};
use phocommander_viewer::content::ContentKind;
use phocommander_viewer::hex::HexDump;
use phocommander_viewer::reader::ViewerReader;

/// The number of the lines that the quick view shows.
const MAX_LINES: usize = 200;

/// The source of the quick view.
pub struct ViewerPreviewSource {
    /// The reader that opens the file.
    reader: ViewerReader,
}

impl ViewerPreviewSource {
    /// Builds the source out of the reader.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_desktop::preview::ViewerPreviewSource;
    /// use phocommander_viewer::policy::ReadPolicy;
    /// use phocommander_viewer::reader::ViewerReader;
    ///
    /// let source = ViewerPreviewSource::new(ViewerReader::new(ReadPolicy::strict(), 1024));
    /// let _ = source;
    /// ```
    pub fn new(reader: ViewerReader) -> Self {
        Self { reader }
    }
}

impl PreviewSource for ViewerPreviewSource {
    fn preview(&self, path: &Path) -> Option<Preview> {
        // WHY: the reader decides whether the file may be read at all. The
        //       rule "read_only_on_demand" refuses the reading of a file
        //       that the user has not opened, and the quick view honours it.
        let datei = self.reader.preview(path).ok()?;

        let (zeilen, binaer) = match datei.kind() {
            ContentKind::Text(zeichensatz) => {
                // WHY: the text of the file is decoded with the character set
                //       that was recognised, and the mark at the head of the
                //       file is not a character of the text.
                let text = zeichensatz.decode(datei.bytes())?;
                let zeilen: Vec<String> = text.lines().map(str::to_string).collect();
                (zeilen, false)
            }
            ContentKind::Binary => {
                // WHY: a file that is no text is shown as a hex dump. The
                //       dump shows the bytes and nothing else.
                let dump = HexDump::new(datei.bytes(), 0);
                (dump.to_lines(), true)
            }
        };

        Some(Preview::new(
            datei.path().to_path_buf(),
            datei.size(),
            zeilen.into_iter().take(MAX_LINES).collect(),
            binaer,
        ))
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use phocommander_viewer::policy::ReadPolicy;
    use phocommander_viewer::reader::ViewerReader;

    /// A temporary file that removes itself.
    struct TempFile {
        path: std::path::PathBuf,
    }

    impl TempFile {
        /// Writes the content into a file of its own.
        fn with_content(name: &str, content: &[u8]) -> Self {
            let pfad = std::env::temp_dir().join(name);
            std::fs::write(&pfad, content)
                .unwrap_or_else(|error| panic!("the file should have been written: {error}"));
            Self { path: pfad }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    /// A source that may read everything.
    fn source() -> ViewerPreviewSource {
        ViewerPreviewSource::new(ViewerReader::new(ReadPolicy::new(false), 1_048_576))
    }

    #[test]
    fn a_text_is_shown_with_its_lines() {
        // arrange
        let datei = TempFile::with_content("phocommander-preview-text.txt", b"hallo\nwelt");
        let source = source();

        // act
        let vorschau = source.preview(&datei.path);

        // assert
        // WHY: the quick view shows the text of the file — the lines and
        //       nothing else (FR-51).
        let vorschau = vorschau.unwrap_or_else(|| panic!("the file should have been read"));
        assert!(!vorschau.is_binary());
        assert_eq!(vorschau.lines(), &["hallo".to_string(), "welt".to_string()]);
        assert_eq!(vorschau.size(), 10);
    }

    #[test]
    fn a_text_of_utf_16_is_read_in_its_own_order() {
        // arrange
        // WHY: the German text with the umlauts is the case that the
        //       requirement names, and it comes from a machine that writes
        //       UTF-16.
        let inhalt: Vec<u8> = "Grüße"
            .encode_utf16()
            .flat_map(|w| w.to_le_bytes())
            .collect();
        let datei = TempFile::with_content("phocommander-preview-utf16.txt", &inhalt);
        let source = source();

        // act
        let vorschau = source.preview(&datei.path);

        // assert
        let vorschau = vorschau.unwrap_or_else(|| panic!("the file should have been read"));
        assert!(!vorschau.is_binary());
        assert_eq!(vorschau.lines(), &["Grüße".to_string()]);
    }

    #[test]
    fn a_file_that_is_no_text_is_shown_as_a_hex_dump() {
        // arrange
        let datei = TempFile::with_content(
            "phocommander-preview-binary.bin",
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x01],
        );
        let source = source();

        // act
        let vorschau = source.preview(&datei.path);

        // assert
        // WHY: the hex dump shows the bytes and nothing else (FR-52).
        let vorschau = vorschau.unwrap_or_else(|| panic!("the file should have been read"));
        assert!(vorschau.is_binary());
        assert_eq!(vorschau.lines().len(), 1);
        let zeile = &vorschau.lines()[0];
        assert!(zeile.contains("89 50 4e 47"), "{zeile}");
        assert!(zeile.starts_with("00000000"), "{zeile}");
    }

    #[test]
    fn a_file_that_does_not_exist_has_no_preview() {
        // arrange
        let source = source();

        // act, assert
        assert!(
            source
                .preview(std::path::Path::new("/gibt-es-nicht.txt"))
                .is_none()
        );
    }

    #[test]
    fn the_rule_that_the_file_may_only_be_read_on_demand_is_obeyed() {
        // arrange
        // WHY: the quick view reads a file that the user has not opened.
        //       The rule "read_only_on_demand" forbids it, and the quick
        //       view has to obey it.
        let datei = TempFile::with_content("phocommander-preview-refused.txt", b"hallo");
        let source = ViewerPreviewSource::new(ViewerReader::new(ReadPolicy::new(true), 1_048_576));

        // act, assert
        assert!(source.preview(&datei.path).is_none());
    }
}
