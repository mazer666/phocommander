//! The complete keymap of Phocommander.
//!
//! WHAT: the bindings of every input context ([{Keymap}]).
//! WHY: the same key combination may be used in different contexts — {F5}
//!       on the keyboard and a palette entry on the television remote
//!       control do not collide. Only the bindings inside one context can
//!       conflict, so the keymap keeps the contexts apart and reports the
//!       conflicts of each of them separately.
//! EXAMPLE:
//! ```no_run
//! use std::collections::BTreeMap;
//! use phocommander_keymap::{Context, Keymap};
//!
//! # fn main() -> Result<(), phocommander_keymap::BindingError> {
//! let mut keyboard = BTreeMap::new();
//! keyboard.insert("panel.copy".to_string(), "F5".to_string());
//!
//! let mut tables = BTreeMap::new();
//! tables.insert(Context::Keyboard, keyboard);
//!
//! let keymap = Keymap::from_context_tables(&tables)?;
//! assert!(keymap.conflicts().is_empty());
//! # Ok(())
//! # }
//! ```

use std::collections::BTreeMap;

use crate::bindings::{BindingConflict, BindingError, Bindings};
use crate::catalog::{ActionId, Context};

/// The complete keymap of Phocommander.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Keymap {
    contexts: BTreeMap<Context, Bindings>,
}

impl Keymap {
/// Creates a keymap without any binding.
    pub fn new() -> Self {
        Self::default()
    }

/// Reads the bindings of every context from the tables of the
/// configuration file.
///
/// WHY: the keymap is built from the tables of the configuration file, so
/// that the file stays the only place a binding is written down. A
/// malformed action identifier is refused here, with its name.
///
/// EXAMPLE:
/// ```no_run
/// use std::collections::BTreeMap;
/// use phocommander_keymap::{Context, Keymap};
///
/// # fn main() -> Result<(), phocommander_keymap::BindingError> {
/// let mut keyboard = BTreeMap::new();
/// keyboard.insert("panel.copy".to_string(), "F5".to_string());
///
/// let mut tables = BTreeMap::new();
/// tables.insert(Context::Keyboard, keyboard);
///
/// let keymap = Keymap::from_context_tables(&tables)?;
/// assert!(keymap.conflicts().is_empty());
/// # Ok(())
/// # }
/// ```
    pub fn from_context_tables(
        tables: &BTreeMap<Context, BTreeMap<String, String>>,
    ) -> Result<Self, BindingError> {
        let mut contexts = BTreeMap::new();
        for (context, table) in tables {
            contexts.insert(*context, Bindings::from_table(*context, table)?);
        }
        Ok(Self { contexts })
    }

/// The bindings of a context.
    pub fn bindings(&self, context: Context) -> Option<&Bindings> {
        self.contexts.get(&context)
    }

/// The key combinations that are bound to an action in a context.
    pub fn keys_for(&self, context: Context, action: &ActionId) -> Option<&[String]> {
        self.contexts.get(&context)?.keys_for(action)
    }

/// Every conflict of the keymap.
///
/// WHY: the same key combination may be used in different contexts — {F5}
/// on the keyboard and a palette entry on the television remote control
/// do not collide. Only the bindings inside one context can conflict.
    pub fn conflicts(&self) -> Vec<BindingConflict> {
        let mut conflicts = Vec::new();
        for bindings in self.contexts.values() {
            conflicts.extend(bindings.conflicts());
        }
        conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_report_a_conflict_between_two_contexts() {
        // arrange
        let mut keyboard = BTreeMap::new();
        keyboard.insert("panel.copy".to_string(), "F5".to_string());
        let mut dpad = BTreeMap::new();
        dpad.insert("file.copy".to_string(), "F5".to_string());

        let mut tables = BTreeMap::new();
        tables.insert(Context::Keyboard, keyboard);
        tables.insert(Context::Dpad, dpad);

        // act
        let keymap = match Keymap::from_context_tables(&tables) {
            Ok(keymap) => keymap,
            Err(error) => panic!("the tables should be valid: {error}"),
        };

        // assert
        assert!(keymap.conflicts().is_empty());
    }

    #[test]
    fn reports_a_conflict_inside_one_context() {
        // arrange
        let mut keyboard = BTreeMap::new();
        keyboard.insert("panel.copy".to_string(), "F5".to_string());
        keyboard.insert("file.copy".to_string(), "F5".to_string());

        let mut tables = BTreeMap::new();
        tables.insert(Context::Keyboard, keyboard);

        // act
        let keymap = match Keymap::from_context_tables(&tables) {
            Ok(keymap) => keymap,
            Err(error) => panic!("the tables should be valid: {error}"),
        };

        // assert
        let conflicts = keymap.conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].context(), Context::Keyboard);
    }

    #[test]
    fn finds_the_bindings_of_a_context() {
        // arrange
        let mut keyboard = BTreeMap::new();
        keyboard.insert("panel.copy".to_string(), "F5".to_string());

        let mut tables = BTreeMap::new();
        tables.insert(Context::Keyboard, keyboard);

        let keymap = match Keymap::from_context_tables(&tables) {
            Ok(keymap) => keymap,
            Err(error) => panic!("the tables should be valid: {error}"),
        };

        // act
        let bindings = keymap.bindings(Context::Keyboard);
        let missing = keymap.bindings(Context::Touch);
        let action = match ActionId::new("panel.copy") {
            Ok(action) => action,
            Err(error) => panic!("the identifier should be valid: {error}"),
        };
        let keys = keymap.keys_for(Context::Keyboard, &action);

        // assert
        assert!(bindings.is_some());
        assert!(missing.is_none());
        assert_eq!(keys, Some(&["F5".to_string()][..]));
    }

    #[test]
    fn an_empty_keymap_has_no_conflicts() {
        // act
        let keymap = Keymap::new();

        // assert
        assert!(keymap.conflicts().is_empty());
    }
}
