// The tests of the application shell.
//
// WHY: the tests live in a file of their own, so that the shell itself stays
// readable. They may fail loudly when their arrangement is wrong — the rule
// that forbids a panic applies to the production code, and it is enforced
// there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::Argument;
use crate::keymap_view::KeymapView;
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
