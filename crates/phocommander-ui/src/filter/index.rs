//! The index of the entries that pass the filter.
//!
//! WHAT: the positions of the entries that pass the filter, in the order
//!       of the screen, and the row of every entry ([`FilteredIndex`]).
//! WHY: the index is built once per change of the filter or of the
//!       directory and not once per frame. The drawing asks for the visible
//!       slice of it, so that a directory with a hundred thousand entries
//!       costs the same in every frame as a directory with thirty entries
//!       (FR-106).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::filter::{Filter, FilteredIndex};
//! use phocommander_ui::listing::{Entry, EntryKind};
//!
//! let entries = vec![Entry::new("notes.txt", EntryKind::File)];
//! let order = vec![0];
//! let filter = Filter::from_text("*.txt", Default::default(), Default::default());
//!
//! let index = FilteredIndex::build(&entries, &order, &filter);
//! assert_eq!(index.len(), 1);
//! ```

use crate::filter::Filter;
use crate::listing::Entry;

/// The entries that pass the filter, with the row of every entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilteredIndex {
    positions: Vec<usize>,
    rows: Vec<usize>,
}

impl FilteredIndex {
    /// The index of the entries that pass the filter.
    ///
    /// WHY: the index follows the order of the screen, so that the filtered
    /// list is sorted like the list without a filter. The rows of the
    /// entries that do not pass are unknown, and the panel asks for them
    /// with [`Self::row_of_position`].
    pub fn build(entries: &[Entry], order: &[usize], filter: &Filter) -> Self {
        let mut positions = Vec::with_capacity(order.len());
        let mut rows = vec![usize::MAX; entries.len()];

        for position in order {
            let Some(entry) = entries.get(*position) else {
                continue;
            };
            if !filter.is_active() || filter.matches_entry(entry) {
                rows[*position] = positions.len();
                positions.push(*position);
            }
        }

        Self { positions, rows }
    }

    /// The number of entries that pass the filter.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Whether no entry passes the filter.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// The positions of the entries that pass the filter, in the order of
    /// the screen.
    pub fn positions(&self) -> &[usize] {
        &self.positions
    }

    /// The row of the screen in which the entry with the position is shown.
    pub fn row_of_position(&self, position: usize) -> Option<usize> {
        match self.rows.get(position) {
            Some(row) if *row != usize::MAX => Some(*row),
            _ => None,
        }
    }

    /// The position of the entry in the given row of the screen.
    pub fn position_of_row(&self, row: usize) -> Option<usize> {
        self.positions.get(row).copied()
    }

    /// Whether the entry with the position passes the filter.
    pub fn contains(&self, position: usize) -> bool {
        self.row_of_position(position).is_some()
    }
}
