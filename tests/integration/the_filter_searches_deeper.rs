//! The filter of a panel reaches deeper than the current directory.
//!
//! WHAT: the test builds a directory tree of three levels, reads the tree
//!       with the depths 1, 2 and 3 and filters the entries with the filter
//!       of the matching depth.
//! WHY: FR-74d — the filter can search deeper than the current directory,
//!       with a configurable depth. The depth is a property of the filter
//!       and not of the pattern, and the entries carry the distance from
//!       the directory that is shown.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test the_filter_searches_deeper -- --exact
//! ```

use std::path::Path;

use phocommander_core::source::Source;
use phocommander_testkit::TempTree;
use phocommander_ui::filter::{Filter, FilterDepth, FilterKind, FilteredIndex};
use phocommander_ui::listing::Entry;

/// The entries of the tree, with the distance from the directory that is
/// shown.
fn entries_of_tree(root: &Path, levels: u16) -> Vec<Entry> {
    let source = phocommander_core::local::LocalSource::new();
    let mut entries = Vec::new();
    collect(root, 0, levels, &source, &mut entries);
    entries
}

/// Reads the directories of one level.
fn collect(
    directory: &Path,
    depth: u16,
    levels: u16,
    source: &phocommander_core::local::LocalSource,
    entries: &mut Vec<Entry>,
) {
    if depth > levels {
        return;
    }

    let listed = match source.list(directory) {
        Ok(listed) => listed,
        Err(error) => panic!("the directory {directory:?} should be readable: {error}"),
    };

    for entry in listed {
        let kind = if entry.kind.is_directory() {
            phocommander_ui::listing::EntryKind::Directory
        } else {
            phocommander_ui::listing::EntryKind::File
        };
        let is_directory = kind == phocommander_ui::listing::EntryKind::Directory;
        // WHY: the distance counts the directories between the shown
        // directory and the directory of the entry. An entry of the shown
        // directory has the distance zero.
        entries.push(Entry::new(entry.name.clone(), kind).with_depth(depth));

        if is_directory {
            collect(
                &directory.join(&entry.name),
                depth + 1,
                levels,
                source,
                entries,
            );
        }
    }
}

/// The names of the entries that pass the filter of the given depth.
///
/// WHY: the test goes the way the panel goes. The panel builds the
/// [`FilteredIndex`] with the order of the screen, and the index keeps that
/// order. A test that filters the list directly would prove something
/// different from what the user sees.
fn matching(entries: &[Entry], depth: FilterDepth) -> Vec<String> {
    let filter = Filter::from_text("*.txt", FilterKind::All, depth);

    // The order of the screen: the directories first, then the files, each
    // group sorted by name.
    let mut order: Vec<usize> = (0..entries.len()).collect();
    order.sort_by(|left, right| {
        let left_entry = &entries[*left];
        let right_entry = &entries[*right];
        right_entry
            .kind
            .is_directory()
            .cmp(&left_entry.kind.is_directory())
            .then_with(|| left_entry.name.cmp(&right_entry.name))
    });

    let index = FilteredIndex::build(entries, &order, &filter);
    index
        .positions()
        .iter()
        .map(|position| entries[*position].name.clone())
        .collect()
}

#[test]
fn the_filter_searches_deeper() {
    // arrange — a tree of three levels with one text file on every level.
    let tree = match TempTree::new("filter-depth") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory should be created: {error}"),
    };
    tree.write_file("top.txt", b"top");
    tree.write_file("one/middle.txt", b"middle");
    tree.write_file("one/two/deep.txt", b"deep");
    tree.write_file("one/two/other.png", b"not a text file");

    // act — the level 0 reaches the directory that is shown only.
    let entries = entries_of_tree(tree.path(), 3);
    let shallow = matching(&entries, FilterDepth::Recursive { levels: 0 });

    // assert
    assert_eq!(shallow, vec!["top.txt"]);

    // WHY: the entries are sorted by name, like the list of a panel. The
    // order of the tree is therefore not the order of the result.
    //
    // act — the level 1 also reaches the direct subdirectory.
    let middle = matching(&entries, FilterDepth::Recursive { levels: 1 });

    // assert
    assert_eq!(middle, vec!["middle.txt", "top.txt"]);

    // act — the level 2 also reaches the subdirectory of the subdirectory.
    let deep = matching(&entries, FilterDepth::Recursive { levels: 2 });

    // assert
    assert_eq!(deep, vec!["deep.txt", "middle.txt", "top.txt"]);

    // assert — the depth that reaches only the current directory finds the
    // entry of the current directory and nothing else.
    let current = matching(&entries, FilterDepth::Current);

    // assert
    assert_eq!(current, vec!["top.txt"]);
}
