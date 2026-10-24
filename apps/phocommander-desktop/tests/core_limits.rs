//! Integration test: the canonical limits of the core crate and the shipped
//! configuration file.
//!
//! WHAT: the limits that `phocommander_core` publishes as constants are
//!       compared with the values that the shipped configuration file carries
//!       for the same limits.
//! WHY: the specification demands that every limit is written down exactly
//!       once. The core crate owns the canonical value, the configuration
//!       file lets the user change it — and the file that ships with the
//!       repository must carry exactly the canonical value. If the two drift
//!       apart, the user gets a limit that the code never uses.
//!
//!       This test crosses the crate boundary on purpose: the core crate
//!       cannot see the configuration crate — the workspace is flat, no
//!       crate below `crates/` may depend on a sibling — so only the
//!       composition root, the application, can hold both sides against each
//!       other. That is the role the application has in this workspace.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-desktop --test core_limits
//! ```

use std::path::{Path, PathBuf};

use phocommander_config::load_file;
use phocommander_core::{
    MAX_ARCHIVE_ENTRIES, MAX_EXTRACTED_BYTES, MAX_VIEWER_FILE_SIZE, MAX_VIEWER_TEXT_SIZE,
};

/// The directory of the repository.
///
/// WHY: a test binary is started with the directory of its package as the
/// working directory, not with the root of the repository. The path of the
/// repository is therefore taken from the compile time constant of cargo, so
/// that the test does not depend on the working directory.
fn repository_root() -> PathBuf {
    let package_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match package_directory.parent().and_then(Path::parent) {
        Some(root) => root.to_path_buf(),
        None => package_directory,
    }
}

/// The path of the configuration file that ships with the repository.
fn shipped_defaults() -> PathBuf {
    repository_root().join("config").join("defaults.toml")
}

#[test]
fn the_shipped_configuration_file_carries_the_canonical_archive_limits() {
    // arrange
    let path = shipped_defaults();

    // act
    let config = match load_file(&path) {
        Ok(config) => config,
        Err(error) => panic!("the shipped configuration file {path:?} was refused: {error}"),
    };

    // assert
    assert_eq!(
        config.archives.max_extracted_bytes,
        Some(MAX_EXTRACTED_BYTES),
        "the archive byte limit of the configuration file differs from the canonical limit"
    );
    assert_eq!(
        config.archives.max_entries,
        Some(MAX_ARCHIVE_ENTRIES),
        "the entry limit of the configuration file differs from the canonical limit"
    );
}

#[test]
fn the_shipped_configuration_file_carries_the_canonical_viewer_limits() {
    // arrange
    let path = shipped_defaults();

    // act
    let config = match load_file(&path) {
        Ok(config) => config,
        Err(error) => panic!("the shipped configuration file {path:?} was refused: {error}"),
    };

    // assert
    assert_eq!(
        config.viewer.max_file_size,
        Some(MAX_VIEWER_FILE_SIZE),
        "the viewer file limit of the configuration file differs from the canonical limit"
    );
    assert_eq!(
        config.viewer.max_text_size,
        Some(MAX_VIEWER_TEXT_SIZE),
        "the viewer text limit of the configuration file differs from the canonical limit"
    );
}

#[test]
fn the_shipped_configuration_file_is_read_by_the_reader_of_the_crate() {
    // WHY: the reader of the configuration crate has to accept the file that
    // ships with the repository. A file that is refused breaks every start of
    // the application before it shows anything.
    // arrange
    let path = shipped_defaults();

    // act
    let result = load_file(&path);

    // assert
    match result {
        Ok(config) => assert_eq!(config.schema_version, 1),
        Err(error) => panic!("the shipped configuration file {path:?} was refused: {error}"),
    }
}
