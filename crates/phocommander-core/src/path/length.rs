//! The limit that a platform puts on the length of a path.
//!
//! WHAT: the unit in which the length of a path is counted ([{LengthUnit}]),
//!       the platform whose limit is in force ([{PathPlatform}]), the limit
//!       itself ([{PathLengthLimit}]), the measurement of a path
//!       ([{measure_path_length}]) and the refusal of a path that is too
//!       long ([{check_path_length}], [{PathTooLong}], [{PathAdvice}]).
//! WHY: Windows limits a path to 260 characters, and the program has to say
//!       that in words that the user understands — which path, how long it
//!       is and what the user can do (FR-28). The rule lives in the core,
//!       because it is a rule of the path and not a rule of the window.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::path::length::{PathLengthLimit, check_path_length};
//! use std::path::Path;
//!
//! let limit = PathLengthLimit::windows();
//! assert!(check_path_length(Path::new("C:\\photos\\sunset.jpg"), limit).is_ok());
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

/// The number of characters that Windows accepts in a path.
///
/// WHY: Windows limits a path to MAX_PATH, which is 260 characters. The
///       limit counts the characters of the *whole* path — the name of the
///       drive, the separators and the name of the file — and it applies to
///       every program, unless the program declares in its manifest that it
///       can handle longer paths (Microsoft, "Maximum path length
///       limitation", the reference [35] of the specification).
pub const WINDOWS_MAX_PATH_CHARACTERS: usize = 260;

/// The number of characters that Windows accepts in a path when the system
/// uses the long paths.
///
/// WHY: with the setting `LongPathsEnabled` of the system and a manifest
///       that declares `longPathAware`, Windows accepts paths of up to
///       32,767 characters. The number is a limit of the file system and
///       not a value that the program chooses.
pub const WINDOWS_LONG_PATH_CHARACTERS: usize = 32_767;

/// The number of bytes that Linux accepts in a path.
///
/// WHY: the constant PATH_MAX of Linux is 4096 bytes, and the terminating
///       zero counts towards it, so that 4095 bytes of a path are usable
///       (see `pathconf(3)`).
pub const LINUX_MAX_PATH_BYTES: usize = 4095;

/// The unit in which the length of a path is counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LengthUnit {
    /// The number of the characters of the path.
    ///
    /// WHY: Windows counts the characters of a path and not its bytes. A
    ///       character outside the basic plane counts as two characters in
    ///       Windows and as one here, so that a path of such characters is
    ///       measured shorter here than Windows measures it.
    Characters,

    /// The number of the bytes of the path.
    ///
    /// WHY: Linux counts the bytes of a path. A name of a hundred
    ///       characters that is written in a script of two bytes per
    ///       character is two hundred bytes long for the system.
    Bytes,
}

/// The platform whose limit of the length of a path is in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathPlatform {
    /// Windows, which counts the characters of a path.
    Windows,

    /// Linux, which counts the bytes of a path.
    Linux,

    /// A platform that the core does not know.
    ///
    /// WHY: the message calls such a platform "the system", because the core
    ///       knows no other name for it.
    Other,
}

impl PathPlatform {
    /// The name that the message uses for the platform.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::path::length::PathPlatform;
    ///
    /// assert_eq!(PathPlatform::Windows.name(), "Windows");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            PathPlatform::Windows => "Windows",
            PathPlatform::Linux => "Linux",
            PathPlatform::Other => "the system",
        }
    }
}

/// The limit that a platform puts on the length of a path.
///
/// WHY: the limit is data and not a constant, because the platforms
///       disagree — Windows counts 260 characters, Linux counts 4096 bytes —
///       and because the message has to name the limit that was checked when
///       it refuses a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathLengthLimit {
    platform: PathPlatform,
    amount: usize,
    unit: LengthUnit,
}

impl PathLengthLimit {
    /// The limit of Windows without the long paths.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::path::length::PathLengthLimit;
    ///
    /// assert_eq!(PathLengthLimit::windows().amount(), 260);
    /// ```
    pub fn windows() -> Self {
        Self {
            platform: PathPlatform::Windows,
            amount: WINDOWS_MAX_PATH_CHARACTERS,
            unit: LengthUnit::Characters,
        }
    }

    /// The limit of Windows with the long paths.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::path::length::PathLengthLimit;
    ///
    /// assert_eq!(PathLengthLimit::windows_with_long_paths().amount(), 32_767);
    /// ```
    pub fn windows_with_long_paths() -> Self {
        Self {
            platform: PathPlatform::Windows,
            amount: WINDOWS_LONG_PATH_CHARACTERS,
            unit: LengthUnit::Characters,
        }
    }

    /// The limit of Linux.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::path::length::PathLengthLimit;
    ///
    /// assert_eq!(PathLengthLimit::linux().amount(), 4095);
    /// ```
    pub fn linux() -> Self {
        Self {
            platform: PathPlatform::Linux,
            amount: LINUX_MAX_PATH_BYTES,
            unit: LengthUnit::Bytes,
        }
    }

    /// A limit of the caller's own.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::path::length::{LengthUnit, PathLengthLimit, PathPlatform};
    ///
    /// let limit = PathLengthLimit::new(PathPlatform::Other, 512, LengthUnit::Bytes);
    /// assert_eq!(limit.amount(), 512);
    /// ```
    pub fn new(platform: PathPlatform, amount: usize, unit: LengthUnit) -> Self {
        Self {
            platform,
            amount,
            unit,
        }
    }

    /// The platform the limit belongs to.
    pub fn platform(&self) -> PathPlatform {
        self.platform
    }

    /// The number that the platform accepts.
    pub fn amount(&self) -> usize {
        self.amount
    }

    /// The unit in which the number is counted.
    pub fn unit(&self) -> LengthUnit {
        self.unit
    }

    /// Whether the limit is the limit of Windows without the long paths.
    ///
    /// WHY: the advice of the message depends on it — only when the long
    ///       paths are switched off can the user switch them on.
    pub fn is_windows_without_long_paths(&self) -> bool {
        self.platform == PathPlatform::Windows && self.amount == WINDOWS_MAX_PATH_CHARACTERS
    }
}

/// One thing that the user can do when a path is too long.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathAdvice {
    /// Give the file a shorter name.
    ShorterName,

    /// Give one of the directories on the way to the file a shorter name.
    ShorterDirectory,

    /// Move the file into a directory with a shorter name.
    MoveToShorterDirectory,

    /// Map the directory to a drive letter, so that the path becomes shorter.
    DriveLetter,

    /// Let the system use the long paths.
    LongPathsOfTheSystem,
}

impl PathAdvice {
    /// The text of the advice.
    ///
    /// WHY: the text is English here, because the core knows no catalogue of
    ///       the interface. The interface shows its own text for the advice
    ///       it knows; the log shows this one.
    pub fn text(&self) -> &'static str {
        match self {
            PathAdvice::ShorterName => "give the file a shorter name",
            PathAdvice::ShorterDirectory => {
                "give one of the directories on the way to it a shorter name"
            }
            PathAdvice::MoveToShorterDirectory => {
                "move the file into a directory with a shorter name"
            }
            PathAdvice::DriveLetter => {
                "map the directory to a drive letter, so that the path becomes shorter"
            }
            PathAdvice::LongPathsOfTheSystem => {
                "let the system use the long paths: the setting LongPathsEnabled has to be 1 \
                 and the program has to declare that it can handle them"
            }
        }
    }
}

/// The refusal of a path that is longer than the platform accepts.
///
/// WHY: the operating system reports such a path with a number and a code
///       that the user does not understand. This type names the path, the
///       length that was measured and the things that the user can do, and
///       it does so without a window — the interface may show the advice in
///       its own words, the log shows the message as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathTooLong {
    path: PathBuf,
    amount: usize,
    limit: PathLengthLimit,
}

impl PathTooLong {
    /// The path that was refused.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The length that was measured.
    pub fn amount(&self) -> usize {
        self.amount
    }

    /// The limit that the path exceeded.
    pub fn limit(&self) -> PathLengthLimit {
        self.limit
    }

    /// The things that the user can do.
    ///
    /// WHY: the advice is data and not only text, so that the interface can
    ///       show it in the language of the user and as a list of buttons.
    pub fn advice(&self) -> Vec<PathAdvice> {
        let mut advice = vec![
            PathAdvice::ShorterName,
            PathAdvice::ShorterDirectory,
            PathAdvice::MoveToShorterDirectory,
        ];

        if self.limit.platform() == PathPlatform::Windows {
            advice.push(PathAdvice::DriveLetter);
        }

        if self.limit.is_windows_without_long_paths() {
            advice.push(PathAdvice::LongPathsOfTheSystem);
        }

        advice
    }

    /// The message that the user reads.
    ///
    /// WHY: the message names the path, the length that was measured and the
    ///       limit, and it ends with the things that the user can do. A user
    ///       who reads only the first sentence knows what happened; a user
    ///       who reads the second knows what to do.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::path::length::{PathLengthLimit, check_path_length};
    /// use std::path::Path;
    ///
    /// let long = Path::new("C:\\a-very-long-path-that-does-not-fit-into-the-limit-of-windows.txt");
    /// let error = match check_path_length(long, PathLengthLimit::windows()) {
    ///     Ok(()) => return,
    ///     Err(error) => error,
    /// };
    /// assert!(error.message().contains("260"));
    /// ```
    pub fn message(&self) -> String {
        let mut message = format!(
            "the path {} is {} {} long, and {} accepts at most {} {} for a path",
            self.path.display(),
            self.amount,
            unit_text(self.limit.unit(), self.amount),
            self.limit.platform().name(),
            self.limit.amount(),
            unit_text(self.limit.unit(), self.limit.amount()),
        );

        let mut advice = advice_sentence(&self.advice());
        if let Some(first) = advice.get_mut(0..1) {
            first.make_ascii_uppercase();
        }

        if !advice.is_empty() {
            message.push_str(". ");
            message.push_str(&advice);
            message.push('.');
        }

        message
    }
}

impl fmt::Display for PathTooLong {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message())
    }
}

impl std::error::Error for PathTooLong {}

/// The length of a path, counted in the unit of the limit.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::path::length::{LengthUnit, measure_path_length};
/// use std::path::Path;
///
/// assert_eq!(measure_path_length(Path::new("/tmp/a.txt"), LengthUnit::Characters), 10);
/// assert_eq!(measure_path_length(Path::new("/tmp/a.txt"), LengthUnit::Bytes), 10);
/// ```
pub fn measure_path_length(path: &Path, unit: LengthUnit) -> usize {
    match unit {
        LengthUnit::Characters => path.to_string_lossy().chars().count(),
        LengthUnit::Bytes => path.as_os_str().len(),
    }
}

/// Checks whether the path fits into the limit of the platform.
///
/// WHY: the operating system refuses a path that is too long with a message
///       that names a number and nothing else. The program checks the length
///       itself, before it calls the system, so that it can tell the user
///       which path is too long and what the user can do about it (FR-28).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::path::length::{PathLengthLimit, check_path_length};
/// use std::path::Path;
///
/// let limit = PathLengthLimit::windows();
/// assert!(check_path_length(Path::new("C:\\photos\\sunset.jpg"), limit).is_ok());
/// ```
pub fn check_path_length(path: &Path, limit: PathLengthLimit) -> Result<(), PathTooLong> {
    let amount = measure_path_length(path, limit.unit());

    if amount > limit.amount() {
        return Err(PathTooLong {
            path: path.to_path_buf(),
            amount,
            limit,
        });
    }

    Ok(())
}

/// The name of the unit, in the singular and in the plural.
fn unit_text(unit: LengthUnit, amount: usize) -> &'static str {
    match (unit, amount) {
        (LengthUnit::Characters, 1) => "character",
        (LengthUnit::Characters, _) => "characters",
        (LengthUnit::Bytes, 1) => "byte",
        (LengthUnit::Bytes, _) => "bytes",
    }
}

/// The advice of a refusal, as one sentence.
fn advice_sentence(advice: &[PathAdvice]) -> String {
    let texts: Vec<&str> = advice.iter().map(|item| item.text()).collect();

    match texts.len() {
        0 => String::new(),
        1 => texts[0].to_string(),
        _ => {
            let (first, last) = texts.split_at(texts.len() - 1);
            format!("{} or {}", first.join(", "), last[0])
        }
    }
}

#[cfg(test)]
mod tests;
