//! The language of the interface of the desktop build.
//!
//! WHAT: the language of the interface ([`InterfaceLanguage`]), the locale of
//!       a setting of the configuration file ([`locale_of`]) and the watcher
//!       of that file ([`ConfigWatcher`]).
//! WHY: the language of the interface can be changed while the program runs
//!       (FR-124). The language of the running program is therefore built
//!       again and handed to the shell of the interface, and the choice of
//!       the user is read from the configuration file — the place where the
//!       user writes it down.
//! BOUNDARY: this module builds the language of the interface and watches the
//!       file that names it. It draws nothing, and it reads no file beyond
//!       the modification time of the configuration file.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_desktop::language::{InterfaceLanguage, locale_of};
//! use phocommander_config::Language;
//!
//! let locale = locale_of(Language::De);
//! let language = InterfaceLanguage::of(locale);
//! assert_eq!(language.locale().as_str(), "de");
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use phocommander_config::Language;
use phocommander_i18n::Locale;
use phocommander_ui::{Catalogue, Formats, PlainFormats};

use crate::catalogue::{EmptyCatalogue, catalogue_for, formats_for};

/// The time between two readings of the configuration file.
///
/// WHY: the file is read often enough that a change of the language is
///       noticed while the user is still looking at the screen, and seldom
///       enough that the program does not spend its time on the file.
pub const CHECK_INTERVAL: Duration = Duration::from_secs(1);

/// The language of the interface.
///
/// WHY: a language is not only the texts: it is also the way the numbers,
///       the sizes, the dates and the times are written (FR-123). The two
///       belong together, and the shell of the interface takes both of them.
pub struct InterfaceLanguage {
    /// The locale of the language.
    locale: Locale,

    /// The catalogue of the texts, until the shell takes it.
    catalogue: Option<Box<dyn Catalogue>>,

    /// The formats of the values, until the shell takes them.
    formats: Option<Box<dyn Formats>>,
}

impl InterfaceLanguage {
    /// Builds the language of a locale.
    ///
    /// WHY: a locale whose catalogue cannot be read and whose data of CLDR
    ///       is unknown does not stop the program: the empty catalogue and
    ///       the plain formats take its place, and the interface stays
    ///       readable (FR-122).
    pub fn of(locale: Locale) -> Self {
        let catalogue = catalogue_for(&locale);
        let formats = formats_for(locale.as_str());
        Self {
            locale,
            catalogue: Some(catalogue),
            formats: Some(formats),
        }
    }

    /// The locale of the language.
    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    /// Takes the catalogue of the texts out of the language.
    ///
    /// WHY: the shell of the interface keeps the catalogue, and the language
    ///       keeps the locale — so that the caller can tell whether the
    ///       language has changed. A language whose catalogue was taken
    ///       already hands out the empty catalogue, which lets the interface
    ///       show its keys (FR-122).
    pub fn take_catalogue(&mut self) -> Box<dyn Catalogue> {
        self.catalogue
            .take()
            .unwrap_or_else(|| Box::new(EmptyCatalogue))
    }

    /// Takes the formats of the values out of the language.
    ///
    /// WHY: a language whose formats were taken already hands out the plain
    ///       formats, so that the interface always writes something.
    pub fn take_formats(&mut self) -> Box<dyn Formats> {
        self.formats
            .take()
            .unwrap_or_else(|| Box::new(PlainFormats))
    }
}

/// The locale of a language of the configuration file.
///
/// WHY: the setting of the configuration file names a language of the
///       product. A language that the product does not ship leads to
///       English, because English is the last step of the chain of the
///       lookup (FR-122).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_desktop::language::locale_of;
/// use phocommander_config::Language;
///
/// assert_eq!(locale_of(Language::De).as_str(), "de");
/// assert_eq!(locale_of(Language::En).as_str(), "en");
/// ```
pub fn locale_of(language: Language) -> Locale {
    match language {
        Language::De => Locale::of("de"),
        Language::En => Locale::english(),
        Language::System => Locale::from_system(),
    }
}

/// The watcher of the configuration file.
///
/// WHY: the language of the interface can be changed while the program runs
///       (FR-124). The configuration file is the place where the user writes
///       that choice down, so the program watches the file and applies the
///       change of the language as soon as the file names another one. Only
///       the language is applied at run time: the other settings are read at
///       the start, because a change of the security rules needs a restart
///       of the program.
pub struct ConfigWatcher {
    /// The path of the file that is watched, if there is one.
    path: Option<PathBuf>,

    /// The state of the file at the last reading.
    stamp: Option<FileStamp>,

    /// The moment of the next reading of the file.
    next_check: Instant,
}

/// The state of the watched file.
///
/// WHY: the moment of the last change alone is not enough. A file system
///       that counts the moments in seconds would not notice a change that
///       happens within the same second. The length of the file is part of
///       the state as well, and a file that is written to carries a new
///       length in nearly every case.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileStamp {
    /// The moment of the last change of the file.
    modified: Option<SystemTime>,

    /// The length of the file in bytes.
    length: u64,
}

impl ConfigWatcher {
    /// Watches a file.
    pub fn new(path: PathBuf) -> Self {
        let stamp = stamp_of(&path);
        Self {
            path: Some(path),
            stamp,
            next_check: Instant::now(),
        }
    }

    /// Watches no file at all.
    ///
    /// WHY: a machine without a configuration directory has no file to
    ///       watch. The watcher then never reports a change, and the
    ///       language of the interface stays the one of the start.
    pub fn without_file() -> Self {
        Self {
            path: None,
            stamp: None,
            next_check: Instant::now(),
        }
    }

    /// The path of the file that is watched, if there is one.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Whether the file has changed since the last reading.
    ///
    /// WHY: the file is read at most once per interval, so that a program
    ///       that runs for hours does not spend its time on the file system.
    pub fn changed(&mut self) -> bool {
        let now = Instant::now();
        if now < self.next_check {
            return false;
        }
        self.next_check = now + CHECK_INTERVAL;

        let Some(path) = self.path.as_deref() else {
            return false;
        };

        let stamp = stamp_of(path);
        if stamp == self.stamp {
            return false;
        }
        self.stamp = stamp;
        true
    }
}

/// The state of a file, if it can be read.
fn stamp_of(path: &Path) -> Option<FileStamp> {
    let metadata = std::fs::metadata(path).ok()?;
    Some(FileStamp {
        modified: metadata.modified().ok(),
        length: metadata.len(),
    })
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::fs;

    /// A file in a directory of its own.
    fn temporary_file(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("phocommander-{name}"));
        fs::create_dir_all(&directory).unwrap_or_else(|error| {
            panic!("the directory should be writable: {error}");
        });
        directory.join("config.toml")
    }

    #[test]
    fn the_locale_of_a_setting_is_the_locale_of_the_product() {
        // act
        let german = locale_of(Language::De);
        let english = locale_of(Language::En);

        // assert
        assert_eq!(german.as_str(), "de");
        assert_eq!(english.as_str(), "en");
    }

    #[test]
    fn a_file_that_does_not_change_is_not_reported() {
        // arrange
        let path = temporary_file("watcher-quiet");
        fs::write(&path, "schema_version = 1\n").unwrap_or_else(|error| {
            panic!("the file should be writable: {error}");
        });
        let mut watcher = ConfigWatcher::new(path);

        // act
        let changed = watcher.changed();

        // assert
        assert!(!changed, "the file was not written to");
    }

    #[test]
    fn a_file_that_changes_is_reported() {
        // WHY: the language of the interface follows the file, and the file
        //       is the place where the user writes that choice down.
        // arrange
        let path = temporary_file("watcher-loud");
        fs::write(&path, "schema_version = 1\n").unwrap_or_else(|error| {
            panic!("the file should be writable: {error}");
        });
        let mut watcher = ConfigWatcher::new(path.clone());

        // act
        fs::write(&path, "schema_version = 1\nlanguage = \"de\"\n").unwrap_or_else(|error| {
            panic!("the file should be writable: {error}");
        });
        let changed = watcher.changed();

        // assert
        assert!(changed, "the file was written to");
    }

    #[test]
    fn a_watcher_without_a_file_reports_nothing() {
        // arrange
        let mut watcher = ConfigWatcher::without_file();

        // act
        let changed = watcher.changed();

        // assert
        assert!(!changed);
        assert!(watcher.path().is_none());
    }

    #[test]
    fn the_language_of_a_locale_carries_a_catalogue_and_the_formats() {
        // arrange
        let mut language = InterfaceLanguage::of(Locale::of("de"));

        // act
        let catalogue = language.take_catalogue();
        let formats = language.take_formats();

        // assert
        assert!(catalogue.lookup("panel-column-name").is_some());
        assert_eq!(formats.size(1_200_000), "1,2 MB");
    }
}
