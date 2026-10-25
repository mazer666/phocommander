//! The rule of a rename as the dialog edits it.
//!
//! WHAT: the rule of a rename ([`RenameRule`]), the search and the
//!       replacement ([`ReplaceRule`]), the counter ([`CounterRule`]), the
//!       order in which the files receive their number ([`CounterOrder`]) and
//!       the values of a series ([`SeriesRule`]).
//! WHY: the dialog edits a rule and shows it to the user. The engine that
//!       forms the names lives in the crate of the operations, and the
//!       interface crate may not depend on a sibling crate of the same level.
//!       The rule is therefore data here — it can be stored, read back and
//!       compared without running anything — and the composition root
//!       translates it into the rule of the engine (the seam of
//!       [`crate::rename::engine`]).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::rename::RenameRule;
//!
//! let rule = RenameRule::new("[name] - [n].[ext]");
//! assert_eq!(rule.effective_format(), "[name] - [n].[ext]");
//! assert!(rule.counter.is_none());
//! ```

/// The rule of a rename.
///
/// WHY: a rule is data and not behaviour. It carries the format of the new
///       name, the replacement rules, the counter and the values of a series,
///       and it can be stored, read back and compared without running
///       anything.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RenameRule {
    /// The format of the new name.
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
    /// use phocommander_ui::rename::RenameRule;
    ///
    /// let rule = RenameRule::new("[name].[ext]");
    /// assert_eq!(rule.format, "[name].[ext]");
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
    /// use phocommander_ui::rename::{RenameRule, ReplaceRule};
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
    /// use phocommander_ui::rename::{CounterRule, RenameRule};
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
    /// use phocommander_ui::rename::{RenameRule, SeriesRule};
    ///
    /// let rule = RenameRule::new("[series]").with_series(SeriesRule::new());
    /// assert!(rule.series.is_some());
    /// ```
    pub fn with_series(mut self, series: SeriesRule) -> Self {
        self.series = Some(series);
        self
    }

    /// The format that the formation uses.
    ///
    /// WHY: a rule that renames the episodes of a series and carries no
    ///       format of its own uses the format of its series.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::RenameRule;
    ///
    /// let rule = RenameRule::new("");
    /// assert_eq!(rule.effective_format(), "");
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

/// A rule that searches a text and replaces it.
///
/// WHY: the switch that turns the search into a regular expression is off by
///       default (FR-31). A search that is not a regular expression is what
///       a user expects when the text carries a dot or a bracket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplaceRule {
    /// The text that is searched.
    pub search: String,

    /// The text that takes its place.
    pub replacement: String,

    /// Whether the search is a regular expression.
    pub regex: bool,
}

impl ReplaceRule {
    /// Builds a rule that searches the text literally.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::ReplaceRule;
    ///
    /// let rule = ReplaceRule::literal("a.b", "c");
    /// assert_eq!(rule.search, "a.b");
    /// assert!(!rule.regex);
    /// ```
    pub fn literal(search: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            search: search.into(),
            replacement: replacement.into(),
            regex: false,
        }
    }

    /// Builds a rule that searches the text with a regular expression.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::ReplaceRule;
    ///
    /// let rule = ReplaceRule::pattern(r"^a(\d+)$", "b$1");
    /// assert!(rule.regex);
    /// ```
    pub fn pattern(search: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            search: search.into(),
            replacement: replacement.into(),
            regex: true,
        }
    }
}

/// The rule of the counter.
///
/// WHY: the counter numbers the files of a selection (FR-32): it starts at a
///       value, it grows by a step, and it is written with a fixed number of
///       digits, so that the files of one run carry the same width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterRule {
    /// The number of the first file.
    pub start: i64,

    /// The distance between two numbers.
    pub step: i64,

    /// The number of digits the number is written with.
    pub width: usize,

    /// The order in which the files receive the number.
    pub order: CounterOrder,
}

impl Default for CounterRule {
    /// Builds the counter that numbers the files from one.
    fn default() -> Self {
        Self {
            start: 1,
            step: 1,
            width: 3,
            order: CounterOrder::Selection,
        }
    }
}

impl CounterRule {
    /// Builds a counter.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::CounterRule;
    ///
    /// let counter = CounterRule::new(10, 5, 2);
    /// assert_eq!(counter.start, 10);
    /// assert_eq!(counter.width, 2);
    /// ```
    pub fn new(start: i64, step: i64, width: usize) -> Self {
        Self {
            start,
            step,
            width,
            order: CounterOrder::Selection,
        }
    }

    /// Sets the order in which the files receive the number.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{CounterOrder, CounterRule};
    ///
    /// let counter = CounterRule::new(1, 1, 3).with_order(CounterOrder::Name);
    /// assert_eq!(counter.order, CounterOrder::Name);
    /// ```
    pub fn with_order(mut self, order: CounterOrder) -> Self {
        self.order = order;
        self
    }
}

/// The order in which the files of a selection receive their number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CounterOrder {
    /// The order of the selection.
    #[default]
    Selection,

    /// The order of the names of the files.
    Name,

    /// The order of the time of the last change of the files.
    Date,
}

impl CounterOrder {
    /// The name of the order, as the configuration file writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::CounterOrder;
    ///
    /// assert_eq!(CounterOrder::Selection.as_str(), "selection");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            CounterOrder::Selection => "selection",
            CounterOrder::Name => "name",
            CounterOrder::Date => "date",
        }
    }
}

/// The values of a series.
///
/// WHY: the name of a series is the name of the directory the files live in
///       — but a series that is filed in a collection folder has a directory
///       that is not its name. The value is therefore always shown before the
///       rename and can be overridden (FR-38).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesRule {
    /// The name of the series.
    pub series_name: Option<String>,

    /// The season of the files whose name carries no number.
    pub season: Option<u64>,

    /// The episode of the files whose name carries no number.
    pub episode: Option<u64>,

    /// The format of the new name, if the series carries one of its own.
    pub format: Option<String>,

    /// The width of the season number.
    pub season_width: usize,

    /// The width of the episode number.
    pub episode_width: usize,
}

impl Default for SeriesRule {
    /// Builds the values of a series with the standard widths.
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
    /// Builds the values of a series.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::SeriesRule;
    ///
    /// let series = SeriesRule::new();
    /// assert_eq!(series.season_width, 2);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of the series.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::SeriesRule;
    ///
    /// let series = SeriesRule::new().with_series_name("The Expanse");
    /// assert_eq!(series.series_name.as_deref(), Some("The Expanse"));
    /// ```
    pub fn with_series_name(mut self, name: impl Into<String>) -> Self {
        self.series_name = Some(name.into());
        self
    }

    /// Sets the format of the new name.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::SeriesRule;
    ///
    /// let series = SeriesRule::new().with_format("[series].[ext]");
    /// assert_eq!(series.effective_format(), "[series].[ext]");
    /// ```
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// The format that the formation uses.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::SeriesRule;
    ///
    /// assert_eq!(SeriesRule::new().effective_format(), "");
    /// ```
    pub fn effective_format(&self) -> &str {
        self.format.as_deref().unwrap_or("")
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn a_rule_without_a_format_uses_the_format_of_its_series() {
        // arrange
        let series = SeriesRule::new().with_format("[series].[ext]");
        let rule = RenameRule::new("").with_series(series);

        // act
        let format = rule.effective_format();

        // assert
        assert_eq!(format, "[series].[ext]");
    }

    #[test]
    fn the_format_of_the_rule_wins_over_the_format_of_the_series() {
        // arrange
        let series = SeriesRule::new().with_format("[series].[ext]");
        let rule = RenameRule::new("[name].[ext]").with_series(series);

        // act
        let format = rule.effective_format();

        // assert
        assert_eq!(format, "[name].[ext]");
    }

    #[test]
    fn the_counter_of_the_rule_carries_its_three_numbers() {
        // arrange
        let rule = RenameRule::new("[n]").with_counter(CounterRule::new(10, 5, 2));

        // act
        let counter = rule.counter;

        // assert
        let counter = match counter {
            Some(counter) => counter,
            None => panic!("the rule should carry a counter"),
        };
        assert_eq!(counter.start, 10);
        assert_eq!(counter.step, 5);
        assert_eq!(counter.width, 2);
        assert_eq!(counter.order, CounterOrder::Selection);
    }

    #[test]
    fn every_order_of_the_counter_has_a_name_of_its_own() {
        // act
        let names = [
            CounterOrder::Selection.as_str(),
            CounterOrder::Name.as_str(),
            CounterOrder::Date.as_str(),
        ];

        // assert
        assert_eq!(names, ["selection", "name", "date"]);
    }
}
