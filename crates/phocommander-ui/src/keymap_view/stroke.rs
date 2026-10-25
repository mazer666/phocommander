//! The keys, the modifiers and the strokes of the input.
//!
//! WHAT: a key of the keyboard ([`Key`]), the modifiers of a key
//!       ([`Modifiers`]), a key with its modifiers ([`KeyStroke`]) and the
//!       event of the input layer ([`InputEvent`]).
//! WHY: the input layer of the platform delivers a key, a click, a gesture
//!       or a button. The surface only knows the two shapes that reach it as
//!       a command: a key and an entry of the action palette. These types
//!       are free of any framework type, so that the surface can compare a
//!       written binding with a pressed key (FR-100).
//! SIZE: the types of the input live in a file of their own, because the
//!       table of the bindings and the types of the input are two different
//!       things. The table refers to the types; the types know nothing of
//!       the table.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::keymap_view::{Key, KeyStroke};
//!
//! let stroke = KeyStroke::parse("Ctrl+R")
//!     .unwrap_or_else(|_| KeyStroke::escape());
//! assert!(matches!(stroke, KeyStroke { .. }));
//! ```

use std::fmt;

use super::{BindingError, MalformedReason};

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
