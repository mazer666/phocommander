//! The shape of the file list of a panel.
//!
//! WHAT: the columns of the list ([`Column`]) and the way the panel shows
//!       its entries ([`ViewMode`]).
//! WHY: the set of the columns is a decision of the user (FR-17) and the
//!       view is a decision of the user as well. Both belong to the tab and
//!       not to the drawing code, so that the drawing never has to guess
//!       what the user wanted to see.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::Column;
//!
//! assert_eq!(Column::all().len(), 4);
//! assert_eq!(Column::from_config_name("size"), Some(Column::Size));
//! ```

/// A column of the file list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Column {
    /// The name of the entry.
    Name,

    /// The size of the entry.
    Size,

    /// The time of the last change.
    Date,

    /// The type of the entry.
    Type,
}

impl Column {
    /// Every column, in the order the specification names them.
    pub const fn all() -> [Column; 4] {
        [Column::Name, Column::Size, Column::Date, Column::Type]
    }

    /// The spelling of the column in the configuration file.
    pub const fn config_name(&self) -> &'static str {
        match self {
            Column::Name => "name",
            Column::Size => "size",
            Column::Date => "date",
            Column::Type => "type",
        }
    }

    /// The column with the given spelling in the configuration file.
    pub fn from_config_name(name: &str) -> Option<Column> {
        Column::all()
            .into_iter()
            .find(|column| column.config_name() == name)
    }
}

/// The way a panel shows its entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ViewMode {
    /// The detailed list with the columns.
    #[default]
    Details,

    /// The compact list with the names only.
    Compact,
}
