//! The place in an archive that a tab shows.
//!
//! WHAT: the view of a tab that shows the inside of an archive
//!       ([`ArchiveView`]).
//! WHY: an archive appears in the listing like every other entry, and
//!       opening it shows its content in the pane — a **virtual
//!       directory** (FR-65). The pane does not know where the entries
//!       come from: the tab remembers the place inside the archive, and
//!       the shell reads the entries and hands them over (FR-66).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::ArchiveView;
//! use std::path::PathBuf;
//!
//! let view = ArchiveView::new(PathBuf::from("/photos/archiv.zip"));
//! assert_eq!(view.archive(), std::path::Path::new("/photos/archiv.zip"));
//! assert!(view.at_root());
//! ```

use std::path::{Path, PathBuf};

/// The place inside an archive that a tab shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveView {
    /// The path of the archive on the disk.
    archive: PathBuf,

    /// The path inside the archive, with the separator of the system.
    ///
    /// WHY: the path inside the archive is the path that the container
    ///       carries — the name of the entry and the names of the
    ///       directories above it, joined by a slash. It is not a path of
    ///       the disk, and it must not be resolved against one.
    inside: String,
}

impl ArchiveView {
    /// The root of an archive.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::panel::ArchiveView;
    /// use std::path::PathBuf;
    ///
    /// let view = ArchiveView::new(PathBuf::from("/photos/archiv.zip"));
    /// assert!(view.at_root());
    /// ```
    pub fn new(archive: PathBuf) -> Self {
        Self {
            archive,
            inside: String::new(),
        }
    }

    /// The path of the archive on the disk.
    pub fn archive(&self) -> &Path {
        &self.archive
    }

    /// The path inside the archive.
    pub fn inside(&self) -> &str {
        &self.inside
    }

    /// Whether the view is at the root of the archive.
    pub fn at_root(&self) -> bool {
        self.inside.is_empty()
    }

    /// The view of a place inside the archive.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::panel::ArchiveView;
    /// use std::path::PathBuf;
    ///
    /// let view = ArchiveView::new(PathBuf::from("/photos/archiv.zip")).at("ordner");
    /// assert_eq!(view.inside(), "ordner");
    /// ```
    pub fn at(mut self, inside: impl Into<String>) -> Self {
        self.inside = inside.into();
        self
    }

    /// The place below a directory of the archive.
    ///
    /// WHY: the path inside the archive is joined with a slash and not
    ///       with the separator of the disk. The container carries the
    ///       names of the entries, and the names are joined the way the
    ///       container joins them.
    pub fn below(&self, name: &str) -> Self {
        let name = name.trim_end_matches('/');
        if self.inside.is_empty() {
            return self.clone().at(name);
        }
        self.clone().at(format!("{}/{name}", self.inside))
    }

    /// The place above the current one.
    ///
    /// WHY: the user walks out of the archive again. The place above the
    ///       root is the directory of the archive itself — and that is
    ///       where the user came from.
    pub fn above(&self) -> Option<Self> {
        if self.at_root() {
            return None;
        }
        let mut teile: Vec<&str> = self.inside.split('/').collect();
        teile.pop();
        Some(self.clone().at(teile.join("/")))
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_view_walks_down_and_up_in_the_archive() {
        // arrange
        let view = ArchiveView::new(PathBuf::from("/photos/archiv.zip"));

        // act
        let tiefe = view.below("ordner").below("unterordner");
        let hoeher = tiefe.above();

        // assert
        // WHY: the path inside the archive is joined with a slash, and the
        //       way out is the way in, one level at a time.
        assert_eq!(tiefe.inside(), "ordner/unterordner");
        assert_eq!(
            hoeher.map(|v| v.inside().to_string()),
            Some("ordner".to_string())
        );
    }

    #[test]
    fn the_root_of_the_archive_has_no_place_above_it() {
        // arrange
        let view = ArchiveView::new(PathBuf::from("/photos/archiv.zip"));

        // act, assert
        // WHY: above the root is the directory of the archive itself — and
        //       that is not inside the archive.
        assert!(view.at_root());
        assert_eq!(view.above(), None);
    }

    #[test]
    fn the_slash_of_a_directory_is_not_carried_into_the_path() {
        // arrange
        let view = ArchiveView::new(PathBuf::from("/photos/archiv.zip"));

        // act
        let tiefe = view.below("ordner/");

        // assert
        // WHY: the ZIP container writes the name of a directory with a
        //       slash at the end.
        assert_eq!(tiefe.inside(), "ordner");
    }
}
