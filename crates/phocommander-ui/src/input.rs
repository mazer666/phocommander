//! The input of the frame in the form of the interface.
//!
//! WHAT: the key that carries the primary meaning on the platform
//!       ([`PrimaryModifier`]) and the conversion of a key event of the
//!       framework into a key stroke of the catalogue ([`stroke_of`]).
//! WHY: the binding table of the configuration file is written in words and
//!       the platform delivers a key of the framework. The conversion is
//!       therefore one function, and it lives here and not in the shell —
//!       so that the shell stays readable and the conversion can be tested
//!       on its own.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::input::{PrimaryModifier, stroke_of};
//!
//! let stroke = stroke_of(egui::Key::F5, egui::Modifiers::default(), PrimaryModifier::Control);
//! assert!(stroke.is_some());
//! ```

use crate::keymap_view::{Key, KeyStroke, Modifiers};

/// The key that carries the primary meaning on the platform.
///
/// WHY: the actions are the same on every platform, only the key that
/// triggers them differs. `Cmd` is the primary key on macOS and on iPadOS,
/// `Ctrl` everywhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrimaryModifier {
    /// The command key — macOS and iPadOS.
    Command,

    /// The control key — Windows, Linux and BSD.
    #[default]
    Control,
}

/// The key stroke of an event of the framework.
///
/// WHY: the binding table of the configuration file is written in words and
/// the platform delivers a key of the framework. The stroke is the form in
/// which both can be compared.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::input::{PrimaryModifier, stroke_of};
///
/// let stroke = stroke_of(egui::Key::F5, egui::Modifiers::default(), PrimaryModifier::Control);
/// assert!(stroke.is_some());
/// ```
pub fn stroke_of(
    key: egui::Key,
    modifiers: egui::Modifiers,
    primary: PrimaryModifier,
) -> Option<KeyStroke> {
    let mut key = Key::from_name(key.name())?;

    let mut shift = modifiers.shift;
    let alt = modifiers.alt;
    let primary_pressed = match primary {
        PrimaryModifier::Command => modifiers.command,
        PrimaryModifier::Control => modifiers.ctrl,
    };

    // WHY: the star key of the numeric keypad is Shift+8 on a keyboard with
    // an American layout, and egui delivers the shift and the eight. The
    // combination is normalised into the key of the keypad, so that the
    // binding of the catalogue can be reached.
    if shift && key == Key::Digit(8) {
        key = Key::NumpadMultiply;
        shift = false;
    }

    Some(KeyStroke {
        key,
        modifiers: Modifiers {
            primary: primary_pressed,
            shift,
            alt,
        },
    })
}
