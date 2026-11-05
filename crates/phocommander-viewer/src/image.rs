//! The image formats that the viewer knows.
//!
//! WHAT: the formats of the images that the viewer shows ([`ImageFormat`])
//!       and the recognition of them ([`ImageFormat::detect`]).
//! WHY: an image has no name that says what it is. The extension of the
//! file lies often enough — a file that is called `.png` may hold
//! something else — and the viewer would then show nonsense. The format is
//! therefore recognised by its mark, the bytes at the head of the file
//! that the writer of the format has put there on purpose (FR-53).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_viewer::image::ImageFormat;
//!
//! let format = ImageFormat::detect(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
//! assert_eq!(format, Some(ImageFormat::Png));
//! ```

/// The format of an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageFormat {
    /// The Portable Network Graphics.
    Png,

    /// The Joint Photographic Experts Group.
    Jpeg,

    /// The Graphics Interchange Format.
    Gif,

    /// The Windows bitmap.
    Bmp,

    /// The WebP of the Google.
    WebP,
}

impl ImageFormat {
    /// Every format, in a stable order.
    pub const fn all() -> [ImageFormat; 5] {
        [
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::Gif,
            ImageFormat::Bmp,
            ImageFormat::WebP,
        ]
    }

    /// The name of the format, as it is shown to the user.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Gif => "GIF",
            Self::Bmp => "BMP",
            Self::WebP => "WebP",
        }
    }

    /// The extension of a file of the format.
    ///
    /// WHY: the framework of the interface needs the extension to know how
    ///       the bytes are to be read. The extension is the name of the
    ///       format and nothing else.
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::WebP => "webp",
        }
    }

    /// The mark at the head of a file of the format.
    ///
    /// WHY: the marks are the bytes that the writer of the format puts at
    ///       the head of every file. They are the only sign that does not
    ///       lie — the name of the file can be anything.
    pub const fn mark(&self) -> &'static [u8] {
        match self {
            Self::Png => &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            Self::Jpeg => &[0xFF, 0xD8, 0xFF],
            Self::Gif => b"GIF8",
            Self::Bmp => b"BM",
            Self::WebP => b"RIFF",
        }
    }

    /// The number of the bytes that the mark needs.
    ///
    /// WHY: the mark of the RIFF format says nothing about the format
    ///       inside it — the name of the format comes four bytes later.
    ///       The reader therefore has to look at more bytes than the mark
    ///       alone.
    pub const fn mark_length(&self) -> usize {
        match self {
            Self::WebP => 12,
            _ => self.mark().len(),
        }
    }

    /// Recognises the format of the bytes.
    ///
    /// WHY: the bytes that are not an image are no image. The function
    ///       answers with `None` and not with a guess — a file that is
    ///       shown as a picture and is none shows the user something that
    ///       is not there.
    pub fn detect(bytes: &[u8]) -> Option<ImageFormat> {
        for format in ImageFormat::all() {
            let laenge = format.mark_length();
            if bytes.len() < laenge {
                continue;
            }
            let passt = match format {
                // WHY: the RIFF format carries the name of the format
                //       inside it, four bytes after the mark.
                ImageFormat::WebP => bytes.starts_with(format.mark()) && &bytes[8..12] == b"WEBP",
                _ => bytes.starts_with(format.mark()),
            };
            if passt {
                return Some(format);
            }
        }
        None
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    /// The bytes that begin a file of the format.
    fn mark_of(format: ImageFormat) -> Vec<u8> {
        let mut bytes = format.mark().to_vec();
        if format == ImageFormat::WebP {
            bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
            bytes.extend_from_slice(b"WEBP");
        }
        bytes
    }

    #[test]
    fn every_format_is_recognised_by_its_mark() {
        // act, assert
        // WHY: the requirement names a test per format.
        for format in ImageFormat::all() {
            let bytes = mark_of(format);
            assert_eq!(
                ImageFormat::detect(&bytes),
                Some(format),
                "the format {format:?} should be recognised"
            );
        }
    }

    #[test]
    fn a_riff_file_that_is_no_web_picture_is_no_picture() {
        // arrange
        // WHY: the RIFF container carries many formats — a sound file, a
        //       video. The name inside it decides, and a container without
        //       the name is no picture.
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        bytes.extend_from_slice(b"WAVE");

        // act, assert
        assert_eq!(ImageFormat::detect(&bytes), None);
    }

    #[test]
    fn the_bytes_that_are_no_picture_are_no_picture() {
        // act, assert
        assert_eq!(ImageFormat::detect(b"hello, world"), None);
        assert_eq!(ImageFormat::detect(b""), None);
        // WHY: a file whose head is cut off cannot be recognised, and a
        //       guess would be wrong.
        assert_eq!(ImageFormat::detect(&[0x89, 0x50]), None);
    }

    #[test]
    fn the_name_of_the_format_is_the_name_that_the_world_knows() {
        // act, assert
        assert_eq!(ImageFormat::Png.name(), "PNG");
        assert_eq!(ImageFormat::Jpeg.name(), "JPEG");
        assert_eq!(ImageFormat::Gif.name(), "GIF");
        assert_eq!(ImageFormat::Bmp.name(), "BMP");
        assert_eq!(ImageFormat::WebP.name(), "WebP");
    }
}
