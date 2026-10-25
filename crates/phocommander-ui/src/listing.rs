//! The content of a directory, held outside of the egui context.
//!
//! WHAT: the entries of a directory ([`Entry`]), the order in which they
//!       are shown ([`Sort`]) and the state object that holds them
//!       ([`ListingState`]).
//! WHY: egui is an immediate mode: whatever is drawn is rebuilt in every
//!       frame. A directory with a hundred thousand entries therefore may
//!       not live in the drawing code — it lives in this state object, and
//!       only the visible slice of it is handed to the widgets (FR-106,
//!       NFR-01). The state object is therefore the answer to "what is in
//!       this directory", and it knows nothing about pixels. The sizes and
//!       the dates of the rows are written by the formats of the interface
//!       (FR-123), so that the locale of the user decides how they look.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::listing::{Entry, EntryKind, ListingState};
//! use std::path::PathBuf;
//!
//! let mut listing = ListingState::new(PathBuf::from("/tmp"));
//! listing.set_entries(vec![Entry::new("notes.txt", EntryKind::File)]);
//! assert_eq!(listing.len(), 1);
//! ```

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::format::Formats;
use crate::i18n::TextKey;

/// The kind of an entry of a directory listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryKind {
    /// A regular file.
    File,

    /// A directory.
    Directory,

    /// A symbolic link.
    Symlink,

    /// Anything else — a socket, a device node or a named pipe.
    Other,
}

impl EntryKind {
    /// Whether the entry is a regular file.
    pub const fn is_file(&self) -> bool {
        matches!(self, EntryKind::File)
    }

    /// Whether the entry is a directory.
    pub const fn is_directory(&self) -> bool {
        matches!(self, EntryKind::Directory)
    }
}

/// One entry of a directory listing.
///
/// WHY: the surface must not depend on the model crate — the dependency
/// direction of the workspace forbids it — and it needs four things of an
/// entry: the name, the kind, the size and the time of the last change. The
/// shell converts the entries of the model into these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The name of the entry, without the path of its directory.
    pub name: String,

    /// The kind of the entry.
    pub kind: EntryKind,

    /// The size of the entry in bytes, if the source knows it.
    pub size: Option<u64>,

    /// The time of the last change of the entry, if the source knows it.
    pub modified: Option<SystemTime>,

    /// The distance of the entry from the directory that is listed.
    ///
    /// WHY: an entry of the directory that is shown has the distance zero.
    /// The filter uses the distance to decide whether it reaches into the
    /// directories below (FR-74d), and a reader that walks deeper than one
    /// directory fills the value in.
    pub depth: u16,
}

impl Entry {
    /// Builds an entry with a name and a kind.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::listing::{Entry, EntryKind};
    ///
    /// let entry = Entry::new("notes.txt", EntryKind::File);
    /// assert_eq!(entry.name, "notes.txt");
    /// ```
    pub fn new(name: impl Into<String>, kind: EntryKind) -> Self {
        Self {
            name: name.into(),
            kind,
            size: None,
            modified: None,
            depth: 0,
        }
    }

    /// The entry with the given size.
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// The entry with the given time of the last change.
    pub fn with_modified(mut self, modified: SystemTime) -> Self {
        self.modified = Some(modified);
        self
    }

    /// The entry at the given distance from the directory that is listed.
    ///
    /// WHY: the distance is zero for an entry of the directory that is
    /// shown. A reader that walks deeper fills the value in, and the filter
    /// uses it to decide how far it reaches (FR-74d).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::listing::{Entry, EntryKind};
    ///
    /// let entry = Entry::new("notes.txt", EntryKind::File).with_depth(2);
    /// assert_eq!(entry.depth(), 2);
    /// ```
    pub fn with_depth(mut self, depth: u16) -> Self {
        self.depth = depth;
        self
    }

    /// The distance of the entry from the directory that is listed.
    pub const fn depth(&self) -> u16 {
        self.depth
    }
}

/// The property the entries are ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SortKey {
    /// The name of the entry.
    #[default]
    Name,

    /// The extension of the name.
    Extension,

    /// The size of the entry.
    Size,

    /// The time of the last change.
    Modified,
}

/// The direction of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SortDirection {
    /// From small to large.
    #[default]
    Ascending,

    /// From large to small.
    Descending,
}

/// The order in which the entries of a listing are shown.
///
/// WHY: the order belongs to the tab and not to the panel, so that two tabs
/// of the same panel can be ordered differently (FR-18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Sort {
    /// The property the entries are ordered by.
    pub key: SortKey,

    /// The direction of the order.
    pub direction: SortDirection,
}

impl Sort {
    /// The order by a property, from small to large.
    pub const fn by(key: SortKey) -> Self {
        Self {
            key,
            direction: SortDirection::Ascending,
        }
    }

    /// The order by a property, in the given direction.
    pub const fn with_direction(key: SortKey, direction: SortDirection) -> Self {
        Self { key, direction }
    }
}

/// The content of a directory, with the order in which it is shown.
///
/// WHY: the object holds the entries in the order in which the source
/// delivered them and keeps the order of the screen as a permutation of
/// them. A selection that holds *positions* therefore keeps its meaning when
/// the order changes, and a cursor that holds a position points at the same
/// entry in every order.
#[derive(Debug)]
pub struct ListingState {
    path: PathBuf,
    entries: Vec<Entry>,
    order: Vec<usize>,
    inverse: Vec<usize>,
    sort: Sort,
    materialised: Cell<u64>,
    revision: u64,
}

impl ListingState {
    /// The empty listing of a path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            entries: Vec::new(),
            order: Vec::new(),
            inverse: Vec::new(),
            sort: Sort::default(),
            materialised: Cell::new(0),
            revision: 0,
        }
    }

    /// The path this listing belongs to.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Changes the path this listing belongs to.
    ///
    /// WHY: the path belongs to the tab and to the listing of the tab, and
    /// the two must not drift apart. The shell sets the path first and the
    /// entries afterwards.
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }

    /// The order in which the entries are shown.
    pub fn sort(&self) -> Sort {
        self.sort
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the listing is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Replaces the entries and applies the order.
    ///
    /// WHY: the order is applied once, when the entries arrive, and not in
    /// every frame. The shell therefore fills the listing from the worker
    /// that read the directory — the interface thread never sorts.
    pub fn set_entries(&mut self, entries: Vec<Entry>) {
        self.entries = entries;
        self.revision += 1;
        self.apply_sort();
    }

    /// Changes the order and applies it.
    pub fn set_sort(&mut self, sort: Sort) {
        self.sort = sort;
        self.revision += 1;
        self.apply_sort();
    }

    /// The number of times the content or the order of the listing changed.
    ///
    /// WHY: the panel holds the index of the entries that pass the filter,
    /// and the index has to be rebuilt when the listing changes. The
    /// counter tells the panel whether it has to rebuild it, without
    /// comparing the entries themselves.
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// The order of the screen: the position of the entry in every row.
    ///
    /// WHY: the index of the filter follows the order of the screen, so
    /// that the filtered list is sorted like the list without a filter.
    pub fn order(&self) -> &[usize] {
        &self.order
    }

    /// The position of the entry in the row of the screen.
    ///
    /// WHY: the position is the index in the order in which the source
    /// delivered the entries. It does not change when the order of the
    /// screen changes, and a selection that holds positions therefore keeps
    /// its meaning.
    pub fn position_of_view_row(&self, view_row: usize) -> Option<usize> {
        self.order.get(view_row).copied()
    }

    /// The row of the screen in which the entry with the position is shown.
    pub fn view_row_of_position(&self, position: usize) -> Option<usize> {
        self.inverse.get(position).copied()
    }

    /// The entry with the given position.
    pub fn entry_at(&self, position: usize) -> Option<&Entry> {
        self.entries.get(position)
    }

    /// The entries of the listing, in the order in which the source
    /// delivered them.
    ///
    /// WHY: the index of the filter is built from the entries and from the
    /// order of the screen, and it lives outside of the listing.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The row of the entry with the given position.
    ///
    /// WHY: the row is the unit the drawing works with. The counter of the
    /// materialised rows grows with every row that is handed to the
    /// widgets, and it is the evidence that only the visible slice of the
    /// list is drawn (FR-106).
    pub fn row_at(&self, position: usize, offset: usize) -> Option<Row<'_>> {
        let entry = self.entries.get(position)?;
        self.materialised.set(self.materialised.get() + 1);
        Some(Row {
            entry,
            position,
            offset,
        })
    }

    /// The rows of the screen that are visible.
    ///
    /// WHY: only the visible slice of the listing is handed to the widgets.
    /// A directory with a hundred thousand entries therefore costs the same
    /// in every frame as a directory with thirty entries (FR-106).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::listing::{Entry, EntryKind, ListingState};
    /// use std::path::PathBuf;
    ///
    /// let mut listing = ListingState::new(PathBuf::from("/tmp"));
    /// listing.set_entries(
    ///     (0..1000).map(|index| Entry::new(format!("file-{index}"), EntryKind::File)).collect(),
    /// );
    ///
    /// let rows: Vec<_> = listing.visible_rows(0, 10).collect();
    /// assert_eq!(rows.len(), 10);
    /// ```
    pub fn visible_rows(&self, first: usize, count: usize) -> impl Iterator<Item = Row<'_>> {
        let last = first.saturating_add(count).min(self.order.len());
        let order = &self.order[first.min(self.order.len())..last];
        let entries = &self.entries;
        let counter = &self.materialised;

        order.iter().enumerate().map(move |(offset, position)| {
            counter.set(counter.get() + 1);
            Row {
                entry: &entries[*position],
                position: *position,
                offset: first + offset,
            }
        })
    }

    /// The number of rows that were handed to the widgets.
    ///
    /// WHY: the counter is the evidence for the claim that only the visible
    /// slice is drawn. The test of the workspace reads it after a frame.
    pub fn rows_materialised(&self) -> u64 {
        self.materialised.get()
    }

    /// Rebuilds the order of the screen.
    fn apply_sort(&mut self) {
        let mut order: Vec<usize> = (0..self.entries.len()).collect();
        let entries = &self.entries;
        let sort = self.sort;

        order.sort_by(|left, right| {
            let comparison = compare_entries(&entries[*left], &entries[*right], sort.key);
            match sort.direction {
                SortDirection::Ascending => comparison,
                SortDirection::Descending => comparison.reverse(),
            }
        });

        let mut inverse = vec![0usize; order.len()];
        for (view_row, position) in order.iter().enumerate() {
            inverse[*position] = view_row;
        }

        self.order = order;
        self.inverse = inverse;
    }
}

/// Compares two entries by a property.
fn compare_entries(left: &Entry, right: &Entry, key: SortKey) -> std::cmp::Ordering {
    match key {
        SortKey::Name => compare_names(&left.name, &right.name),
        SortKey::Extension => compare_names(&extension_of(&left.name), &extension_of(&right.name))
            .then_with(|| compare_names(&left.name, &right.name)),
        SortKey::Size => left.size.unwrap_or(0).cmp(&right.size.unwrap_or(0)),
        SortKey::Modified => left.modified.cmp(&right.modified),
    }
}

/// Compares two names, ignoring the case.
fn compare_names(left: &str, right: &str) -> std::cmp::Ordering {
    left.to_lowercase().cmp(&right.to_lowercase())
}

/// The extension of a name, in lower case.
fn extension_of(name: &str) -> String {
    match name.rfind('.') {
        Some(position) if position + 1 < name.len() => name[position + 1..].to_lowercase(),
        _ => String::new(),
    }
}

/// A row of the listing that is ready to be drawn.
///
/// WHY: the row holds a reference to the entry and the position of the
/// entry. The text of the columns is built only for the rows that are
/// drawn, so that a directory with a hundred thousand entries does not cost
/// a hundred thousand formatted strings per frame.
#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    entry: &'a Entry,
    position: usize,
    offset: usize,
}

impl<'a> Row<'a> {
    /// The entry of the row.
    pub fn entry(&self) -> &'a Entry {
        self.entry
    }

    /// The position of the entry in the listing.
    pub fn position(&self) -> usize {
        self.position
    }

    /// The number of the row on the screen.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The name of the entry.
    pub fn name(&self) -> &str {
        &self.entry.name
    }

    /// The kind of the entry.
    pub fn kind(&self) -> EntryKind {
        self.entry.kind
    }

    /// The size of the entry, written the way the locale of the interface
    /// writes sizes.
    ///
    /// WHY: the format of a size follows the locale of the user (FR-123).
    ///       The row therefore asks the formats of the interface and knows
    ///       no rule of a locale itself.
    pub fn size_text(&self, formats: &dyn Formats) -> String {
        match self.entry.size {
            Some(size) => formats.size(size),
            None => String::new(),
        }
    }

    /// The time of the last change, written the way the locale of the
    /// interface writes dates and times.
    ///
    /// WHY: the format of a date follows the locale of the user (FR-123).
    ///       The column of the date carries the date and the time together,
    ///       so the row asks for both at once.
    pub fn date_text(&self, formats: &dyn Formats) -> String {
        match self.entry.modified {
            Some(modified) => formats.date_time(modified),
            None => String::new(),
        }
    }

    /// The translation key of the type of the entry.
    ///
    /// WHY: the name of a type is a visible text, and a visible text goes
    /// through a translation key (FR-05). The listing therefore answers with
    /// the key and lets the surface ask the catalogue for the text.
    pub fn type_key(&self) -> TextKey {
        match self.entry.kind {
            EntryKind::File => TextKey::PanelTypeFile,
            EntryKind::Directory => TextKey::PanelTypeDirectory,
            EntryKind::Symlink => TextKey::PanelTypeLink,
            EntryKind::Other => TextKey::PanelTypeOther,
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;
