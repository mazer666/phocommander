//! The marked entries of a panel.
//!
//! WHAT: the positions of the entries that the user marked
//!       ([`Selection`]).
//! WHY: the marks hold the *positions* of the entries and not their paths.
//!       A position is the index in the order in which the source delivered
//!       the entries, and it does not change when the order of the screen
//!       changes. The marks therefore stay on the same files when the user
//!       sorts the list, and they survive a filter as well.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::Selection;
//!
//! let mut selection = Selection::new();
//! selection.toggle(2);
//! assert!(selection.contains(2));
//! assert_eq!(selection.len(), 1);
//! ```

use std::collections::BTreeSet;

/// The marked entries of a panel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Selection {
    positions: BTreeSet<usize>,
}

impl Selection {
    /// The empty selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of marked entries.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Whether no entry is marked.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Whether the entry with the position is marked.
    pub fn contains(&self, position: usize) -> bool {
        self.positions.contains(&position)
    }

    /// Marks the entry with the position.
    pub fn insert(&mut self, position: usize) {
        self.positions.insert(position);
    }

    /// Removes the mark of the entry with the position.
    pub fn remove(&mut self, position: usize) {
        self.positions.remove(&position);
    }

    /// Switches the mark of the entry with the position.
    pub fn toggle(&mut self, position: usize) {
        if !self.positions.remove(&position) {
            self.positions.insert(position);
        }
    }

    /// Removes every mark.
    pub fn clear(&mut self) {
        self.positions.clear();
    }

    /// The marked positions, in the order of the listing.
    pub fn positions(&self) -> impl Iterator<Item = usize> + '_ {
        self.positions.iter().copied()
    }
}
