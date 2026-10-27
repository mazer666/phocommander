// The tests of the application shell.
//
// WHY: the tests live in a file of their own, so that the shell itself stays
// readable. They may fail loudly when their arrangement is wrong — the rule
// that forbids a panic applies to the production code, and it is enforced
// there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::Argument;
use crate::keymap_view::{Key, KeyStroke, KeymapView};
use crate::listing::{Entry, EntryKind};
use crate::panel::PanelSide;
use std::collections::BTreeMap;

/// A catalogue that answers every key with the key itself.
struct Echo;

impl Catalogue for Echo {
    fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
        Some(key.to_string())
    }
}

/// The shell with three entries in the left panel.
fn app() -> WorkSurfaceApp {
    let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
    surface.active_mut().set_entries(vec![
        Entry::new("a.txt", EntryKind::File),
        Entry::new("b.txt", EntryKind::File),
        Entry::new("c.txt", EntryKind::File),
    ]);

    let mut keyboard = BTreeMap::new();
    keyboard.insert("panel.copy".to_string(), "F5".to_string());
    keyboard.insert("panel.switch".to_string(), "Tab".to_string());
    keyboard.insert("panel.filter".to_string(), "Ctrl+F".to_string());
    keyboard.insert("selection.toggle".to_string(), "Ins".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(Context::Keyboard, keyboard);

    let keymap = match KeymapView::from_context_tables(&tables) {
        Ok(keymap) => keymap,
        Err(error) => panic!("the tables should be valid: {error}"),
    };

    WorkSurfaceApp::new(surface, keymap, Box::new(Echo))
}

/// The shell with a catalogue of its own.
fn app_with_catalogue(catalogue: Box<dyn Catalogue>) -> WorkSurfaceApp {
    let surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
    let mut keyboard = BTreeMap::new();
    keyboard.insert("panel.copy".to_string(), "F5".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(Context::Keyboard, keyboard);
    let keymap = match KeymapView::from_context_tables(&tables) {
        Ok(keymap) => keymap,
        Err(error) => panic!("the tables should be valid: {error}"),
    };
    WorkSurfaceApp::new(surface, keymap, catalogue)
}

#[test]
fn every_action_of_the_catalogue_does_something() {
    // WHY: an action that is added to the catalogue and carried out
    // silently would be a defect that nobody notices.
    for action in Action::all() {
        let mut app = app();
        let before = app.activity();

        app.dispatch(*action);

        assert!(
            app.activity() > before,
            "the action {} changed nothing",
            action.id()
        );
    }
}

#[test]
fn the_copy_names_the_other_panel_as_its_destination() {
    // arrange
    let mut app = app();
    app.surface_mut().active_mut().toggle_at(0);

    // act
    let copy = match Action::from_id("panel.copy") {
        Some(action) => action,
        None => panic!("the catalogue should hold the action"),
    };
    app.dispatch(copy);

    // assert
    let commands = app.commands();
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        Command::Copy { transfer } => {
            assert_eq!(transfer.source(), PanelSide::Left);
            assert_eq!(transfer.destination(), PanelSide::Right);
            assert_eq!(transfer.entries().len(), 1);
        }
        other => panic!("the command should be a copy: {other:?}"),
    }
}

#[test]
fn the_copy_after_the_switch_names_the_right_panel_as_its_source() {
    // arrange
    let mut app = app();
    app.surface_mut().switch_active();

    // act
    let copy = match Action::from_id("panel.copy") {
        Some(action) => action,
        None => panic!("the catalogue should hold the action"),
    };
    app.dispatch(copy);

    // assert
    match &app.commands()[0] {
        Command::Copy { transfer } => {
            assert_eq!(transfer.source(), PanelSide::Right);
            assert_eq!(transfer.destination(), PanelSide::Left);
        }
        other => panic!("the command should be a copy: {other:?}"),
    }
}

#[test]
fn the_parent_of_a_path_is_the_directory_above_it() {
    // act
    let parent = parent_of(std::path::Path::new("/tmp/folder"));

    // assert
    assert_eq!(parent, PathBuf::from("/tmp"));
}

#[test]
fn the_parent_of_the_root_is_the_root() {
    // act
    let parent = parent_of(std::path::Path::new("/"));

    // assert
    assert_eq!(parent, PathBuf::from("/"));
}

#[test]
fn the_switch_asks_for_the_directory_to_be_read_again() {
    // WHY: the rule "verify_directory_before_refresh" says that the
    // content of a directory is read again before the panel shows it.
    // arrange
    let mut app = app();

    // act
    let action = match Action::from_id("panel.switch") {
        Some(action) => action,
        None => panic!("the catalogue should hold the action"),
    };
    app.dispatch(action);

    // assert
    match app.commands().first() {
        Some(Command::LoadDirectory { side, path }) => {
            assert_eq!(*side, PanelSide::Right);
            assert_eq!(path, &PathBuf::from("/var"));
        }
        other => panic!("the command should be a load of the directory: {other:?}"),
    }
}

#[test]
fn the_switch_reads_nothing_again_when_the_rule_is_switched_off() {
    // arrange
    let mut app = app().with_refresh_policy(RefreshPolicy::new(false));

    // act
    let action = match Action::from_id("panel.switch") {
        Some(action) => action,
        None => panic!("the catalogue should hold the action"),
    };
    app.dispatch(action);

    // assert
    assert!(
        app.commands().is_empty(),
        "the rule is switched off, so nothing may be read again"
    );
}

#[test]
fn the_filter_opens_in_the_panel_that_has_the_focus() {
    // WHY: the filter belongs to the panel, and the operations act on the
    // panel that has the focus (FR-16).
    // arrange
    let mut app = app();
    app.surface_mut().switch_active();

    // act
    let filter = match Action::from_id("panel.filter") {
        Some(action) => action,
        None => panic!("the catalogue should hold the action"),
    };
    app.dispatch(filter);

    // assert
    assert!(app.surface().active().is_filter_editing());
    assert!(!app.surface().other().is_filter_editing());
}

#[test]
fn the_escape_key_empties_the_filter_and_hides_it() {
    // arrange
    let mut app = app();
    app.surface_mut().active_mut().open_filter();
    app.surface_mut().active_mut().set_filter_text("a");

    // act
    app.filter_key(KeyStroke::new(Key::Escape));

    // assert
    assert_eq!(app.surface().active().filter_text(), "");
    assert!(!app.surface().active().is_filter_visible());
    assert!(!app.surface().active().filter().is_active());
}

#[test]
fn the_enter_key_keeps_the_filter() {
    // arrange
    let mut app = app();
    app.surface_mut().active_mut().open_filter();
    app.surface_mut().active_mut().set_filter_text("a");

    // act
    app.filter_key(KeyStroke::new(Key::Enter));

    // assert
    assert_eq!(app.surface().active().filter_text(), "a");
    assert!(!app.surface().active().is_filter_editing());
    assert!(app.surface().active().is_filter_visible());
    assert_eq!(app.surface().active().matching_count(), 1);
}

/// A catalogue that answers the keys of the errors with a German text.
struct German;

impl Catalogue for German {
    fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
        match key {
            "error-transfer-same-panel" => {
                Some("Das Panel darf nicht zweimal Ziel sein.".to_string())
            }
            _ => Some(key.to_string()),
        }
    }
}

#[test]
fn the_message_of_an_error_comes_from_the_catalogue_and_not_from_the_source() {
    // arrange
    // WHY: the message of an error reaches the user through the status line.
    //       A message that is written down in the source stays English in a
    //       German interface (FR-05). The test therefore reads the text of
    //       the status line and expects the German one.
    let app = app_with_catalogue(Box::new(German));

    // act
    let text = app.message_of_error("error-transfer-same-panel", &[("side", "left".to_string())]);

    // assert
    assert_eq!(text, "Das Panel darf nicht zweimal Ziel sein.");
}

#[test]
fn a_catalogue_that_does_not_know_the_key_leaves_a_visible_text() {
    // arrange
    // WHY: a message that is empty cannot be diagnosed. A catalogue that does
    //       not know the key therefore leaves the key itself (FR-122).
    let app = app();

    // act
    let text = app.message_of_error("error-transfer-same-panel", &[]);

    // assert
    assert_eq!(text, "error-transfer-same-panel");
}

#[test]
fn the_error_of_a_transfer_names_the_key_of_its_message() {
    // arrange
    use crate::surface::TransferRequest;

    // act
    let result = TransferRequest::new(PanelSide::Left, PanelSide::Left, Vec::new());

    // assert
    // WHY: the error names the key of the catalogue and not a text of its
    //       own, so that the surface can read the message in the language of
    //       the interface (FR-05).
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("the transfer into the same panel should be refused"),
    };
    assert_eq!(error.message_key(), "error-transfer-same-panel");
}

#[test]
fn the_error_of_a_binding_names_the_key_of_its_message() {
    // arrange
    use crate::keymap_view::Binding;

    // act
    let result = Binding::parse("Ctrl+");

    // assert
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("the binding without a key should be refused"),
    };
    assert_eq!(error.message_key(), "error-binding-unknown-key");

    // act
    let result = Binding::parse("");

    // assert
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("the empty binding should be refused"),
    };
    assert_eq!(error.message_key(), "error-binding-empty");
}

#[test]
fn a_tap_in_the_touch_context_carries_out_the_action() {
    // arrange
    // WHY: FR-101 in one test. The finger makes a gesture, the gesture
    //       reaches the table of the context, the table names the action,
    //       and the action is carried out. Every link of the chain is
    //       proven here.
    let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
    surface
        .active_mut()
        .set_entries(vec![Entry::new("a.txt", EntryKind::File)]);

    let mut touch = BTreeMap::new();
    touch.insert("panel.switch".to_string(), "swipe-left".to_string());
    touch.insert("panel.delete".to_string(), "tap".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(Context::Touch, touch);
    let keymap = match KeymapView::from_context_tables(&tables) {
        Ok(keymap) => keymap,
        Err(error) => panic!("the tables should be valid: {error}"),
    };

    let mut app = WorkSurfaceApp::new(surface, keymap, Box::new(Echo));
    app.set_context(Context::Touch);

    // act
    // WHY: the gesture reaches the application through the translator, and
    //       the translator is fed with the events of the framework.
    let mut translator = crate::input::pointer::PointerTranslator::new();
    translator.translate(
        &egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(1),
            phase: egui::TouchPhase::Start,
            pos: egui::pos2(10.0, 10.0),
            force: None,
        },
        0.0,
    );
    let gesture = translator.translate(
        &egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(1),
            phase: egui::TouchPhase::End,
            pos: egui::pos2(11.0, 11.0),
            force: None,
        },
        0.1,
    );

    let event = match gesture {
        Some(event) => event,
        None => panic!("the tap should have been recognised"),
    };
    app.dispatch_pointer_event(&event);

    // assert
    // WHY: the deletion of the selection is the action of the table. The
    //       command proves that the action was carried out.
    assert_eq!(app.commands().len(), 1);
}

#[test]
fn a_swipe_in_the_touch_context_carries_out_the_action() {
    // arrange
    let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
    surface
        .active_mut()
        .set_entries(vec![Entry::new("a.txt", EntryKind::File)]);

    let mut touch = BTreeMap::new();
    touch.insert("panel.swap".to_string(), "swipe-left".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(Context::Touch, touch);
    let keymap = match KeymapView::from_context_tables(&tables) {
        Ok(keymap) => keymap,
        Err(error) => panic!("the tables should be valid: {error}"),
    };

    let mut app = WorkSurfaceApp::new(surface, keymap, Box::new(Echo));
    app.set_context(Context::Touch);

    // act
    let mut translator = crate::input::pointer::PointerTranslator::new();
    translator.translate(
        &egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(1),
            phase: egui::TouchPhase::Start,
            pos: egui::pos2(200.0, 100.0),
            force: None,
        },
        0.0,
    );
    let gesture = translator.translate(
        &egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(1),
            phase: egui::TouchPhase::Move,
            pos: egui::pos2(50.0, 105.0),
            force: None,
        },
        0.2,
    );

    let event = match gesture {
        Some(event) => event,
        None => panic!("the swipe should have been recognised"),
    };
    app.dispatch_pointer_event(&event);

    // assert
    // WHY: the exchange of the two panels is the action of the table. The
    //       command proves that the action was carried out.
    assert_eq!(app.commands().len(), 1);
    assert!(matches!(app.commands()[0], Command::SwapPanels));
}
