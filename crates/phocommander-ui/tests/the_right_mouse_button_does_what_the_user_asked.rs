//! The right mouse button does what the user asked for.
//!
//! WHAT: the test presses the right mouse button once with the setting
//!       "toggle_selection" and once with the setting "context_menu", and
//!       reads the selection of the panel.
//! WHY: the user writes in the configuration file whether the right mouse
//!       button opens a menu or marks the file under the pointer. The
//!       setting was read and never used, and the button had no binding at
//!       all — the user pressed it and nothing happened (FR-17).
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_right_mouse_button_does_what_the_user_asked
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{app_with, harness};
use phocommander_ui::Context;
use phocommander_ui::listing::{Entry, EntryKind};
use phocommander_ui::visibility::RightClickAction;

#[test]
fn the_right_mouse_button_marks_the_file_when_the_user_asked_for_it() {
    // arrange — a shell that marks the file under the pointer, and a panel
    // with one entry.
    let mut app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    app = app
        .with_context(Context::Pointer)
        .with_right_click(RightClickAction::ToggleSelection);
    let mut harness = harness(app);

    // act — the user presses the right mouse button.
    harness.event(egui::Event::PointerButton {
        pos: egui::pos2(10.0, 10.0),
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.step();

    // assert — the file under the pointer is marked.
    let panel = harness.state().surface().active();
    assert!(
        !panel.marked_entries().is_empty(),
        "the right mouse button should mark the file"
    );
}

#[test]
fn the_right_mouse_button_says_that_the_menu_comes_later() {
    // arrange — a shell that opens the menu of the system, and a panel
    // with one entry.
    let mut app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    app = app
        .with_context(Context::Pointer)
        .with_right_click(RightClickAction::ContextMenu);
    let mut harness = harness(app);

    // act — the user presses the right mouse button.
    harness.event(egui::Event::PointerButton {
        pos: egui::pos2(10.0, 10.0),
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.step();

    // assert — nothing is marked, and the status line says that the menu
    // belongs to a later milestone.
    let panel = harness.state().surface().active();
    assert!(
        !!panel.marked_entries().is_empty(),
        "the menu should not mark the file"
    );
    let status = harness.state().surface().status();
    let status = status.expect("the status line should carry the notice");
    assert_eq!(status.key, "action-deferred");
}
