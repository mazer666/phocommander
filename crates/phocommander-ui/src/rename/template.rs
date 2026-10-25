//! The named templates of the multiple rename.
//!
//! WHAT: a template with a name ([`NamedTemplate`]), the seam that stores the
//!       templates ([`TemplateStore`]), the reasons why a store fails
//!       ([`TemplateStoreError`]) and the text in which the templates are
//!       written ([`encode_templates`], [`decode_templates`]).
//! WHY: a named template can be stored and applied again (FR-36). The dialog
//!       holds the list of the templates and writes through the seam; the
//!       composition root decides where the templates live — in a file of the
//!       configuration directory, for example. The text of the file belongs
//!       to the interface crate as well, so that the composition root moves
//!       bytes and does not know the format of a template.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::rename::{NamedTemplate, RenameRule, decode_templates, encode_templates};
//!
//! let template = NamedTemplate::new("Camera", RenameRule::new("[name]_[n].[ext]"));
//! let text = encode_templates(&[template]);
//!
//! let read = decode_templates(&text).unwrap_or_default();
//! assert_eq!(read.len(), 1);
//! assert_eq!(read[0].name, "Camera");
//! ```

use std::path::PathBuf;

use crate::i18n::TextKey;
use crate::rename::rule::RenameRule;

/// A template with a name.
///
/// WHY: a template is a rule under a name. The name is what the user sees in
///       the list of the dialog, and the rule is what the engine forms the
///       names with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedTemplate {
    /// The name of the template.
    pub name: String,

    /// The rule that the template carries.
    pub rule: RenameRule,
}

impl NamedTemplate {
    /// Builds a template.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{NamedTemplate, RenameRule};
    ///
    /// let template = NamedTemplate::new("Camera", RenameRule::new("[name]_[n].[ext]"));
    /// assert_eq!(template.name, "Camera");
    /// ```
    pub fn new(name: impl Into<String>, rule: RenameRule) -> Self {
        Self {
            name: name.into(),
            rule,
        }
    }
}

/// The seam that stores the named templates.
///
/// WHY: the interface crate may not depend on the crate of the
///       configuration, and it performs no file operation of its own. The
///       seam is therefore the place where the templates are stored, and the
///       composition root hands the implementation over.
pub trait TemplateStore {
    /// The templates that are stored.
    ///
    /// WHY: a store that cannot be read answers with the reason and not with
    ///       an empty list that looks like a store without templates.
    fn templates(&self) -> Result<Vec<NamedTemplate>, TemplateStoreError>;

    /// Stores a template under its name.
    ///
    /// WHY: a template with the same name is replaced. A name stands for one
    ///       rule, and a second rule under the same name would make the list
    ///       of the dialog ambiguous.
    fn store(&self, template: &NamedTemplate) -> Result<(), TemplateStoreError>;
}

/// The reasons why a store of the templates fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TemplateStoreError {
    /// The stored text carries a line that could not be read.
    #[error("the line {line} of the stored templates could not be read: {reason}")]
    Malformed {
        /// The number of the line, counted from one.
        line: usize,

        /// The reason why the line could not be read.
        reason: String,
    },

    /// The file of the templates could not be read or written.
    #[error("the file {path} of the templates could not be used: {message}")]
    Io {
        /// The path of the file.
        path: PathBuf,

        /// The message of the operating system.
        message: String,
    },
}

impl TemplateStoreError {
    /// The key of the catalogue that carries the message of this error.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::TemplateStoreError;
    /// use phocommander_ui::TextKey;
    ///
    /// let error = TemplateStoreError::malformed(3, "the key is unknown");
    /// assert_eq!(error.message_key(), TextKey::RenameTemplateStoreFailed);
    /// ```
    pub const fn message_key(&self) -> TextKey {
        TextKey::RenameTemplateStoreFailed
    }

    /// Builds the reason of a line that could not be read.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::TemplateStoreError;
    ///
    /// let error = TemplateStoreError::malformed(3, "the key is unknown");
    /// assert_eq!(error.line(), 3);
    /// ```
    pub fn malformed(line: usize, reason: impl Into<String>) -> Self {
        Self::Malformed {
            line,
            reason: reason.into(),
        }
    }

    /// The number of the line that could not be read.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::TemplateStoreError;
    ///
    /// let error = TemplateStoreError::malformed(3, "the key is unknown");
    /// assert_eq!(error.line(), 3);
    /// ```
    pub const fn line(&self) -> usize {
        match self {
            Self::Malformed { line, .. } => *line,
            Self::Io { .. } => 0,
        }
    }
}

/// The head of a stored template.
const TEMPLATE_HEAD: &str = "@";

/// Writes the templates into the text of the store.
///
/// WHY: the format is a line for a value and a line for the head of a
///       template. It is deliberately simple: a user may read the file of
///       the templates and change a rule in it, and a line that the reader
///       does not know is reported instead of being ignored.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::rename::{NamedTemplate, RenameRule, encode_templates};
///
/// let template = NamedTemplate::new("Camera", RenameRule::new("[name]_[n].[ext]"));
/// let text = encode_templates(&[template]);
/// assert!(text.contains("@ Camera"));
/// assert!(text.contains("format [name]_[n].[ext]"));
/// ```
pub fn encode_templates(templates: &[NamedTemplate]) -> String {
    let mut text = String::new();
    text.push_str("# The named templates of the multiple rename.\n");
    for template in templates {
        text.push('\n');
        text.push_str(TEMPLATE_HEAD);
        text.push(' ');
        text.push_str(template.name.as_str());
        text.push('\n');

        let rule = &template.rule;
        text.push_str("format ");
        text.push_str(rule.effective_format());
        text.push('\n');

        for replacement in &rule.replacements {
            text.push_str("search ");
            text.push_str(replacement.search.as_str());
            text.push('\n');
            text.push_str("replace ");
            text.push_str(replacement.replacement.as_str());
            text.push('\n');
            text.push_str("regex ");
            text.push_str(if replacement.regex { "true" } else { "false" });
            text.push('\n');
        }

        if let Some(counter) = &rule.counter {
            text.push_str("counter-start ");
            text.push_str(&counter.start.to_string());
            text.push('\n');
            text.push_str("counter-step ");
            text.push_str(&counter.step.to_string());
            text.push('\n');
            text.push_str("counter-width ");
            text.push_str(&counter.width.to_string());
            text.push('\n');
            text.push_str("counter-order ");
            text.push_str(counter.order.as_str());
            text.push('\n');
        }

        if let Some(series) = &rule.series
            && let Some(name) = &series.series_name
        {
            text.push_str("series ");
            text.push_str(name.as_str());
            text.push('\n');
        }
    }
    text
}

/// Reads the templates out of the text of the store.
///
/// WHY: a line that the reader does not know is an error and not a line that
///       is ignored. A typo in the file of the templates must be visible, or
///       the user would not know why a template is missing.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::rename::decode_templates;
///
/// let text = "@ Camera\nformat [name]_[n].[ext]\n";
/// let templates = decode_templates(text).unwrap_or_default();
/// assert_eq!(templates.len(), 1);
/// assert_eq!(templates[0].rule.format, "[name]_[n].[ext]");
/// ```
pub fn decode_templates(text: &str) -> Result<Vec<NamedTemplate>, TemplateStoreError> {
    let mut templates: Vec<NamedTemplate> = Vec::new();
    let mut current: Option<NamedTemplate> = None;

    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(name) = trimmed.strip_prefix(TEMPLATE_HEAD) {
            if let Some(template) = current.take() {
                templates.push(template);
            }
            let name = name.trim();
            if name.is_empty() {
                return Err(TemplateStoreError::malformed(number, "name-empty"));
            }
            current = Some(NamedTemplate::new(name, RenameRule::default()));
            continue;
        }

        let Some(template) = current.as_mut() else {
            return Err(TemplateStoreError::malformed(
                number,
                "line-before-template",
            ));
        };

        let (key, value) = split_value(trimmed);
        match key {
            "format" => {
                template.rule.format = value.to_string();
            }
            "search" => {
                let replacement = crate::rename::rule::ReplaceRule {
                    search: value.to_string(),
                    replacement: String::new(),
                    regex: false,
                };
                template.rule.replacements.push(replacement);
            }
            "replace" => {
                let Some(last) = template.rule.replacements.last_mut() else {
                    return Err(TemplateStoreError::malformed(
                        number,
                        "replacement-before-search",
                    ));
                };
                last.replacement = value.to_string();
            }
            "regex" => {
                let Some(last) = template.rule.replacements.last_mut() else {
                    return Err(TemplateStoreError::malformed(number, "regex-before-search"));
                };
                last.regex = match value {
                    "true" => true,
                    "false" => false,
                    other => {
                        return Err(TemplateStoreError::malformed(
                            number,
                            format!("value-not-a-boolean:{other}"),
                        ));
                    }
                };
            }
            "counter-start" => {
                let counter = counter_of(&mut template.rule);
                counter.start = number_of(value)
                    .ok_or_else(|| TemplateStoreError::malformed(number, "start-not-a-number"))?;
            }
            "counter-step" => {
                let counter = counter_of(&mut template.rule);
                counter.step = number_of(value)
                    .ok_or_else(|| TemplateStoreError::malformed(number, "step-not-a-number"))?;
            }
            "counter-width" => {
                let counter = counter_of(&mut template.rule);
                counter.width = width_of(value)
                    .ok_or_else(|| TemplateStoreError::malformed(number, "width-not-a-number"))?;
                if counter.width == 0 {
                    return Err(TemplateStoreError::malformed(
                        number,
                        "width-not-greater-than-zero",
                    ));
                }
            }
            "counter-order" => {
                let counter = counter_of(&mut template.rule);
                counter.order = match value {
                    "selection" => crate::rename::rule::CounterOrder::Selection,
                    "name" => crate::rename::rule::CounterOrder::Name,
                    "date" => crate::rename::rule::CounterOrder::Date,
                    other => {
                        return Err(TemplateStoreError::malformed(
                            number,
                            format!("order-unknown:{other}"),
                        ));
                    }
                };
            }
            "series" => {
                let series = series_of(&mut template.rule);
                series.series_name = Some(value.to_string());
            }
            other => {
                return Err(TemplateStoreError::malformed(
                    number,
                    format!("key-unknown:{other}"),
                ));
            }
        }
    }

    if let Some(template) = current.take() {
        templates.push(template);
    }
    Ok(templates)
}

/// Splits a line into its key and its value.
fn split_value(line: &str) -> (&str, &str) {
    match line.split_once(' ') {
        Some((key, value)) => (key, value.trim()),
        None => (line, ""),
    }
}

/// The counter of a rule, created when it carries none.
fn counter_of(rule: &mut RenameRule) -> &mut crate::rename::rule::CounterRule {
    rule.counter
        .get_or_insert_with(crate::rename::rule::CounterRule::default)
}

/// The values of a series of a rule, created when the rule carries none.
fn series_of(rule: &mut RenameRule) -> &mut crate::rename::rule::SeriesRule {
    rule.series
        .get_or_insert_with(crate::rename::rule::SeriesRule::default)
}

/// The number of a value, if the value is a number.
fn number_of(value: &str) -> Option<i64> {
    value.trim().parse::<i64>().ok()
}

/// The width of a value, if the value is a width.
fn width_of(value: &str) -> Option<usize> {
    value.trim().parse::<usize>().ok()
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::rename::rule::{CounterOrder, CounterRule, ReplaceRule};

    #[test]
    fn a_template_survives_the_round_trip_through_the_text() {
        // arrange
        let rule = RenameRule::new("[name]_[n].[ext]")
            .with_replacement(ReplaceRule::pattern(r"^a(\d+)$", "b$1"))
            .with_counter(CounterRule::new(10, 5, 2).with_order(CounterOrder::Name));
        let template = NamedTemplate::new("Camera", rule);

        // act
        let text = encode_templates(std::slice::from_ref(&template));
        let read = decode_templates(&text).unwrap_or_default();

        // assert
        assert_eq!(read.len(), 1);
        let read = match read.first() {
            Some(template) => template,
            None => panic!("the template should have been read back"),
        };
        assert_eq!(read.name, "Camera");
        assert_eq!(read.rule.format, template.rule.format);
        assert_eq!(read.rule.replacements.len(), 1);
        assert_eq!(read.rule.replacements[0].search, r"^a(\d+)$");
        assert_eq!(read.rule.replacements[0].replacement, "b$1");
        assert!(read.rule.replacements[0].regex);
        let counter = match read.rule.counter.as_ref() {
            Some(counter) => counter,
            None => panic!("the counter should have been read back"),
        };
        assert_eq!(counter.start, 10);
        assert_eq!(counter.step, 5);
        assert_eq!(counter.width, 2);
        assert_eq!(counter.order, CounterOrder::Name);
    }

    #[test]
    fn the_name_of_a_series_survives_the_round_trip() {
        // arrange
        let rule = RenameRule::new("[series]")
            .with_series(crate::rename::rule::SeriesRule::new().with_series_name("The Expanse"));
        let template = NamedTemplate::new("Series", rule);

        // act
        let text = encode_templates(&[template]);
        let read = decode_templates(&text).unwrap_or_default();

        // assert
        let series = read
            .first()
            .and_then(|template| template.rule.series.as_ref())
            .and_then(|series| series.series_name.as_deref());
        assert_eq!(series, Some("The Expanse"));
    }

    #[test]
    fn a_line_with_an_unknown_key_is_an_error() {
        // arrange
        let text = "@ Camera\nformat [name]\ncolour red\n";

        // act
        let result = decode_templates(text);

        // assert
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("the unknown key should have been reported"),
        };
        assert_eq!(error.line(), 3);
    }

    #[test]
    fn a_line_before_the_name_of_a_template_is_an_error() {
        // arrange
        let text = "format [name]\n";

        // act
        let result = decode_templates(text);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn a_template_without_a_name_is_an_error() {
        // arrange
        let text = "@\nformat [name]\n";

        // act
        let result = decode_templates(text);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn a_counter_width_of_zero_is_an_error() {
        // arrange
        let text = "@ Camera\ncounter-width 0\n";

        // act
        let result = decode_templates(text);

        // assert
        assert!(result.is_err());
    }

    #[test]
    fn comments_and_empty_lines_are_ignored() {
        // arrange
        let text = "# a comment\n\n@ Camera\n\nformat [name]\n";

        // act
        let templates = decode_templates(text).unwrap_or_default();

        // assert
        assert_eq!(templates.len(), 1);
    }
}
