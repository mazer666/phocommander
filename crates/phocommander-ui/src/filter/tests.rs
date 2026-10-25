// The unit tests of the filter.
//
// WHY: the tests live in a file of their own, so that the model itself
// stays readable. They may fail loudly when their arrangement is wrong —
// the rule that forbids a panic applies to the production code, and it is
// enforced there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::listing::EntryKind;

/// An entry with a name and a kind.
fn entry(name: &str, kind: EntryKind) -> Entry {
    Entry::new(name, kind)
}

#[test]
fn a_plain_text_matches_anywhere_in_the_name() {
    // WHY: FR-74b — a plain text matches anywhere in the name.
    // act
    let pattern = FilterPattern::parse("report");

    // assert
    assert!(pattern.matches("report"));
    assert!(pattern.matches("the-report-2026.txt"));
    assert!(!pattern.matches("notes.txt"));
}

#[test]
fn a_star_matches_any_number_of_characters() {
    // act
    let pattern = FilterPattern::parse("*.txt");

    // assert
    assert!(pattern.matches("notes.txt"));
    assert!(pattern.matches(".txt"));
    assert!(pattern.matches("a-very-long-name.txt"));
    assert!(!pattern.matches("notes.md"));
}

#[test]
fn a_question_mark_matches_exactly_one_character() {
    // act
    let pattern = FilterPattern::parse("a?c");

    // assert
    assert!(pattern.matches("abc"));
    assert!(!pattern.matches("ac"));
    assert!(!pattern.matches("abbc"));
}

#[test]
fn the_tilde_ignores_the_case() {
    // act
    let pattern = FilterPattern::parse("~*.TXT");

    // assert
    assert!(pattern.matches("notes.txt"));
    assert!(pattern.matches("NOTES.TXT"));
    assert!(!FilterPattern::parse("*.TXT").matches("notes.txt"));
}

#[test]
fn an_empty_pattern_lets_every_entry_through() {
    // act
    let pattern = FilterPattern::parse("");

    // assert
    assert!(pattern.is_empty());
    assert!(pattern.matches("anything"));
    assert!(pattern.matches(""));
}

#[test]
fn a_pattern_with_several_stars_matches_the_general_case() {
    // act
    let pattern = FilterPattern::parse("a*b*c");

    // assert
    assert!(pattern.matches("abc"));
    assert!(pattern.matches("aXXbYYc"));
    assert!(!pattern.matches("acb"));
}

#[test]
fn the_kind_limits_the_filter_to_files_or_to_directories() {
    // WHY: FR-74c — the filter can be limited to files, to directories
    // or to both.
    // arrange
    let file = entry("notes.txt", EntryKind::File);
    let directory = entry("notes.txt", EntryKind::Directory);

    // act
    let files = Filter::from_text("notes", FilterKind::Files, Default::default());
    let directories = Filter::from_text("notes", FilterKind::Directories, Default::default());
    let all = Filter::from_text("notes", FilterKind::All, Default::default());

    // assert
    assert!(files.matches_entry(&file));
    assert!(!files.matches_entry(&directory));
    assert!(directories.matches_entry(&directory));
    assert!(!directories.matches_entry(&file));
    assert!(all.matches_entry(&file));
    assert!(all.matches_entry(&directory));
}

#[test]
fn the_depth_limits_the_filter_to_the_current_directory() {
    // WHY: FR-74d — the filter can reach deeper than the current
    // directory.
    // arrange
    let shallow = entry("notes.txt", EntryKind::File);
    let deep = entry("notes.txt", EntryKind::File).with_depth(2);

    // act
    let current = Filter::from_text("notes", FilterKind::All, FilterDepth::Current);
    let recursive = Filter::from_text(
        "notes",
        FilterKind::All,
        FilterDepth::Recursive { levels: 2 },
    );

    // assert
    assert!(current.matches_entry(&shallow));
    assert!(!current.matches_entry(&deep));
    assert!(recursive.matches_entry(&shallow));
    assert!(recursive.matches_entry(&deep));
}

#[test]
fn the_index_holds_the_entries_that_pass_the_filter() {
    // arrange
    let entries = vec![
        entry("a.txt", EntryKind::File),
        entry("b.md", EntryKind::File),
        entry("c.txt", EntryKind::Directory),
    ];
    let order = vec![0, 1, 2];
    let filter = Filter::from_text("*.txt", FilterKind::Files, Default::default());

    // act
    let index = FilteredIndex::build(&entries, &order, &filter);

    // assert
    assert_eq!(index.len(), 1);
    assert_eq!(index.positions(), &[0]);
    assert_eq!(index.row_of_position(0), Some(0));
    assert_eq!(index.row_of_position(1), None);
}

#[test]
fn the_field_edits_the_text_at_the_caret() {
    // arrange
    let mut field = FilterField::new();
    field.open();

    // act
    field.insert("ab");
    field.move_caret(-1);
    field.insert("X");

    // assert
    assert_eq!(field.text(), "aXb");
    assert_eq!(field.caret(), 2);
    field.backspace();
    assert_eq!(field.text(), "ab");
    field.delete();
    assert_eq!(field.text(), "a");
}

#[test]
fn the_field_cannot_leave_its_text() {
    // arrange
    let mut field = FilterField::new();
    field.open();
    field.insert("ab");

    // act
    field.move_caret(-10);
    field.backspace();

    // assert
    assert_eq!(field.text(), "ab");
    assert_eq!(field.caret(), 0);
}
