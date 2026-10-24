//! Integration test: the shipped key binding file and the keymap crate.
//!
//! WHAT: the file `config/keymap.toml` that ships with the repository is read,
//!       its three context tables are handed to `phocommander_keymap`, and the
//!       resulting keymap is checked for conflicts and for completeness.
//! WHY:  the keymap file is the only place where a binding is written down.
//!       Two actions on the same key combination mean that the second one can
//!       never be reached — the conflict has to be found by the gate, not by
//!       the user in the middle of a copy operation.
//!
//!       The completeness check is the requirement of decision E9 of the
//!       specification: "Jede Aktion ist im `dpad`-Kontext erreichbar". A
//!       function that exists on the keyboard but not in the action palette is
//!       a function that a television user cannot reach, and Google Play
//!       checks exactly that. The test therefore compares the action sets of
//!       the contexts with each other.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test keymap_bindings
//! ```

use std::collections::BTreeMap;
use std::fs;

use phocommander_keymap::{ActionId, Context, Keymap};
use phocommander_testkit::repository_root;

/// The action that opens the action palette.
///
/// WHY: it is not part of the action catalogue — it is a function of the
/// surface — but it is bound in the dpad and in the controller context.
const PALETTE_OPEN: &str = "palette.open";

/// Reads the context tables of the shipped key binding file.
fn shipped_context_tables() -> BTreeMap<Context, BTreeMap<String, String>> {
    let path = repository_root().join("config").join("keymap.toml");
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => panic!("the key binding file {path:?} could not be read: {error}"),
    };

    let document: toml::Table = match toml::from_str(&source) {
        Ok(document) => document,
        Err(error) => panic!("the key binding file {path:?} is not valid TOML: {error}"),
    };

    let input = match document.get("input") {
        Some(input) => input,
        None => panic!("the key binding file {path:?} has no [input] section"),
    };

    let mut tables: BTreeMap<Context, BTreeMap<String, String>> = BTreeMap::new();
    for context in [Context::Keyboard, Context::Dpad, Context::Controller] {
        let table = match input.get(context.config_name()) {
            Some(table) => table,
            None => panic!(
                "the key binding file {path:?} has no [input.{}] section",
                context.config_name()
            ),
        };

        let mut bindings: BTreeMap<String, String> = BTreeMap::new();
        for (action, key) in match table.as_table() {
            Some(table) => table,
            None => panic!(
                "the section [input.{}] is not a table",
                context.config_name()
            ),
        } {
            let key = match key.as_str() {
                Some(key) => key,
                None => panic!("the binding of {action} is not a string"),
            };
            bindings.insert(action.clone(), key.to_string());
        }
        tables.insert(context, bindings);
    }
    tables
}

/// Builds the keymap of the shipped file.
fn shipped_keymap() -> Keymap {
    let tables = shipped_context_tables();
    match Keymap::from_context_tables(&tables) {
        Ok(keymap) => keymap,
        Err(error) => panic!("the shipped key binding file was refused: {error}"),
    }
}

/// Returns the action identifiers of a context table.
fn actions_of(
    tables: &BTreeMap<Context, BTreeMap<String, String>>,
    context: Context,
) -> Vec<String> {
    match tables.get(&context) {
        Some(table) => table.keys().cloned().collect(),
        None => Vec::new(),
    }
}

#[test]
fn the_shipped_keymap_has_no_conflict_in_any_context() {
    // arrange
    let keymap = shipped_keymap();

    // act
    let conflicts = keymap.conflicts();

    // assert — the only bindings that are used twice are the two that the
    // specification names: `F3` searches forward in the viewer and jumps to
    // the next conflict in the rename dialog, `Shift+F3` does the opposite.
    // Both dialogs are modal and never open at the same time, so the two
    // meanings never collide. Every other conflict would be a defect.
    let documented: Vec<(Context, &str)> =
        vec![(Context::Keyboard, "F3"), (Context::Keyboard, "Shift+F3")];
    for conflict in &conflicts {
        let is_documented = documented
            .iter()
            .any(|(context, key)| *context == conflict.context() && *key == conflict.key());
        assert!(
            is_documented,
            "the shipped key binding file binds {:?} twice in the {} context: {:?}",
            conflict.key(),
            conflict.context(),
            conflict.actions()
        );
    }
    assert_eq!(
        conflicts.len(),
        documented.len(),
        "the shipped key binding file has fewer conflicts than the two documented ones: {conflicts:?}"
    );
}

#[test]
fn every_action_of_the_keyboard_is_reachable_in_the_dpad_context() {
    // arrange
    let tables = shipped_context_tables();

    // act
    let keyboard = actions_of(&tables, Context::Keyboard);
    let dpad = actions_of(&tables, Context::Dpad);

    // assert — every action of the keyboard is in the action palette, and the
    // palette adds the action that opens it.
    for action in &keyboard {
        assert!(
            dpad.contains(action),
            "the action {action} exists on the keyboard but is not reachable with a remote control"
        );
    }
    let extra: Vec<&String> = dpad
        .iter()
        .filter(|action| !keyboard.contains(action))
        .collect();
    assert_eq!(
        extra,
        vec![&PALETTE_OPEN.to_string()],
        "the dpad context binds actions that are not part of the catalogue"
    );
}

#[test]
fn every_action_of_the_keyboard_is_reachable_in_the_controller_context() {
    // arrange
    let tables = shipped_context_tables();

    // act
    let keyboard = actions_of(&tables, Context::Keyboard);
    let controller = actions_of(&tables, Context::Controller);

    // assert
    for action in &keyboard {
        assert!(
            controller.contains(action),
            "the action {action} exists on the keyboard but is not reachable with a controller"
        );
    }
}

#[test]
fn the_dpad_and_the_controller_bind_their_actions_to_the_action_palette() {
    // arrange
    let tables = shipped_context_tables();

    // act
    let dpad = tables.get(&Context::Dpad).cloned().unwrap_or_default();
    let controller = tables
        .get(&Context::Controller)
        .cloned()
        .unwrap_or_default();

    // assert — the remote control and the controller have no function keys,
    // so every action is reached through the palette. The two documented
    // exceptions are the action that opens the palette and the grip, which
    // toggles the marking of an entry.
    for (action, key) in &dpad {
        if action == PALETTE_OPEN {
            assert_eq!(key, "Menu", "the palette must open with the Menu key");
            continue;
        }
        assert!(
            key.starts_with("palette:"),
            "the action {action} is bound to {key} in the dpad context, but a remote control has no function keys"
        );
    }
    for (action, key) in &controller {
        if action == PALETTE_OPEN || action == "selection.toggle" {
            continue;
        }
        assert!(
            key.starts_with("palette:"),
            "the action {action} is bound to {key} in the controller context, but a controller has no function keys"
        );
    }
    assert_eq!(
        controller.get("selection.toggle"),
        Some(&"Grip".to_string()),
        "the grip toggles the marking of an entry"
    );
}

#[test]
fn the_action_identifiers_of_the_shipped_file_are_well_formed() {
    // arrange
    let tables = shipped_context_tables();

    // act
    let mut refused: Vec<String> = Vec::new();
    for (context, table) in &tables {
        for action in table.keys() {
            if ActionId::new(action).is_err() {
                refused.push(format!("{}: {action}", context.config_name()));
            }
        }
    }

    // assert
    assert!(
        refused.is_empty(),
        "the shipped key binding file contains malformed action identifiers: {refused:?}"
    );
}
