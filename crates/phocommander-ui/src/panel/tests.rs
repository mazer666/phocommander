// The tests of the state of a panel.
//
// WHY: the tests live in a file of their own, so that the state itself
// stays readable. They may fail loudly when their arrangement is wrong —
// the rule that forbids a panic applies to the production code, and it is
// enforced there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::filter::{FilterDepth, FilterKind};
use crate::listing::{EntryKind, SortKey};

/// A panel with three entries.
fn panel() -> PanelState {
    let mut panel = PanelState::new(PathBuf::from("/tmp"));
    panel.set_entries(vec![
        Entry::new("a.txt", EntryKind::File),
        Entry::new("b.txt", EntryKind::File),
        Entry::new("c.txt", EntryKind::File),
    ]);
    panel
}

#[test]
fn the_marks_hold_positions_and_not_paths() {
    // arrange
    let mut panel = panel();

    // act
    panel.toggle_at(1);

    // assert
    assert!(panel.is_marked(1));
    assert!(!panel.is_marked(0));
    assert_eq!(panel.selection().len(), 1);
}

#[test]
fn the_marks_survive_a_change_of_the_order() {
    // arrange
    let mut panel = panel();
    panel.toggle_at(0);

    // act
    panel.set_sort(Sort::by(SortKey::Size));

    // assert
    assert!(panel.is_marked(0));
}

#[test]
fn the_cursor_stays_inside_the_list() {
    // arrange
    let mut panel = panel();

    // act
    panel.set_cursor(99);

    // assert
    assert_eq!(panel.cursor(), 2);
}

#[test]
fn the_cursor_moves_by_rows_and_not_by_positions() {
    // arrange
    let mut panel = panel();
    panel.set_sort(Sort::by(SortKey::Size));

    // act
    panel.move_cursor(1);

    // assert
    let row = panel.cursor_row();
    assert_eq!(row, Some(1));
}

#[test]
fn selecting_all_marks_every_entry() {
    // arrange
    let mut panel = panel();

    // act
    panel.select_all();

    // assert
    assert_eq!(panel.selection().len(), 3);
    panel.invert_selection();
    assert_eq!(panel.selection().len(), 0);
}

#[test]
fn a_tab_that_does_not_exist_is_refused() {
    // arrange
    let mut panel = panel();

    // act
    let refused = panel.switch_tab(3);

    // assert
    assert!(!refused);
    assert_eq!(panel.tab_count(), 1);
}

#[test]
fn the_scroll_follows_the_cursor() {
    // arrange
    let mut panel = panel();

    // act
    panel.set_cursor_row(2);
    panel.ensure_cursor_visible(2);

    // assert
    assert_eq!(panel.scroll_row(), 1);
}

#[test]
fn the_filter_hides_the_entries_that_do_not_match() {
    // WHY: FR-74a — the current directory is filtered while the user
    // types.
    // arrange
    let mut panel = panel();

    // act
    panel.set_filter_text("a");

    // assert
    assert_eq!(panel.matching_count(), 1);
    assert_eq!(panel.row_count(), 1);
    assert_eq!(panel.total_count(), 3);
    let names: Vec<String> = panel
        .visible_rows(0, 10)
        .into_iter()
        .map(|row| row.name().to_string())
        .collect();
    assert_eq!(names, vec!["a.txt"]);
}

#[test]
fn the_filter_survives_a_change_of_the_directory() {
    // WHY: FR-74f — the filter belongs to the panel, so that the user does
    // not have to type it again for every directory.
    // arrange
    let mut panel = panel();
    panel.set_filter_text("*.txt");

    // act
    panel.set_path(PathBuf::from("/var"));

    // assert
    assert_eq!(panel.filter_text(), "*.txt");
    assert!(panel.filter().is_active());
}

#[test]
fn the_cursor_goes_to_the_first_entry_that_matches() {
    // WHY: the cursor must never point at an entry that the filter hides.
    // arrange
    let mut panel = panel();
    panel.set_cursor_row(2);

    // act
    panel.set_filter_text("a");

    // assert
    assert_eq!(panel.cursor(), 0);
    assert_eq!(panel.cursor_row(), Some(0));
}

#[test]
fn the_cursor_never_points_at_an_empty_list() {
    // arrange
    let mut panel = panel();
    panel.set_cursor_row(2);

    // act
    panel.set_filter_text("nothing-matches-this");

    // assert
    assert_eq!(panel.matching_count(), 0);
    assert_eq!(panel.row_count(), 0);
    assert!(panel.cursor_entry().is_none());
}

#[test]
fn the_filter_of_a_kind_limits_the_list() {
    // WHY: FR-74c — the filter can be limited to files or to directories.
    // arrange
    let mut panel = PanelState::new(PathBuf::from("/tmp"));
    panel.set_entries(vec![
        Entry::new("a.txt", EntryKind::File),
        Entry::new("folder", EntryKind::Directory),
    ]);

    // act
    panel.set_filter_text("*");
    panel.set_filter_kind(FilterKind::Directories);

    // assert
    assert_eq!(panel.matching_count(), 1);
    assert_eq!(panel.visible_rows(0, 10).len(), 1);

    // act
    panel.set_filter_kind(FilterKind::Files);

    // assert
    assert_eq!(panel.matching_count(), 1);
    assert_eq!(panel.visible_rows(0, 10)[0].name(), "a.txt");
}

#[test]
fn the_depth_of_the_filter_decides_how_far_it_reaches() {
    // WHY: FR-74d — the filter can reach deeper than the current
    // directory.
    // arrange
    let mut panel = PanelState::new(PathBuf::from("/tmp"));
    panel.set_entries(vec![
        Entry::new("a.txt", EntryKind::File),
        Entry::new("b.txt", EntryKind::File).with_depth(1),
    ]);

    // act
    panel.set_filter_kind(FilterKind::All);

    // assert — the default depth reaches only the directory that is shown.
    assert_eq!(panel.matching_count(), 2);

    // act
    panel.set_filter_depth(FilterDepth::Recursive { levels: 1 });
    panel.set_filter_text("b");

    // assert
    assert_eq!(panel.matching_count(), 1);
}

#[test]
fn the_marks_follow_the_filter() {
    // WHY: the user marks what he sees.
    // arrange
    let mut panel = panel();

    // act
    panel.set_filter_text("a");
    panel.select_all();

    // assert
    assert_eq!(panel.selection().len(), 1);
    assert!(panel.is_marked(0));
}

#[test]
fn cancelling_the_filter_clears_the_text_and_the_line() {
    // arrange
    let mut panel = panel();
    panel.open_filter();
    panel.set_filter_text("a");

    // act
    panel.cancel_filter();

    // assert
    assert_eq!(panel.filter_text(), "");
    assert!(!panel.filter().is_active());
    assert!(!panel.is_filter_visible());
    assert_eq!(panel.matching_count(), 3);
}

#[test]
fn confirming_the_filter_keeps_it() {
    // arrange
    let mut panel = panel();
    panel.open_filter();
    panel.set_filter_text("a");

    // act
    panel.confirm_filter();

    // assert
    assert!(!panel.is_filter_editing());
    assert!(panel.is_filter_visible());
    assert_eq!(panel.matching_count(), 1);
}

#[test]
fn the_field_is_only_editing_while_it_has_the_focus() {
    // arrange
    let mut panel = panel();

    // act
    panel.open_filter();

    // assert
    assert!(panel.is_filter_editing());
    assert!(panel.is_filter_visible());
}
