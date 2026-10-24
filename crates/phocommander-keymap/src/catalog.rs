//! The action catalogue of Phocommander.
//!
//! WHAT: the stable identifier of a command ([`ActionId`]) and the input
//!       contexts ([`Context`]) in which a command can be triggered.
//! WHY: the user interface never asks *which key* was pressed, it asks *which
//!       command* was requested. The identifier is therefore the contract
//!       between the input layer, the configuration file and the user
//!       interface — and it has to stay stable, because a user writes it into
//!       the configuration file and into the documentation.
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

use std::fmt;
use std::str::FromStr;

/// The context in which an action can be triggered.
///
/// WHY: the same command needs a different binding in every context. A
/// television remote control has no function keys, so the action palette
/// takes their place there. The context is part of the binding, not of the
/// action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Context {
    /// A physical keyboard.
    Keyboard,

    /// A mouse or a trackpad.
    Pointer,

    /// A finger on a touch screen.
    Touch,

    /// The directional pad of a television remote control.
    Dpad,

    /// A gamepad or a VR controller.
    Controller,
}

impl Context {
    /// Every context, in a stable order.
    ///
    /// WHY: the order is fixed so that a list of contexts — in the
    /// configuration file, in the documentation or in the user interface —
    /// always reads the same way.
    pub const fn all() -> [Context; 5] {
        [
            Context::Keyboard,
            Context::Pointer,
            Context::Touch,
            Context::Dpad,
            Context::Controller,
        ]
    }

    /// The spelling of this context in the configuration file.
    pub const fn config_name(&self) -> &'static str {
        match self {
            Context::Keyboard => "keyboard",
            Context::Pointer => "pointer",
            Context::Touch => "touch",
            Context::Dpad => "dpad",
            Context::Controller => "controller",
        }
    }

    /// The context with the given spelling in the configuration file.
    pub fn from_config_name(name: &str) -> Option<Self> {
        Context::all()
            .into_iter()
            .find(|context| context.config_name() == name)
    }
}

impl fmt::Display for Context {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.config_name())
    }
}

/// The stable identifier of a command, for example `panel.copy`.
///
/// WHY: the identifier is written into the configuration file and into the
/// documentation. It therefore has one shape everywhere: lower case letters,
/// digits, `-` and `_`, separated into segments by `.`, with no empty segment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionId(String);

impl ActionId {
    /// Creates an action identifier from its spelling.
    ///
    /// WHY: the identifier travels through the configuration file and through
    /// the documentation. A malformed identifier is therefore refused at the
    /// moment it is read, with the reason named, instead of silently doing
    /// nothing later.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_keymap::ActionId;
    ///
    /// # fn main() -> Result<(), phocommander_keymap::ActionIdError> {
    /// assert!(ActionId::new("panel.copy").is_ok());
    /// assert!(ActionId::new("panel..copy").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(raw: &str) -> Result<Self, ActionIdError> {
        if raw.is_empty() {
            return Err(ActionIdError::Empty);
        }

        if !raw.contains('.') {
            return Err(ActionIdError::MissingSeparator {
                name: raw.to_string(),
            });
        }

        if raw.split('.').any(str::is_empty) {
            return Err(ActionIdError::EmptySegment {
                name: raw.to_string(),
            });
        }

        if let Some(character) = raw.chars().find(|character| {
            !(character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '_' | '.'))
        }) {
            return Err(ActionIdError::ForbiddenCharacter {
                name: raw.to_string(),
                character,
            });
        }

        Ok(Self(raw.to_string()))
    }

    /// The identifier as it is written in the configuration file.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ActionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ActionId {
    type Err = ActionIdError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        ActionId::new(raw)
    }
}

/// The reasons why an action identifier was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ActionIdError {
    /// The identifier was empty.
    #[error("an action identifier must not be empty")]
    Empty,

    /// The identifier contained an empty segment, for example `panel..copy`.
    #[error("the action identifier {name:?} contains an empty segment")]
    EmptySegment {
        /// The identifier that was refused.
        name: String,
    },

    /// The identifier contained a character that is not allowed.
    #[error("the action identifier {name:?} contains the forbidden character {character:?}")]
    ForbiddenCharacter {
        /// The identifier that was refused.
        name: String,
        /// The character that is not allowed.
        character: char,
    },

    /// The identifier did not separate the group from the action.
    #[error("the action identifier {name:?} must contain a '.' between the group and the action")]
    MissingSeparator {
        /// The identifier that was refused.
        name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_identifiers_the_specification_uses() {
        // arrange
        let identifiers = [
            "panel.copy",
            "panel.view",
            "selection.toggle",
            "palette.open",
        ];

        // act
        let results: Vec<bool> = identifiers
            .iter()
            .map(|identifier| ActionId::new(identifier).is_ok())
            .collect();

        // assert
        assert_eq!(results, vec![true, true, true, true]);
    }

    #[test]
    fn refuses_an_empty_identifier() {
        // act
        let result = ActionId::new("");

        // assert
        assert_eq!(result.err(), Some(ActionIdError::Empty));
    }

    #[test]
    fn refuses_an_empty_segment() {
        // act
        let results = [
            ActionId::new("panel..copy"),
            ActionId::new(".copy"),
            ActionId::new("copy."),
        ];

        // assert
        for result in results {
            assert!(
                matches!(result, Err(ActionIdError::EmptySegment { .. })),
                "{result:?}"
            );
        }
    }

    #[test]
    fn refuses_an_identifier_without_a_group() {
        // WHY: a forgotten dot is the most likely typo in a hand-written
        // configuration file, and it would silently disable the binding.
        // act
        let result = ActionId::new("panelcopy");

        // assert
        assert!(
            matches!(result, Err(ActionIdError::MissingSeparator { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn refuses_a_forbidden_character() {
        // act
        let results = [
            ActionId::new("panel.copy "),
            ActionId::new("Panel.Copy"),
            ActionId::new("panel.copy!"),
        ];

        // assert
        for result in results {
            assert!(
                matches!(result, Err(ActionIdError::ForbiddenCharacter { .. })),
                "{result:?}"
            );
        }
    }

    #[test]
    fn reads_itself_from_a_string() {
        // act
        let action: ActionId = match "panel.copy".parse() {
            Ok(action) => action,
            Err(error) => panic!("the identifier should be valid: {error}"),
        };

        // assert
        assert_eq!(action.as_str(), "panel.copy");
        assert_eq!(action.to_string(), "panel.copy");
    }

    #[test]
    fn every_context_has_a_unique_configuration_name() {
        // act
        let names: Vec<&str> = Context::all()
            .iter()
            .map(|context| context.config_name())
            .collect();

        // assert
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn finds_a_context_by_its_configuration_name() {
        // act
        let found = Context::from_config_name("dpad");

        // assert
        assert_eq!(found, Some(Context::Dpad));
        assert_eq!(Context::from_config_name("gamepad"), None);
    }
}
