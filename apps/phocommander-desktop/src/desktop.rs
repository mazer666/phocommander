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

use std::sync::Arc;

use phocommander_config::EffectiveSecurity;
use phocommander_i18n::Locale;
use phocommander_ops::copy::{CopyMode, CopyPolicy};
use phocommander_ops::port::FileSystem;
use phocommander_ops::queue::JobQueue;
use phocommander_ops::undo::UndoLedger;
use phocommander_ui::WorkSurfaceApp;
use phocommander_ui::commands::Command;
use phocommander_viewer::policy::ReadPolicy;
use phocommander_viewer::reader::ViewerReader;

use crate::execute::{Adapter, Ways, submit};
use crate::filesystem::HostFileSystem;
use crate::preview::ViewerPreviewSource;

use crate::language::{CHECK_INTERVAL, ConfigWatcher, InterfaceLanguage, locale_of};

/// A waiting room that has no file system.
///
/// WHY: the host of the sandbox has no file system of its own, and a
///       waiting room without one can hold the jobs but cannot carry them
///       out. It is the fallback for the tests and for the places where the
///       application is built without a host.
fn queue_without_a_file() -> Arc<JobQueue> {
    Arc::new(JobQueue::new(
        Arc::new(phocommander_ops::fake_fs::FakeFileSystem::new()),
        1,
    ))
}

/// The application of the desktop build.
pub struct DesktopApp {
    /// The shell of the interface.
    surface: WorkSurfaceApp,

    /// The locale of the language that is on the screen.
    locale: Locale,

    /// The watcher of the configuration file.
    watcher: ConfigWatcher,

    /// The waiting room of the operations.
    ///
    /// WHY: the commands of the surface become the jobs of the waiting
    ///       room, and the worker of the room carries them out. Without
    ///       the room the commands of the surface would stay commands and
    ///       nothing would ever be copied.
    queue: Arc<JobQueue>,

    /// The way a file is copied.
    ///
    /// WHY: the way is a setting of the user (FR-22b), and it stands in
    ///       the configuration file.
    copy_mode: CopyMode,
    sync_mode: CopyMode,
    /// What has been copied and can be taken back.
    ///
    /// WHY: the undo takes a copy back, and it can only take back what it
    ///       knows. The desktop keeps the ledger, because it is the place
    ///       that copies (FR-26).
    undo: UndoLedger,

    /// The rules that decide whether the state of the session is written.
    ///
    /// WHY: the rule of the security section says that the program leaves
    ///       nothing behind. The rule decides whether the state of the
    ///       session is written at the end, and the application carries it
    ///       because the application is what ends (FR-16).
    security_rules: EffectiveSecurity,

    /// The source of the content of the files.
    ///
    /// WHY: the viewer opens a file that the user has named himself, and
    ///       the quick view shows the file that is marked. Both go through
    ///       the reader of the viewer, and the reader knows the rules that
    ///       say which file may be read (FR-50).
    viewer: Box<ViewerPreviewSource>,

    /// The file system of the host.
    ///
    /// WHY: the commands of the surface ask for directories, and the
    ///       directory is read from the disk. The file system of the host
    ///       is therefore part of the application and not of the surface.
    file_system: Box<dyn FileSystem>,

    /// The rules that decide how a copy treats the files it copies.
    ///
    /// WHY: the rules come from the configuration of the user, and they
    ///       decide whether the copy compares the checksums and whether a
    ///       link is dereferenced. A program that decided them on its own
    ///       would behave the same on a private machine and on a managed
    ///       one — and the difference is the setting, not the program
    ///       (section 9.8).
    copy_rules: CopyPolicy,
}

impl DesktopApp {
    /// The application with a shell, a locale and a watcher.
    pub fn new(surface: WorkSurfaceApp, locale: Locale, watcher: ConfigWatcher) -> Self {
        Self::with_queue(surface, locale, watcher, queue_without_a_file())
    }

    /// The application with a waiting room of its own.
    ///
    /// WHY: the waiting room holds the file system of the host, and the
    ///       host of the sandbox has no file system of its own. The
    ///       application therefore takes the room from outside, and the
    ///       composition root builds it with the file system of the host.
    pub fn with_queue(
        surface: WorkSurfaceApp,
        locale: Locale,
        watcher: ConfigWatcher,
        queue: Arc<JobQueue>,
    ) -> Self {
        Self::with_file_system(
            surface,
            locale,
            watcher,
            queue,
            Box::new(HostFileSystem::new()),
        )
    }

    /// The application with a file system of its own.
    ///
    /// WHY: the host of the sandbox has no file system of its own, and a
    ///       test needs one that it can fill. The file system is therefore
    ///       a parameter of the construction, and the composition root
    ///       hands the file system of the host over.
    pub fn with_file_system(
        surface: WorkSurfaceApp,
        locale: Locale,
        watcher: ConfigWatcher,
        queue: Arc<JobQueue>,
        file_system: Box<dyn FileSystem>,
    ) -> Self {
        Self::with_everything(
            surface,
            locale,
            watcher,
            queue,
            file_system,
            Box::new(ViewerPreviewSource::new(ViewerReader::new(
                ReadPolicy::strict(),
                10_485_760,
            ))),
        )
    }

    /// The application with a source of the content of the files.
    ///
    /// WHY: the reader of the viewer needs the rule of the security
    ///       section, and the limit of the size of a file. The composition
    ///       root knows both and hands them over.
    pub fn with_everything(
        surface: WorkSurfaceApp,
        locale: Locale,
        watcher: ConfigWatcher,
        queue: Arc<JobQueue>,
        file_system: Box<dyn FileSystem>,
        viewer: Box<ViewerPreviewSource>,
    ) -> Self {
        Self {
            surface,
            locale,
            watcher,
            queue,
            file_system,
            viewer,
            copy_mode: CopyMode::Verified,
            sync_mode: CopyMode::SyncPreview,
            undo: UndoLedger::new(),
            copy_rules: CopyPolicy::strict(),
            security_rules: EffectiveSecurity::safe(),
        }
    }

    /// Writes the state of the session, if the rules allow it.
    ///
    /// WHY: the rule of the security section decides whether the program
    ///       leaves anything behind. The state of the session is the two
    ///       directories that the panels showed, and nothing else — no
    ///       history, no names of files, no credentials (FR-16).
    pub fn write_the_session(&self) {
        let Some(directory) = crate::composition::configuration_directory() else {
            return;
        };
        let state = phocommander_config::SessionState::new(
            self.surface
                .surface()
                .panel(phocommander_ui::panel::PanelSide::Left)
                .path(),
            self.surface
                .surface()
                .panel(phocommander_ui::panel::PanelSide::Right)
                .path(),
        );
        if let Err(error) =
            phocommander_config::write_session_state(&directory, &state, &self.security_rules)
        {
            eprintln!("the state of the session could not be written: {error}");
        }
    }

    /// Sets the rules that decide whether the state of the session is written.
    ///
    /// WHY: the rule of the security section decides whether the program
    ///       leaves anything behind, and the state of the session is such
    ///       a thing (FR-16).
    pub fn set_security_rules(&mut self, rules: EffectiveSecurity) {
        self.security_rules = rules;
    }

    /// Sets the way a file is copied.
    pub fn set_copy_mode(&mut self, mode: CopyMode) {
        self.copy_mode = mode;
    }

    /// The way a directory is synchronised.
    pub const fn sync_mode(&self) -> CopyMode {
        self.sync_mode
    }

    /// Sets the way a directory is synchronised.
    ///
    /// WHY: the way of the synchronisation is a setting of the user
    ///       (FR-22b). The setting decides whether the synchronisation
    ///       only shows the plan or also brings the target into the shape
    ///       of the source.
    pub fn set_sync_mode(&mut self, mode: CopyMode) {
        self.sync_mode = mode;
    }

    /// Sets the rules that decide how a copy treats the files it copies.
    pub fn set_copy_rules(&mut self, rules: CopyPolicy) {
        self.copy_rules = rules;
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
    fn on_exit(&mut self) {
        // WHY: the state of the session is written when the program ends,
        //       and not earlier: the state of the panels is the state of
        //       the last moment, and the program writes it exactly once
        //       (FR-16).
        self.write_the_session();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // WHY: the choice of the user comes first. It is the language the
        //       user has asked for, and the file may still name the old one
        //       until the writing is done.
        self.apply_the_chosen_language();
        self.apply_the_language_of_the_configuration();

        // WHY: the commands of the surface are the requests of the user, and
        //       they become the jobs of the waiting room. Without this the
        //       copy, the move, the deletion and the rename would stay
        //       commands and nothing would ever be carried out.
        let commands = self.surface.take_commands();
        if !commands.is_empty() {
            // WHY: the command to leave the application belongs to the
            //       frame and not to the file system. The window closes,
            //       and the program ends with it (FR-04).
            if commands
                .iter()
                .any(|command| matches!(command, Command::Quit))
            {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
            let mut adapter = Adapter {
                queue: &self.queue,
                fs: self.file_system.as_ref(),
                viewer: self.viewer.as_ref(),
                ways: Ways {
                    copy: self.copy_mode,
                    sync: self.sync_mode,
                },
                rules: self.copy_rules,
                undo: &mut self.undo,
            };
            submit(&commands, self.surface.surface_mut(), &mut adapter);
        }
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
        let directory = std::env::temp_dir().join(format!(
            "phocommander-language-write-test-{}",
            std::process::id()
        ));
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
        let path = std::env::temp_dir().join(format!(
            "phocommander-language-missing-test-{}/config.toml",
            std::process::id()
        ));

        // act
        let ergebnis = write_the_language(&path, "en");

        // assert
        assert!(ergebnis.is_err());
    }
}
