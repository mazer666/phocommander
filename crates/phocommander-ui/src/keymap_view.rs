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
//! SIZE: the table of the bindings and the types of the input are two
//!       different things, and they live in two files. The table refers to
//!       the types; the types know nothing of the table.
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

use crate::actions::Action;
use crate::context::Context;

mod controller;
pub mod defaults;
mod gesture;
mod stroke;

// WHY: the types of the input live in a file of their own, and they are
//       handed on here. The public path of every type stays the same, so
//       that the callers do not have to know about the split.
pub use controller::ControllerButton;
pub use gesture::{Direction, Gesture, PinchDirection, PointerEvent};
pub use stroke::{InputEvent, Key, KeyStroke, Modifiers};

/// The prefix of a binding that names an entry of the action palette.
const PALETTE_PREFIX: &str = "palette:";

/// The binding of an action: a key or an entry of the action palette.
///
/// WHY: a remote control has no function keys, so the action palette takes
/// their place there. The binding therefore has two shapes, and the shape
/// decides how the action is reached.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Binding {
    /// A key with its modifier keys.
    Key(KeyStroke),

    /// An event of the pointer.
    Pointer(PointerEvent),

    /// A gesture of a finger.
    Gesture(Gesture),

    /// A button of a gamepad or of a VR controller.
    Controller(ControllerButton),

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

        // WHY: the pointer and the finger come before the key. A name like
        //       "click" is not a key, and a key stroke would refuse it with
        //       an error that names the wrong thing.
        if let Some(event) = PointerEvent::parse(text) {
            return Ok(Binding::Pointer(event));
        }
        if let Some(gesture) = Gesture::parse(text) {
            return Ok(Binding::Gesture(gesture));
        }
        // WHY: the buttons of the controller come after the pointer and the
        //       finger, because their names are short and could collide.
        if let Some(button) = ControllerButton::parse(text) {
            return Ok(Binding::Controller(button));
        }

        Ok(Binding::Key(KeyStroke::parse(text)?))
    }

    /// The spelling of the binding as the configuration file writes it.
    pub fn to_binding(&self) -> String {
        match self {
            Binding::Key(stroke) => stroke.to_binding(),
            Binding::Pointer(event) => event.as_str().to_string(),
            Binding::Gesture(gesture) => gesture.as_str(),
            Binding::Controller(button) => button.as_str().to_string(),
            Binding::Palette(entry) => format!("{PALETTE_PREFIX}{entry}"),
        }
    }
}

/// An event of the input layer.
///
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

    /// The view of the default bindings.
    ///
    /// WHY: the file of the configuration is the one that the user edits,
    ///       and a file that is missing must not leave the application
    ///       without any binding. The defaults come first, and the file
    ///       overrides them (FR-101).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::keymap_view::KeymapView;
    ///
    /// # fn main() -> Result<(), phocommander_ui::BindingError> {
    /// let view = KeymapView::with_defaults(&Default::default())?;
    /// assert!(view.has_context(phocommander_ui::Context::Touch));
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_defaults(
        tables: &BTreeMap<Context, BTreeMap<String, String>>,
    ) -> Result<Self, BindingError> {
        let mut zusammen = defaults::default_bindings();
        for (context, table) in tables {
            let ziel = zusammen.entry(*context).or_default();
            for (aktion, bindung) in table {
                ziel.insert(aktion.clone(), bindung.clone());
            }
        }
        Self::from_context_tables(&zusammen)
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
            InputEvent::Pointer(event) => self.lookup.get(&Binding::Pointer(*event)).copied(),
            InputEvent::Gesture(gesture) => self.lookup.get(&Binding::Gesture(*gesture)).copied(),
            InputEvent::Controller(button) => {
                self.lookup.get(&Binding::Controller(*button)).copied()
            }
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

    /// The values that the message of this error carries, with their names.
    ///
    /// WHY: the message of the catalogue has placeholders for the values that
    ///       the error knows — the identifier of the action, the name of the
    ///       key. The name belongs to the placeholder of the message, and
    ///       only the message knows it.
    pub fn message_arguments(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::UnknownAction { identifier } => vec![("identifier", identifier.clone())],
            Self::UnknownKey { binding, key } => {
                vec![("binding", binding.clone()), ("key", key.clone())]
            }
            Self::Malformed { binding, reason } => {
                let mut values = vec![("binding", binding.clone())];
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

    /// The values that the message of this reason carries, with their names.
    pub fn message_arguments(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::KeyTwice(name) => vec![("key", name.clone())],
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
