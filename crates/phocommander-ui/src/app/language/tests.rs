// The tests of the language of the interface.
//
// WHAT: the tests of the ways in which the shell takes a new language, and
//       the test of the harness of egui that proves that a change of the
//       language reaches the screen in the next frame.
// WHY: FR-124 — the language of the interface can be changed while the
//       program runs, without a restart and without the user having to cause
//       a repaint. The test of the harness draws the surface, changes the
//       catalogue and draws again: the labels of the second frame come from
//       the new catalogue.
// EXAMPLE: `cargo test -p phocommander-ui app::language`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use super::*;
use crate::format::PlainFormats;
use crate::i18n::Argument;
use crate::keymap_view::KeymapView;
use crate::listing::{Entry, EntryKind};
use crate::surface::WorkSurface;
use std::path::PathBuf;

/// A catalogue that answers every key with the key itself.
struct Echo;

impl Catalogue for Echo {
    fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
        Some(key.to_string())
    }
}

/// A catalogue that answers with a German text.
struct German;

impl Catalogue for German {
    fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
        match key {
            "common-cancel" => Some("Abbrechen".to_string()),
            _ => Some(key.to_string()),
        }
    }
}

/// A catalogue that answers every key with a marked text.
///
/// WHY: the marker shows in the labels of the buttons of the toolbar, and
///       the harness reads those labels out of the tree of the accessibility.
///       The test therefore sees what the user sees.
struct Marked {
    /// The mark that every text of this catalogue carries.
    marker: &'static str,
}

impl Catalogue for Marked {
    fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
        Some(format!("{}{key}", self.marker))
    }
}

/// The shell with a catalogue of its own.
fn app() -> WorkSurfaceApp {
    WorkSurfaceApp::new(
        WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
        KeymapView::default(),
        Box::new(Echo),
    )
}

/// The shell with a marked catalogue and one entry in the left panel.
fn marked_app(marker: &'static str) -> WorkSurfaceApp {
    let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
    surface.active_mut().set_entries(vec![
        Entry::new("a.txt", EntryKind::File).with_size(1_200_000),
    ]);

    WorkSurfaceApp::new(surface, KeymapView::default(), Box::new(Marked { marker }))
}

#[test]
fn the_catalogue_can_be_replaced_while_the_program_runs() {
    // WHY: FR-124 — the language of the interface can be changed at run
    //       time, without a restart of the program.
    // arrange
    let mut app = app();

    // act
    app.set_catalogue(Box::new(German));

    // assert
    assert_eq!(
        app.catalogue().lookup("common-cancel"),
        Some("Abbrechen".to_string())
    );
}

#[test]
fn the_formats_can_be_replaced_while_the_program_runs() {
    // arrange
    let mut app = app();

    // act
    app.set_formats(Box::new(PlainFormats));

    // assert
    assert_eq!(app.formats().size(512), "512 B");
}

#[test]
fn a_shell_without_formats_writes_the_plain_formats() {
    // WHY: a caller that has no locale needs no decision. The shell writes
    //       the plain formats until a composition root hands other ones
    //       over.
    // act
    let app = app();

    // assert
    assert_eq!(app.formats().size(512), "512 B");
}

#[test]
fn a_change_of_the_language_reaches_the_screen_in_the_next_frame() {
    // arrange
    let mut harness = Harness::new_ui_state(
        |ui, app: &mut WorkSurfaceApp| app.update(ui),
        marked_app("en-"),
    );
    harness.run_steps(1);

    // assert
    // WHY: the surface is drawn with the catalogue of the start. The
    //       buttons of the toolbar carry the texts of that catalogue.
    assert!(
        harness.query_all_by_label_contains("en-").count() > 0,
        "the interface should carry the texts of the first catalogue"
    );

    // act
    harness
        .state_mut()
        .set_catalogue(Box::new(Marked { marker: "de-" }));
    harness.run_steps(1);

    // assert
    assert!(
        harness.query_all_by_label_contains("de-").count() > 0,
        "the interface should carry the texts of the new catalogue"
    );
    assert_eq!(
        harness.query_all_by_label_contains("en-").count(),
        0,
        "the interface should not carry the texts of the old catalogue"
    );
}
