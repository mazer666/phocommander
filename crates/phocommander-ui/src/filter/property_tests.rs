// The property tests of the filter.
//
// WHY: the filter is evaluated on every key stroke, on every entry of a
// directory. The evaluation is therefore a pure function, and the property
// tests prove that it stays one: the same input yields the same result,
// always, and the rules of the pattern language hold for every input and
// not only for the examples of the unit tests.
//
// The rule that forbids a panic applies to the production code; a property
// test is allowed to fail loudly.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use proptest::prelude::*;

use crate::filter::{Filter, FilterKind, FilterPattern};
use crate::listing::{Entry, EntryKind};

/// The text of a pattern: the wildcards, the modifier and plain letters.
fn pattern_text() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        "[a-zA-Z0-9]{1,6}",
        "\\*[a-zA-Z0-9]{0,4}",
        "[a-zA-Z0-9]{0,4}\\?",
        "\\*[a-zA-Z0-9]{0,3}\\*[a-zA-Z0-9]{0,3}",
        "~[a-zA-Z0-9]{1,5}",
        "~\\*[a-zA-Z0-9]{0,4}",
    ]
}

/// The name of an entry.
fn entry_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9._ -]{0,12}"
}

proptest! {
    /// The evaluation of a pattern is a pure function: the same name and the
    /// same pattern always yield the same answer.
    #[test]
    fn the_evaluation_is_deterministic(text in pattern_text(), name in entry_name()) {
        let pattern = FilterPattern::parse(&text);
        let first = pattern.matches(&name);
        let second = pattern.matches(&name);
        prop_assert_eq!(first, second);
    }

    /// A pattern without a wildcard is a text that is searched anywhere in
    /// the name.
    #[test]
    fn a_plain_text_is_searched_anywhere(text in "[a-z]{1,5}", name in entry_name()) {
        let pattern = FilterPattern::parse(&text);
        prop_assert_eq!(pattern.matches(&name), name.contains(&text));
    }

    /// A text that ignores the case matches the name whatever its case is.
    #[test]
    fn the_tilde_ignores_the_case(text in "[a-z]{1,5}") {
        let pattern = FilterPattern::parse(&format!("~{text}"));
        prop_assert!(pattern.matches(&text));
        prop_assert!(pattern.matches(&text.to_uppercase()));
        let name = format!("prefix-{}-suffix", text);
        prop_assert!(pattern.matches(&name));
    }

    /// The star matches every name.
    #[test]
    fn the_star_matches_every_name(name in entry_name()) {
        let pattern = FilterPattern::parse("*");
        prop_assert!(pattern.matches(&name));
    }

    /// The question mark matches exactly one character.
    #[test]
    fn the_question_mark_matches_exactly_one_character(name in entry_name()) {
        let pattern = FilterPattern::parse("?");
        prop_assert_eq!(pattern.matches(&name), name.chars().count() == 1);
    }

    /// An empty pattern lets every entry through.
    #[test]
    fn an_empty_pattern_matches_every_name(name in entry_name()) {
        let pattern = FilterPattern::parse("");
        prop_assert!(pattern.is_empty());
        prop_assert!(pattern.matches(&name));
    }

    /// The filter of a panel is a pure function as well: the same entries
    /// and the same filter yield the same index.
    #[test]
    fn the_index_is_deterministic(text in pattern_text(), count in 0usize..24) {
        let entries: Vec<Entry> = (0..count)
            .map(|index| {
                let name = format!("file-{index}.txt");
                let kind = if index % 2 == 0 { EntryKind::File } else { EntryKind::Directory };
                Entry::new(name, kind)
            })
            .collect();
        let order: Vec<usize> = (0..count).collect();
        let filter = Filter::from_text(&text, FilterKind::All, Default::default());

        let first = crate::filter::FilteredIndex::build(&entries, &order, &filter);
        let second = crate::filter::FilteredIndex::build(&entries, &order, &filter);

        prop_assert_eq!(first.len(), second.len());
        prop_assert_eq!(first, second);
    }
}
