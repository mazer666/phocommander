//! The state of the input line of the filter.
//!
//! WHAT: the text of the field, the position of the caret and the two
//!       states of the field — shown and being edited ([`FilterField`]).
//! WHY: the field belongs to the panel and not to the frame, so that the
//!       text the user typed is still there when the panel is drawn again.
//!       The caret is a position between the characters and not a byte
//!       offset, so that a name with a character outside of ASCII can be
//!       edited.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::filter::FilterField;
//!
//! let mut field = FilterField::new();
//! field.open();
//! field.insert("notes");
//! assert_eq!(field.text(), "notes");
//! ```

/// The state of the input line of the filter.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FilterField {
    text: String,
    caret: usize,
    visible: bool,
    editing: bool,
}

impl FilterField {
    /// An empty field that is not shown.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text in the field.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The position of the caret between the characters.
    pub fn caret(&self) -> usize {
        self.caret
    }

    /// Whether the line with the field is drawn.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Whether the field has the focus.
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Shows the field and puts the caret into it.
    ///
    /// WHY: the caret goes to the end of the text, because the user
    /// continues a text that is already there and does not rewrite it.
    pub fn open(&mut self) {
        self.visible = true;
        self.editing = true;
        self.caret = self.text.chars().count();
    }

    /// Takes the focus from the field and keeps the text.
    pub fn confirm(&mut self) {
        self.editing = false;
    }

    /// Empties the field and hides it.
    pub fn clear(&mut self) {
        self.text.clear();
        self.caret = 0;
        self.visible = false;
        self.editing = false;
    }

    /// Writes the text into the field.
    ///
    /// WHY: the text of the field is the pattern of the filter, and the
    /// filter is applied while the user types (FR-74a). There is no delay
    /// and no confirmation between the key stroke and the new list.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.caret = self.text.chars().count();
    }

    /// Inserts text at the caret.
    pub fn insert(&mut self, text: &str) {
        let position = byte_index_of(&self.text, self.caret);
        self.text.insert_str(position, text);
        self.caret += text.chars().count();
    }

    /// Deletes the character before the caret.
    pub fn backspace(&mut self) {
        if self.caret == 0 {
            return;
        }
        let position = byte_index_of(&self.text, self.caret - 1);
        let end = byte_index_of(&self.text, self.caret);
        self.text.replace_range(position..end, "");
        self.caret -= 1;
    }

    /// Deletes the character at the caret.
    pub fn delete(&mut self) {
        if self.caret >= self.text.chars().count() {
            return;
        }
        let position = byte_index_of(&self.text, self.caret);
        let end = byte_index_of(&self.text, self.caret + 1);
        self.text.replace_range(position..end, "");
    }

    /// Moves the caret by a number of characters.
    pub fn move_caret(&mut self, delta: isize) {
        let last = self.text.chars().count() as isize;
        let target = (self.caret as isize + delta).clamp(0, last);
        self.caret = target as usize;
    }

    /// Puts the caret in front of the text.
    pub fn caret_to_start(&mut self) {
        self.caret = 0;
    }

    /// Puts the caret behind the text.
    pub fn caret_to_end(&mut self) {
        self.caret = self.text.chars().count();
    }
}

/// The byte index of the given character position.
fn byte_index_of(text: &str, position: usize) -> usize {
    text.char_indices()
        .nth(position)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}
