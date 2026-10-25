//! The series template of a rename rule.
//!
//! WHAT: the rule of a series ([{SeriesRule}]), the values of a series for
//!       one file ([{SeriesValues}]) and the widths of its numbers.
//! WHY: the files of a series episode carry their numbers in their name —
//!       "The.Expanse.S01E03.avi". The rule reads these numbers out of the
//!       name (FR-37), takes the name of the series from the directory the
//!       files live in (FR-38) and writes the numbers with a fixed width
//!       that grows as soon as the selection carries a number with more
//!       digits (FR-39). The name of the series and the numbers may be
//!       overridden by the user, because a series that is filed in a
//!       collection folder has a directory that is not its name.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::template::SeriesRule;
//!
//! let rule = SeriesRule::new().with_series_name("The Expanse").with_season(1);
//! assert_eq!(rule.effective_format(), "[series] - S[season]E[episode] - [episode_title].[ext]");
//! assert_eq!(rule.season, Some(1));
//! ```
//!
//! BOUNDARY: this module reads the numbers of a series out of a name and
//!       writes them back. It reads no file and it draws nothing.

use crate::series::{SeriesPattern, detect_series_pattern};
use crate::template::{RenameInput, base_name_of, file_name_of, parent_name_of};

/// The format that a series rename uses when the rule carries none.
pub const STANDARD_FORMAT: &str = "[series] - S[season]E[episode] - [episode_title].[ext]";

/// The rule of a series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesRule {
    /// The name of the series.
    ///
    /// WHY: the name of the series is the name of the directory the files
    ///       live in — but a series that is filed in a collection folder has
    ///       a directory that is not its name. The value is therefore always
    ///       shown before the rename and can be overridden (FR-38).
    pub series_name: Option<String>,

    /// The season of the files whose name carries no pattern.
    pub season: Option<u64>,

    /// The episode of the files whose name carries no pattern.
    pub episode: Option<u64>,

    /// The format of the new name.
    ///
    /// WHY: a rule that carries no format uses the standard format of a
    ///       series, so that the dialog needs no further knowledge.
    pub format: Option<String>,

    /// The width of the season number.
    pub season_width: usize,

    /// The width of the episode number.
    pub episode_width: usize,
}

impl Default for SeriesRule {
    /// Builds the rule of a series with the standard widths.
    fn default() -> Self {
        Self {
            series_name: None,
            season: None,
            episode: None,
            format: None,
            season_width: 2,
            episode_width: 2,
        }
    }
}

impl SeriesRule {
    /// Builds the rule of a series.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::SeriesRule;
    ///
    /// let rule = SeriesRule::new();
    /// assert_eq!(rule.season_width, 2);
    /// assert_eq!(rule.episode_width, 2);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of the series.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::SeriesRule;
    ///
    /// let rule = SeriesRule::new().with_series_name("The Expanse");
    /// assert_eq!(rule.series_name.as_deref(), Some("The Expanse"));
    /// ```
    pub fn with_series_name(mut self, name: impl Into<String>) -> Self {
        self.series_name = Some(name.into());
        self
    }

    /// Sets the season of the files whose name carries no pattern.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::SeriesRule;
    ///
    /// let rule = SeriesRule::new().with_season(2);
    /// assert_eq!(rule.season, Some(2));
    /// ```
    pub fn with_season(mut self, season: u64) -> Self {
        self.season = Some(season);
        self
    }

    /// Sets the episode of the files whose name carries no pattern.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::SeriesRule;
    ///
    /// let rule = SeriesRule::new().with_episode(3);
    /// assert_eq!(rule.episode, Some(3));
    /// ```
    pub fn with_episode(mut self, episode: u64) -> Self {
        self.episode = Some(episode);
        self
    }

    /// Sets the format of the new name.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::SeriesRule;
    ///
    /// let rule = SeriesRule::new().with_format("[series] - [episode_title].[ext]");
    /// assert_eq!(rule.effective_format(), "[series] - [episode_title].[ext]");
    /// ```
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// The format that the formation uses.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::SeriesRule;
    ///
    /// assert_eq!(
    ///     SeriesRule::new().effective_format(),
    ///     "[series] - S[season]E[episode] - [episode_title].[ext]"
    /// );
    /// ```
    pub fn effective_format(&self) -> &str {
        self.format.as_deref().unwrap_or(STANDARD_FORMAT)
    }
}

/// The values of a series for one file of the selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SeriesValues {
    /// The name of the series.
    pub name: String,

    /// The number of the season, if it is known.
    pub season: Option<u64>,

    /// The number of the episode, if it is known.
    pub episode: Option<u64>,

    /// The title of the episode — the name of the file without the pattern of
    /// the season and the episode.
    pub title: String,
}

/// Reads the values of the series for one file.
///
/// WHY: the numbers are looked for in the name of the file first (FR-37).
///       The value that the user confirmed wins over the value that was
///       found, and the counter of the rule numbers the files whose name
///       carries no pattern at all — the dialog asks for the first episode
///       and the counter continues the series.
pub(crate) fn values_of(
    rule: &SeriesRule,
    input: &RenameInput,
    counter: Option<i64>,
) -> SeriesValues {
    let pattern = detect_series_pattern(&file_name_of(input));
    let name = rule
        .series_name
        .clone()
        .unwrap_or_else(|| parent_name_of(input));
    let season = input
        .series
        .map(|numbers| numbers.season)
        .or_else(|| pattern.map(|pattern| pattern.season))
        .or(rule.season);
    let episode = input
        .series
        .map(|numbers| numbers.episode)
        .or_else(|| pattern.map(|pattern| pattern.episode))
        .or_else(|| counter.and_then(|value| u64::try_from(value).ok()))
        .or(rule.episode);
    let title = episode_title(&base_name_of(input), pattern.as_ref());

    SeriesValues {
        name,
        season,
        episode,
        title,
    }
}

/// The widths of the season and of the episode number.
///
/// WHY: the width grows as soon as the selection carries a number with more
///       digits (FR-39), so that every file of one run is named with the
///       same width. The files of a run that would be named "S01E099" and
///       "S01E100" therefore both carry three digits.
pub(crate) fn widths_of(rule: &SeriesRule, values: &[SeriesValues]) -> (usize, usize) {
    let season = values
        .iter()
        .filter_map(|values| values.season)
        .max()
        .map_or(0, digits);
    let episode = values
        .iter()
        .filter_map(|values| values.episode)
        .max()
        .map_or(0, digits);
    (
        rule.season_width.max(season),
        rule.episode_width.max(episode),
    )
}

/// The title of the episode — the name of the file without the pattern.
///
/// WHY: the name of an episode that is already filed carries the pattern of
///       the season and the episode between the parts of the name — "Show.
///       S01E03.Title". The title is what is left of it.
pub(crate) fn episode_title(base: &str, pattern: Option<&SeriesPattern>) -> String {
    let Some(pattern) = pattern else {
        return base.to_string();
    };
    // WHY: the pattern is looked for in the name of the file, and the title
    //       is cut out of the name without the extension. A pattern that
    //       lies behind the extension therefore reaches beyond the title.
    if pattern.end > base.len() {
        return base.to_string();
    }

    let before = &base[..pattern.start];
    let after = &base[pattern.end..];
    let mut before_characters: Vec<char> = before.chars().collect();
    let after_characters: Vec<char> = after.chars().collect();
    // WHY: removing the pattern leaves two separators next to each other,
    //       and the title would read "Show..Title".
    if before_characters
        .last()
        .is_some_and(|character| is_separator(*character))
        && after_characters
            .first()
            .is_some_and(|character| is_separator(*character))
    {
        before_characters.pop();
    }

    let title: String = before_characters
        .into_iter()
        .chain(after_characters)
        .collect();
    title.trim_matches(is_separator).to_string()
}

/// Whether the character separates the parts of a name.
fn is_separator(character: char) -> bool {
    matches!(character, '.' | ' ' | '-' | '_')
}

/// The number of digits of a number.
fn digits(value: u64) -> usize {
    let mut count = 1;
    let mut rest = value;
    while rest >= 10 {
        rest /= 10;
        count += 1;
    }
    count
}
