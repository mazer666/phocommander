//! The action closes the viewer again.
//!
//! WHAT: the test opens a file in the viewer and closes it with the
//!       action `viewer.close`.
//! WHY: the viewer opens in an area below the panels, and the area takes
//!       the room of the panels. The user must be able to give the room
//!       back — the function `close_viewer` existed and nobody called it,
//!       and the action did not exist at all. A viewer that cannot be
//!       closed is a window that stays open (FR-50).
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_action_closes_the_viewer
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{app_with, harness};
use egui_kittest::kittest::Queryable;
use phocommander_ui::listing::{Entry, EntryKind};
use phocommander_ui::preview::Preview;

#[test]
fn the_action_closes_the_viewer() {
    // arrange — a file in the viewer.
    let mut app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    app.surface_mut().set_viewer(Preview::new(
        std::path::PathBuf::from("/tmp/notes.txt"),
        11,
        vec!["hallo welt".to_string()],
        false,
    ));
    let mut harness = harness(app);

    // act — the user presses the key of the action.
    harness.key_press_modifiers(
        egui::Modifiers {
            ctrl: true,
            shift: true,
            ..egui::Modifiers::default()
        },
        egui::Key::W,
    );
    harness.step();

    // assert — the viewer is gone.
    let inhalt = harness.query_by_label("hallo welt");
    assert!(inhalt.is_none(), "the action should close the viewer again");
}
