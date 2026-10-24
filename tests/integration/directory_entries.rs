//! Integration test: the directory model of the core crate.
//!
//! **STATUS: waiting for the implementation.** The file system model — the
//! entry structure and the listing of a directory — arrives with the file
//! operations in milestone M1 (`crates/phocommander-ops`, `phocommander-core`).
//! Until it exists, this test cannot be compiled, and a test that cannot be
//! compiled is not a test: it is a piece of text.
//!
//! It is therefore **not** registered as a target of the test package. As soon
//! as the API below exists, one line in `tests/Cargo.toml` makes it part of
//! the gate:
//!
//! ```toml
//! [[test]]
//! name = "directory_entries"
//! path = "integration/directory_entries.rs"
//! ```
//!
//! The expected API, as far as it is fixed today:
//!
//! ```text
//! phocommander_core::fs::Entry
//!     name:        String          — the name of the entry, without the directory
//!     path:        PathBuf         — the full path of the entry
//!     size:        u64             — the size in bytes
//!     is_directory: bool           — whether the entry is a directory
//!
//! phocommander_core::fs::list_directory(path: &Path) -> Result<Vec<Entry>, Error>
//! ```
//!
//! The sort order — by name, case-insensitive, directories first — is the
//! order the specification describes for the file list and the order the
//! benchmark of `benches/directory_listing.rs` measures.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test directory_entries
//! ```

use phocommander_core::fs::{Entry, list_directory};
use phocommander_testkit::TempTree;

#[test]
fn the_listing_holds_every_entry_of_the_directory() {
    // arrange
    let tree = match TempTree::new("listing") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    tree.create_tree(&["beta.txt", "alpha.txt", "gamma/"]);

    // act
    let entries = match list_directory(tree.path()) {
        Ok(entries) => entries,
        Err(error) => panic!("the directory could not be listed: {error}"),
    };

    // assert
    let names: Vec<&str> = entries
        .iter()
        .map(|entry: &Entry| entry.name.as_str())
        .collect();
    assert_eq!(names.len(), 3, "the listing holds {names:?}");
}

#[test]
fn the_entry_carries_the_path_inside_the_listed_directory() {
    // arrange
    let tree = match TempTree::new("listing") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    tree.create_tree(&["alpha.txt"]);

    // act
    let entries = match list_directory(tree.path()) {
        Ok(entries) => entries,
        Err(error) => panic!("the directory could not be listed: {error}"),
    };

    // assert
    for entry in &entries {
        assert!(
            entry.path.starts_with(tree.path()),
            "the entry {:?} points outside of the listed directory",
            entry.name
        );
        assert!(!entry.name.contains('/'), "the name carries a separator");
    }
}

#[test]
fn a_directory_that_does_not_exist_is_an_error_and_not_a_panic() {
    // arrange
    let tree = match TempTree::new("listing-missing") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let missing = tree.path().join("does-not-exist");

    // act
    let result = list_directory(&missing);

    // assert
    assert!(
        result.is_err(),
        "a missing directory was listed without an error"
    );
}
