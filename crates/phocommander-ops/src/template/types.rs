//! The data of a rename rule.
//!
//! WHAT: the rule of a rename ([{RenameRule}]), the facts of the files of a
//!       selection ([{RenameInput}]), the outcome of the formation for one
//!       file ([{FormedName}]) and the failure of the formation
//!       ([{TemplateError}]).
//! WHY: a rule is data and not behaviour. It carries the format of the new
//!       name, the replacement rules, the counter and the values of a series,
//!       and it can be stored, read back and compared without running
//!       anything. The behaviour lives in the module
//!       [{crate::template}], so that the dialog can hold a rule, show it to
//!       the user and hand it to the engine without knowing how the engine
//!       works.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::template::{RenameInput, RenameRule};
//!
//! let rule = RenameRule::new("[name] - [n].[ext]");
//! let input = RenameInput::new("/photos/sunset.jpg");
//! assert_eq!(rule.format, "[name] - [n].[ext]");
//! ```
//!
//! BOUNDARY: this file describes data. It reads no file, it forms no name
//!       and it draws nothing.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::NameProblem;
use crate::rename::RenamePair;
use crate::series::SeriesNumbers;
use crate::template::counter::CounterRule;
use crate::template::replace::ReplaceRule;
use crate::template::series::SeriesRule;

/// The rule of a multiple rename.
///
/// WHY: a rule is data and not behaviour: it carries the format of the new
///       name, the replacement rules, the counter and the values of a series.
///       The behaviour lives in the functions of this module, so that a rule
///       can be stored, read back and compared without running anything.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::template::RenameRule;
///
/// let rule = RenameRule::new("[name] - [n].[ext]");
/// assert_eq!(rule.format, "[name] - [n].[ext]");
/// assert!(rule.counter.is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenameRule {
    /// The format of the new name.
    ///
    /// WHY: the format carries the placeholders that name the parts of the
    ///       new name. A rule that renames a series episode may leave it
    ///       empty — the standard format of a series is then used.
    pub format: String,

    /// The rules that search a text and replace it (FR-31).
    pub replacements: Vec<ReplaceRule>,

    /// The counter that numbers the files of the selection (FR-32).
    pub counter: Option<CounterRule>,

    /// The values of a series, if the files belong to one (FR-37, FR-38).
    pub series: Option<SeriesRule>,
}

impl RenameRule {
    /// Builds a rule with a format.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::RenameRule;
    ///
    /// let rule = RenameRule::new("[name].[ext]");
    /// assert_eq!(rule.effective_format(), "[name].[ext]");
    /// ```
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            replacements: Vec::new(),
            counter: None,
            series: None,
        }
    }

    /// Adds a rule that searches a text and replaces it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{RenameRule, ReplaceRule};
    ///
    /// let rule = RenameRule::new("[name]").with_replacement(ReplaceRule::literal("a", "b"));
    /// assert_eq!(rule.replacements.len(), 1);
    /// ```
    pub fn with_replacement(mut self, replacement: ReplaceRule) -> Self {
        self.replacements.push(replacement);
        self
    }

    /// Adds the counter that numbers the files.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{CounterRule, RenameRule};
    ///
    /// let rule = RenameRule::new("[n]").with_counter(CounterRule::new(1, 1, 3));
    /// assert_eq!(rule.counter.map(|counter| counter.width), Some(3));
    /// ```
    pub fn with_counter(mut self, counter: CounterRule) -> Self {
        self.counter = Some(counter);
        self
    }

    /// Adds the values of a series.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{RenameRule, SeriesRule};
    ///
    /// let rule = RenameRule::new("").with_series(SeriesRule::new().with_season(1));
    /// let format = rule.effective_format();
    /// assert!(format.contains("[season]"));
    /// assert!(format.contains("[episode]"));
    /// ```
    pub fn with_series(mut self, series: SeriesRule) -> Self {
        self.series = Some(series);
        self
    }

    /// The format that the formation uses.
    ///
    /// WHY: a rule that renames a series episode and carries no format of its
    ///       own uses the standard format of a series.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{RenameRule, SeriesRule};
    ///
    /// let rule = RenameRule::new("").with_series(SeriesRule::new());
    /// assert!(rule.effective_format().contains("[season]"));
    /// ```
    pub fn effective_format(&self) -> &str {
        if !self.format.trim().is_empty() {
            return self.format.as_str();
        }
        match &self.series {
            Some(series) => series.effective_format(),
            None => "",
        }
    }
}

/// What the formation knows about one file of the selection.
///
/// WHY: the formation reads no file. Everything it needs — the size, the
///       time of the last change, the checksum — is handed to it by the
///       caller, which has the file system at hand. That keeps the formation
///       a pure function of its arguments (FR-33) and keeps the reading of a
///       large file out of the loop that draws the preview.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::template::RenameInput;
///
/// let input = RenameInput::new("/photos/sunset.jpg").with_size(4096);
/// assert_eq!(input.size, Some(4096));
/// assert!(input.checksum.is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenameInput {
    /// The path of the file that is renamed.
    pub path: PathBuf,

    /// The size of the file in bytes.
    pub size: Option<u64>,

    /// The time of the last change of the file.
    pub modified: Option<SystemTime>,

    /// The checksum of the content of the file.
    pub checksum: Option<u64>,

    /// The season and the episode the user entered for this file.
    ///
    /// WHY: the dialog offers the numbers it found in the name as the
    ///       pre-filled values (FR-37) and the user may change them. The
    ///       value the user confirmed therefore wins over the value that was
    ///       detected.
    pub series: Option<SeriesNumbers>,

    /// The value of the placeholder `[rand]`, if the caller draws one.
    ///
    /// WHY: the value that the placeholder carries is derived from the name of
    ///       the file, so that the same name always yields the same name
    ///       (FR-33). A caller that wants a real random value — the value
    ///       changes with every run — hands it in here. The caller then owns
    ///       the randomness, and the formation stays a pure function.
    pub random: Option<String>,
}

impl RenameInput {
    /// Builds the facts of a file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::RenameInput;
    ///
    /// let input = RenameInput::new("/photos/sunset.jpg");
    /// assert_eq!(input.path.to_string_lossy(), "/photos/sunset.jpg");
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            size: None,
            modified: None,
            checksum: None,
            series: None,
            random: None,
        }
    }

    /// Adds the size of the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::RenameInput;
    ///
    /// let input = RenameInput::new("/photos/sunset.jpg").with_size(4096);
    /// assert_eq!(input.size, Some(4096));
    /// ```
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Adds the time of the last change of the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::RenameInput;
    /// use std::time::{Duration, SystemTime};
    ///
    /// let time = SystemTime::UNIX_EPOCH + Duration::from_secs(60);
    /// let input = RenameInput::new("/photos/sunset.jpg").with_modified(time);
    /// assert_eq!(input.modified, Some(time));
    /// ```
    pub fn with_modified(mut self, modified: SystemTime) -> Self {
        self.modified = Some(modified);
        self
    }

    /// Adds the checksum of the content of the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::RenameInput;
    ///
    /// let input = RenameInput::new("/photos/sunset.jpg").with_checksum(0x1234_5678);
    /// assert_eq!(input.checksum, Some(0x1234_5678));
    /// ```
    pub fn with_checksum(mut self, checksum: u64) -> Self {
        self.checksum = Some(checksum);
        self
    }

    /// Adds the season and the episode the user entered.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::series::SeriesNumbers;
    /// use phocommander_ops::template::RenameInput;
    ///
    /// let input = RenameInput::new("/photos/episode.avi").with_series(SeriesNumbers {
    ///     season: 1,
    ///     episode: 3,
    /// });
    /// assert_eq!(input.series.map(|numbers| numbers.episode), Some(3));
    /// ```
    pub fn with_series(mut self, series: SeriesNumbers) -> Self {
        self.series = Some(series);
        self
    }

    /// Adds the value of the placeholder `[rand]`.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::RenameInput;
    ///
    /// let input = RenameInput::new("/photos/sunset.jpg").with_random("a1b2c3d4");
    /// assert_eq!(input.random.as_deref(), Some("a1b2c3d4"));
    /// ```
    pub fn with_random(mut self, random: impl Into<String>) -> Self {
        self.random = Some(random.into());
        self
    }
}

/// The outcome of the formation for one file of the selection.
///
/// WHY: a file whose name cannot be formed does not stop the others. The
///       dialog shows the failure next to that file and forms the rest, so
///       that the user sees where the rule does not fit.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::template::{RenameInput, RenameRule, form_names};
/// use std::time::SystemTime;
///
/// let rule = RenameRule::new("[name].[ext]");
/// let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
/// let names = form_names(&rule, &inputs, SystemTime::UNIX_EPOCH)?;
///
/// assert_eq!(names[0].name(), Some("sunset.jpg"));
/// assert!(names[0].problem().is_none());
/// # Ok::<(), phocommander_ops::template::TemplateError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormedName {
    /// The file received a new name.
    Named {
        /// The path of the file that is renamed.
        source: PathBuf,

        /// The new name of the file.
        name: String,

        /// The path the file receives.
        target: PathBuf,
    },

    /// The file could not be named.
    Failed {
        /// The path of the file that could not be renamed.
        source: PathBuf,

        /// The reason why the name could not be formed.
        problem: TemplateError,
    },
}

impl FormedName {
    /// Builds the outcome of a file that received a new name.
    pub(crate) fn named(input: &RenameInput, name: String) -> Self {
        let target = match input.path.parent() {
            Some(parent) => parent.join(name.as_str()),
            None => PathBuf::from(name.as_str()),
        };
        FormedName::Named {
            source: input.path.clone(),
            name,
            target,
        }
    }

    /// Builds the outcome of a file that could not be named.
    pub(crate) fn failed(input: &RenameInput, problem: TemplateError) -> Self {
        FormedName::Failed {
            source: input.path.clone(),
            problem,
        }
    }

    /// The path of the file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{RenameInput, RenameRule, form_names};
    /// use std::time::SystemTime;
    ///
    /// let rule = RenameRule::new("[name].[ext]");
    /// let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
    /// let names = form_names(&rule, &inputs, SystemTime::UNIX_EPOCH)?;
    ///
    /// assert_eq!(names[0].source().to_string_lossy(), "/photos/sunset.jpg");
    /// # Ok::<(), phocommander_ops::template::TemplateError>(())
    /// ```
    pub fn source(&self) -> &Path {
        match self {
            FormedName::Named { source, .. } => source.as_path(),
            FormedName::Failed { source, .. } => source.as_path(),
        }
    }

    /// The new name of the file, if it could be named.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{RenameInput, RenameRule, form_names};
    /// use std::time::SystemTime;
    ///
    /// let rule = RenameRule::new("[name].[ext]");
    /// let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
    /// let names = form_names(&rule, &inputs, SystemTime::UNIX_EPOCH)?;
    ///
    /// assert_eq!(names[0].name(), Some("sunset.jpg"));
    /// # Ok::<(), phocommander_ops::template::TemplateError>(())
    /// ```
    pub fn name(&self) -> Option<&str> {
        match self {
            FormedName::Named { name, .. } => Some(name.as_str()),
            FormedName::Failed { .. } => None,
        }
    }

    /// The path the file receives, if it could be named.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{RenameInput, RenameRule, form_names};
    /// use std::time::SystemTime;
    ///
    /// let rule = RenameRule::new("[name].[ext]");
    /// let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
    /// let names = form_names(&rule, &inputs, SystemTime::UNIX_EPOCH)?;
    ///
    /// assert_eq!(names[0].target().unwrap().to_string_lossy(), "/photos/sunset.jpg");
    /// # Ok::<(), phocommander_ops::template::TemplateError>(())
    /// ```
    pub fn target(&self) -> Option<&Path> {
        match self {
            FormedName::Named { target, .. } => Some(target.as_path()),
            FormedName::Failed { .. } => None,
        }
    }

    /// The reason why the file could not be named.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{RenameInput, RenameRule, form_names};
    /// use std::time::SystemTime;
    ///
    /// let rule = RenameRule::new("[size]");
    /// let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
    /// let names = form_names(&rule, &inputs, SystemTime::UNIX_EPOCH)?;
    ///
    /// assert!(names[0].problem().is_some());
    /// # Ok::<(), phocommander_ops::template::TemplateError>(())
    /// ```
    pub fn problem(&self) -> Option<&TemplateError> {
        match self {
            FormedName::Named { .. } => None,
            FormedName::Failed { problem, .. } => Some(problem),
        }
    }

    /// The pair of the rename, if the file could be named.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::{RenameInput, RenameRule, form_names};
    /// use std::time::SystemTime;
    ///
    /// let rule = RenameRule::new("[name].bak");
    /// let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
    /// let names = form_names(&rule, &inputs, SystemTime::UNIX_EPOCH)?;
    ///
    /// assert_eq!(names[0].pair().unwrap().target.to_string_lossy(), "/photos/sunset.bak");
    /// # Ok::<(), phocommander_ops::template::TemplateError>(())
    /// ```
    pub fn pair(&self) -> Option<RenamePair> {
        match self {
            FormedName::Named { source, target, .. } => {
                Some(RenamePair::new(source.clone(), target.clone()))
            }
            FormedName::Failed { .. } => None,
        }
    }
}

/// The reason why a name could not be formed.
///
/// WHY: a name that cannot be formed is a result and never a panic. The
///       reasons are the ones the user can act on: a rule without a format, a
///       value that the caller did not supply, a broken regular expression
///       and a name that no platform accepts.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::template::{RenameInput, RenameRule, form_names};
/// use std::time::SystemTime;
///
/// let rule = RenameRule::new("[hash]");
/// let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
/// let names = form_names(&rule, &inputs, SystemTime::UNIX_EPOCH)?;
///
/// match names[0].problem() {
///     Some(problem) => assert!(problem.to_string().contains("hash")),
///     None => panic!("the problem should have been reported"),
/// }
/// # Ok::<(), phocommander_ops::template::TemplateError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TemplateError {
    /// The rule carries no format.
    #[error("the rule carries no format")]
    EmptyFormat,

    /// The value of a placeholder is not known.
    #[error("the value of the placeholder {placeholder} is not known for {path}")]
    UnknownValue {
        /// The name of the placeholder.
        placeholder: &'static str,

        /// The path of the file the value was missing for.
        path: PathBuf,
    },

    /// The pattern of a replacement rule is not a valid regular expression.
    #[error("the pattern {pattern} is not a valid regular expression: {message}")]
    InvalidPattern {
        /// The pattern that could not be read.
        pattern: String,

        /// The message of the regular expression engine.
        message: String,
    },

    /// The formed name cannot be used as a file name.
    #[error("the name {name} cannot be used: {problem}")]
    InvalidName {
        /// The name that was refused.
        name: String,

        /// The reason why the name was refused.
        problem: NameProblem,
    },
}
