//! The application of the desktop build.
//!
//! WHAT: the application that holds the shell of the interface and the
//!       watcher of the configuration file ([`DesktopApp`]).
//! WHY: the language of the interface can be changed while the program runs
//!       (FR-124). The application therefore watches the configuration file
//!       and hands a new language to the shell of the interface as soon as
//!       the file names another one. The shell draws the surface; this type
//!       holds it and the watcher beside it.
//! BOUNDARY: this module is the composition of the desktop build. It reads
//!       the configuration file and it applies the language. It performs no
//!       file operation of its own and it draws nothing.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::desktop::DesktopApp;
//! use phocommander_desktop::language::ConfigWatcher;
//! use phocommander_i18n::Locale;
//! use phocommander_ui::keymap_view::KeymapView;
//! use phocommander_ui::surface::WorkSurface;
//! use phocommander_ui::WorkSurfaceApp;
//! use std::path::PathBuf;
//!
//! # struct Empty;
//! # impl phocommander_ui::Catalogue for Empty {
//! #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
//! #         None
//! #     }
//! # }
//! let shell = WorkSurfaceApp::new(
//!     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
//!     KeymapView::default(),
//!     Box::new(Empty),
//! );
//! let app = DesktopApp::new(shell, Locale::english(), ConfigWatcher::without_file());
//! assert_eq!(app.locale().as_str(), "en");
//! ```

use std::path::Path;

use phocommander_i18n::Locale;
use phocommander_ui::WorkSurfaceApp;

use crate::language::{CHECK_INTERVAL, ConfigWatcher, InterfaceLanguage, locale_of};

/// The application of the desktop build.
pub struct DesktopApp {
    /// The shell of the interface.
    surface: WorkSurfaceApp,

    /// The locale of the language that is on the screen.
    locale: Locale,

    /// The watcher of the configuration file.
    watcher: ConfigWatcher,
}

impl DesktopApp {
    /// The application with a shell, a locale and a watcher.
    pub fn new(surface: WorkSurfaceApp, locale: Locale, watcher: ConfigWatcher) -> Self {
        Self {
            surface,
            locale,
            watcher,
        }
    }

    /// The shell of the interface.
    pub fn surface(&self) -> &WorkSurfaceApp {
        &self.surface
    }

    /// The shell of the interface.
    pub fn surface_mut(&mut self) -> &mut WorkSurfaceApp {
        &mut self.surface
    }

    /// The locale of the language that is on the screen.
    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    /// Applies a language to the shell of the interface.
    ///
    /// WHY: the language of the interface can be changed while the program
    ///       runs (FR-124). The shell takes the new catalogue of the texts
    ///       and the new formats of the values, and the interface shows them
    ///       in the next frame — without a restart and without a lost state.
    pub fn set_language(&mut self, locale: Locale) {
        let mut language = InterfaceLanguage::of(locale.clone());
        self.surface.set_catalogue(language.take_catalogue());
        self.surface.set_formats(language.take_formats());
        self.locale = locale;
    }

    /// Applies the language of the configuration file, if the file names
    /// another language than the one on the screen.
    ///
    /// WHY: the file is the place where the user writes the choice of the
    ///       language down. The program reads the file again and applies the
    ///       change — the rest of the settings stays as it was read at the
    ///       start, because a change of the security rules needs a restart.
    pub fn apply_the_language_of_the_configuration(&mut self) {
        if !self.watcher.changed() {
            return;
        }

        let Some(path) = self.watcher.path().map(Path::to_path_buf) else {
            return;
        };

        let (config, _notices) = phocommander_config::load_file_or_safe(path);
        let locale = locale_of(config.general.language);
        if locale == self.locale {
            return;
        }
        self.set_language(locale);
    }
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.apply_the_language_of_the_configuration();
        self.surface.update(ui);
        // WHY: the configuration file is watched, so the program wakes up
        //       once a second to look at it. Without that wake-up a change
        //       of the language would only be noticed when the user happens
        //       to press a key (FR-124).
        ui.ctx().request_repaint_after(CHECK_INTERVAL);
    }
}
