//! A tab of a panel.
//!
//! WHAT: the path, the order, the columns, the view and the content of a
//!       tab ([`Tab`]).
//! WHY: a tab holds its own path, its own order and its own view state
//!       (FR-12). The order and the view are therefore properties of the
//!       tab and not of the panel, so that two tabs of one panel can be
//!       ordered differently.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::Tab;
//! use std::path::PathBuf;
//!
//! let tab = Tab::new(PathBuf::from("/tmp"));
//! assert_eq!(tab.path(), std::path::Path::new("/tmp"));
//! ```

use std::path::{Path, PathBuf};

use crate::listing::{ListingState, Sort, SortKey};
use crate::panel::columns::{Column, ViewMode};

/// A tab of a panel.
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
