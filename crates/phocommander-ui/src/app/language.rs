//! The language of the interface, seen by the shell.
//!
//! WHAT: the ways in which the shell takes a language — a new catalogue of
//!       texts ([`WorkSurfaceApp::set_catalogue`]) and new formats of the
//!       values ([`WorkSurfaceApp::set_formats`]).
//! WHY: the language of the interface can be changed while the program runs
//!       (FR-124). The catalogue and the formats are therefore not fixed at
//!       the start: the shell takes them from the composition root and can
//!       take them again. The change reaches the screen in the next frame —
//!       without a restart and without the user having to cause a repaint.
//! BOUNDARY: this module knows the shell and nothing else. It builds no
//!       catalogue and no formats of its own: the composition root owns
//!       them, because the interface crate may not depend on the crate of
//!       the localisation (ADR-0001, option B).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::app::WorkSurfaceApp;
//! use phocommander_ui::keymap_view::KeymapView;
//! use phocommander_ui::surface::WorkSurface;
//! use phocommander_ui::Catalogue;
//! use std::path::PathBuf;
//!
//! # struct Empty;
//! # impl phocommander_ui::Catalogue for Empty {
//! #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
//! #         None
//! #     }
//! # }
//! let mut app = WorkSurfaceApp::new(
//!     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
//!     KeymapView::default(),
//!     Box::new(Empty),
//! );
//! app.set_catalogue(Box::new(Empty));
//! assert!(app.catalogue().lookup("common-ok").is_none());
//! ```

use crate::app::WorkSurfaceApp;
use crate::format::Formats;
use crate::i18n::Catalogue;

impl WorkSurfaceApp {
    /// The shell with other formats for the values of the interface.
    ///
    /// WHY: the numbers, the sizes, the dates and the times follow the rules
    ///       of the locale of the user (FR-123). The shell therefore takes
    ///       the formats from the composition root, exactly as it takes the
    ///       catalogue of the texts. A shell without formats writes the
    ///       plain formats, and a caller that has no locale needs no
    ///       decision.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::format::PlainFormats;
    /// use phocommander_ui::keymap_view::KeymapView;
    /// use phocommander_ui::surface::WorkSurface;
    /// use phocommander_ui::Formats;
    /// use std::path::PathBuf;
    ///
    /// # struct Empty;
    /// # impl phocommander_ui::Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     KeymapView::default(),
    ///     Box::new(Empty),
    /// )
    /// .with_formats(Box::new(PlainFormats));
    /// assert_eq!(app.formats().size(512), "512 B");
    /// ```
    pub fn with_formats(mut self, formats: Box<dyn Formats>) -> Self {
        self.formats = formats;
        self
    }

    /// Replaces the catalogue of the texts.
    ///
    /// WHY: the language of the interface can be changed while the program
    ///       runs (FR-124). The catalogue is therefore not fixed at the
    ///       start, and the shell asks for a repaint — the new language is on
    ///       the screen in the next frame, without a restart and without the
    ///       user having to cause a repaint. The state of the panels is not
    ///       touched by the change: only the language of the texts and of the
    ///       values changes.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::keymap_view::KeymapView;
    /// use phocommander_ui::surface::WorkSurface;
    /// use phocommander_ui::Catalogue;
    /// use std::path::PathBuf;
    ///
    /// # struct German;
    /// # impl phocommander_ui::Catalogue for German {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
    /// #         Some("Abbrechen".to_string())
    /// #     }
    /// # }
    /// let mut app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     KeymapView::default(),
    ///     Box::new(German),
    /// );
    /// app.set_catalogue(Box::new(German));
    /// assert_eq!(app.catalogue().lookup("common-cancel"), Some("Abbrechen".to_string()));
    /// ```
    pub fn set_catalogue(&mut self, catalogue: Box<dyn Catalogue>) {
        self.catalogue = catalogue;
        self.request_repaint();
    }

    /// Replaces the formats of the values of the interface.
    ///
    /// WHY: the formats belong to the language of the interface, and a
    ///       change of the language changes them as well (FR-123, FR-124).
    ///       The shell asks for a repaint, so that the new formats are on the
    ///       screen in the next frame.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::format::PlainFormats;
    /// use phocommander_ui::keymap_view::KeymapView;
    /// use phocommander_ui::surface::WorkSurface;
    /// use phocommander_ui::Formats;
    /// use std::path::PathBuf;
    ///
    /// # struct Empty;
    /// # impl phocommander_ui::Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let mut app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     KeymapView::default(),
    ///     Box::new(Empty),
    /// );
    /// app.set_formats(Box::new(PlainFormats));
    /// assert_eq!(app.formats().size(2_048), "2.0 KiB");
    /// ```
    pub fn set_formats(&mut self, formats: Box<dyn Formats>) {
        self.formats = formats;
        self.request_repaint();
    }

    /// The catalogue of the texts that is in force.
    ///
    /// WHY: a caller that wants to know the language of the interface — a
    ///       test, a report — asks the shell and does not keep a copy of
    ///       the catalogue.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::keymap_view::KeymapView;
    /// use phocommander_ui::surface::WorkSurface;
    /// use phocommander_ui::Catalogue;
    /// use std::path::PathBuf;
    ///
    /// # struct Empty;
    /// # impl phocommander_ui::Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     KeymapView::default(),
    ///     Box::new(Empty),
    /// );
    /// assert!(app.catalogue().lookup("common-ok").is_none());
    /// ```
    pub fn catalogue(&self) -> &dyn Catalogue {
        self.catalogue.as_ref()
    }

    /// The formats of the values that are in force.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::keymap_view::KeymapView;
    /// use phocommander_ui::surface::WorkSurface;
    /// use phocommander_ui::Formats;
    /// use std::path::PathBuf;
    ///
    /// # struct Empty;
    /// # impl phocommander_ui::Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     KeymapView::default(),
    ///     Box::new(Empty),
    /// );
    /// assert_eq!(app.formats().size(512), "512 B");
    /// ```
    pub fn formats(&self) -> &dyn Formats {
        self.formats.as_ref()
    }

    /// Asks for a repaint, so that a change of the language reaches the
    /// screen.
    ///
    /// WHY: the program may stand still between two frames. A change of the
    ///       language would then stay invisible until the user happens to
    ///       press a key. The shell therefore asks egui for a repaint — the
    ///       new language is on the screen in the next frame (FR-124).
    fn request_repaint(&self) {
        if let Some(context) = &self.egui_context {
            context.request_repaint();
        }
    }
}

#[cfg(test)]
// WHY: the tests of the module live in a file of their own, so that the
// implementation and its proof can grow independently. The file of the
// tests carries its own exception, because a test may fail loudly when its
// arrangement is wrong.
mod tests;
