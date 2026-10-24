//! The key bindings of one input context.
//!
//! WHAT: the bindings of one input context ([`Bindings`]), the conflict
//!       between two of them ([`BindingConflict`]) and the reasons why a
//!       binding could not be read ([`BindingError`]).
//! WHY: a binding is a relation between a key combination and an action, and
//!       two actions on the same key cannot both be reachable. The conflict
//!       check therefore lives next to the bindings, so that no caller can
//!       forget it, and it reports every conflict of a context in a stable
//!       order.
//! EXAMPLE:
//! ```no_run
//! use std::collections::BTreeMap;
//! use phocommander_keymap::{Bindings, Context};
//!
//! # fn main() -> Result<(), phocommander_keymap::BindingError> {
//! let mut table = BTreeMap::new();
//! table.insert("panel.copy".to_string(), "F5".to_string());
//!
//! let bindings = Bindings::from_table(Context::Keyboard, &table)?;
//! assert!(bindings.conflicts().is_empty());
//! # Ok(())
//! # }
//! ```

use std::collections::BTreeMap;

use crate::catalog::{ActionId, ActionIdError, Context};

/// The key bindings of one input context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bindings {
    context: Context,
    entries: BTreeMap<ActionId, Vec<String>>,
}

impl Bindings {
    /// Creates the empty bindings of a context.
    pub fn new(context: Context) -> Self {
        Self {
            context,
            entries: BTreeMap::new(),
        }
    }

    /// Reads the bindings of a context from a table of the configuration file.
    ///
    /// WHY: the table is the shape the configuration file uses — an action
    /// identifier and the key combination that triggers it. Reading it here
    /// means that a malformed action identifier is reported with its name,
    /// at the moment the file is read.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use std::collections::BTreeMap;
    /// use phocommander_keymap::{Bindings, Context};
    ///
    /// # fn main() -> Result<(), phocommander_keymap::BindingError> {
    /// let mut table = BTreeMap::new();
    /// table.insert("panel.copy".to_string(), "F5".to_string());
    /// let bindings = Bindings::from_table(Context::Keyboard, &table)?;
    /// assert_eq!(bindings.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_table(
        context: Context,
        table: &BTreeMap<String, String>,
    ) -> Result<Self, BindingError> {
        let mut bindings = Bindings::new(context);
        for (action, key) in table {
            let action = ActionId::new(action)?;
            bindings.bind(action, key.clone())?;
        }
        Ok(bindings)
    }

    /// Adds a key combination for an action.
    ///
    /// WHY: an empty key combination would be bound to an action that can
    /// never be triggered, and would hide a mistake in the configuration
    /// file. It is therefore refused.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_keymap::{ActionId, Bindings, Context};
    ///
    /// # fn main() -> Result<(), phocommander_keymap::BindingError> {
    /// let mut bindings = Bindings::new(Context::Keyboard);
    /// bindings.bind(ActionId::new("panel.copy")?, "F5")?;
    /// assert!(bindings.bind(ActionId::new("panel.view")?, "").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn bind(&mut self, action: ActionId, key: impl Into<String>) -> Result<(), BindingError> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(BindingError::EmptyKey { action });
        }
        let keys = self.entries.entry(action).or_default();
        if !keys.contains(&key) {
            keys.push(key);
        }
        Ok(())
    }

    /// The key combinations that are bound to an action.
    pub fn keys_for(&self, action: &ActionId) -> Option<&[String]> {
        self.entries.get(action).map(Vec::as_slice)
    }

    /// The actions that are bound to a key combination.
    pub fn actions_for(&self, key: &str) -> Vec<&ActionId> {
        self.entries
            .iter()
            .filter(|(_, keys)| keys.iter().any(|bound| bound == key))
            .map(|(action, _)| action)
            .collect()
    }

    /// The number of actions that are bound in this context.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no action is bound in this context.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The conflicts inside these bindings.
    ///
    /// WHY: two actions on the same key combination cannot both be triggered
    /// — the second one would never be reached. The conflicts are reported
    /// with every action that is affected, sorted by the action identifier,
    /// so that the same file always produces the same report.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_keymap::{ActionId, Bindings, Context};
    ///
    /// # fn main() -> Result<(), phocommander_keymap::BindingError> {
    /// let mut bindings = Bindings::new(Context::Keyboard);
    /// bindings.bind(ActionId::new("panel.copy")?, "F5")?;
    /// bindings.bind(ActionId::new("file.copy")?, "F5")?;
    ///
    /// let conflicts = bindings.conflicts();
    /// assert_eq!(conflicts.len(), 1);
    /// assert_eq!(conflicts[0].actions().len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn conflicts(&self) -> Vec<BindingConflict> {
        // WHY: the conflicts are collected per key combination and not per
        // action. Two actions on one key produce one conflict that names
        // both of them — a report that lists the same collision twice is
        // harder to read, not easier.
        let mut actions_per_key: BTreeMap<&str, Vec<ActionId>> = BTreeMap::new();
        for (action, keys) in &self.entries {
            for key in keys {
                let actions = actions_per_key.entry(key.as_str()).or_default();
                actions.push(action.clone());
            }
        }

        actions_per_key
            .into_iter()
            .filter(|(_, actions)| actions.len() > 1)
            .map(|(key, actions)| BindingConflict {
                context: self.context,
                key: key.to_string(),
                actions,
            })
            .collect()
    }
}

/// Two actions that are bound to the same key combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingConflict {
    context: Context,
    key: String,
    actions: Vec<ActionId>,
}

impl BindingConflict {
    /// The context the conflict happened in.
    pub fn context(&self) -> Context {
        self.context
    }

    /// The key combination that is bound twice.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The actions that are bound to the key combination.
    pub fn actions(&self) -> &[ActionId] {
        &self.actions
    }
}

/// The reasons why a binding could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BindingError {
    /// An action identifier was malformed.
    #[error("the action identifier is not valid: {0}")]
    InvalidAction(#[from] ActionIdError),

    /// A key combination was empty.
    #[error("the key combination of the action {action} is empty")]
    EmptyKey {
        /// The action the empty key combination was meant for.
        action: ActionId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the bindings of a table that has to be valid.
    fn valid(result: Result<Bindings, BindingError>) -> Bindings {
        match result {
            Ok(bindings) => bindings,
            Err(error) => panic!("the table should be valid: {error}"),
        }
    }

    /// Returns an action identifier that has to be valid.
    fn action(identifier: &str) -> ActionId {
        match ActionId::new(identifier) {
            Ok(action) => action,
            Err(error) => panic!("the identifier should be valid: {error}"),
        }
    }

    #[test]
    fn reads_a_table_of_the_configuration_file() {
        // arrange
        let mut table = BTreeMap::new();
        table.insert("panel.copy".to_string(), "F5".to_string());
        table.insert("panel.move".to_string(), "F6".to_string());

        // act
        let bindings = valid(Bindings::from_table(Context::Keyboard, &table));

        // assert
        assert_eq!(bindings.len(), 2);
        assert_eq!(
            bindings.keys_for(&action("panel.copy")),
            Some(&["F5".to_string()][..])
        );
    }

    #[test]
    fn refuses_a_malformed_action_identifier() {
        // arrange
        let mut table = BTreeMap::new();
        table.insert("panelcopy".to_string(), "F5".to_string());

        // act
        let result = Bindings::from_table(Context::Keyboard, &table);

        // assert
        assert!(matches!(
            result,
            Err(BindingError::InvalidAction(
                ActionIdError::MissingSeparator { .. }
            ))
        ));
    }

    #[test]
    fn refuses_an_empty_key_combination() {
        // arrange
        let mut bindings = Bindings::new(Context::Keyboard);
        let action = action("panel.copy");

        // act
        let result = bindings.bind(action.clone(), "   ");

        // assert
        assert_eq!(result, Err(BindingError::EmptyKey { action }));
    }

    #[test]
    fn reports_a_conflict_between_two_actions() {
        // arrange
        let mut bindings = Bindings::new(Context::Keyboard);
        for identifier in ["panel.copy", "file.copy"] {
            match bindings.bind(action(identifier), "F5") {
                Ok(()) => {}
                Err(error) => panic!("the binding should be accepted: {error}"),
            }
        }

        // act
        let conflicts = bindings.conflicts();

        // assert
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].key(), "F5");
        assert_eq!(conflicts[0].context(), Context::Keyboard);
        assert_eq!(conflicts[0].actions().len(), 2);
    }

    #[test]
    fn finds_the_key_combinations_of_an_action() {
        // arrange
        let mut table = BTreeMap::new();
        table.insert("panel.copy".to_string(), "F5".to_string());
        let bindings = valid(Bindings::from_table(Context::Keyboard, &table));
        let action = action("panel.copy");

        // act
        let keys = bindings.keys_for(&action);
        let actions = bindings.actions_for("F5");

        // assert
        assert_eq!(keys, Some(&["F5".to_string()][..]));
        assert_eq!(actions, vec![&action]);
    }

    #[test]
    fn an_empty_context_has_no_bindings() {
        // act
        let bindings = Bindings::new(Context::Touch);

        // assert
        assert!(bindings.is_empty());
        assert_eq!(bindings.len(), 0);
    }
}
