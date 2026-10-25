//! The search and the replacement of the rename rule (FR-31).
//!
//! WHAT: the test searches a name with a regular expression and with a
//!       literal text, and it offers a pattern that the engine cannot read.
//! WHY: the criterion of the milestone asks for the search with the regular
//!       expression switched on and switched off, and for a broken pattern
//!       that changes no name.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ops --test rename_search_and_replace -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::{Duration, SystemTime};

use phocommander_ops::template::{RenameInput, RenameRule, ReplaceRule, form_names};

/// The moment that the test fixes.
fn moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

/// The name that the rule forms for the file.
fn formed(rule: &RenameRule, input: &RenameInput) -> String {
    let names = form_names(rule, std::slice::from_ref(input), moment());
    let names = match names {
        Ok(names) => names,
        Err(error) => panic!("the name should have been formed: {error}"),
    };
    match names.first().and_then(|formed| formed.name()) {
        Some(name) => name.to_string(),
        None => panic!("the name should have been formed"),
    }
}

#[test]
fn rename_search_and_replace() {
    // arrange
    let input = RenameInput::new("/photos/a42.txt");

    // act, assert
    // WHY: with the switch on the pattern `^a(\d+)$` matches the name and
    //       the group `$1` keeps the number.
    let with_pattern =
        RenameRule::new("[name]").with_replacement(ReplaceRule::pattern(r"^a(\d+)$", "b$1"));
    assert_eq!(formed(&with_pattern, &input), "b42");

    // WHY: with the switch off the text is matched literally, and the
    //       pattern `^a(\d+)$` is a text that the name does not carry.
    let with_literal =
        RenameRule::new("[name]").with_replacement(ReplaceRule::literal(r"^a(\d+)$", "b$1"));
    assert_eq!(formed(&with_literal, &input), "a42");
}

#[test]
fn a_search_that_is_not_a_regular_expression_matches_the_text() {
    // arrange
    let input = RenameInput::new("/photos/axb.txt");

    // act, assert
    // WHY: the dot of a literal search is a dot and not "any character".
    let literal = RenameRule::new("[name]").with_replacement(ReplaceRule::literal("a.b", "c"));
    assert_eq!(formed(&literal, &input), "axb");

    let pattern = RenameRule::new("[name]").with_replacement(ReplaceRule::pattern("a.b", "c"));
    assert_eq!(formed(&pattern, &input), "c");
}

#[test]
fn a_broken_pattern_produces_an_error_and_changes_no_name() {
    // arrange
    let rule = RenameRule::new("[name]").with_replacement(ReplaceRule::pattern("a(", "b"));
    let inputs = vec![RenameInput::new("/photos/a42.txt")];

    // act
    let names = form_names(&rule, &inputs, moment());

    // assert
    let error = match names {
        Ok(_) => panic!("the broken pattern should have been refused"),
        Err(error) => error,
    };
    assert!(
        matches!(
            error,
            phocommander_ops::template::TemplateError::InvalidPattern { .. }
        ),
        "{error}"
    );
}
