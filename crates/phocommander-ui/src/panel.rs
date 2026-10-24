//! The state of one panel.
//!
//! WHAT: the side of a panel ([`PanelSide`]), the columns of a file list
//!       ([`Column`]), the view mode ([`ViewMode`]), the marked entries
//!       ([`Selection`]), a tab ([`Tab`]) and the state of a panel
//!       ([`PanelState`]).
//! WHY: a panel owns its tabs, and a tab holds its own path, its own order
//!       and its own view state (FR-12). The cursor and the marks belong to
//!       the panel, and the marks hold *positions* and not paths: a position
//!       keeps its meaning when the order of the list changes, while a path
//!       does not survive a rename that happened somewhere else.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::PanelState;
//! use std::path::PathBuf;
//!
//! let mut panel = PanelState::new(PathBuf::from("/tmp"));
//! panel.toggle_at(0);
//! assert_eq!(panel.selection().len(), 1);
//! ```

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::listing::{Entry, ListingState, Sort, SortKey};

/// The side of a panel in the work surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PanelSide {
    /// The panel on the left.
    Left,

    /// The panel on the right.
    Right,
}

impl PanelSide {
    /// The other side.
    ///
    /// WHY: the transfer to the other panel is always explicit (FR-16). The
    /// other side is therefore a value of the type and not a decision that
    /// a caller has to make again and again.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::panel::PanelSide;
    ///
    /// assert_eq!(PanelSide::Left.other(), PanelSide::Right);
    /// ```
    pub const fn other(&self) -> PanelSide {
        match self {
            PanelSide::Left => PanelSide::Right,
            PanelSide::Right => PanelSide::Left,
        }
    }

    /// The spelling of the side in the interface.
    pub const fn as_str(&self) -> &'static str {
        match self {
            PanelSide::Left => "left",
            PanelSide::Right => "right",
        }
    }
}

impl std::fmt::Display for PanelSide {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

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

/// The marked entries of a panel.
///
/// WHY: the marks hold the *positions* of the entries and not their paths.
/// A position is the index in the order in which the source delivered the
/// entries, and it does not change when the order of the screen changes.
/// The marks therefore stay on the same files when the user sorts the list.
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

    /// Inverts the marks of the first `count` entries.
    pub fn invert(&mut self, count: usize) {
        for position in 0..count {
            self.toggle(position);
        }
    }

    /// The marked positions, in the order of the listing.
    pub fn positions(&self) -> impl Iterator<Item = usize> + '_ {
        self.positions.iter().copied()
    }
}

/// A tab of a panel.
///
/// WHY: a tab holds its own path, its own order and its own view state
/// (FR-12). The order and the view are therefore properties of the tab and
/// not of the panel, so that two tabs of one panel can be ordered
/// differently.
#[derive(Debug)]
pub struct Tab {
    path: PathBuf,
    sort: Sort,
    columns: Vec<Column>,
    view: ViewMode,
    listing: ListingState,
}

impl Tab {
    /// A tab that shows a path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            listing: ListingState::new(path.clone()),
            path,
            sort: Sort::by(SortKey::Name),
            columns: Column::all().to_vec(),
            view: ViewMode::default(),
        }
    }

    /// The path of the tab.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Changes the path of the tab.
    ///
    /// WHY: the path belongs to the tab and to the listing of the tab, and
    /// the two must not drift apart. The method therefore sets both.
    pub fn set_path(&mut self, path: PathBuf) {
        self.listing.set_path(path.clone());
        self.path = path;
    }

    /// The order of the entries.
    pub fn sort(&self) -> Sort {
        self.sort
    }

    /// The columns of the file list.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// The way the tab shows its entries.
    pub fn view(&self) -> ViewMode {
        self.view
    }

    /// The content of the directory of the tab.
    pub fn listing(&self) -> &ListingState {
        &self.listing
    }

    /// The content of the directory of the tab.
    pub fn listing_mut(&mut self) -> &mut ListingState {
        &mut self.listing
    }
}

/// The state of a panel.
///
/// WHY: the panel owns its tabs and its cursor and its marks. Every
/// operation of the interface acts on the panel that has the focus; the
/// transfer into the other panel is always explicit and is therefore not
/// part of this type (FR-16).
#[derive(Debug)]
pub struct PanelState {
    tabs: Vec<Tab>,
    active_tab: usize,
    cursor: usize,
    selection: Selection,
    scroll_row: usize,
    fallback_tab: Tab,
}

impl PanelState {
    /// A panel with one tab that shows a path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            tabs: vec![Tab::new(path)],
            active_tab: 0,
            cursor: 0,
            selection: Selection::new(),
            scroll_row: 0,
            fallback_tab: Tab::new(PathBuf::new()),
        }
    }

    /// The tabs of the panel.
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// The tab that is shown.
    pub fn active_tab(&self) -> &Tab {
        let index = self.active_tab.min(self.tabs.len().saturating_sub(1));
        self.tabs
            .get(index)
            .unwrap_or(&self.fallback_tab)
    }

    /// The tab that is shown.
    pub fn active_tab_mut(&mut self) -> &mut Tab {
        let index = self.active_tab.min(self.tabs.len().saturating_sub(1));
        self.tabs
            .get_mut(index)
            .unwrap_or(&mut self.fallback_tab)
    }

    /// The number of tabs.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// The number of the tab that is shown.
    pub fn active_tab_index(&self) -> usize {
        self.active_tab.min(self.tabs.len().saturating_sub(1))
    }

    /// Shows another tab.
    ///
    /// WHY: a tab that does not exist is refused instead of creating an
    /// empty one, so that a binding to a tab that was closed does not
    /// change the panel.
    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        self.active_tab = index;
        self.clamp_cursor();
        true
    }

    /// Shows the next tab.
    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
        self.clamp_cursor();
    }

    /// Shows the previous tab.
    pub fn previous_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = (self.active_tab + self.tabs.len() - 1) % self.tabs.len();
        self.clamp_cursor();
    }

    /// The path of the tab that is shown.
    pub fn path(&self) -> &Path {
        self.active_tab().path()
    }

    /// Changes the path of the tab that is shown.
    pub fn set_path(&mut self, path: PathBuf) {
        self.active_tab_mut().set_path(path);
        self.clamp_cursor();
    }

    /// The order of the entries of the tab that is shown.
    pub fn sort(&self) -> Sort {
        self.active_tab().sort()
    }

    /// The columns of the file list.
    pub fn columns(&self) -> &[Column] {
        self.active_tab().columns()
    }

    /// The way the panel shows its entries.
    pub fn view(&self) -> ViewMode {
        self.active_tab().view()
    }

    /// The content of the directory of the tab that is shown.
    pub fn listing(&self) -> &ListingState {
        self.active_tab().listing()
    }

    /// The content of the directory of the tab that is shown.
    pub fn listing_mut(&mut self) -> &mut ListingState {
        self.active_tab_mut().listing_mut()
    }

    /// The number of entries of the panel.
    pub fn row_count(&self) -> usize {
        self.listing().len()
    }

    /// The position of the entry the cursor is on.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The entry the cursor is on.
    pub fn cursor_entry(&self) -> Option<&Entry> {
        self.listing().entry_at(self.cursor)
    }

    /// Moves the cursor to the given position.
    ///
    /// WHY: a position that is outside the list is clamped instead of
    /// rejected, because the list can become shorter while the cursor is on
    /// it — after a reload, for example.
    pub fn set_cursor(&mut self, position: usize) {
        self.cursor = position.min(self.row_count().saturating_sub(1));
    }

    /// Moves the cursor by a number of rows.
    pub fn move_cursor(&mut self, rows: isize) {
        let current = self.listing().view_row_of_position(self.cursor).unwrap_or(0);
        let last = self.row_count().saturating_sub(1) as isize;
        let target = (current as isize + rows).clamp(0, last);
        if let Some(position) = self.listing().position_of_view_row(target as usize) {
            self.cursor = position;
        }
    }

    /// Puts the cursor back into the list.
    pub fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.min(self.row_count().saturating_sub(1));
    }

    /// The marked entries of the panel.
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Whether the entry with the position is marked.
    pub fn is_marked(&self, position: usize) -> bool {
        self.selection.contains(position)
    }

    /// Switches the mark of the entry with the position.
    pub fn toggle_at(&mut self, position: usize) {
        self.selection.toggle(position);
    }

    /// Switches the mark of the entry the cursor is on.
    pub fn toggle_at_cursor(&mut self) {
        if self.row_count() > 0 {
            self.selection.toggle(self.cursor);
        }
    }

    /// Marks every entry.
    pub fn select_all(&mut self) {
        self.selection = Selection::new();
        for position in 0..self.row_count() {
            self.selection.insert(position);
        }
    }

    /// Removes every mark.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Inverts the marks of every entry.
    pub fn invert_selection(&mut self) {
        self.selection.invert(self.row_count());
    }

    /// The marked entries, in the order of the listing.
    pub fn marked_entries(&self) -> Vec<&Entry> {
        let listing = self.listing();
        self.selection
            .positions()
            .filter_map(|position| listing.entry_at(position))
            .collect()
    }

    /// The first row of the screen.
    pub fn scroll_row(&self) -> usize {
        self.scroll_row
    }

    /// Scrolls to the given row.
    pub fn set_scroll_row(&mut self, row: usize) {
        self.scroll_row = row.min(self.row_count().saturating_sub(1));
    }

    /// Scrolls by a number of rows.
    pub fn scroll_by(&mut self, rows: isize) {
        let target = (self.scroll_row as isize + rows).max(0) as usize;
        self.set_scroll_row(target);
    }

    /// Scrolls so that the cursor is inside the window of `rows` rows.
    ///
    /// WHY: the cursor and the scroll position are two states, and the
    /// cursor must never leave the screen — a file manager that scrolls away
    /// from its own cursor has lost the place the user was at.
    pub fn ensure_cursor_visible(&mut self, rows: usize) {
        if rows == 0 {
            return;
        }
        let Some(view_row) = self.listing().view_row_of_position(self.cursor) else {
            return;
        };
        let first = self.scroll_row;
        if view_row < first {
            self.scroll_row = view_row;
        } else if view_row >= first + rows {
            self.scroll_row = view_row + 1 - rows;
        }
        self.set_scroll_row(self.scroll_row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::listing::EntryKind;

    /// A panel with three entries.
    fn panel() -> PanelState {
        let mut panel = PanelState::new(PathBuf::from("/tmp"));
        panel.listing_mut().set_entries(vec![
            Entry::new("a.txt", EntryKind::File),
            Entry::new("b.txt", EntryKind::File),
            Entry::new("c.txt", EntryKind::File),
        ]);
        panel
    }

    #[test]
    fn the_marks_hold_positions_and_not_paths() {
        // arrange
        let mut panel = panel();

        // act
        panel.toggle_at(1);

        // assert
        assert!(panel.is_marked(1));
        assert!(!panel.is_marked(0));
        assert_eq!(panel.selection().len(), 1);
    }

    #[test]
    fn the_marks_survive_a_change_of_the_order() {
        // arrange
        let mut panel = panel();
        panel.toggle_at(0);

        // act
        panel.listing_mut().set_sort(Sort::by(SortKey::Size));

        // assert
        assert!(panel.is_marked(0));
    }

    #[test]
    fn the_cursor_stays_inside_the_list() {
        // arrange
        let mut panel = panel();

        // act
        panel.set_cursor(99);

        // assert
        assert_eq!(panel.cursor(), 2);
    }

    #[test]
    fn the_cursor_moves_by_rows_and_not_by_positions() {
        // arrange
        let mut panel = panel();
        panel.listing_mut().set_sort(Sort::by(SortKey::Size));

        // act
        panel.move_cursor(1);

        // assert
        let row = panel.listing().view_row_of_position(panel.cursor());
        assert_eq!(row, Some(1));
    }

    #[test]
    fn selecting_all_marks_every_entry() {
        // arrange
        let mut panel = panel();

        // act
        panel.select_all();

        // assert
        assert_eq!(panel.selection().len(), 3);
        panel.invert_selection();
        assert_eq!(panel.selection().len(), 0);
    }

    #[test]
    fn a_tab_that_does_not_exist_is_refused() {
        // arrange
        let mut panel = panel();

        // act
        let refused = panel.switch_tab(3);

        // assert
        assert!(!refused);
        assert_eq!(panel.tab_count(), 1);
    }

    #[test]
    fn the_scroll_follows_the_cursor() {
        // arrange
        let mut panel = panel();

        // act
        panel.set_cursor(2);
        panel.ensure_cursor_visible(2);

        // assert
        assert_eq!(panel.scroll_row(), 1);
    }
}
