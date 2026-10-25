//! The pattern language of the filter.
//!
//! WHAT: the test matches names against one case per form of the pattern
//!       language: `*`, `?`, `~` and a plain text.
//! WHY: FR-74b — the filter matches the name with a pattern language: `*`
//!       and `?` are wildcards, `~` matches without regard to the case and
//!       a plain text matches anywhere in the name. The evaluation is a pure
//!       function, so the same pattern and the same name always yield the
//!       same answer.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_filter_pattern_language -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_ui::filter::FilterPattern;

/// The names of the entries of the test.
const NAMES: [&str; 6] = [
    "notes.txt",
    "report-2026.pdf",
    "REPORT-2026.PDF",
    "image.png",
    "a.txt",
    "readme",
];

/// The names that match the given pattern.
fn matching(pattern: &str) -> Vec<&'static str> {
    let pattern = FilterPattern::parse(pattern);
    NAMES
        .iter()
        .copied()
        .filter(|name| pattern.matches(name))
        .collect()
}

#[test]
fn the_star_matches_any_sequence_of_characters() {
    // act
    let matched = matching("*.txt");

    // assert
    assert_eq!(matched, vec!["notes.txt", "a.txt"]);
}

#[test]
fn the_question_mark_matches_exactly_one_character() {
    // act
    let matched = matching("?.txt");

    // assert
    assert_eq!(matched, vec!["a.txt"]);
}

#[test]
fn the_tilde_matches_without_regard_to_the_case() {
    // act
    let matched = matching("~report-2026.pdf");

    // assert — the tilde ignores the case of the whole pattern.
    assert_eq!(matched, vec!["report-2026.pdf", "REPORT-2026.PDF"]);
}

#[test]
fn a_plain_text_matches_anywhere_in_the_name() {
    // act
    let matched = matching("report");

    // assert — the text is searched as it is written; only the tilde
    // ignores the case.
    assert_eq!(matched, vec!["report-2026.pdf"]);
}

#[test]
fn the_evaluation_is_the_same_for_the_same_input() {
    // arrange
    let pattern = FilterPattern::parse("*.txt");

    // act
    let first: Vec<bool> = NAMES.iter().map(|name| pattern.matches(name)).collect();
    let second: Vec<bool> = NAMES.iter().map(|name| pattern.matches(name)).collect();

    // assert
    assert_eq!(first, second);
    assert_eq!(first, vec![true, false, false, false, true, false]);
}
