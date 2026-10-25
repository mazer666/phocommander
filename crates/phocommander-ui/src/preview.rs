//! The quick view of the marked file.
//!
//! WHAT: the content of a file as the quick view shows it ([`Preview`]), the
//!       source that produces it ([`PreviewSource`]) and the state of the
//!       quick view ([`QuickView`]).
//! WHY: the quick view shows the content of the marked file without opening
//!       it (FR-15). The interface crate may not depend on the viewer crate —
//!       no crate below `crates/` may depend on a sibling — so the seam is a
//!       trait, and the composition root fills it with the reader of the
//!       viewer. The quick view reads exactly one file — the marked one —
//!       and it reads it only when the marked file changes.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::preview::{Preview, QuickView};
//! use std::path::{Path, PathBuf};
//!
//! # struct Source;
//! # impl phocommander_ui::preview::PreviewSource for Source {
//! #     fn preview(&self, _path: &Path) -> Option<Preview> { None }
//! # }
//! let mut view = QuickView::new();
//! view.update(&Source, Some(Path::new("/notes.txt")));
//! assert!(view.is_empty());
//! ```

use std::path::{Path, PathBuf};

/// The content of a file, prepared for the quick view.
///
/// WHY: the quick view shows text. A file that is not text is shown as a
/// notice and not as a wall of replacement characters, because the user
/// wants to know that the file is binary and not read garbage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    path: PathBuf,
    size: u64,
    lines: Vec<String>,
    binary: bool,
}

impl Preview {
    /// The content of a file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::preview::Preview;
    /// use std::path::PathBuf;
    ///
    /// let preview = Preview::new(PathBuf::from("/notes.txt"), 5, vec!["hello".to_string()], false);
    /// assert_eq!(preview.lines(), &["hello".to_string()]);
    /// ```
    pub fn new(path: PathBuf, size: u64, lines: Vec<String>, binary: bool) -> Self {
        Self {
            path,
            size,
            lines,
            binary,
        }
    }

    /// The path of the file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The name of the file, without the path above it.
    pub fn name(&self) -> String {
        match self.path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => self.path.to_string_lossy().to_string(),
        }
    }

    /// The size of the file in bytes.
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// The lines of the file.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Whether the file is not text.
    pub const fn is_binary(&self) -> bool {
        self.binary
    }
}

/// The source that produces the content of the quick view.
///
/// WHY: the trait is the seam between the interface and the reader of the
/// viewer. The interface asks for one file and gets the content of that
/// file or nothing — it never reads a second file and never reads a file
/// that is not marked.
pub trait PreviewSource {
    /// The content of the file at the given path, if it can be read.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::preview::{Preview, PreviewSource};
    /// use std::path::Path;
    ///
    /// # struct Source;
    /// # impl PreviewSource for Source {
    /// #     fn preview(&self, _path: &Path) -> Option<Preview> { None }
    /// # }
    /// # let source = Source;
    /// assert!(source.preview(Path::new("/notes.txt")).is_none());
    /// ```
    fn preview(&self, path: &Path) -> Option<Preview>;
}

/// The state of the quick view.
///
/// WHY: the quick view remembers the file it shows. A file that is read in
/// every frame would cost a system call per frame and would slow the
/// interface down for no reason — the file only changes when the marked
/// entry changes.
#[derive(Debug)]
pub struct QuickView {
    visible: bool,
    path: Option<PathBuf>,
    preview: Option<Preview>,
    message: Option<String>,
}

impl Default for QuickView {
    /// A quick view that is shown and that shows nothing yet.
    ///
    /// WHY: the quick view is an area of the layout of the desktop, and the
    /// layout decides that it exists. The user switches it off with the
    /// action of the catalogue.
    fn default() -> Self {
        Self {
            visible: true,
            path: None,
            preview: None,
            message: None,
        }
    }
}

impl QuickView {
    /// A quick view that is shown and that shows nothing yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the quick view is shown.
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Shows or hides the quick view.
    pub fn toggle(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }

    /// The content that is shown.
    pub fn preview(&self) -> Option<&Preview> {
        self.preview.as_ref()
    }

    /// The message that is shown instead of the content.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Whether the quick view shows nothing.
    pub fn is_empty(&self) -> bool {
        self.preview.is_none() && self.message.is_none()
    }

    /// The path of the file that is shown.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Reads the content of the given file and shows it.
    ///
    /// WHY: the file is read only when the marked file changes. The path
    /// that is shown is remembered, and a second call with the same path
    /// does nothing — the quick view therefore costs one read per marked
    /// file and not one read per frame (FR-15, NFR-09).
    pub fn update(&mut self, source: &dyn PreviewSource, path: Option<&Path>) {
        let wanted = path.map(Path::to_path_buf);
        if wanted == self.path && (self.preview.is_some() || self.message.is_some()) {
            return;
        }
        self.path = wanted.clone();

        let Some(path) = wanted else {
            self.preview = None;
            self.message = None;
            return;
        };

        match source.preview(&path) {
            Some(preview) => {
                self.preview = Some(preview);
                self.message = None;
            }
            None => {
                self.preview = None;
                self.message = Some(path.to_string_lossy().to_string());
            }
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::cell::Cell;

    /// A source that counts how often it was asked.
    struct Counting {
        reads: Cell<usize>,
    }

    impl Counting {
        fn new() -> Self {
            Self {
                reads: Cell::new(0),
            }
        }
    }

    impl PreviewSource for Counting {
        fn preview(&self, path: &Path) -> Option<Preview> {
            self.reads.set(self.reads.get() + 1);
            Some(Preview::new(
                path.to_path_buf(),
                5,
                vec!["hello".to_string()],
                false,
            ))
        }
    }

    #[test]
    fn the_quick_view_reads_the_file_only_once() {
        // arrange
        let source = Counting::new();
        let mut view = QuickView::new();

        // act
        view.update(&source, Some(Path::new("/notes.txt")));
        view.update(&source, Some(Path::new("/notes.txt")));

        // assert
        assert_eq!(source.reads.get(), 1);
        assert!(view.preview().is_some());
    }

    #[test]
    fn the_quick_view_reads_the_file_again_when_the_marked_file_changes() {
        // arrange
        let source = Counting::new();
        let mut view = QuickView::new();

        // act
        view.update(&source, Some(Path::new("/notes.txt")));
        view.update(&source, Some(Path::new("/other.txt")));

        // assert
        assert_eq!(source.reads.get(), 2);
        assert_eq!(view.path(), Some(Path::new("/other.txt")));
    }

    #[test]
    fn the_quick_view_forgets_the_content_when_nothing_is_marked() {
        // arrange
        let source = Counting::new();
        let mut view = QuickView::new();
        view.update(&source, Some(Path::new("/notes.txt")));

        // act
        view.update(&source, None);

        // assert
        assert!(view.is_empty());
        assert!(view.path().is_none());
    }
}
