//! The fields of the dialog of the multiple rename.
//!
//! WHAT: the buffers of the dialog ([`Fields`]).
//! WHY: the fields are the truth of the dialog: the rule is built out of them
//!       every time the engine is asked, so that a rule and the fields can
//!       never drift apart. The caret belongs to the fields as well, because
//!       only one field is edited at a time and the caret of the field that
//!       has the focus is the caret of the dialog.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::PanelSide;
//! use phocommander_ui::rename::{RenameDialog, RenameFocus};
//!
//! let dialog = RenameDialog::new(PanelSide::Left, Vec::new(), Vec::new());
//! assert_eq!(dialog.text_of(RenameFocus::Format), "[name]_[n].[ext]");
//! ```

use crate::rename::focus::RenameFocus;
use crate::rename::rule::{CounterOrder, CounterRule, RenameRule, ReplaceRule, SeriesRule};

/// The buffers of the dialog and the caret in them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Fields {
    /// The format of the new name.
    format: String,

    /// The text that is searched.
    search: String,

    /// The text that takes its place.
    replacement: String,

    /// Whether the search is a regular expression.
    regex: bool,

    /// The first number of the counter.
    counter_start: String,

    /// The step of the counter.
    counter_step: String,

    /// The width of the counter.
    counter_width: String,

    /// The name of the series.
    series_name: String,

    /// The name of the template that is to be stored.
    template_name: String,

    /// The position of the caret in the field that has the focus.
    caret: usize,
}

impl Default for Fields {
    /// Builds the fields of a dialog that was just opened.
    ///
    /// WHY: the format numbers the files of the selection, so that the
    ///       preview shows what the dialog is for. A format that leaves the
    ///       names alone would show a list of names that do not change, and
    ///       every file of it would look like a conflict.
    fn default() -> Self {
        Self {
            format: "[name]_[n].[ext]".to_string(),
            search: String::new(),
            replacement: String::new(),
            regex: false,
            counter_start: "1".to_string(),
            counter_step: "1".to_string(),
            counter_width: "3".to_string(),
            series_name: String::new(),
            template_name: String::new(),
            caret: 0,
        }
    }
}

impl Fields {
    /// The text of a field.
    pub(crate) fn text_of(&self, focus: RenameFocus) -> &str {
        match focus {
            RenameFocus::Format => self.format.as_str(),
            RenameFocus::Search => self.search.as_str(),
            RenameFocus::Replacement => self.replacement.as_str(),
            RenameFocus::CounterStart => self.counter_start.as_str(),
            RenameFocus::CounterStep => self.counter_step.as_str(),
            RenameFocus::CounterWidth => self.counter_width.as_str(),
            RenameFocus::SeriesName => self.series_name.as_str(),
            RenameFocus::TemplateName => self.template_name.as_str(),
            RenameFocus::Regex | RenameFocus::Templates | RenameFocus::Preview => "",
        }
    }

    /// The position of the caret in the field that has the focus.
    pub(crate) fn caret(&self) -> usize {
        self.caret
    }

    /// Writes text into a field at the caret.
    ///
    /// WHY: the preview follows every key stroke of the field (FR-34). The
    ///       text therefore goes into the buffer of the dialog and the engine
    ///       is asked in the frame that follows.
    pub(crate) fn insert(&mut self, focus: RenameFocus, text: &str) {
        // WHY: the caret is read before the buffer is borrowed, and the new
        //       caret is written after the borrow is over. The borrow of the
        //       buffer holds the whole field, and the caret is a part of it.
        let caret = self.caret;
        let count = text.chars().count();
        let next = {
            let Some(buffer) = self.buffer_mut(focus) else {
                return;
            };
            let position = byte_index_of(buffer, caret);
            buffer.insert_str(position, text);
            caret + count
        };
        self.caret = next;
    }

    /// Deletes the character before the caret.
    pub(crate) fn backspace(&mut self, focus: RenameFocus) {
        let caret = self.caret;
        let next = {
            let Some(buffer) = self.buffer_mut(focus) else {
                return;
            };
            if caret == 0 {
                return;
            }
            let position = byte_index_of(buffer, caret - 1);
            let end = byte_index_of(buffer, caret);
            buffer.replace_range(position..end, "");
            caret - 1
        };
        self.caret = next;
    }

    /// Deletes the character at the caret.
    pub(crate) fn delete(&mut self, focus: RenameFocus) {
        let caret = self.caret;
        let next = {
            let Some(buffer) = self.buffer_mut(focus) else {
                return;
            };
            if caret >= buffer.chars().count() {
                return;
            }
            let position = byte_index_of(buffer, caret);
            let end = byte_index_of(buffer, caret + 1);
            buffer.replace_range(position..end, "");
            caret
        };
        self.caret = next;
    }

    /// Moves the caret by a number of characters.
    pub(crate) fn move_caret(&mut self, focus: RenameFocus, delta: isize) {
        let last = self.text_of(focus).chars().count() as isize;
        let target = (self.caret as isize + delta).clamp(0, last);
        self.caret = target as usize;
    }

    /// Puts the caret in front of the text.
    pub(crate) fn caret_to_start(&mut self) {
        self.caret = 0;
    }

    /// Puts the caret behind the text of a field.
    ///
    /// WHY: the caret goes to the end of the text, because the user continues
    ///       a text that is already there and does not rewrite it.
    pub(crate) fn caret_to_end(&mut self, focus: RenameFocus) {
        self.caret = self.text_of(focus).chars().count();
    }

    /// Whether the search is a regular expression.
    pub(crate) fn regex(&self) -> bool {
        self.regex
    }

    /// Turns the search into a regular expression, or back.
    pub(crate) fn set_regex(&mut self, regex: bool) {
        self.regex = regex;
    }

    /// The name of the template that is to be stored.
    pub(crate) fn template_name(&self) -> &str {
        self.template_name.trim()
    }

    /// Whether a field carries a value that is not a number.
    ///
    /// WHY: a field that carries no number is drawn in the colour of a
    ///       warning, so that the user sees the mistake before the rename
    ///       runs.
    pub(crate) fn is_invalid(&self, focus: RenameFocus) -> bool {
        match focus {
            RenameFocus::CounterStart => number_of(self.counter_start.as_str()).is_none(),
            RenameFocus::CounterStep => number_of(self.counter_step.as_str()).is_none(),
            RenameFocus::CounterWidth => width_of(self.counter_width.as_str()).is_none(),
            _ => false,
        }
    }

    /// The rule that the fields describe.
    pub(crate) fn rule(&self) -> RenameRule {
        let mut rule = RenameRule::new(self.format.clone());
        if !self.search.is_empty() {
            rule = rule.with_replacement(ReplaceRule {
                search: self.search.clone(),
                replacement: self.replacement.clone(),
                regex: self.regex,
            });
        }
        rule = rule.with_counter(CounterRule {
            start: number_of(self.counter_start.as_str()).unwrap_or(1),
            step: number_of(self.counter_step.as_str()).unwrap_or(1),
            width: width_of(self.counter_width.as_str()).unwrap_or(3),
            order: CounterOrder::Selection,
        });
        rule = rule.with_series(SeriesRule {
            series_name: if self.series_name.is_empty() {
                None
            } else {
                Some(self.series_name.clone())
            },
            ..SeriesRule::default()
        });
        rule
    }

    /// Puts the rule of a template into the fields.
    ///
    /// WHY: a template that is applied shows its rule in the fields, so that
    ///       the user sees what the template does before the rename runs.
    pub(crate) fn apply_rule(&mut self, rule: &RenameRule) {
        self.format = rule.effective_format().to_string();
        self.search = rule
            .replacements
            .first()
            .map(|replacement| replacement.search.clone())
            .unwrap_or_default();
        self.replacement = rule
            .replacements
            .first()
            .map(|replacement| replacement.replacement.clone())
            .unwrap_or_default();
        self.regex = rule
            .replacements
            .first()
            .map(|replacement| replacement.regex)
            .unwrap_or(false);
        if let Some(counter) = &rule.counter {
            self.counter_start = counter.start.to_string();
            self.counter_step = counter.step.to_string();
            self.counter_width = counter.width.to_string();
        }
        self.series_name = rule
            .series
            .as_ref()
            .and_then(|series| series.series_name.clone())
            .unwrap_or_default();
    }

    /// The buffer of a field.
    fn buffer_mut(&mut self, focus: RenameFocus) -> Option<&mut String> {
        match focus {
            RenameFocus::Format => Some(&mut self.format),
            RenameFocus::Search => Some(&mut self.search),
            RenameFocus::Replacement => Some(&mut self.replacement),
            RenameFocus::CounterStart => Some(&mut self.counter_start),
            RenameFocus::CounterStep => Some(&mut self.counter_step),
            RenameFocus::CounterWidth => Some(&mut self.counter_width),
            RenameFocus::SeriesName => Some(&mut self.series_name),
            RenameFocus::TemplateName => Some(&mut self.template_name),
            RenameFocus::Regex | RenameFocus::Templates | RenameFocus::Preview => None,
        }
    }
}

/// The byte index of the given character position.
fn byte_index_of(text: &str, position: usize) -> usize {
    text.char_indices()
        .nth(position)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

/// The number of a value, if the value is a number.
fn number_of(text: &str) -> Option<i64> {
    text.trim().parse::<i64>().ok()
}

/// The width of a value, if the value is a width.
fn width_of(text: &str) -> Option<usize> {
    text.trim().parse::<usize>().ok().filter(|width| *width > 0)
}
