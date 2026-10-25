//! The state of one panel.
//!
//! WHAT: the side of a panel ([`PanelSide`]), the columns of a file list
//!       ([`Column`]), the view mode ([`ViewMode`]), the marked entries
//!       ([`Selection`]), a tab ([`Tab`]), the directory tree
//!       ([`DirectoryTree`]), the place the keyboard is in ([`PanelFocus`])
//!       and the state of a panel ([`PanelState`]).
//! WHY: a panel owns its tabs, and a tab holds its own path, its own order,
//!       its own view, its own cursor, its own marks and its own filter
//!       (FR-12). The panel itself holds the list of the tabs, the tab that
//!       is shown, the tree and the place the keyboard is in. The transfer
//!       into the other panel is always explicit and is therefore not part
//!       of this type (FR-16).
//! SIZE: the state of a panel and the parts it owns stay together. A tab
//!       holds its own path, order, view, cursor, marks and filter, and the
//!       panel holds the list of the tabs, the tree and the focus. Splitting
//!       the state from the types that describe it would scatter the
//!       invariants of FR-12 over two modules.
//!
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
pub mod tree;

use std::path::{Path, PathBuf};

use crate::filter::{Filter, FilterDepth, FilterKind};
use crate::listing::{Entry, ListingState, Row, Sort};

pub use columns::{Column, ViewMode};
pub use selection::Selection;
pub use tab::Tab;
pub use tree::{DirectoryTree, TreeNode, TreeNodeKind};

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

/// The place the keyboard is in a panel.
///
/// WHY: a panel holds two places the keyboard can be in — the list of the
/// entries and the tree of the directories. The place is a state of the
/// panel and not a decision of the drawing code, so that the arrow keys
/// always move the cursor in the place the user is looking at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum PanelFocus {
    /// The list of the entries.
    #[default]
    List,

    /// The tree of the directories.
    Tree,
}

/// The state of a panel.
///
/// WHY: the panel owns its tabs and the tab that is shown. Every operation
/// of the interface acts on the panel that has the focus; the transfer into
/// the other panel is always explicit and is therefore not part of this type
/// (FR-16). The tree belongs to the panel and can be switched on and off
/// for it alone (FR-14).
#[derive(Debug)]
pub struct PanelState {
    tabs: Vec<Tab>,
    active_tab: usize,
    tree: DirectoryTree,
    tree_visible: bool,
    tree_touched: bool,
    focus: PanelFocus,
    fallback_tab: Tab,
}

impl PanelState {
    /// A panel with one tab that shows a path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            tabs: vec![Tab::new(path)],
            active_tab: 0,
            tree: DirectoryTree::new(),
            tree_visible: false,
            tree_touched: false,
            focus: PanelFocus::List,
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
        match self.tabs.get(index) {
            Some(tab) => tab,
            None => &self.fallback_tab,
        }
    }

    /// The tab that is shown.
    pub fn active_tab_mut(&mut self) -> &mut Tab {
        let index = self.active_tab.min(self.tabs.len().saturating_sub(1));
        match self.tabs.get_mut(index) {
            Some(tab) => tab,
            None => &mut self.fallback_tab,
        }
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

    /// Opens a new tab that shows the path of the tab that is shown.
    ///
    /// WHY: a new tab opens at the place the user is at. The user who wants
    /// another directory types it into the path field afterwards.
    pub fn new_tab(&mut self) {
        let path = self.active_tab().path().to_path_buf();
        self.tabs.push(Tab::new(path));
        self.active_tab = self.tabs.len() - 1;
        self.reconcile_cursor();
    }

    /// Closes the tab that is shown.
    ///
    /// WHY: the last tab of a panel is never closed. A panel without a tab
    /// has nothing to show, and the user would have to restart the program
    /// to get a panel back.
    pub fn close_tab(&mut self) -> bool {
        let index = self.active_tab_index();
        self.close_tab_at(index)
    }

    /// Closes the tab with the given number.
    ///
    /// WHY: the tab that is closed is named, so that a click on the button
    /// of a tab closes that tab and not the one that is shown.
    pub fn close_tab_at(&mut self, index: usize) -> bool {
        if self.tabs.len() <= 1 || index >= self.tabs.len() {
            return false;
        }
        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        self.reconcile_cursor();
        true
    }

    /// Takes the tab with the given number out of the panel.
    ///
    /// WHY: the move of a tab into the other panel is explicit (FR-13). The
    /// method hands the tab to the caller and therefore leaves the decision
    /// of where it goes to the caller.
    pub fn take_tab(&mut self, index: usize) -> Option<Tab> {
        if index >= self.tabs.len() {
            return None;
        }
        let tab = self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len().saturating_sub(1);
        }
        self.reconcile_cursor();
        Some(tab)
    }

    /// Puts a tab into the panel and shows it.
    ///
    /// WHY: the tab that comes from the other panel keeps its own state —
    /// its path, its order, its cursor, its marks and its filter (FR-12).
    pub fn insert_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.reconcile_cursor();
    }

    /// The path of the tab that is shown.
    pub fn path(&self) -> &Path {
        self.active_tab().path()
    }

    /// Changes the path of the tab that is shown.
    ///
    /// WHY: the filter belongs to the tab, so that it survives the change
    /// of the directory of that tab (FR-74f). The method therefore changes
    /// the path and nothing else.
    pub fn set_path(&mut self, path: PathBuf) {
        self.active_tab_mut().set_path(path);
        self.sync_tree();
        self.reconcile_cursor();
    }

    /// Puts the entries into the listing of the tab that is shown.
    ///
    /// WHY: the method refreshes the index of the filter, so that the
    /// entries that pass the filter are known right after they arrived, and
    /// it feeds the tree with the directories below the current path.
    pub fn set_entries(&mut self, entries: Vec<Entry>) {
        self.active_tab_mut().set_entries(entries);
        self.sync_tree();
    }

    /// Changes the order of the entries and applies it.
    pub fn set_sort(&mut self, sort: Sort) {
        self.active_tab_mut().set_sort(sort);
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
        self.active_tab().row_count()
    }

    /// The number of entries of the directory, without the filter.
    pub fn total_count(&self) -> usize {
        self.active_tab().total_count()
    }

    /// The rows of the screen that are visible.
    ///
    /// WHY: only the visible slice of the list is handed to the widgets, so
    /// that a directory with a hundred thousand entries costs the same in
    /// every frame as a directory with thirty entries (FR-106).
    pub fn visible_rows(&self, first: usize, count: usize) -> Vec<Row<'_>> {
        self.active_tab().visible_rows(first, count)
    }

    /// The position of the entry the cursor is on.
    pub fn cursor(&self) -> usize {
        self.active_tab().cursor()
    }

    /// The row of the screen in which the entry under the cursor is shown.
    pub fn cursor_row(&self) -> Option<usize> {
        self.active_tab().cursor_row()
    }

    /// The entry the cursor is on.
    ///
    /// WHY: an entry that the filter hides is not on the screen, and the
    /// cursor does not point at it. The answer is therefore `None` when the
    /// entry under the cursor does not pass the filter.
    pub fn cursor_entry(&self) -> Option<&Entry> {
        self.active_tab().cursor_entry()
    }

    /// Moves the cursor to the given position.
    ///
    /// WHY: a position that is not on the screen is refused, because the
    /// cursor has to point at an entry the user can see. The cursor then
    /// goes to the last entry that is on the screen.
    pub fn set_cursor(&mut self, position: usize) {
        self.active_tab_mut().set_cursor(position);
    }

    /// Moves the cursor to the entry in the given row of the screen.
    ///
    /// WHY: the row is the unit the user sees. The home key goes to the
    /// first row and the end key to the last one, and both are rows of the
    /// list that the filter makes visible.
    pub fn set_cursor_row(&mut self, row: usize) {
        self.active_tab_mut().set_cursor_row(row);
    }

    /// Moves the cursor by a number of rows.
    pub fn move_cursor(&mut self, rows: isize) {
        self.active_tab_mut().move_cursor(rows);
    }

    /// Puts the cursor back onto an entry that is on the screen.
    ///
    /// WHY: when the filter changes, the entry under the cursor can stop
    /// passing the filter. The cursor then goes to the first entry that
    /// passes it, and it never points at an entry that is not on the screen.
    pub fn reconcile_cursor(&mut self) {
        self.active_tab_mut().reconcile_cursor();
    }

    /// Rebuilds the index of the entries that pass the filter.
    ///
    /// WHY: the index is built when the filter or the directory changes and
    /// not in every frame. The method compares the filter and the revision
    /// of the listing and does nothing when both are the same.
    pub fn refresh(&mut self) {
        self.active_tab_mut().refresh();
    }

    /// The marked entries of the panel.
    pub fn selection(&self) -> &Selection {
        self.active_tab().selection()
    }

    /// Whether the entry with the position is marked.
    pub fn is_marked(&self, position: usize) -> bool {
        self.active_tab().is_marked(position)
    }

    /// Switches the mark of the entry with the position.
    pub fn toggle_at(&mut self, position: usize) {
        self.active_tab_mut().toggle_at(position);
    }

    /// Switches the mark of the entry the cursor is on.
    pub fn toggle_at_cursor(&mut self) {
        self.active_tab_mut().toggle_at_cursor();
    }

    /// Marks every entry that is on the screen.
    ///
    /// WHY: with a filter the user marks what he sees, and not the entries
    /// that the filter hides.
    pub fn select_all(&mut self) {
        self.active_tab_mut().select_all();
    }

    /// Removes every mark.
    pub fn clear_selection(&mut self) {
        self.active_tab_mut().clear_selection();
    }

    /// Inverts the marks of every entry that is on the screen.
    pub fn invert_selection(&mut self) {
        self.active_tab_mut().invert_selection();
    }

    /// The marked entries, in the order of the listing.
    pub fn marked_entries(&self) -> Vec<&Entry> {
        self.active_tab().marked_entries()
    }

    /// The first row of the screen.
    pub fn scroll_row(&self) -> usize {
        self.active_tab().scroll_row()
    }

    /// Scrolls to the given row.
    pub fn set_scroll_row(&mut self, row: usize) {
        self.active_tab_mut().set_scroll_row(row);
    }

    /// Scrolls by a number of rows.
    pub fn scroll_by(&mut self, rows: isize) {
        self.active_tab_mut().scroll_by(rows);
    }

    /// Scrolls so that the cursor is inside the window of `rows` rows.
    ///
    /// WHY: the cursor and the scroll position are two states, and the
    /// cursor must never leave the screen — a file manager that scrolls away
    /// from its own cursor has lost the place the user was at.
    pub fn ensure_cursor_visible(&mut self, rows: usize) {
        self.active_tab_mut().ensure_cursor_visible(rows);
    }

    /// The filter of the panel.
    pub fn filter(&self) -> &Filter {
        self.active_tab().filter()
    }

    /// The text in the filter field.
    pub fn filter_text(&self) -> &str {
        self.active_tab().filter_text()
    }

    /// Whether the line with the filter field is drawn.
    ///
    /// WHY: the line is drawn while the field is being edited and while the
    /// filter is in use. A filter that was cancelled leaves no line behind.
    pub fn is_filter_visible(&self) -> bool {
        self.active_tab().is_filter_visible()
    }

    /// Whether the filter field has the focus.
    pub fn is_filter_editing(&self) -> bool {
        self.active_tab().is_filter_editing()
    }

    /// The number of entries that pass the filter.
    pub fn matching_count(&self) -> usize {
        self.active_tab().matching_count()
    }

    /// Shows the filter field and puts the caret into it.
    pub fn open_filter(&mut self) {
        self.active_tab_mut().open_filter();
    }

    /// Writes the text into the filter field and applies it.
    pub fn set_filter_text(&mut self, text: &str) {
        self.active_tab_mut().set_filter_text(text);
    }

    /// Inserts text into the filter field and applies it.
    pub fn filter_insert(&mut self, text: &str) {
        self.active_tab_mut().filter_insert(text);
    }

    /// Deletes the character before the caret and applies the filter.
    pub fn filter_backspace(&mut self) {
        self.active_tab_mut().filter_backspace();
    }

    /// Deletes the character at the caret and applies the filter.
    pub fn filter_delete(&mut self) {
        self.active_tab_mut().filter_delete();
    }

    /// Moves the caret in the filter field.
    pub fn filter_move_caret(&mut self, delta: isize) {
        self.active_tab_mut().filter_move_caret(delta);
    }

    /// The position of the caret in the filter field.
    pub fn filter_caret(&self) -> usize {
        self.active_tab().filter_caret()
    }

    /// Puts the caret in front of the text of the filter field.
    pub fn filter_caret_to_start(&mut self) {
        self.active_tab_mut().filter_caret_to_start();
    }

    /// Puts the caret behind the text of the filter field.
    pub fn filter_caret_to_end(&mut self) {
        self.active_tab_mut().filter_caret_to_end();
    }

    /// Confirms the filter: the text stays and the field loses the focus.
    pub fn confirm_filter(&mut self) {
        self.active_tab_mut().confirm_filter();
    }

    /// Cancels the filter: the text is cleared and the line is hidden.
    pub fn cancel_filter(&mut self) {
        self.active_tab_mut().cancel_filter();
    }

    /// Limits the filter to the files, to the directories or to both.
    pub fn set_filter_kind(&mut self, kind: FilterKind) {
        self.active_tab_mut().set_filter_kind(kind);
    }

    /// Changes the depth the filter reaches.
    pub fn set_filter_depth(&mut self, depth: FilterDepth) {
        self.active_tab_mut().set_filter_depth(depth);
    }

    /// The tree of the directories of the panel.
    pub fn tree(&self) -> &DirectoryTree {
        &self.tree
    }

    /// The tree of the directories of the panel.
    pub fn tree_mut(&mut self) -> &mut DirectoryTree {
        &mut self.tree
    }

    /// Whether the tree is shown.
    pub fn is_tree_visible(&self) -> bool {
        self.tree_visible
    }

    /// Shows or hides the tree.
    ///
    /// WHY: the tree belongs to one panel and not to the surface (FR-14).
    /// The place the keyboard is in follows the tree: a tree that appears
    /// takes the keyboard, so that the arrow keys move the cursor in the
    /// tree and not somewhere the user does not see.
    pub fn toggle_tree(&mut self) -> bool {
        self.tree_visible = !self.tree_visible;
        self.tree_touched = true;
        self.focus = if self.tree_visible {
            PanelFocus::Tree
        } else {
            PanelFocus::List
        };
        self.tree_visible
    }

    /// Applies the layout of the context to the panel.
    ///
    /// WHY: the layout decides whether a tree exists at all. A panel whose
    /// tree the user has switched keeps its own decision — the layout never
    /// takes a decision away from the user.
    pub fn apply_profile(&mut self, profile: &crate::layout::LayoutProfile) {
        if self.tree_touched {
            return;
        }
        self.tree_visible = profile.areas().directory_tree();
        if !self.tree_visible {
            self.focus = PanelFocus::List;
        }
    }

    /// The place the keyboard is in.
    pub fn focus(&self) -> PanelFocus {
        self.focus
    }

    /// Puts the keyboard in the given place.
    pub fn set_focus(&mut self, focus: PanelFocus) {
        self.focus = focus;
    }

    /// Feeds the tree with the path that is shown and the directories below
    /// it.
    ///
    /// WHY: the tree reads nothing by itself. It shows the directories the
    /// user has opened and the directories that lie directly below the
    /// current path — nothing else, and never recursively (FR-14).
    fn sync_tree(&mut self) {
        let path = self.active_tab().path().to_path_buf();
        let children: Vec<PathBuf> = self
            .active_tab()
            .listing()
            .entries()
            .iter()
            .filter(|entry| entry.kind.is_directory())
            .map(|entry| path.join(&entry.name))
            .collect();
        self.tree.set_root(&path, children);
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;
