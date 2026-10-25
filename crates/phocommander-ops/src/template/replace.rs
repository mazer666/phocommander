//! The search and the replacement of a rename rule.
//!
//! WHAT: the rule that searches a text and replaces it ([{ReplaceRule}]) and
//!       the compiled rules that the formation applies.
//! WHY: the dialog offers a search text, a replacement text and a switch that
//!       turns the search into a regular expression (FR-31). The switch is off
//!       by default: a search that is not a regular expression is what a user
//!       expects when the text carries a dot or a bracket. A pattern that the
//!       engine cannot read is an error and not a silent change of the rule.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ops::template::ReplaceRule;
//!
//! let literal = ReplaceRule::literal("a", "b");
//! assert!(!literal.regex);
//!
//! let pattern = ReplaceRule::pattern(r"^a(\d+)$", "b$1");
//! assert!(pattern.regex);
//! ```
//!
//! BOUNDARY: this module replaces a text in a text. It reads no file and it
//!       knows nothing about the names of files.

use regex::Regex;

use crate::template::TemplateError;

/// A rule that searches a text and replaces it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceRule {
    /// The text that is searched.
    pub search: String,

    /// The text that takes its place.
    pub replacement: String,

    /// Whether the search is a regular expression.
    ///
    /// WHY: with the switch off the text is matched literally, so that a dot
    ///       in the search text is a dot and not "any character". With the
    ///       switch on the text is a regular expression, and the replacement
    ///       may name a group of the pattern with `$1`.
    pub regex: bool,
}

impl ReplaceRule {
    /// Builds a rule that searches the text literally.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ops::template::ReplaceRule;
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
    /// use phocommander_ops::template::ReplaceRule;
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

/// A rule that is ready to be applied.
pub(crate) enum Compiled {
    /// The text is searched literally.
    Literal {
        /// The text that is searched.
        search: String,

        /// The text that takes its place.
        replacement: String,
    },

    /// The text is searched with a regular expression.
    Pattern {
        /// The pattern that is searched.
        expression: Regex,

        /// The text that takes its place.
        replacement: String,
    },
}

/// Reads the rules of a rename rule and prepares them.
///
/// WHY: the patterns are read once for the whole selection and not once per
///       file. A pattern that the engine cannot read is an error that stops
///       the formation, so that a broken rule changes no name at all.
pub(crate) fn compile(rules: &[ReplaceRule]) -> Result<Vec<Compiled>, TemplateError> {
    let mut compiled = Vec::with_capacity(rules.len());
    for rule in rules {
        // WHY: a search that is empty would match at every place of the name
        // and would fill it with the replacement. A rule without a search
        // text does nothing.
        if rule.search.is_empty() {
            continue;
        }
        if rule.regex {
            match Regex::new(rule.search.as_str()) {
                Ok(expression) => compiled.push(Compiled::Pattern {
                    expression,
                    replacement: rule.replacement.clone(),
                }),
                Err(error) => {
                    return Err(TemplateError::InvalidPattern {
                        pattern: rule.search.clone(),
                        message: error.to_string(),
                    });
                }
            }
        } else {
            compiled.push(Compiled::Literal {
                search: rule.search.clone(),
                replacement: rule.replacement.clone(),
            });
        }
    }
    Ok(compiled)
}

/// Applies the rules to a text.
///
/// WHY: the rules are applied one after the other, in the order in which the
///       user wrote them. A rule therefore sees what the rule before it has
///       written.
pub(crate) fn apply(rules: &[Compiled], text: &str) -> String {
    let mut result = text.to_string();
    for rule in rules {
        result = match rule {
            Compiled::Literal {
                search,
                replacement,
            } => result.replace(search.as_str(), replacement.as_str()),
            Compiled::Pattern {
                expression,
                replacement,
            } => expression
                .replace_all(result.as_str(), replacement.as_str())
                .into_owned(),
        };
    }
    result
}
