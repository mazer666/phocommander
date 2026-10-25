//! The filter can be limited to a kind of the entries.
//!
//! WHAT: the test filters one list with one case per kind: the files, the
//!       directories and both.
//! WHY: FR-74c — the filter can be limited to files, to directories or to
//!       both. The two kinds partition the list: no entry disappears when
//!       the user switches between them.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-ui --test the_filter_can_be_limited_to_a_kind -- --exact
//! ```

#![allow(clippy::unwrap_used, clippy::expect_used)]

use phocommander_ui::filter::{Filter, FilterKind};
use phocommander_ui::listing::{Entry, EntryKind};

/// The entries of the test.
fn entries() -> Vec<Entry> {
    vec![
        Entry::new("notes.txt", EntryKind::File),
        Entry::new("picture.png", EntryKind::File),
        Entry::new("folder", EntryKind::Directory),
        Entry::new("other", EntryKind::Directory),
    ]
}

/// The names of the entries that pass the filter.
fn matching(kind: FilterKind) -> Vec<String> {
    let entries = entries();
    // WHY: an empty pattern is not a filter — it lets every entry through.
    // The star is the pattern that matches every name.
    let filter = Filter::from_text("*", kind, Default::default());
    entries
        .iter()
        .filter(|entry| filter.matches_entry(entry))
        .map(|entry| entry.name.clone())
        .collect()
}

#[test]
fn the_filter_can_be_limited_to_a_kind() {
    // act — the files.
    let files = matching(FilterKind::Files);

    // assert
    assert_eq!(files, vec!["notes.txt", "picture.png"]);

    // act — the directories.
    let directories = matching(FilterKind::Directories);

    // assert
    assert_eq!(directories, vec!["folder", "other"]);

    // act — both.
    let all = matching(FilterKind::All);

    // assert
    assert_eq!(all, vec!["notes.txt", "picture.png", "folder", "other"]);

    // assert — the two kinds partition the list: no entry is lost.
    assert_eq!(files.len() + directories.len(), all.len());
}
