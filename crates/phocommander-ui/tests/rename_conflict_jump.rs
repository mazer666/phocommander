//! The conflict jump list of the rename dialog is reachable with F3.
//!
//! WHAT: the test opens the dialog of the multiple rename with three marked
//!       files, presses F3 and Shift+F3 and reads the cursor of the preview.
//! WHY: FR-35 — the conflicts are marked and listed in a conflict jump list
//!       that is reachable with F3 and Shift+F3. The test drives the real
//!       shell with real key events, and it reads the state of the dialog —
//!       not a picture.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test rename_conflict_jump -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use phocommander_ui::app::WorkSurfaceApp;
use phocommander_ui::context::Context;
use phocommander_ui::i18n::{Argument, Catalogue};
use phocommander_ui::keymap_view::KeymapView;
use phocommander_ui::listing::{Entry, EntryKind};
use phocommander_ui::rename::{
    Conflict, ConflictKind, PreviewRow, RenameEngine, RenamePreview, RenameRequest,
};
use phocommander_ui::surface::WorkSurface;

/// A catalogue that answers every key with a readable text.
struct Names;

impl Catalogue for Names {
    fn lookup_with(&self, key: &str, arguments: &[Argument<'_>]) -> Option<String> {
        let mut text = key.replace('-', " ");
        if let Some(Argument::Number { value: count, .. }) = arguments.first() {
            text = format!("{text} {count}");
        }
        Some(text)
    }
}

/// The binding table of the test.
///
/// WHY: the test needs the bindings of the rename — F2 opens the dialog,
///       F3 and Shift+F3 walk the conflicts — and the binding that marks
///       every entry of the panel.
fn keymap() -> KeymapView {
    let mut keyboard = BTreeMap::new();
    keyboard.insert("panel.rename".to_string(), "F2".to_string());
    keyboard.insert("selection.all".to_string(), "Ctrl+A".to_string());
    keyboard.insert("rename.conflict_next".to_string(), "F3".to_string());
    keyboard.insert(
        "rename.conflict_previous".to_string(),
        "Shift+F3".to_string(),
    );

    let mut tables = BTreeMap::new();
    tables.insert(Context::Keyboard, keyboard);

    match KeymapView::from_context_tables(&tables) {
        Ok(keymap) => keymap,
        Err(error) => panic!("the binding table of the test should be valid: {error}"),
    }
}

/// An engine that marks the first and the last file as a conflict.
struct Conflicting;

impl RenameEngine for Conflicting {
    fn preview(&self, request: &RenameRequest) -> RenamePreview {
        let mut preview = RenamePreview::empty();
        for (index, file) in request.files.iter().enumerate() {
            let name = format!("{index:03}.jpg");
            preview
                .rows
                .push(PreviewRow::named(file.path.clone(), name));
        }
        for index in [0usize, 2usize] {
            if let Some(row) = preview.rows.get(index)
                && let Some(target) = row.target.clone()
            {
                preview
                    .conflicts
                    .push(Conflict::new(ConflictKind::TargetExists, target));
            }
        }
        preview
    }
}

/// The shell with three entries and the engine of the test.
fn app() -> WorkSurfaceApp {
    let mut surface = WorkSurface::new("/tmp".into(), "/var".into());
    surface.active_mut().set_entries(vec![
        Entry::new("a.jpg", EntryKind::File),
        Entry::new("b.jpg", EntryKind::File),
        Entry::new("c.jpg", EntryKind::File),
    ]);

    WorkSurfaceApp::new(surface, keymap(), Box::new(Names))
        .with_rename_engine(Box::new(Conflicting))
}

/// The harness that draws the shell.
fn harness(app: WorkSurfaceApp) -> egui_kittest::Harness<'static, WorkSurfaceApp> {
    egui_kittest::Harness::builder()
        .with_size(egui::vec2(900.0, 600.0))
        .build_ui_state(
            |ui, app: &mut WorkSurfaceApp| {
                app.update(ui);
            },
            app,
        )
}

/// The modifiers of the control key.
fn ctrl() -> egui::Modifiers {
    egui::Modifiers {
        ctrl: true,
        ..egui::Modifiers::default()
    }
}

/// The modifiers of the shift key.
fn shift() -> egui::Modifiers {
    egui::Modifiers {
        shift: true,
        ..egui::Modifiers::default()
    }
}

#[test]
fn the_conflict_jump_list_is_reachable_with_f3_and_shift_f3() {
    // arrange — three entries, all of them marked.
    let mut harness = harness(app());
    harness.key_press_modifiers(ctrl(), egui::Key::A);
    harness.step();

    // act — F2 opens the dialog of the multiple rename.
    harness.key_press(egui::Key::F2);
    harness.step();

    // assert — the dialog is open, the three files are in it, and the two
    // conflicts of the engine are marked.
    let dialog = match harness.state().rename_dialog() {
        Some(dialog) => dialog,
        None => panic!("the dialog should be open"),
    };
    assert_eq!(dialog.files().len(), 3);
    assert_eq!(dialog.conflicts(), &[0, 2]);

    // act — F3 jumps to the next conflict.
    harness.key_press(egui::Key::F3);
    harness.step();

    // assert — the cursor stands on the first conflict that follows the
    // cursor of the preview.
    let cursor = harness
        .state()
        .rename_dialog()
        .map(|dialog| dialog.cursor());
    assert_eq!(cursor, Some(2));

    // act — F3 again: the list has no conflict behind the last one, so the
    // cursor stays on the last entry.
    harness.key_press(egui::Key::F3);
    harness.step();

    // assert
    let cursor = harness
        .state()
        .rename_dialog()
        .map(|dialog| dialog.cursor());
    assert_eq!(cursor, Some(2));

    // act — Shift+F3 jumps to the previous conflict.
    harness.key_press_modifiers(shift(), egui::Key::F3);
    harness.step();

    // assert
    let cursor = harness
        .state()
        .rename_dialog()
        .map(|dialog| dialog.cursor());
    assert_eq!(cursor, Some(0));

    // The tree of the drawn interface is the evidence of what the user sees.
    // It is printed, so that the handover can quote it.
    println!("{:?}", harness.root());
}
