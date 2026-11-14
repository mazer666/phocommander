//! A key that cannot do anything says so.
//!
//! WHAT: the test presses a key of an action that belongs to a later
//!       milestone and reads the status line.
//! WHY: the user presses a key and the program does nothing. A program that
//!       says nothing about it leaves the user with the impression that the
//!       key is broken. The notice names the action — and it names it in
//!       the language of the interface, which only the shell knows
//!       (FR-05, FR-124).
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test a_key_that_cannot_do_anything_says_so
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{app_with, harness};
use phocommander_ui::listing::{Entry, EntryKind};

#[test]
fn a_key_that_cannot_do_anything_says_so() {
    // arrange — the shell with an entry in the panel that has the focus.
    let app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    let mut harness = harness(app);

    // act — the user presses the key of the drive list. The drive list
    // belongs to a later milestone, and the action cannot do anything yet.
    harness.key_press_modifiers(
        egui::Modifiers {
            alt: true,
            ..egui::Modifiers::default()
        },
        egui::Key::F1,
    );
    harness.step();

    // assert — the status line carries the notice, and it names the action.
    let status = harness.state().surface().status();
    let status = status.expect("the status line should carry the notice");
    assert_eq!(status.key, "action-deferred");
    assert_eq!(status.arguments.len(), 1);
    assert!(
        !status.arguments[0].1.is_empty(),
        "the notice should name the action"
    );
}
