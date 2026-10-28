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

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn every_column_has_a_name_in_the_configuration_file() {
        // WHY: the column set of FR-17 is a setting of the user, and the
        //       names of the configuration file are the way the user writes
        //       it down. A column without a name could not be switched.
        for column in Column::all() {
            assert_eq!(
                Column::from_config_name(column.config_name()),
                Some(column),
                "the column {column:?} should be found by its name"
            );
        }
    }

    #[test]
    fn the_names_of_the_columns_are_the_ones_the_specification_names() {
        // act, assert
        assert_eq!(Column::all().len(), 4);
        assert_eq!(Column::Name.config_name(), "name");
        assert_eq!(Column::Size.config_name(), "size");
        assert_eq!(Column::Date.config_name(), "date");
        assert_eq!(Column::Type.config_name(), "type");
    }

    #[test]
    fn a_name_that_is_no_column_gives_no_column() {
        // act, assert
        // WHY: a file of the configuration that names a column that does
        //       not exist has to be refused, and not silently ignored.
        assert_eq!(Column::from_config_name("colour"), None);
        assert_eq!(Column::from_config_name(""), None);
        assert_eq!(Column::from_config_name("Name"), None);
    }

    #[test]
    fn the_view_mode_of_a_panel_is_the_detailed_list_by_default() {
        // act, assert
        // WHY: the default is the mode that shows the most, so that the
        //       user sees the size and the date without asking for them.
        assert_eq!(ViewMode::default(), ViewMode::Details);
    }
}
