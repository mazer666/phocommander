//! The filter of a panel survives a change of the directory.
//!
//! WHAT: the test filters a panel, changes its directory and reads the
//!       filter of both panels.
//! WHY: FR-74f — the filter belongs to the panel, so that the user does not
//!       have to type it again for every directory. The second panel keeps
//!       its own pattern: the filter of one panel is never the filter of the
//!       other one.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_filter_survives_a_directory_change -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_ui::filter::FilterKind;
use phocommander_ui::listing::{Entry, EntryKind};
use phocommander_ui::panel::PanelSide;
use phocommander_ui::surface::WorkSurface;

#[test]
fn the_filter_survives_a_directory_change() {
    // arrange — two panels, each with its own list.
    let mut surface = WorkSurface::new("/tmp".into(), "/var".into());
    surface.panel_mut(PanelSide::Left).set_entries(vec![
        Entry::new("notes.txt", EntryKind::File),
        Entry::new("picture.png", EntryKind::File),
    ]);
    surface.panel_mut(PanelSide::Right).set_entries(vec![
        Entry::new("notes.txt", EntryKind::File),
        Entry::new("picture.png", EntryKind::File),
    ]);

    // act — the left panel is filtered and changes its directory.
    surface
        .panel_mut(PanelSide::Left)
        .set_filter_kind(FilterKind::Files);
    surface.panel_mut(PanelSide::Left).set_filter_text("*.txt");
    surface.panel_mut(PanelSide::Left).set_path("/home".into());

    // assert — the pattern is still in the field of the panel, and the
    // filter is still in use.
    let left = surface.panel(PanelSide::Left);
    assert_eq!(left.filter_text(), "*.txt");
    assert!(left.filter().is_active());
    assert_eq!(left.path().to_string_lossy(), "/home");

    // assert — the second panel keeps its own pattern.
    let right = surface.panel(PanelSide::Right);
    assert_eq!(right.filter_text(), "");
    assert!(!right.filter().is_active());
    assert_eq!(right.matching_count(), 2);
}
