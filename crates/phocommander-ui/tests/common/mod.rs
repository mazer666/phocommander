//! The helpers of the tests of the user interface.
//!
//! WHAT: the shell with a catalogue and a binding table ([`app_with`]), the
//!       harness that draws it ([`harness`]) and a catalogue that answers
//!       with a readable text ([`Names`]).
//! WHY: the tests of the interface drive the real shell with real key
//!       events. The helpers build the shell once, so that every test says
//!       what it does and not how the shell is assembled.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use phocommander_ui::app::WorkSurfaceApp;
use phocommander_ui::context::Context;
use phocommander_ui::i18n::{Argument, Catalogue};
use phocommander_ui::keymap_view::KeymapView;
use phocommander_ui::listing::Entry;
use phocommander_ui::surface::WorkSurface;

/// A catalogue that answers every key with a readable text.
///
/// WHY: the tests read the drawn interface. A catalogue that answers with
/// the key itself makes every label predictable, and the number of the
/// matches is appended to the text of the key.
pub struct Names;

impl Catalogue for Names {
    fn lookup_with(&self, key: &str, arguments: &[Argument<'_>]) -> Option<String> {
        let mut text = key.replace('-', " ");
        if let Some(Argument::Number { value: count, .. }) = arguments.first() {
            text = format!("{text} {count}");
        }
        Some(text)
    }
}

/// The binding table of the tests.
fn keymap() -> KeymapView {
    let mut keyboard = BTreeMap::new();
    keyboard.insert("panel.filter".to_string(), "Ctrl+F".to_string());
    keyboard.insert("panel.switch".to_string(), "Tab".to_string());
    keyboard.insert("selection.toggle".to_string(), "Ins".to_string());

    let mut tables = BTreeMap::new();
    tables.insert(Context::Keyboard, keyboard);

    match KeymapView::from_context_tables(&tables) {
        Ok(keymap) => keymap,
        Err(error) => panic!("the binding table of the test should be valid: {error}"),
    }
}

/// The shell with the given entries in the panel that has the focus.
pub fn app_with(entries: Vec<Entry>) -> WorkSurfaceApp {
    let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
    surface.active_mut().set_entries(entries);
    WorkSurfaceApp::new(surface, keymap(), Box::new(Names))
}

/// The harness that draws the shell.
pub fn harness(app: WorkSurfaceApp) -> egui_kittest::Harness<'static, WorkSurfaceApp> {
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
pub fn ctrl() -> egui::Modifiers {
    egui::Modifiers {
        ctrl: true,
        ..egui::Modifiers::default()
    }
}
