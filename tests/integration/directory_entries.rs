//! The entries of a listed directory.
//!
//! WHAT: the test of the listing of a directory — that it holds every entry,
//!       that the name of an entry carries no path and that a directory that
//!       does not exist is an error and not a panic.
//! WHY: the listing is the first thing the user sees, and a listing that
//!       loses an entry or that panics on a missing directory is a fault that
//!       the user notices at once.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test directory_entries
//! ```

use std::path::Path;

use phocommander_core::source::Source;
use phocommander_core::{LocalSource, entry::Entry};
use phocommander_testkit::TempTree;

/// The source that lists the directory of the tree.
fn source_of(tree: &TempTree) -> LocalSource {
    LocalSource::rooted(tree.path())
}

/// The entries of the directory, or a panic with the reason.
fn list(tree: &TempTree) -> Vec<Entry> {
    let source = source_of(tree);
    match source.list(Path::new("")) {
        Ok(entries) => entries,
        Err(error) => panic!("the directory could not be listed: {error}"),
    }
}

#[test]
fn the_listing_holds_every_entry_of_the_directory() {
    // arrange
    let tree = match TempTree::new("listing") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    tree.create_tree(&["beta.txt", "alpha.txt"]);
    // WHY: the testkit writes files. A directory is therefore created here,
    //       so that the listing holds a file and a directory at the same
    //       time.
    std::fs::create_dir(tree.path().join("gamma"))
        .unwrap_or_else(|error| panic!("the directory could not be created: {error}"));

    // act
    let entries = list(&tree);

    // assert
    let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
    assert_eq!(names.len(), 3, "the listing holds {names:?}");
    assert!(names.contains(&"alpha.txt"), "the listing holds {names:?}");
    assert!(names.contains(&"beta.txt"), "the listing holds {names:?}");
    assert!(names.contains(&"gamma"), "the listing holds {names:?}");
}

#[test]
fn the_name_of_an_entry_carries_no_path() {
    // arrange
    let tree = match TempTree::new("listing-names") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    tree.create_tree(&["alpha.txt"]);

    // act
    let entries = list(&tree);

    // assert
    // WHY: the name of an entry is what the list shows. A name that carried
    //       a path would make the list unreadable, and a source that is
    //       remote has no path to offer.
    for entry in &entries {
        assert!(
            !entry.name.contains('/'),
            "the name {:?} carries a separator",
            entry.name
        );
        assert!(
            !entry.name.contains('\\'),
            "the name {:?} carries a separator",
            entry.name
        );
    }
}

#[test]
fn a_directory_that_does_not_exist_is_an_error_and_not_a_panic() {
    // arrange
    let tree = match TempTree::new("listing-missing") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let source = LocalSource::rooted(tree.path().join("does-not-exist"));

    // act
    let result = source.list(Path::new(""));

    // assert
    assert!(
        result.is_err(),
        "a missing directory was listed without an error"
    );
}
