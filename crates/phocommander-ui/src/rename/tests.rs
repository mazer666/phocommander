// The tests of the dialog of the multiple rename.
//
// WHY: the tests live in a file of their own, so that the dialog itself stays
// readable. They may fail loudly when their arrangement is wrong — the rule
// that forbids a panic applies to the production code, and it is enforced
// there.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::panel::PanelSide;
use crate::rename::engine::{ConflictKind, RenameEngine};

/// An engine that forms the names out of the format of the rule.
struct Forming {
    /// The number of the answers.
    calls: std::cell::Cell<usize>,

    /// The row that carries a conflict, if there is one.
    conflict: Option<usize>,
}

impl Forming {
    /// An engine without a conflict.
    fn new() -> Self {
        Self {
            calls: std::cell::Cell::new(0),
            conflict: None,
        }
    }

    /// An engine that marks the given row as a conflict.
    fn with_conflict(row: usize) -> Self {
        Self {
            calls: std::cell::Cell::new(0),
            conflict: Some(row),
        }
    }
}

impl RenameEngine for Forming {
    fn preview(&self, request: &RenameRequest) -> RenamePreview {
        self.calls.set(self.calls.get() + 1);
        let mut preview = RenamePreview::empty();
        for (index, file) in request.files.iter().enumerate() {
            let stem = file
                .path
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_default();
            let extension = file
                .path
                .extension()
                .map(|extension| extension.to_string_lossy().to_string())
                .unwrap_or_default();
            let name = request
                .rule
                .format
                .replace("[name]", stem.as_str())
                .replace("[n]", &format!("{:03}", index + 1))
                .replace("[ext]", extension.as_str());
            preview
                .rows
                .push(PreviewRow::named(file.path.clone(), name));
        }
        if let Some(entry) = self.conflict.and_then(|row| preview.rows.get(row)) {
            let path = entry.target.clone().unwrap_or_else(|| entry.source.clone());
            preview
                .conflicts
                .push(Conflict::new(ConflictKind::TargetExists, path));
        }
        preview
    }
}

/// The dialog with three files.
fn dialog() -> RenameDialog {
    RenameDialog::new(
        PanelSide::Left,
        vec![
            RenameFile::new("/photos/a.jpg"),
            RenameFile::new("/photos/b.jpg"),
            RenameFile::new("/photos/c.jpg"),
        ],
        Vec::new(),
    )
}

/// The number of characters of the format of a new dialog.
fn format_length() -> usize {
    "[name]_[n].[ext]".chars().count()
}

#[test]
fn the_preview_follows_every_change_of_the_rule() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::new();
    dialog.update_preview(&engine);

    // act
    dialog.set_focus(RenameFocus::Format);
    dialog.caret_to_start();
    dialog.insert("x");
    dialog.update_preview(&engine);

    // assert
    assert_eq!(engine.calls.get(), 2);
    assert_eq!(dialog.rows()[0].name.as_deref(), Some("xa_001.jpg"));
}

#[test]
fn the_engine_is_not_asked_again_when_nothing_changed() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::new();
    dialog.update_preview(&engine);

    // act — three frames that draw the same rule.
    dialog.update_preview(&engine);
    dialog.update_preview(&engine);

    // assert
    assert_eq!(engine.calls.get(), 1);
    assert_eq!(dialog.engine_calls(), 1);
}

#[test]
fn a_conflict_is_marked_and_the_jump_list_names_it() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::with_conflict(1);

    // act
    dialog.update_preview(&engine);

    // assert
    assert_eq!(dialog.conflicts(), &[1]);
    assert!(dialog.is_conflict(1));
    assert!(!dialog.is_conflict(0));
}

#[test]
fn the_next_conflict_is_reachable_and_the_end_of_the_list_stays() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::with_conflict(1);
    dialog.update_preview(&engine);

    // act
    let jumped = dialog.jump_to_next_conflict();

    // assert
    assert!(jumped);
    assert_eq!(dialog.cursor(), 1);

    // act — the list has no second entry, so the cursor stays.
    let jumped = dialog.jump_to_next_conflict();

    // assert
    assert!(!jumped);
    assert_eq!(dialog.cursor(), 1);
}

#[test]
fn the_previous_conflict_is_reachable() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::with_conflict(1);
    dialog.update_preview(&engine);

    // act — the cursor stands behind the conflict.
    dialog.move_cursor(2);
    let jumped = dialog.jump_to_previous_conflict();

    // assert
    assert!(jumped);
    assert_eq!(dialog.cursor(), 1);
}

#[test]
fn the_rename_is_refused_while_a_conflict_is_on_the_list() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::with_conflict(1);
    dialog.update_preview(&engine);

    // act
    let pairs = dialog.pairs();

    // assert
    assert!(pairs.is_none());
    assert!(!dialog.is_ready());
}

#[test]
fn the_rename_is_possible_without_a_conflict() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::new();
    dialog.update_preview(&engine);

    // act
    let pairs = dialog.pairs();

    // assert
    let pairs = match pairs {
        Some(pairs) => pairs,
        None => panic!("the rename should be possible"),
    };
    assert_eq!(pairs.len(), 3);
    assert_eq!(pairs[0].target_name(), "a_001.jpg");
}

#[test]
fn a_file_that_keeps_its_name_is_not_renamed() {
    // arrange
    let mut dialog = RenameDialog::new(
        PanelSide::Left,
        vec![RenameFile::new("/photos/a.jpg")],
        Vec::new(),
    );
    let engine = Forming::new();

    // act — the format of the rule is the name of the file itself.
    dialog.set_focus(RenameFocus::Format);
    for _ in 0..format_length() {
        dialog.backspace();
    }
    dialog.insert("[name].[ext]");
    dialog.update_preview(&engine);

    // assert
    let pairs = dialog.pairs();
    assert!(pairs.is_some());
    let pairs = pairs.unwrap_or_default();
    assert!(pairs.is_empty());
}

#[test]
fn a_stored_template_can_be_applied_again() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::new();
    dialog.update_preview(&engine);
    let before = dialog.rows()[0].name.clone();

    // act — the rule is stored under a name, changed and applied again.
    dialog.set_focus(RenameFocus::TemplateName);
    dialog.insert("Camera");
    let template = match dialog.current_template() {
        Some(template) => template,
        None => panic!("the template should be built"),
    };
    dialog.add_template(template);
    dialog.set_focus(RenameFocus::Format);
    for _ in 0..format_length() {
        dialog.backspace();
    }
    dialog.insert("[name]-[n].[ext]");
    dialog.update_preview(&engine);
    dialog.apply_template(0);
    dialog.update_preview(&engine);

    // assert
    assert_eq!(dialog.templates().len(), 1);
    assert_eq!(dialog.templates()[0].name, "Camera");
    assert_eq!(dialog.rows()[0].name, before);
}

#[test]
fn a_template_without_a_name_is_not_stored() {
    // arrange
    let dialog = dialog();

    // act
    let template = dialog.current_template();

    // assert
    assert!(template.is_none());
}

#[test]
fn a_template_with_the_same_name_replaces_the_old_one() {
    // arrange
    let mut dialog = dialog();
    let first = NamedTemplate::new("Camera", RenameRule::new("[name]"));
    let second = NamedTemplate::new("Camera", RenameRule::new("[n]"));

    // act
    dialog.add_template(first);
    dialog.add_template(second);

    // assert
    assert_eq!(dialog.templates().len(), 1);
    assert_eq!(dialog.templates()[0].rule.format, "[n]");
}

#[test]
fn the_focus_walks_the_dialog_in_a_fixed_order() {
    // arrange
    let mut dialog = dialog();

    // act — the tab key walks the parts of the dialog and comes back.
    let mut visited = Vec::new();
    for _ in 0..RenameFocus::all().len() {
        visited.push(dialog.focus());
        dialog.focus_next(false);
    }

    // assert
    assert_eq!(visited[0], RenameFocus::Format);
    assert_eq!(visited[visited.len() - 1], RenameFocus::Preview);
    assert_eq!(dialog.focus(), RenameFocus::Format);
}

#[test]
fn the_backward_focus_stays_at_the_beginning() {
    // arrange
    let mut dialog = dialog();

    // act
    dialog.focus_next(true);

    // assert
    assert_eq!(dialog.focus(), RenameFocus::Preview);
}

#[test]
fn the_name_of_a_template_does_not_change_the_rule() {
    // arrange
    let mut dialog = dialog();
    let engine = Forming::new();
    dialog.update_preview(&engine);

    // act
    dialog.set_focus(RenameFocus::TemplateName);
    dialog.insert("Camera");
    dialog.update_preview(&engine);

    // assert
    assert_eq!(engine.calls.get(), 1);
}

#[test]
fn a_field_that_carries_no_number_is_reported() {
    // arrange
    let mut dialog = dialog();

    // act
    dialog.set_focus(RenameFocus::CounterStart);
    dialog.insert("x");

    // assert
    assert!(dialog.is_invalid(RenameFocus::CounterStart));
    assert!(!dialog.is_invalid(RenameFocus::CounterStep));
}
