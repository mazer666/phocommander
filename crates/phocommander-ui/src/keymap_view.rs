//! The resolution of an input event to an action.
//!
//! WHAT: the keys of the keyboard ([`Key`]), a key with its modifiers
//!       ([`KeyStroke`]), the binding of an action ([`Binding`]), the event
//!       of the input layer ([`InputEvent`]) and the binding table of the
//!       contexts ([`KeymapView`]).
//! WHY: the surface never asks which key was pressed, it asks which command
//!       was requested (FR-100). The resolution is a pure function of the
//!       event, the context and the binding table — and it is deterministic:
//!       when two actions of one context are bound to the same key, the
//!       action that comes first in the catalogue wins (FR-101).
//! SIZE: this module holds the whole input layer in one place on purpose —
//!       the keys, the strokes, the bindings, the events and the table of the
//!       contexts. The table of the contexts refers to the bindings of every
//!       other table, and a split would make the resolution of an event
//!       readable in three files instead of one.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::{Binding, Context, InputEvent, KeyStroke};
//!
//! let stroke = KeyStroke::parse("Ctrl+R").unwrap_or(KeyStroke::parse("F1").unwrap_or(KeyStroke::escape()));
//! assert!(matches!(stroke, KeyStroke { .. }));
//! assert!(matches!(Binding::parse("palette:file.copy"), Ok(Binding::Palette(_))));
//! ```

use std::collections::BTreeMap;
use std::fmt;

use crate::actions::Action;
use crate::context::Context;

/// The prefix of a binding that names an entry of the action palette.
const PALETTE_PREFIX: &str = "palette:";

/// A key of the keyboard, free of any framework type.
///
/// WHY: the binding table of the configuration file is written in words
/// ("F5", "Ctrl+Shift+Tab", "Ctrl+Num-"), and the surface must be able to
/// compare a written binding with a pressed key. The key is therefore a type
/// of its own that both sides can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    /// A function key, `F1` to `F12`.
    Function(u8),

    /// A letter of the keyboard.
    Letter(char),

    /// A digit of the main block.
    Digit(u8),

    /// A digit of the numeric keypad.
    NumpadDigit(u8),

    /// The plus key of the numeric keypad.
    NumpadAdd,

    /// The minus key of the numeric keypad.
    NumpadSubtract,

    /// The star key of the numeric keypad.
    NumpadMultiply,

    /// The slash key of the numeric keypad.
    NumpadDivide,

    /// The point key of the numeric keypad.
    NumpadDecimal,

    /// The tab key.
    Tab,

    /// The enter key.
    Enter,

    /// The space bar.
    Space,

    /// The insert key.
    Insert,

    /// The delete key.
    Delete,

    /// The backspace key.
    Backspace,

    /// The escape key.
    Escape,

    /// The home key.
    Home,

    /// The end key.
    End,

    /// The page up key.
    PageUp,

    /// The page down key.
    PageDown,

    /// The arrow key up.
    ArrowUp,

    /// The arrow key down.
    ArrowDown,

    /// The arrow key left.
    ArrowLeft,

    /// The arrow key right.
    ArrowRight,

    /// The menu key of the keyboard.
    Menu,

    /// The play and pause key of a remote control.
    MediaPlayPause,
}

impl Default for Key {
    /// The key that a key stroke without a key carries.
    ///
    /// WHY: the escape key is on every keyboard and triggers no action of the
    /// catalogue on its own.
    fn default() -> Self {
        Key::Escape
    }
}

impl Key {
    /// The key with the name the configuration file uses.
    ///
    /// WHY: the names in the catalogue are the names of the keys as the
    /// documentation writes them ("Ins", "PgUp", "Num-"). A name that the
    /// surface does not know is refused with its name, so that the mistake
    /// can be found in the file that caused it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::Key;
    ///
    /// assert_eq!(Key::from_name("F5"), Some(Key::Function(5)));
    /// assert_eq!(Key::from_name("Ins"), Some(Key::Insert));
    /// assert_eq!(Key::from_name("Num-"), Some(Key::NumpadSubtract));
    /// ```
    pub fn from_name(name: &str) -> Option<Key> {
        if let Some(number) = name.strip_prefix('F')
            && let Ok(number) = number.parse::<u8>()
            && (1..=12).contains(&number)
        {
            return Some(Key::Function(number));
        }

        if let Some(number) = name.strip_prefix("Num") {
            return match number {
                "+" => Some(Key::NumpadAdd),
                "-" => Some(Key::NumpadSubtract),
                "*" => Some(Key::NumpadMultiply),
                "/" => Some(Key::NumpadDivide),
                "." => Some(Key::NumpadDecimal),
                digits => digits
                    .parse::<u8>()
                    .ok()
                    .filter(|digit| *digit <= 9)
                    .map(Key::NumpadDigit),
            };
        }

        if let Some(digit) = name.parse::<u8>().ok().filter(|digit| *digit <= 9) {
            return Some(Key::Digit(digit));
        }

        if name.len() == 1
            && let Some(letter) = name.chars().next()
            && letter.is_ascii_alphabetic()
        {
            return Some(Key::Letter(letter.to_ascii_lowercase()));
        }

        match name {
            "Tab" => Some(Key::Tab),
            "Enter" | "Return" => Some(Key::Enter),
            "Space" => Some(Key::Space),
            "Ins" | "Insert" => Some(Key::Insert),
            "Delete" | "Del" => Some(Key::Delete),
            "Backspace" => Some(Key::Backspace),
            "Esc" | "Escape" => Some(Key::Escape),
            "Home" => Some(Key::Home),
            "End" => Some(Key::End),
            "PgUp" | "PageUp" => Some(Key::PageUp),
            "PgDn" | "PageDown" => Some(Key::PageDown),
            "Up" | "ArrowUp" => Some(Key::ArrowUp),
            "Down" | "ArrowDown" => Some(Key::ArrowDown),
            "Left" | "ArrowLeft" => Some(Key::ArrowLeft),
            "Right" | "ArrowRight" => Some(Key::ArrowRight),
            "Menu" => Some(Key::Menu),
            "Play" | "PlayPause" | "MediaPlayPause" => Some(Key::MediaPlayPause),
            _ => None,
        }
    }

    /// The name of the key as the configuration file writes it.
    pub fn name(&self) -> String {
        match self {
            Key::Function(number) => format!("F{number}"),
            Key::Letter(letter) => letter.to_string(),
            Key::Digit(digit) => digit.to_string(),
            Key::NumpadDigit(digit) => format!("Num{digit}"),
            Key::NumpadAdd => "Num+".to_string(),
            Key::NumpadSubtract => "Num-".to_string(),
            Key::NumpadMultiply => "Num*".to_string(),
            Key::NumpadDivide => "Num/".to_string(),
            Key::NumpadDecimal => "Num.".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::Enter => "Enter".to_string(),
            Key::Space => "Space".to_string(),
            Key::Insert => "Ins".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::Backspace => "Backspace".to_string(),
            Key::Escape => "Esc".to_string(),
            Key::Home => "Home".to_string(),
            Key::End => "End".to_string(),
            Key::PageUp => "PgUp".to_string(),
            Key::PageDown => "PgDn".to_string(),
            Key::ArrowUp => "Up".to_string(),
            Key::ArrowDown => "Down".to_string(),
            Key::ArrowLeft => "Left".to_string(),
            Key::ArrowRight => "Right".to_string(),
            Key::Menu => "Menu".to_string(),
            Key::MediaPlayPause => "Play".to_string(),
        }
    }
}

/// The modifier keys of a key stroke.
///
/// WHY: the configuration file writes `Ctrl` for the *primary* modifier, and
/// the primary modifier is `Cmd` on macOS and on iPadOS. The stroke therefore
/// carries a primary flag instead of a control flag, so that the same table
/// works on every platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Modifiers {
    /// The primary modifier — `Cmd` on macOS, `Ctrl` everywhere else.
    pub primary: bool,

    /// The shift key.
    pub shift: bool,

    /// The alt key.
    pub alt: bool,
}

impl Modifiers {
    /// The modifiers of a key without any modifier key.
    pub const NONE: Modifiers = Modifiers {
        primary: false,
        shift: false,
        alt: false,
    };

    /// Whether no modifier key is pressed.
    pub const fn is_empty(&self) -> bool {
        !self.primary && !self.shift && !self.alt
    }
}

/// A key with its modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct KeyStroke {
    /// The key that was pressed.
    pub key: Key,

    /// The modifier keys that were held down.
    pub modifiers: Modifiers,
}

impl KeyStroke {
    /// A key stroke without any modifier key.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::{Key, KeyStroke};
    ///
    /// let stroke = KeyStroke::new(Key::Function(5));
    /// assert!(stroke.modifiers.is_empty());
    /// ```
    pub const fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::NONE,
        }
    }

    /// The key stroke of the escape key.
    pub const fn escape() -> Self {
        Self::new(Key::Escape)
    }

    /// Reads a key stroke from the notation of the configuration file.
    ///
    /// WHY: the notation of the catalogue is "Ctrl+Shift+Tab". The stroke is
    /// parsed into its parts, so that a written binding and a pressed key can
    /// be compared without going through the text again.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::{Key, KeyStroke};
    ///
    /// # fn main() -> Result<(), phocommander_ui::BindingError> {
    /// let stroke = KeyStroke::parse("Ctrl+Shift+Tab")?;
    /// assert_eq!(stroke.key, Key::Tab);
    /// assert!(stroke.modifiers.primary);
    /// assert!(stroke.modifiers.shift);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(text: &str) -> Result<Self, BindingError> {
        // WHY: the text is checked before it is split. A text that carries
        //       nothing at all is not a binding that names an unknown key —
        //       it is a binding that is empty, and the message of the
        //       catalogue says so.
        if text.trim().is_empty() {
            return Err(BindingError::Malformed {
                binding: text.to_string(),
                reason: MalformedReason::Empty,
            });
        }

        let parts = split_parts(text);
        if parts.is_empty() {
            return Err(BindingError::Malformed {
                binding: text.to_string(),
                reason: MalformedReason::Empty,
            });
        }

        let mut modifiers = Modifiers::NONE;
        let mut key = None;

        for part in &parts {
            match part.as_str() {
                "Ctrl" | "Cmd" | "Command" | "Super" => modifiers.primary = true,
                "Shift" => modifiers.shift = true,
                "Alt" | "Option" => modifiers.alt = true,
                name => {
                    if key.is_some() {
                        return Err(BindingError::Malformed {
                            binding: text.to_string(),
                            reason: MalformedReason::KeyTwice(name.to_string()),
                        });
                    }
                    let parsed = Key::from_name(name).ok_or_else(|| BindingError::UnknownKey {
                        binding: text.to_string(),
                        key: name.to_string(),
                    })?;
                    key = Some(parsed);
                }
            }
        }

        let key = key.ok_or_else(|| BindingError::Malformed {
            binding: text.to_string(),
            reason: MalformedReason::NoKey,
        })?;

        Ok(Self { key, modifiers })
    }

    /// The spelling of the stroke as the configuration file writes it.
    ///
    /// WHY: the status line of the interface shows the binding of a command,
    /// and it has to show it the way the configuration file writes it.
    pub fn to_binding(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers.primary {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.shift {
            parts.push("Shift".to_string());
        }
        if self.modifiers.alt {
            parts.push("Alt".to_string());
        }
        parts.push(self.key.name());
        parts.join("+")
    }
}

impl fmt::Display for KeyStroke {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_binding())
    }
}

/// The binding of an action: a key or an entry of the action palette.
///
/// WHY: a remote control has no function keys, so the action palette takes
/// their place there. The binding therefore has two shapes, and the shape
/// decides how the action is reached.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Binding {
    /// A key with its modifier keys.
    Key(KeyStroke),

    /// An entry of the action palette.
    Palette(String),
}

impl Binding {
    /// Reads a binding from the notation of the configuration file.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::Binding;
    ///
    /// # fn main() -> Result<(), phocommander_ui::BindingError> {
    /// assert!(matches!(Binding::parse("palette:file.copy")?, Binding::Palette(_)));
    /// assert!(matches!(Binding::parse("F5")?, Binding::Key(_)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(text: &str) -> Result<Self, BindingError> {
        if let Some(entry) = text.strip_prefix(PALETTE_PREFIX) {
            if entry.is_empty() {
                return Err(BindingError::Malformed {
                    binding: text.to_string(),
                    reason: MalformedReason::EmptyPaletteEntry,
                });
            }
            return Ok(Binding::Palette(entry.to_string()));
        }

        Ok(Binding::Key(KeyStroke::parse(text)?))
    }

    /// The spelling of the binding as the configuration file writes it.
    pub fn to_binding(&self) -> String {
        match self {
            Binding::Key(stroke) => stroke.to_binding(),
            Binding::Palette(entry) => format!("{PALETTE_PREFIX}{entry}"),
        }
    }
}

/// An event of the input layer.
///
/// WHY: the input layer of the platform delivers a key, a click, a gesture or
/// a button. The surface only knows the two shapes that reach it as a
/// command: a key and an entry of the action palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// A key was pressed.
    Key(KeyStroke),

    /// An entry of the action palette was chosen.
    Palette(String),
}

/// Two actions of one context that are bound to the same key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingConflict {
    context: Context,
    binding: Binding,
    kept: Action,
    dropped: Action,
}

impl BindingConflict {
    /// The context the conflict happened in.
    pub fn context(&self) -> Context {
        self.context
    }

    /// The binding that is bound twice.
    pub fn binding(&self) -> &Binding {
        &self.binding
    }

    /// The action that wins the binding.
    pub fn kept(&self) -> Action {
        self.kept
    }

    /// The action that loses the binding.
    pub fn dropped(&self) -> Action {
        self.dropped
    }
}

/// The binding table of every context.
///
/// WHY: the binding table is the only place where a key becomes an action.
/// The resolution is a pure function, so that it can be tested without a
/// window and so that the same event always produces the same action.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeymapView {
    contexts: BTreeMap<Context, ContextBindings>,
}

impl KeymapView {
    /// Reads the binding table of every context.
    ///
    /// WHY: the tables come from the configuration file, and the surface
    /// must not read the file itself. The tables are therefore the input of
    /// this function, and the catalogue order decides every conflict.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use std::collections::BTreeMap;
    /// use phocommander_ui::{Context, KeymapView};
    ///
    /// # fn main() -> Result<(), phocommander_ui::BindingError> {
    /// let mut keyboard = BTreeMap::new();
    /// keyboard.insert("panel.copy".to_string(), "F5".to_string());
    /// let mut tables = BTreeMap::new();
    /// tables.insert(Context::Keyboard, keyboard);
    ///
    /// let view = KeymapView::from_context_tables(&tables)?;
    /// assert!(view.conflicts().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_context_tables(
        tables: &BTreeMap<Context, BTreeMap<String, String>>,
    ) -> Result<Self, BindingError> {
        let mut contexts = BTreeMap::new();

        for (context, table) in tables {
            let mut bindings = ContextBindings::new(*context);

            // WHY: the actions are read in the order of the catalogue and not
            // in the order of the table, so that the winner of a conflict is
            // the same in every run.
            for action in Action::all() {
                let Some(text) = table.get(action.id()) else {
                    continue;
                };
                let binding = Binding::parse(text)?;
                bindings.insert(*action, binding);
            }

            for identifier in table.keys() {
                if Action::from_id(identifier).is_none() {
                    return Err(BindingError::UnknownAction {
                        identifier: identifier.clone(),
                    });
                }
            }

            contexts.insert(*context, bindings);
        }

        Ok(Self { contexts })
    }

    /// The action that an event triggers in a context.
    ///
    /// WHY: the resolution is deterministic. When two actions of one context
    /// are bound to the same key, the action that comes first in the
    /// catalogue wins, and the loser is reported as a conflict.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use std::collections::BTreeMap;
    /// use phocommander_ui::{Context, InputEvent, KeyStroke, KeymapView};
    ///
    /// # fn main() -> Result<(), phocommander_ui::BindingError> {
    /// let mut keyboard = BTreeMap::new();
    /// keyboard.insert("panel.copy".to_string(), "F5".to_string());
    /// let mut tables = BTreeMap::new();
    /// tables.insert(Context::Keyboard, keyboard);
    /// let view = KeymapView::from_context_tables(&tables)?;
    ///
    /// let event = InputEvent::Key(KeyStroke::parse("F5")?);
    /// assert!(view.resolve(Context::Keyboard, &event).is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolve(&self, context: Context, event: &InputEvent) -> Option<Action> {
        let bindings = self.contexts.get(&context)?;
        bindings.resolve(event)
    }

    /// The binding of an action in a context.
    pub fn binding_for(&self, context: Context, action: Action) -> Option<&Binding> {
        self.contexts.get(&context)?.binding_for(action)
    }

    /// The conflicts of the binding table.
    pub fn conflicts(&self) -> Vec<BindingConflict> {
        self.contexts
            .values()
            .flat_map(ContextBindings::conflicts)
            .collect()
    }

    /// Whether a context has a binding table.
    pub fn has_context(&self, context: Context) -> bool {
        self.contexts.contains_key(&context)
    }
}

/// The binding table of one context.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextBindings {
    context: Context,
    bindings: BTreeMap<Action, Vec<Binding>>,
    lookup: BTreeMap<Binding, Action>,
    conflicts: Vec<BindingConflict>,
}

impl ContextBindings {
    /// The empty binding table of a context.
    fn new(context: Context) -> Self {
        Self {
            context,
            bindings: BTreeMap::new(),
            lookup: BTreeMap::new(),
            conflicts: Vec::new(),
        }
    }

    /// Adds a binding for an action.
    fn insert(&mut self, action: Action, binding: Binding) {
        if let Some(kept) = self.lookup.get(&binding).copied() {
            self.conflicts.push(BindingConflict {
                context: self.context,
                binding: binding.clone(),
                kept,
                dropped: action,
            });
        } else {
            self.lookup.insert(binding.clone(), action);
        }

        self.bindings.entry(action).or_default().push(binding);
    }

    /// The action that an event triggers.
    fn resolve(&self, event: &InputEvent) -> Option<Action> {
        match event {
            InputEvent::Key(stroke) => self.lookup.get(&Binding::Key(*stroke)).copied(),
            InputEvent::Palette(entry) => {
                self.lookup.get(&Binding::Palette(entry.clone())).copied()
            }
        }
    }

    /// The first binding of an action.
    fn binding_for(&self, action: Action) -> Option<&Binding> {
        self.bindings.get(&action)?.first()
    }

    /// The conflicts of this context.
    fn conflicts(&self) -> Vec<BindingConflict> {
        self.conflicts.clone()
    }
}

/// Splits a binding into its parts.
///
/// WHY: the numeric keypad carries a plus key, and "Ctrl+Num+" must not be
/// read as the parts "Ctrl", "Num" and an empty third part.
fn split_parts(text: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for part in text.split('+') {
        if part.is_empty()
            && let Some(last) = parts.last_mut()
        {
            last.push('+');
            continue;
        }
        parts.push(part.to_string());
    }
    parts
}

/// The reasons why a binding could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BindingError {
    /// The action identifier is not part of the catalogue.
    #[error("the action {identifier} is not part of the catalogue")]
    UnknownAction {
        /// The identifier that was refused.
        identifier: String,
    },

    /// The binding names a key that does not exist.
    #[error("the binding {binding} names the unknown key {key}")]
    UnknownKey {
        /// The binding that was refused.
        binding: String,

        /// The key that is not known.
        key: String,
    },

    /// The binding could not be read.
    #[error("the binding {binding} is malformed: {reason}")]
    Malformed {
        /// The binding that was refused.
        binding: String,

        /// The reason why it was refused.
        reason: MalformedReason,
    },
}

impl BindingError {
    /// The key of the catalogue that carries the message of this error.
    ///
    /// WHY: the message of an error reaches the user through the status line
    ///       of the surface. A message that is written down in the source
    ///       stays English in a German interface (FR-05, section 8.4). The
    ///       error therefore names the key of the catalogue, and the surface
    ///       reads the text from there.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::BindingError;
    ///
    /// let error = BindingError::UnknownAction {
    ///     identifier: "panel.copy".to_string(),
    /// };
    /// assert_eq!(error.message_key(), "error-binding-unknown-action");
    /// ```
    pub fn message_key(&self) -> &'static str {
        match self {
            Self::UnknownAction { .. } => "error-binding-unknown-action",
            Self::UnknownKey { .. } => "error-binding-unknown-key",
            Self::Malformed { reason, .. } => reason.message_key(),
        }
    }

    /// The values that the message of this error carries.
    ///
    /// WHY: the message of the catalogue has placeholders for the values that
    ///       the error knows — the identifier of the action, the name of the
    ///       key. The values are handed over in the order in which the
    ///       catalogue expects them.
    pub fn message_arguments(&self) -> Vec<String> {
        match self {
            Self::UnknownAction { identifier } => vec![identifier.clone()],
            Self::UnknownKey { binding, key } => vec![binding.clone(), key.clone()],
            Self::Malformed { binding, reason } => {
                let mut values = vec![binding.clone()];
                values.extend(reason.message_arguments());
                values
            }
        }
    }
}

/// The reasons why a binding could not be read.
///
/// WHY: every reason is a case of its own, and every case has a message of
///       its own in the catalogue. A free text as the reason would make the
///       message of the surface depend on the wording of the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MalformedReason {
    /// The binding carries no part at all.
    Empty,

    /// The binding names the same key twice. The name of the key is carried
    /// along, because the message names it.
    KeyTwice(String),

    /// The binding names no key at all.
    NoKey,

    /// The entry of the action palette carries no name.
    EmptyPaletteEntry,
}

impl MalformedReason {
    /// The key of the catalogue that carries the message of this reason.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::MalformedReason;
    ///
    /// assert_eq!(
    ///     MalformedReason::Empty.message_key(),
    ///     "error-binding-empty"
    /// );
    /// ```
    pub fn message_key(&self) -> &'static str {
        match self {
            Self::Empty => "error-binding-empty",
            Self::KeyTwice(_) => "error-binding-key-twice",
            Self::NoKey => "error-binding-no-key",
            Self::EmptyPaletteEntry => "error-binding-empty-palette-entry",
        }
    }

    /// The values that the message of this reason carries.
    pub fn message_arguments(&self) -> Vec<String> {
        match self {
            Self::KeyTwice(name) => vec![name.clone()],
            _ => Vec::new(),
        }
    }
}

impl std::fmt::Display for MalformedReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Empty => "the binding is empty",
            Self::KeyTwice(_) => "the binding names the key twice",
            Self::NoKey => "the binding names no key",
            Self::EmptyPaletteEntry => "the palette entry has no name",
        };
        write!(formatter, "{text}")
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    /// Reads the binding table of the contexts of a test.
    fn view(tables: &BTreeMap<Context, BTreeMap<String, String>>) -> KeymapView {
        match KeymapView::from_context_tables(tables) {
            Ok(view) => view,
            Err(error) => panic!("the tables should be valid: {error}"),
        }
    }

    /// The stroke of a binding, or the escape key when it is not a key.
    fn stroke(text: &str) -> KeyStroke {
        match KeyStroke::parse(text) {
            Ok(stroke) => stroke,
            Err(error) => panic!("the binding should be valid: {error}"),
        }
    }

    #[test]
    fn reads_a_key_stroke_from_the_notation_of_the_catalogue() {
        // act
        let plain = stroke("F5");
        let modified = stroke("Ctrl+Shift+Tab");

        // assert
        assert_eq!(plain.key, Key::Function(5));
        assert!(plain.modifiers.is_empty());
        assert_eq!(modified.key, Key::Tab);
        assert!(modified.modifiers.primary);
        assert!(modified.modifiers.shift);
        assert!(!modified.modifiers.alt);
    }

    #[test]
    fn reads_the_keys_of_the_numeric_keypad() {
        // act
        let subtract = stroke("Ctrl+Num-");
        let multiply = stroke("Ctrl+Num*");

        // assert
        assert_eq!(subtract.key, Key::NumpadSubtract);
        assert_eq!(multiply.key, Key::NumpadMultiply);
    }

    #[test]
    fn the_spelling_of_a_stroke_is_the_notation_of_the_catalogue() {
        // act
        let text = stroke("Ctrl+Shift+Tab").to_binding();

        // assert
        assert_eq!(text, "Ctrl+Shift+Tab");
    }

    #[test]
    fn refuses_a_binding_without_a_key() {
        // act
        let result = KeyStroke::parse("Ctrl");

        // assert
        assert!(matches!(result, Err(BindingError::Malformed { .. })));
    }

    #[test]
    fn refuses_a_binding_with_an_unknown_key() {
        // act
        let result = KeyStroke::parse("Ctrl+Nonsense");

        // assert
        assert!(matches!(result, Err(BindingError::UnknownKey { .. })));
    }

    #[test]
    fn reads_a_palette_binding() {
        // act
        let binding = Binding::parse("palette:file.copy");

        // assert
        assert_eq!(binding, Ok(Binding::Palette("file.copy".to_string())));
        assert_eq!(
            Binding::parse("F5"),
            Ok(Binding::Key(KeyStroke::new(Key::Function(5))))
        );
    }

    #[test]
    fn the_same_event_resolves_differently_in_two_contexts() {
        // WHY: FR-101 — the key, the pointer, the touch gesture, the D-pad
        // and the controller bind to the same actions, but every context has
        // its own table.
        // arrange
        let mut keyboard = BTreeMap::new();
        keyboard.insert("panel.copy".to_string(), "Enter".to_string());
        let mut dpad = BTreeMap::new();
        dpad.insert("app.quit".to_string(), "Enter".to_string());
        let mut tables = BTreeMap::new();
        tables.insert(Context::Keyboard, keyboard);
        tables.insert(Context::Dpad, dpad);
        let view = view(&tables);
        let event = InputEvent::Key(stroke("Enter"));

        // act
        let in_keyboard = view.resolve(Context::Keyboard, &event);
        let in_dpad = view.resolve(Context::Dpad, &event);

        // assert
        assert_eq!(in_keyboard.map(|action| action.id()), Some("panel.copy"));
        assert_eq!(in_dpad.map(|action| action.id()), Some("app.quit"));
        assert_ne!(in_keyboard, in_dpad);
    }

    #[test]
    fn the_resolution_is_deterministic() {
        // WHY: a conflict is decided by a fixed rule, so that the same table
        // always produces the same action.
        // arrange
        let mut keyboard = BTreeMap::new();
        keyboard.insert("panel.copy".to_string(), "F5".to_string());
        keyboard.insert("panel.move".to_string(), "F5".to_string());
        let mut tables = BTreeMap::new();
        tables.insert(Context::Keyboard, keyboard);
        let view = view(&tables);
        let event = InputEvent::Key(stroke("F5"));

        // act
        let first = view.resolve(Context::Keyboard, &event);
        let second = view.resolve(Context::Keyboard, &event);

        // assert
        assert_eq!(first, second);
        // WHY: "panel.copy" comes before "panel.move" in the catalogue.
        assert_eq!(first.map(|action| action.id()), Some("panel.copy"));
        let conflicts = view.conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].kept().id(), "panel.copy");
        assert_eq!(conflicts[0].dropped().id(), "panel.move");
    }

    #[test]
    fn refuses_an_action_that_is_not_part_of_the_catalogue() {
        // arrange
        let mut keyboard = BTreeMap::new();
        keyboard.insert("panel.nonsense".to_string(), "F5".to_string());
        let mut tables = BTreeMap::new();
        tables.insert(Context::Keyboard, keyboard);

        // act
        let result = KeymapView::from_context_tables(&tables);

        // assert
        assert!(matches!(result, Err(BindingError::UnknownAction { .. })));
    }

    #[test]
    fn a_palette_entry_resolves_in_the_context_that_uses_it() {
        // arrange
        let mut dpad = BTreeMap::new();
        dpad.insert("panel.copy".to_string(), "palette:file.copy".to_string());
        let mut tables = BTreeMap::new();
        tables.insert(Context::Dpad, dpad);
        let view = view(&tables);

        // act
        let resolved = view.resolve(Context::Dpad, &InputEvent::Palette("file.copy".to_string()));

        // assert
        assert_eq!(resolved.map(|action| action.id()), Some("panel.copy"));
        assert!(
            view.resolve(
                Context::Keyboard,
                &InputEvent::Palette("file.copy".to_string())
            )
            .is_none()
        );
    }
}
