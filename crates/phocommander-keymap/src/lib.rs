//! The action catalogue and the key bindings of Phocommander.
//!
//! WHAT: the stable identifiers of the commands of the application
//!       ([`catalog`]), the key bindings of one context ([`bindings`]) and the
//!       complete keymap ([`keymap`]).
//! WHY: the user interface must know *what* a command is, never *how* it was
//!       triggered. The keyboard, the mouse, the finger, the television remote
//!       control and the gamepad all deliver the same action identifier, so
//!       the same command works on every platform without a second
//!       implementation. A binding that is bound twice is a mistake that is
//!       reported here, where it can be found.
//! EXAMPLE:
//! ```no_run
//! use phocommander_keymap::{ActionId, Context};
//!
//! # fn main() -> Result<(), phocommander_keymap::ActionIdError> {
//! let action = ActionId::new("panel.copy")?;
//! assert_eq!(action.as_str(), "panel.copy");
//! assert_eq!(Context::Dpad.config_name(), "dpad");
//! # Ok(())
//! # }
//! ```

pub mod bindings;
pub mod catalog;
pub mod keymap;

pub use bindings::{BindingConflict, BindingError, Bindings};
pub use catalog::{ActionId, ActionIdError, Context};
pub use keymap::Keymap;
