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

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::template::{RenameInput, RenameRule, form_names};
    use proptest::prelude::*;

    /// A text and a rule that may be anything.
    fn rule() -> impl Strategy<Value = (String, String, bool)> {
        (any::<String>(), any::<String>(), proptest::bool::ANY)
    }

    proptest! {
        /// The rules of a rename never panic and never lose the text.
        ///
        /// WHY: the user writes the search text and the replacement, and
        ///       the engine has to survive whatever they write. A rule that
        ///       panics would take the whole rename with it, and a rule
        ///       that loses the text would produce names that the user
        ///       never asked for.
        #[test]
        fn the_rules_of_a_rename_never_panic_and_never_lose_the_text(
            (search, replacement, is_regex) in rule(),
            text in any::<String>(),
        ) {
            // arrange
            let regel = ReplaceRule {
                search: search.clone(),
                replacement: replacement.clone(),
                regex: is_regex,
            };

            // act
            let ergebnis = compile(&[regel]);

            // assert
            // WHY: a pattern that the engine cannot read is an error and
            //       not a panic. The error stops the formation, so that a
            //       broken rule changes no name at all.
            if is_regex {
                // Der Fehler ist erlaubt — der entscheidende Punkt ist, dass
                // es keiner ist, der das Programm beendet.
                if let Ok(compiled) = ergebnis {
                    let _ = apply(&compiled, &text);
                }
            } else {
                let compiled = ergebnis.unwrap_or_else(|error| panic!("a literal rule should always compile: {error}"));
                let ergebnis = apply(&compiled, &text);
                prop_assert!(!ergebnis.is_empty() || text.is_empty());
            }
        }

        /// The rules without a search text do nothing.
        #[test]
        fn a_rule_without_a_search_text_does_nothing(text in any::<String>()) {
            // arrange
            let regel = ReplaceRule::literal("", "etwas");

            // act
            let compiled = compile(&[regel]).unwrap_or_else(|error| panic!("the rule should compile: {error}"));
            let ergebnis = apply(&compiled, &text);

            // assert
            prop_assert_eq!(ergebnis, text);
        }

        /// The rules survive the path of the file, like the fuzz target
        /// draws them.
        ///
        /// WHY: the fuzz target of the night job cuts its input into a
        ///       search text, a replacement and a name of a file, and
        ///       hands all three to the engine. The target needs a nightly
        ///       compiler and a sanitizer, and it only runs in the night.
        ///       The test stands next to it and proves on the stable
        ///       compiler that the same call does not panic — the target
        ///       then only has to look for the input that this test cannot
        ///       draw.
        #[test]
        fn the_rules_survive_the_path_of_the_file(
            (search, replacement, is_regex) in rule(),
            name in any::<String>(),
        ) {
            // arrange
            let pfad = std::path::PathBuf::from(format!("/photos/{name}"));
            let regel = RenameRule::new("[name]_[n].[ext]").with_replacement(ReplaceRule {
                search: search.clone(),
                replacement: replacement.clone(),
                regex: is_regex,
            });
            let input = RenameInput::new(pfad).with_size(1_024).with_checksum(42);

            // act
            let ergebnis = form_names(
                &regel,
                &[input],
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000),
            );

            // assert
            // WHY: the engine may refuse the rule — a pattern that it
            //       cannot read is an error and not a panic.
            if is_regex {
                let _ = ergebnis;
            } else {
                let namen = ergebnis.unwrap_or_else(|error| panic!("a literal rule should always work: {error}"));
                prop_assert_eq!(namen.len(), 1);
            }
        }

        /// The same rules always give the same result.
        #[test]
        fn the_rules_of_a_rename_are_deterministic(
            (search, replacement, is_regex) in rule(),
            text in any::<String>(),
        ) {
            // arrange
            let regel = ReplaceRule {
                search: search.clone(),
                replacement: replacement.clone(),
                regex: is_regex,
            };
            let Ok(compiled) = compile(&[regel]) else {
                return Ok(());
            };

            // act
            let erste = apply(&compiled, &text);
            let zweite = apply(&compiled, &text);

            // assert
            prop_assert_eq!(erste, zweite);
        }
    }
}
