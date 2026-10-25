//! The filter of the panel filters the list while the user types.
//!
//! WHAT: the test types the pattern `*.txt` into the filter field of a
//!       panel with five entries and reads the list of the panel.
//! WHY: FR-74a — the current directory is filtered while the user types
//!       and without a dialog. The filter is a line of the panel, the list
//!       follows every key stroke, and the number of the entries that pass
//!       the filter is shown (FR-74e).
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_filter_matches_without_a_dialog -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{app_with, ctrl, harness};
use egui_kittest::kittest::Queryable;
use phocommander_ui::listing::{Entry, EntryKind};

#[test]
fn the_filter_matches_without_a_dialog() {
    // arrange — five entries, of which one is a text file.
    let app = app_with(vec![
        Entry::new("notes.txt", EntryKind::File),
        Entry::new("picture.png", EntryKind::File),
        Entry::new("music.mp3", EntryKind::File),
        Entry::new("folder", EntryKind::Directory),
        Entry::new("archive.zip", EntryKind::File),
    ]);
    let mut harness = harness(app);

    // act — the control key and F open the filter field of the panel that
    // has the focus, and the pattern is typed into it.
    harness.key_press_modifiers(ctrl(), egui::Key::F);
    harness.step();
    harness.event(egui::Event::Text("*.txt".to_string()));
    harness.step();

    // assert — the list of the panel holds one entry, the count of the
    // matches is one, and the pattern is in the field of the panel.
    let panel = harness.state().surface().active();
    assert_eq!(panel.filter_text(), "*.txt");
    assert_eq!(panel.matching_count(), 1);
    assert_eq!(panel.row_count(), 1);
    assert_eq!(panel.total_count(), 5);
    assert!(panel.is_filter_editing());
    assert!(panel.is_filter_visible());

    // assert — the list holds the entry that matches and no other.
    let names: Vec<String> = panel
        .visible_rows(0, 10)
        .into_iter()
        .map(|row| row.name().to_string())
        .collect();
    assert_eq!(names, vec!["notes.txt"]);

    // assert — the filter is a line of the panel and not a window: the
    // shell produced no command and the drawn interface shows the pattern
    // and the number of the matches.
    assert!(harness.state().commands().is_empty());
    assert!(harness.query_by_label("*.txt").is_some());
    assert!(harness.query_by_label("panel filter matches 1").is_some());

    // The tree of the drawn interface is the evidence of what the user sees.
    // It is printed, so that the handover can quote it.
    println!("{:?}", harness.root());
}
