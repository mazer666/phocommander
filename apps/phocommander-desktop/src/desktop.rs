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

    /// Carries out the language the user has chosen in the interface.
    ///
    /// WHY: the actions `app.language.de`, `app.language.en` and
    ///       `app.language.fr` name the language in the interface. The
    ///       choice belongs to the user, and it belongs in the file of the
    ///       settings — the place where the user keeps what belongs to the
    ///       machine. The choice is therefore written into the file and
    ///       applied to the screen at once, without a restart (FR-124).
    ///
    /// WHY: the file is changed with the editor of the configuration, and
    ///       not written anew. The file belongs to the user: it carries
    ///       comments that explain every setting, and a program that
    ///       rewrites the whole file takes them away.
    pub fn apply_the_chosen_language(&mut self) {
        let Some(locale) = self.surface.take_language_choice() else {
            return;
        };

        if let Some(path) = self.watcher.path().map(Path::to_path_buf)
            && let Err(error) = write_the_language(&path, &locale)
        {
            // WHY: a file that cannot be written is not a reason to stop
            //       the program. The language changes on the screen all the
            //       same, and the file keeps the language it had.
            eprintln!("the language cannot be written down: {error}");
        }

        // WHY: the locale of a catalogue is the name of its directory, and
        //       a locale whose catalogue does not exist falls back to the
        //       chain of the lookup — the interface stays readable.
        let locale = Locale::of(&locale);
        self.set_language(locale);
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

/// Writes the language into the configuration file.
///
/// WHY: the file belongs to the user. The editor of the configuration
///       changes the one line that carries the language and leaves the
///       comments and the order of the file as they are. A program that
///       writes the file anew would take the comments away.
fn write_the_language(path: &Path, locale: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let language = match locale {
        "de" => phocommander_config::Language::De,
        "en" => phocommander_config::Language::En,
        _ => phocommander_config::Language::System,
    };
    let geaendert = phocommander_config::edit::set_language(&text, language)
        .map_err(|error| error.to_string())?;
    std::fs::write(path, geaendert).map_err(|error| error.to_string())
}

impl eframe::App for DesktopApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // WHY: the choice of the user comes first. It is the language the
        //       user has asked for, and the file may still name the old one
        //       until the writing is done.
        self.apply_the_chosen_language();
        self.apply_the_language_of_the_configuration();
        self.surface.update(ui);
        // WHY: the configuration file is watched, so the program wakes up
        //       once a second to look at it. Without that wake-up a change
        //       of the language would only be noticed when the user happens
        //       to press a key (FR-124).
        ui.ctx().request_repaint_after(CHECK_INTERVAL);
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn the_language_is_written_into_the_file_of_the_settings() {
        // arrange
        let directory = std::env::temp_dir().join("phocommander-language-write-test");
        std::fs::create_dir_all(&directory)
            .unwrap_or_else(|error| panic!("the directory should be created: {error}"));
        let path = directory.join("config.toml");
        std::fs::write(
            &path,
            "# The settings of the user.\n[general]\n# The language.\nlanguage = \"de\"\n",
        )
        .unwrap_or_else(|error| panic!("the file should be written: {error}"));

        // act
        write_the_language(&path, "en")
            .unwrap_or_else(|error| panic!("the language should be written: {error}"));

        // assert
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("the file should be read: {error}"));
        // WHY: the comments of the user stay where they are, and only the
        //       value of the one line changes.
        assert_eq!(
            text,
            "# The settings of the user.\n[general]\n# The language.\nlanguage = \"en\"\n"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_file_that_cannot_be_written_gives_an_error_and_no_panic() {
        // arrange
        let path = std::env::temp_dir().join("phocommander-language-missing-test/config.toml");

        // act
        let ergebnis = write_the_language(&path, "en");

        // assert
        assert!(ergebnis.is_err());
    }
}
