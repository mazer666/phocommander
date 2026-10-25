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
//!       does not survive a rename that happened somewhere else. The filter
//!       belongs to the panel as well and not to the tab, so that it survives
//!       a change of the directory (FR-74f).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::PanelState;
//! use std::path::PathBuf;
//!
//! let mut panel = PanelState::new(PathBuf::from("/tmp"));
//! panel.toggle_at(0);
//! assert_eq!(panel.selection().len(), 1);
//! ```

pub mod columns;
pub mod selection;
pub mod tab;

use std::path::{Path, PathBuf};

use crate::filter::{Filter, FilterDepth, FilterField, FilterKind, FilteredIndex};
use crate::listing::{Entry, ListingState, Row, Sort};

pub use columns::{Column, ViewMode};
pub use selection::Selection;
pub use tab::Tab;

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

/// The state of a panel.
///
/// WHY: the panel owns its tabs and its cursor and its marks. Every
/// operation of the interface acts on the panel that has the focus; the
/// transfer into the other panel is always explicit and is therefore not
/// part of this type (FR-16). The panel owns the filter as well, and the
/// filter survives a change of the directory (FR-74f).
#[derive(Debug)]
pub struct PanelState {
    tabs: Vec<Tab>,
    active_tab: usize,
    cursor: usize,
    selection: Selection,
    scroll_row: usize,
    fallback_tab: Tab,
    filter: Filter,
    field: FilterField,
    index: FilteredIndex,
    index_revision: u64,
    index_filter: Filter,
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
            filter: Filter::default(),
            field: FilterField::new(),
            index: FilteredIndex::default(),
            index_revision: 0,
            index_filter: Filter::default(),
        }
    }

    /// The tabs of the panel.
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    /// The tab that is shown.
    pub fn active_tab(&self) -> &Tab {
        let index = self.active_tab.min(self.tabs.len().saturating_sub(1));
        self.tabs.get(index).unwrap_or(&self.fallback_tab)
    }

    /// The tab that is shown.
    pub fn active_tab_mut(&mut self) -> &mut Tab {
        let index = self.active_tab.min(self.tabs.len().saturating_sub(1));
        self.tabs.get_mut(index).unwrap_or(&mut self.fallback_tab)
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
        self.reconcile_cursor();
        true
    }

    /// Shows the next tab.
    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
        self.reconcile_cursor();
    }

    /// Shows the previous tab.
    pub fn previous_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active_tab = (self.active_tab + self.tabs.len() - 1) % self.tabs.len();
        self.reconcile_cursor();
    }

    /// The path of the tab that is shown.
    pub fn path(&self) -> &Path {
        self.active_tab().path()
    }

    /// Changes the path of the tab that is shown.
    ///
    /// WHY: the filter belongs to the panel and not to the tab, so that it
    /// survives the change of the directory (FR-74f). The method therefore
    /// changes the path and nothing else.
    pub fn set_path(&mut self, path: PathBuf) {
        self.active_tab_mut().set_path(path);
        self.reconcile_cursor();
    }

    /// Puts the entries into the listing of the tab that is shown.
    ///
    /// WHY: the method refreshes the index of the filter, so that the
    /// entries that pass the filter are known right after they arrived.
    pub fn set_entries(&mut self, entries: Vec<Entry>) {
        self.active_tab_mut().listing_mut().set_entries(entries);
        self.refresh();
    }

    /// Changes the order of the entries and applies it.
    pub fn set_sort(&mut self, sort: Sort) {
        self.active_tab_mut().listing_mut().set_sort(sort);
        self.refresh();
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
    ///
    /// WHY: the caller changes the content behind the back of the panel and
    /// has to call [`Self::refresh`] afterwards, so that the index of the
    /// filter is rebuilt.
    pub fn listing_mut(&mut self) -> &mut ListingState {
        self.active_tab_mut().listing_mut()
    }

    /// The number of entries that are on the screen.
    ///
    /// WHY: with a filter the screen shows the entries that pass the
    /// filter, so the number of the visible entries is the number of the
    /// matches (FR-74e).
    pub fn row_count(&self) -> usize {
        self.index.len()
    }

    /// The number of entries of the directory, without the filter.
    pub fn total_count(&self) -> usize {
        self.listing().len()
    }

    /// The rows of the screen that are visible.
    ///
    /// WHY: only the visible slice of the list is handed to the widgets, so
    /// that a directory with a hundred thousand entries costs the same in
    /// every frame as a directory with thirty entries (FR-106).
    pub fn visible_rows(&self, first: usize, count: usize) -> Vec<Row<'_>> {
        let listing = self.listing();
        let mut rows = Vec::new();
        for offset in first..first.saturating_add(count) {
            let Some(position) = self.index.position_of_row(offset) else {
                break;
            };
            let Some(row) = listing.row_at(position, offset) else {
                break;
            };
            rows.push(row);
        }
        rows
    }

    /// The position of the entry the cursor is on.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The row of the screen in which the entry under the cursor is shown.
    pub fn cursor_row(&self) -> Option<usize> {
        self.index.row_of_position(self.cursor)
    }

    /// The entry the cursor is on.
    ///
    /// WHY: an entry that the filter hides is not on the screen, and the
    /// cursor does not point at it. The answer is therefore `None` when the
    /// entry under the cursor does not pass the filter.
    pub fn cursor_entry(&self) -> Option<&Entry> {
        if !self.index.contains(self.cursor) {
            return None;
        }
        self.listing().entry_at(self.cursor)
    }

    /// Moves the cursor to the given position.
    ///
    /// WHY: a position that is not on the screen is refused, because the
    /// cursor has to point at an entry the user can see. The cursor then
    /// goes to the last entry that is on the screen.
    pub fn set_cursor(&mut self, position: usize) {
        if self.index.contains(position) {
            self.cursor = position;
            return;
        }
        if let Some(last) = self.index.positions().last() {
            self.cursor = *last;
        } else {
            self.cursor = 0;
        }
    }

    /// Moves the cursor to the entry in the given row of the screen.
    ///
    /// WHY: the row is the unit the user sees. The home key goes to the
    /// first row and the end key to the last one, and both are rows of the
    /// list that the filter makes visible.
    pub fn set_cursor_row(&mut self, row: usize) {
        let last = self.row_count().saturating_sub(1);
        if let Some(position) = self.index.position_of_row(row.min(last)) {
            self.cursor = position;
        }
    }

    /// Moves the cursor by a number of rows.
    pub fn move_cursor(&mut self, rows: isize) {
        let current = self.cursor_row().unwrap_or(0);
        let last = self.row_count().saturating_sub(1) as isize;
        let target = (current as isize + rows).clamp(0, last);
        if let Some(position) = self.index.position_of_row(target as usize) {
            self.cursor = position;
        }
    }

    /// Puts the cursor back onto an entry that is on the screen.
    ///
    /// WHY: when the filter changes, the entry under the cursor can stop
    /// passing the filter. The cursor then goes to the first entry that
    /// passes it, and it never points at an entry that is not on the screen.
    pub fn reconcile_cursor(&mut self) {
        if self.index.is_empty() {
            self.cursor = 0;
            self.scroll_row = 0;
            return;
        }
        if !self.index.contains(self.cursor)
            && let Some(first) = self.index.positions().first()
        {
            self.cursor = *first;
        }
        self.set_scroll_row(self.scroll_row);
    }

    /// Rebuilds the index of the entries that pass the filter.
    ///
    /// WHY: the index is built when the filter or the directory changes and
    /// not in every frame. The method compares the filter and the revision
    /// of the listing and does nothing when both are the same.
    pub fn refresh(&mut self) {
        let revision = self.listing().revision();
        if revision != self.index_revision || self.filter != self.index_filter {
            let index = FilteredIndex::build(
                self.listing().entries(),
                self.listing().order(),
                &self.filter,
            );
            self.index = index;
            self.index_revision = revision;
            self.index_filter = self.filter.clone();
        }
        self.reconcile_cursor();
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

    /// Marks every entry that is on the screen.
    ///
    /// WHY: with a filter the user marks what he sees, and not the entries
    /// that the filter hides.
    pub fn select_all(&mut self) {
        self.selection = Selection::new();
        let positions: Vec<usize> = self.index.positions().to_vec();
        for position in positions {
            self.selection.insert(position);
        }
    }

    /// Removes every mark.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Inverts the marks of every entry that is on the screen.
    pub fn invert_selection(&mut self) {
        let positions: Vec<usize> = self.index.positions().to_vec();
        for position in positions {
            self.selection.toggle(position);
        }
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
        let Some(view_row) = self.cursor_row() else {
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

    /// The filter of the panel.
    pub fn filter(&self) -> &Filter {
        &self.filter
    }

    /// The text in the filter field.
    pub fn filter_text(&self) -> &str {
        self.field.text()
    }

    /// Whether the line with the filter field is drawn.
    ///
    /// WHY: the line is drawn while the field is being edited and while the
    /// filter is in use. A filter that was cancelled leaves no line behind.
    pub fn is_filter_visible(&self) -> bool {
        self.field.is_visible() || self.filter.is_active()
    }

    /// Whether the filter field has the focus.
    pub fn is_filter_editing(&self) -> bool {
        self.field.is_editing()
    }

    /// The number of entries that pass the filter.
    pub fn matching_count(&self) -> usize {
        self.index.len()
    }

    /// Shows the filter field and puts the caret into it.
    pub fn open_filter(&mut self) {
        self.field.open();
    }

    /// Writes the text into the filter field and applies it.
    pub fn set_filter_text(&mut self, text: &str) {
        self.field.set_text(text);
        self.apply_filter();
    }

    /// Inserts text into the filter field and applies it.
    pub fn filter_insert(&mut self, text: &str) {
        self.field.insert(text);
        self.apply_filter();
    }

    /// Deletes the character before the caret and applies the filter.
    pub fn filter_backspace(&mut self) {
        self.field.backspace();
        self.apply_filter();
    }

    /// Deletes the character at the caret and applies the filter.
    pub fn filter_delete(&mut self) {
        self.field.delete();
        self.apply_filter();
    }

    /// Moves the caret in the filter field.
    pub fn filter_move_caret(&mut self, delta: isize) {
        self.field.move_caret(delta);
    }

    /// The position of the caret in the filter field.
    pub fn filter_caret(&self) -> usize {
        self.field.caret()
    }

    /// Puts the caret in front of the text of the filter field.
    pub fn filter_caret_to_start(&mut self) {
        self.field.caret_to_start();
    }

    /// Puts the caret behind the text of the filter field.
    pub fn filter_caret_to_end(&mut self) {
        self.field.caret_to_end();
    }

    /// Confirms the filter: the text stays and the field loses the focus.
    pub fn confirm_filter(&mut self) {
        self.field.confirm();
    }

    /// Cancels the filter: the text is cleared and the line is hidden.
    pub fn cancel_filter(&mut self) {
        self.field.clear();
        self.filter = self.filter.without_pattern();
        self.refresh();
    }

    /// Limits the filter to the files, to the directories or to both.
    pub fn set_filter_kind(&mut self, kind: FilterKind) {
        self.filter = Filter::new(self.filter.pattern().clone(), kind, self.filter.depth());
        self.refresh();
    }

    /// Changes the depth the filter reaches.
    pub fn set_filter_depth(&mut self, depth: FilterDepth) {
        self.filter = Filter::new(self.filter.pattern().clone(), self.filter.kind(), depth);
        self.refresh();
    }

    /// Reads the text of the field into the filter.
    ///
    /// WHY: the text of the field is the pattern, and the pattern is
    /// applied while the user types — there is no delay between the key
    /// stroke and the new list (FR-74a).
    fn apply_filter(&mut self) {
        let kind = self.filter.kind();
        let depth = self.filter.depth();
        self.filter = Filter::from_text(self.field.text(), kind, depth);
        self.refresh();
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;
