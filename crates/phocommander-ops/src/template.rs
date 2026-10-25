//! The formation of the new names of a multiple rename.
//!
//! WHAT: the rule of a rename ([{RenameRule}]), the facts of the files of a
//!       selection ([{RenameInput}]), the outcome of the formation for one
//!       file ([{FormedName}]), the failure of the formation
//!       ([{TemplateError}]) and the formation itself ([{form_names}]).
//! WHY: the rename of a selection answers one question — *what is the new
//!       name of this file?* — and the answer has to be the same every time
//!       the same question is asked (FR-33). A name that changes while the
//!       user moves the mouse is a name the user cannot trust. The formation
//!       therefore reads nothing but its arguments: the time of the day comes
//!       from the caller, the size and the checksum of a file come from the
//!       caller, and the value of the placeholder `[rand]` is derived from the
//!       name of the file instead of being drawn from a random source. The
//!       rules themselves live in the files of this directory: the
//!       placeholders in `placeholders.rs`, the search and the replacement in
//!       `replace.rs`, the counter in `counter.rs` and the series in
//!       `series.rs`.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::template::{RenameInput, RenameRule, form_names};
//! use std::time::{Duration, SystemTime};
//!
//! let rule = RenameRule::new("[name] - [n].[ext]");
//! let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
//! let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
//!
//! let names = form_names(&rule, &inputs, now)?;
//! assert_eq!(
//!     names.first().and_then(|formed| formed.name()),
//!     Some("sunset - [n].jpg")
//! );
//! # Ok::<(), phocommander_ops::template::TemplateError>(())
//! ```
//!
//! BOUNDARY: this module forms names. It reads no file, it draws nothing and
//!       it knows no platform. The characters that a platform forbids live in
//!       [{crate::names}]; this module reports them and suggests a
//!       replacement, but it never changes a name on its own. The rename
//!       itself lives in [{crate::rename}].

use std::time::SystemTime;

use crate::error::NameProblem;
use crate::rename::RenamePair;

pub mod counter;
pub mod placeholders;
pub mod replace;
pub mod series;

mod types;

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;

pub use counter::{CounterOrder, CounterRule};
pub use placeholders::Placeholder;
pub use replace::ReplaceRule;
pub use series::SeriesRule;
pub use types::{FormedName, RenameInput, RenameRule, TemplateError};

/// Forms the new name of every file of the selection.
///
/// WHY: the formation is a pure function of the rule, the files and the time
///       (FR-33). The time comes from the caller, because a name that depends
///       on the clock of the library cannot be tested and cannot be
///       reproduced. The same rule and the same files therefore always yield
///       the same names — also after a restart of the program.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::template::{CounterRule, RenameInput, RenameRule, form_names};
/// use std::time::{Duration, SystemTime};
///
/// let rule = RenameRule::new("[n] - [name]").with_counter(CounterRule::new(1, 1, 3));
/// let inputs = vec![
///     RenameInput::new("/photos/a.jpg"),
///     RenameInput::new("/photos/b.jpg"),
/// ];
/// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
///
/// let names = form_names(&rule, &inputs, now)?;
/// assert_eq!(names[0].name(), Some("001 - a"));
/// assert_eq!(names[1].name(), Some("002 - b"));
/// # Ok::<(), phocommander_ops::template::TemplateError>(())
/// ```
pub fn form_names(
    rule: &RenameRule,
    inputs: &[RenameInput],
    now: SystemTime,
) -> Result<Vec<FormedName>, TemplateError> {
    let formation = Formation::prepare(rule, inputs, now)?;
    let mut names = Vec::with_capacity(inputs.len());
    for (index, input) in inputs.iter().enumerate() {
        names.push(formation.form(index, input));
    }
    Ok(names)
}

/// The pairs of the rename of a selection whose names could all be formed.
///
/// WHY: the rename of a selection is atomic (FR-41) — either every file is
///       renamed or none of them is. A selection with a file that could not
///       be named therefore has no pairs at all, and the caller refuses the
///       rename.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::template::{RenameInput, RenameRule, form_names, rename_pairs};
/// use std::time::SystemTime;
///
/// let rule = RenameRule::new("[name].bak");
/// let inputs = vec![RenameInput::new("/photos/sunset.jpg")];
/// let names = form_names(&rule, &inputs, SystemTime::UNIX_EPOCH)?;
///
/// let pairs = rename_pairs(&names);
/// assert_eq!(pairs.unwrap().len(), 1);
/// # Ok::<(), phocommander_ops::template::TemplateError>(())
/// ```
pub fn rename_pairs(names: &[FormedName]) -> Option<Vec<RenamePair>> {
    let mut pairs = Vec::with_capacity(names.len());
    for name in names {
        pairs.push(name.pair()?);
    }
    Some(pairs)
}

/// The characters of a formed name that the target platform forbids, with
/// the replacement that is suggested.
///
/// WHY: the characters that a platform forbids are reported before the
///       rename and a replacement is suggested (FR-40). Nothing is changed
///       silently — the user decides.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ops::template::name_problems;
///
/// let problems = name_problems("A:B");
/// assert_eq!(problems, vec![(':', '-')]);
/// ```
pub fn name_problems(name: &str) -> Vec<(char, char)> {
    crate::names::invalid_characters(name)
}

/// The state that the formation of a whole selection shares.
struct Formation<'a> {
    /// The format of the new name.
    format: &'a str,

    /// The replacement rules, ready to be applied.
    replacements: Vec<replace::Compiled>,

    /// The number of every file of the selection.
    counter: Vec<Option<i64>>,

    /// The values of the series of every file.
    series: Option<Vec<series::SeriesValues>>,

    /// The time of the rename.
    now: SystemTime,

    /// The width of the season number.
    season_width: usize,

    /// The width of the episode number.
    episode_width: usize,

    /// The width of the counter number.
    counter_width: usize,
}

impl<'a> Formation<'a> {
    /// Reads the rule and prepares the state of the whole selection.
    ///
    /// WHY: the numbers of the counter and the values of the series depend on
    ///       the whole selection and not on a single file. The width of the
    ///       season and of the episode number grows with the largest number
    ///       of the selection (FR-39), so that every file of one run carries
    ///       the same width.
    fn prepare(
        rule: &'a RenameRule,
        inputs: &[RenameInput],
        now: SystemTime,
    ) -> Result<Self, TemplateError> {
        let format = rule.effective_format();
        if format.trim().is_empty() {
            return Err(TemplateError::EmptyFormat);
        }

        let replacements = replace::compile(&rule.replacements)?;
        let counter = counter::counter_values(rule.counter.as_ref(), inputs);
        let series = rule.series.as_ref().map(|series_rule| {
            inputs
                .iter()
                .enumerate()
                .map(|(index, input)| series::values_of(series_rule, input, counter[index]))
                .collect::<Vec<_>>()
        });
        let (season_width, episode_width) = match (&series, &rule.series) {
            (Some(values), Some(series_rule)) => series::widths_of(series_rule, values),
            _ => (0, 0),
        };
        let counter_width = rule.counter.as_ref().map_or(0, |counter| counter.width);

        Ok(Self {
            format,
            replacements,
            counter,
            series,
            now,
            season_width,
            episode_width,
            counter_width,
        })
    }

    /// Forms the new name of one file of the selection.
    ///
    /// WHY: the three parts of the rule are evaluated in the order of the
    ///       documentation: first the placeholders, then the search and the
    ///       replacement, and last the counter. The counter is written last so
    ///       that a replacement rule cannot change a number that the user
    ///       wants to keep.
    fn form(&self, index: usize, input: &RenameInput) -> FormedName {
        let values = placeholders::Values {
            input,
            now: self.now,
            counter: self.counter[index],
            series: self.series.as_ref().and_then(|values| values.get(index)),
            season_width: self.season_width,
            episode_width: self.episode_width,
        };

        let text = match placeholders::evaluate(self.format, &values) {
            Ok(text) => text,
            Err(problem) => return FormedName::failed(input, problem),
        };
        let text = replace::apply(&self.replacements, &text);
        let name = counter::apply(&text, values.counter, self.counter_width);

        let valid = validate_name(name.as_str());
        match valid {
            Ok(()) => FormedName::named(input, name),
            Err(problem) => FormedName::failed(input, TemplateError::InvalidName { name, problem }),
        }
    }
}

/// Checks whether the name can be used as a file name.
///
/// WHY: a name that is empty or that carries a separator names a directory
///       and not a file. Such a name is refused before the rename runs, so
///       that the failure is a result and not a half finished rename.
fn validate_name(name: &str) -> Result<(), NameProblem> {
    if name.is_empty() {
        return Err(NameProblem::Empty);
    }
    if name.contains('/') || name.contains('\\') {
        return Err(NameProblem::PathSeparator);
    }
    Ok(())
}

/// The name of the file without its directory.
pub(crate) fn file_name_of(input: &RenameInput) -> String {
    input
        .path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// The name of the file without its directory and without its extension.
pub(crate) fn base_name_of(input: &RenameInput) -> String {
    match input.path.file_stem() {
        Some(stem) => stem.to_string_lossy().to_string(),
        None => file_name_of(input),
    }
}

/// The extension of the file without the dot.
pub(crate) fn extension_of(input: &RenameInput) -> String {
    input
        .path
        .extension()
        .map(|extension| extension.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// The name of the directory the file lives in.
pub(crate) fn parent_name_of(input: &RenameInput) -> String {
    input
        .path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}
