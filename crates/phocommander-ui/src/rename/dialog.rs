//! The state of the dialog of the multiple rename.
//!
//! WHAT: the state of the dialog ([`RenameDialog`]) and the parts of it that
//!       have the focus ([`RenameFocus`]).
//! WHY: the dialog shows a live preview of every new name before the rename
//!       runs (FR-34), it marks the conflicts and offers them as a jump list
//!       (FR-35), and it stores the named templates (FR-36). The state lives
//!       outside of the immediate mode context of egui, so that the preview is
//!       not rebuilt in every frame: the engine is asked only when the rule
//!       changed, and a frame that draws an unchanged rule costs nothing. The
//!       keys of the dialog are the keys of the catalogue (F3 and Shift+F3 for
//!       the conflict jump) and the keys of the editing of a text field — the
//!       same arrangement as the filter field of a panel.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::PanelSide;
//! use phocommander_ui::rename::{RenameDialog, RenameFile};
//!
//! let dialog = RenameDialog::new(
//!     PanelSide::Left,
//!     vec![RenameFile::new("/photos/sunset.jpg")],
//!     Vec::new(),
//! );
//! assert_eq!(dialog.files().len(), 1);
//! assert!(dialog.preview().is_none());
//! ```

use crate::panel::PanelSide;
use crate::rename::engine::{
    PreviewRow, RenameEngine, RenameFile, RenamePair, RenamePreview, RenameRequest,
};
use crate::rename::fields::Fields;
use crate::rename::focus::RenameFocus;
use crate::rename::rule::RenameRule;
use crate::rename::template::NamedTemplate;

/// The state of the dialog of the multiple rename.
#[derive(Debug)]
pub struct RenameDialog {
    /// The panel the files belong to.
    side: PanelSide,

    /// The files of the selection.
    files: Vec<RenameFile>,

    /// The fields that carry the rule.
    fields: Fields,

    /// The part of the dialog that has the focus.
    focus: RenameFocus,

    /// The last answer of the engine.
    preview: Option<RenamePreview>,

    /// Whether the engine has to be asked again.
    dirty: bool,

    /// The number of the answers of the engine.
    ///
    /// WHY: the counter is the evidence that the preview is not rebuilt in
    ///       every frame. A test reads it and knows that the engine is asked
    ///       only when the rule changed.
    engine_calls: usize,

    /// The row of the preview under the cursor.
    cursor: usize,

    /// The first row of the preview that is shown.
    scroll: usize,

    /// The rows of the preview that carry a conflict — the jump list.
    conflicts: Vec<usize>,

    /// The stored templates.
    templates: Vec<NamedTemplate>,

    /// The template under the cursor.
    template_cursor: usize,

    /// Whether the window of the dialog is open.
    ///
    /// WHY: the user can close the window with the mouse, and the shell has
    ///       to know that the dialog is gone. The flag lives in the state and
    ///       not in the frame, because the frame draws the window again and
    ///       would forget the click.
    window_open: bool,
}

impl RenameDialog {
    /// Builds the dialog for a selection.
    ///
    /// WHY: the dialog knows the panel the files belong to, because an
    ///       operation acts on the panel that has the focus (FR-16) and the
    ///       rename must not land in the other panel.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::panel::PanelSide;
    /// use phocommander_ui::rename::{RenameDialog, RenameFile};
    ///
    /// let dialog = RenameDialog::new(
    ///     PanelSide::Right,
    ///     vec![RenameFile::new("/photos/a.jpg")],
    ///     Vec::new(),
    /// );
    /// assert_eq!(dialog.side(), PanelSide::Right);
    /// ```
    pub fn new(side: PanelSide, files: Vec<RenameFile>, templates: Vec<NamedTemplate>) -> Self {
        let mut fields = Fields::default();
        fields.caret_to_end(RenameFocus::Format);
        Self {
            side,
            files,
            fields,
            focus: RenameFocus::Format,
            preview: None,
            dirty: true,
            engine_calls: 0,
            cursor: 0,
            scroll: 0,
            conflicts: Vec::new(),
            templates,
            template_cursor: 0,
            window_open: true,
        }
    }

    /// The panel the files belong to.
    pub fn side(&self) -> PanelSide {
        self.side
    }

    /// The files of the selection.
    pub fn files(&self) -> &[RenameFile] {
        &self.files
    }

    /// The part of the dialog that has the focus.
    pub fn focus(&self) -> RenameFocus {
        self.focus
    }

    /// Gives the focus to a part of the dialog.
    ///
    /// WHY: the caret goes to the end of the new field, because the user
    ///       continues a text that is already there.
    pub fn set_focus(&mut self, focus: RenameFocus) {
        if self.focus == focus {
            return;
        }
        self.focus = focus;
        self.fields.caret_to_end(focus);
    }

    /// Gives the focus to the next part of the dialog.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{RenameDialog, RenameFocus};
    /// # use phocommander_ui::panel::PanelSide;
    /// let mut dialog = RenameDialog::new(PanelSide::Left, Vec::new(), Vec::new());
    ///
    /// dialog.focus_next(false);
    ///
    /// assert_eq!(dialog.focus(), RenameFocus::Search);
    /// ```
    pub fn focus_next(&mut self, backwards: bool) {
        let all = RenameFocus::all();
        let index = all
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        let next = if backwards {
            if index == 0 { all.len() - 1 } else { index - 1 }
        } else {
            (index + 1) % all.len()
        };
        self.set_focus(all[next]);
    }

    /// The text of the field that has the focus.
    pub fn focused_text(&self) -> &str {
        self.text_of(self.focus)
    }

    /// The text of a field of the dialog.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::rename::{RenameDialog, RenameFocus};
    /// # use phocommander_ui::panel::PanelSide;
    /// let mut dialog = RenameDialog::new(PanelSide::Left, Vec::new(), Vec::new());
    ///
    /// assert_eq!(dialog.text_of(RenameFocus::Format), "[name]_[n].[ext]");
    /// ```
    pub fn text_of(&self, focus: RenameFocus) -> &str {
        self.fields.text_of(focus)
    }

    /// Whether the search is a regular expression.
    pub fn regex(&self) -> bool {
        self.fields.regex()
    }

    /// Turns the search into a regular expression, or back.
    ///
    /// WHY: the switch is off by default (FR-31). A search that is not a
    ///       regular expression is what a user expects when the text carries
    ///       a dot or a bracket.
    pub fn set_regex(&mut self, regex: bool) {
        self.fields.set_regex(regex);
        self.dirty = true;
    }

    /// Turns the search into a regular expression, or back.
    ///
    /// WHY: the switch is a part of the dialog and not a widget of the
    ///       drawing, so that the keyboard reaches it as well as the mouse.
    pub fn toggle_regex(&mut self) {
        let regex = !self.fields.regex();
        self.set_regex(regex);
    }

    /// Whether a field carries a value that is not a number.
    ///
    /// WHY: a field that carries no number is drawn in the colour of a
    ///       warning, so that the user sees the mistake before the rename
    ///       runs.
    pub fn is_invalid(&self, focus: RenameFocus) -> bool {
        self.fields.is_invalid(focus)
    }

    /// The position of the caret in the field that has the focus.
    pub fn caret(&self) -> usize {
        self.fields.caret()
    }

    /// Writes text into the field that has the focus.
    ///
    /// WHY: the preview follows every key stroke of the field (FR-34). The
    ///       dialog therefore writes the text into its own state and asks the
    ///       engine for the new names in the frame that follows.
    pub fn insert(&mut self, text: &str) {
        self.fields.insert(self.focus, text);
        self.mark_changed();
    }

    /// Deletes the character before the caret.
    pub fn backspace(&mut self) {
        self.fields.backspace(self.focus);
        self.mark_changed();
    }

    /// Deletes the character at the caret.
    pub fn delete(&mut self) {
        self.fields.delete(self.focus);
        self.mark_changed();
    }

    /// Moves the caret by a number of characters.
    pub fn move_caret(&mut self, delta: isize) {
        self.fields.move_caret(self.focus, delta);
    }

    /// Puts the caret in front of the text.
    pub fn caret_to_start(&mut self) {
        self.fields.caret_to_start();
    }

    /// Puts the caret behind the text.
    pub fn caret_to_end(&mut self) {
        self.fields.caret_to_end(self.focus);
    }

    /// Marks the rule as changed.
    ///
    /// WHY: the name of a template is not part of the rule, and the preview
    ///       does not change when it is typed.
    fn mark_changed(&mut self) {
        if self.focus != RenameFocus::TemplateName {
            self.dirty = true;
        }
    }

    /// The rule that the fields describe.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::panel::PanelSide;
    /// use phocommander_ui::rename::RenameDialog;
    ///
    /// let mut dialog = RenameDialog::new(PanelSide::Left, Vec::new(), Vec::new());
    /// assert_eq!(dialog.rule().format, "[name]_[n].[ext]");
    /// assert_eq!(dialog.rule().counter.map(|counter| counter.width), Some(3));
    /// ```
    pub fn rule(&self) -> RenameRule {
        self.fields.rule()
    }

    /// The last answer of the engine.
    pub fn preview(&self) -> Option<&RenamePreview> {
        self.preview.as_ref()
    }

    /// The rows of the preview.
    pub fn rows(&self) -> &[PreviewRow] {
        match &self.preview {
            Some(preview) => preview.rows.as_slice(),
            None => &[],
        }
    }

    /// The number of the answers of the engine.
    ///
    /// WHY: the number is the evidence that the preview is not rebuilt in
    ///       every frame. It grows with every change of the rule and with
    ///       nothing else.
    pub fn engine_calls(&self) -> usize {
        self.engine_calls
    }

    /// Asks the engine for the new names, if the rule changed.
    ///
    /// WHY: the preview follows every change of the rule (FR-34), and a
    ///       frame that draws an unchanged rule asks nothing. The engine is
    ///       therefore called once per change and not once per frame.
    pub fn update_preview(&mut self, engine: &dyn RenameEngine) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        self.engine_calls += 1;
        let request = RenameRequest::with_files(self.rule(), self.files.clone());
        let preview = engine.preview(&request);
        self.conflicts = conflict_rows(&preview);
        self.preview = Some(preview);
        let last = self.rows().len().saturating_sub(1);
        if self.cursor > last {
            self.cursor = last;
        }
    }

    /// The rows of the preview that carry a conflict — the jump list.
    ///
    /// WHY: the jump list holds the rows and not the paths, because the
    ///       cursor of the dialog walks the rows of the preview.
    pub fn conflicts(&self) -> &[usize] {
        &self.conflicts
    }

    /// Whether the row of the preview carries a conflict.
    pub fn is_conflict(&self, row: usize) -> bool {
        self.conflicts.contains(&row)
    }

    /// Puts the cursor on the next conflict.
    ///
    /// WHY: at the end of the list the cursor stays on the last entry. A
    ///       jump that wraps around would send the user to a place he has
    ///       already seen, and the list of the conflicts is short.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::panel::PanelSide;
    /// use phocommander_ui::rename::RenameDialog;
    ///
    /// let mut dialog = RenameDialog::new(PanelSide::Left, Vec::new(), Vec::new());
    /// assert!(!dialog.jump_to_next_conflict());
    /// ```
    pub fn jump_to_next_conflict(&mut self) -> bool {
        let Some(next) = self
            .conflicts
            .iter()
            .copied()
            .find(|index| *index > self.cursor)
        else {
            return false;
        };
        self.cursor = next;
        true
    }

    /// Puts the cursor on the previous conflict.
    pub fn jump_to_previous_conflict(&mut self) -> bool {
        let Some(previous) = self
            .conflicts
            .iter()
            .copied()
            .rev()
            .find(|index| *index < self.cursor)
        else {
            return false;
        };
        self.cursor = previous;
        true
    }

    /// The row of the preview under the cursor.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Moves the cursor by a number of rows.
    pub fn move_cursor(&mut self, delta: isize) {
        let last = self.rows().len().saturating_sub(1) as isize;
        let next = (self.cursor as isize + delta).clamp(0, last);
        self.cursor = next as usize;
    }

    /// The first row of the preview that is shown.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Scrolls so that the cursor is inside the window of the given rows.
    ///
    /// WHY: the cursor must never leave the screen — a dialog that scrolls
    ///       away from its own cursor has lost the place the user was at.
    pub fn ensure_cursor_visible(&mut self, rows: usize) {
        if rows == 0 {
            return;
        }
        let cursor = self.cursor;
        if cursor < self.scroll {
            self.scroll = cursor;
        } else if cursor >= self.scroll + rows {
            self.scroll = cursor + 1 - rows;
        }
    }

    /// The pairs of the rename, if the rename may run.
    ///
    /// WHY: the rename of a selection is atomic — either every file is
    ///       renamed or none of them is. A selection with a conflict and a
    ///       selection with a file that could not be named are therefore
    ///       refused as a whole (FR-41).
    pub fn pairs(&self) -> Option<Vec<RenamePair>> {
        let preview = self.preview.as_ref()?;
        if !preview.may_rename() {
            return None;
        }
        let mut pairs = Vec::with_capacity(preview.rows.len());
        for row in &preview.rows {
            let target = row.target.as_ref()?;
            // WHY: a file that keeps its name is not renamed. Such a file
            //       would be a rename of a file onto itself, and the engine
            //       refuses it.
            if *target == row.source {
                continue;
            }
            pairs.push(RenamePair::new(row.source.clone(), target.clone()));
        }
        Some(pairs)
    }

    /// Whether the rename may run.
    pub fn is_ready(&self) -> bool {
        self.pairs().is_some()
    }

    /// The stored templates.
    pub fn templates(&self) -> &[NamedTemplate] {
        &self.templates
    }

    /// The template under the cursor.
    pub fn template_cursor(&self) -> usize {
        self.template_cursor
    }

    /// Moves the cursor of the templates by a number of rows.
    pub fn move_template_cursor(&mut self, delta: isize) {
        let last = self.templates.len().saturating_sub(1) as isize;
        let next = (self.template_cursor as isize + delta).clamp(0, last);
        self.template_cursor = next as usize;
    }

    /// Puts the rule of a template into the fields.
    ///
    /// WHY: a template that is applied shows its rule in the fields, so that
    ///       the user sees what the template does before the rename runs.
    pub fn apply_template(&mut self, index: usize) -> bool {
        let Some(template) = self.templates.get(index) else {
            return false;
        };
        let rule = template.rule.clone();
        self.fields.apply_rule(&rule);
        self.dirty = true;
        self.fields.caret_to_end(self.focus);
        true
    }

    /// The template that the fields describe.
    ///
    /// WHY: the name of the template is the name the user typed. A template
    ///       without a name cannot be stored, because the list of the dialog
    ///       would show two entries without a name.
    pub fn current_template(&self) -> Option<NamedTemplate> {
        let name = self.fields.template_name();
        if name.is_empty() {
            return None;
        }
        Some(NamedTemplate::new(name, self.rule()))
    }

    /// Puts a template into the list.
    ///
    /// WHY: a template with the same name is replaced. A name stands for one
    ///       rule, and a second rule under the same name would make the list
    ///       ambiguous.
    pub fn add_template(&mut self, template: NamedTemplate) {
        let position = self
            .templates
            .iter()
            .position(|other| other.name == template.name);
        match position {
            Some(index) => {
                self.templates[index] = template;
                self.template_cursor = index;
            }
            None => {
                self.templates.push(template);
                self.template_cursor = self.templates.len().saturating_sub(1);
            }
        }
    }

    /// Whether the window of the dialog is open.
    ///
    /// WHY: the user can close the window with the mouse, and the shell has
    ///       to know that the dialog is gone.
    pub fn is_window_open(&self) -> bool {
        self.window_open
    }

    /// Remembers whether the window of the dialog is open.
    pub fn set_window_open(&mut self, open: bool) {
        self.window_open = open;
    }
}

/// The rows of the preview that carry a conflict.
fn conflict_rows(preview: &RenamePreview) -> Vec<usize> {
    let mut rows = Vec::new();
    for (index, row) in preview.rows.iter().enumerate() {
        let marked = preview.conflicts.iter().any(|conflict| {
            conflict.path == row.source
                || row
                    .target
                    .as_ref()
                    .is_some_and(|target| conflict.path == *target)
        });
        if marked {
            rows.push(index);
        }
    }
    rows
}
