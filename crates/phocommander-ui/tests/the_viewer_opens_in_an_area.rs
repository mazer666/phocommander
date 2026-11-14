//! The viewer opens in an area of the surface.
//!
//! WHAT: the test opens a file in the viewer and reads the area of the
//!       surface.
//! WHY: FR-50 — the viewer opens in an additional area without leaving the
//! application. Every test so far proved the *state* of the surface; none
//! of them drew it. A file that the surface carries and never draws is a
//! file that the user does not see — and the area of the viewer is drawn
//! only when a file is open, so the drawing has to be proven as well.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_viewer_opens_in_an_area
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{app_with, harness};
use egui_kittest::kittest::Queryable;
use phocommander_ui::listing::{Entry, EntryKind};
use phocommander_ui::preview::Preview;

#[test]
fn the_viewer_opens_in_an_area() {
    // arrange — a file in the panel that has the focus, and the file in
    // the viewer, as the shell puts it there when the user presses F3 on
    // the marked entry.
    let mut app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    app.surface_mut().set_viewer(Preview::new(
        std::path::PathBuf::from("/tmp/notes.txt"),
        11,
        vec!["hallo welt".to_string()],
        false,
    ));
    let mut harness = harness(app);

    // act — the surface is drawn.
    harness.step();

    // assert — the line of the text stands in the area of the surface.
    // The name of the file is not a proof: the same name stands in the
    // list of the panel as well, and the test found two nodes with it.
    let inhalt = harness.query_by_label("hallo welt");
    assert!(
        inhalt.is_some(),
        "the viewer should show the line of the text"
    );
}

#[test]
fn the_area_of_the_viewer_is_empty_without_a_file() {
    // arrange
    let app = app_with(vec![Entry::new("notes.txt", EntryKind::File)]);
    let mut harness = harness(app);

    // act
    harness.step();

    // assert — the line of the text is not drawn, because no file is
    // open. The name of the file stands in the list of the panel and
    // proves nothing.
    let inhalt = harness.query_by_label("hallo welt");
    assert!(
        inhalt.is_none(),
        "the viewer should show nothing without a file"
    );
}

#[test]
fn the_viewer_shows_the_bytes_of_a_file_that_is_no_text() {
    // arrange — a file that is no text, with the lines of its hex dump.
    let mut app = app_with(vec![Entry::new("image.bin", EntryKind::File)]);
    app.surface_mut().set_viewer(Preview::new(
        std::path::PathBuf::from("/tmp/image.bin"),
        16,
        vec!["00000000  48 65 6c 6c 6f 20 57 6f  72 6c 64 21      |Hello World!|".to_string()],
        true,
    ));
    let mut harness = harness(app);

    // act — the surface is drawn.
    harness.step();

    // assert — the bytes stand in the area of the surface, and the notice
    // says what the lines are.
    let bytes = harness.query_by_label_contains("48 65 6c 6c 6f");
    assert!(
        bytes.is_some(),
        "the viewer should show the bytes of a file that is no text"
    );
}
