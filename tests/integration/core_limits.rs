//! Integration test: the canonical limits of the core crate and the shipped
//! configuration file.
//!
//! WHAT: the limits that `phocommander_core` publishes as constants are
//!       compared with the values the shipped configuration file carries for
//!       the same limits.
//! WHY:  the specification demands that every limit is written down exactly
//!       once. The core crate owns the canonical value, the configuration file
//!       lets the user change it — and the file that ships with the
//!       repository must carry exactly the canonical value. If the two drift
//!       apart, the user gets a limit that the code never uses.
//!
//!       This test crosses the crate boundary on purpose: the core crate
//!       cannot see the configuration crate (the workspace is flat), so only
//!       a test can hold both sides against each other.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test core_limits
//! ```

use phocommander_config::load_file;
use phocommander_core::{
    MAX_ARCHIVE_ENTRIES, MAX_EXTRACTED_BYTES, MAX_VIEWER_FILE_SIZE, MAX_VIEWER_TEXT_SIZE,
};
use phocommander_testkit::repository_root;

#[test]
fn the_shipped_configuration_file_carries_the_canonical_archive_limits() {
    // arrange
    let path = repository_root().join("config").join("defaults.toml");

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
    let path = repository_root().join("config").join("defaults.toml");

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
fn the_canonical_limits_leave_room_for_a_normal_workload() {
    // WHY: a limit that is too low turns a protection into an outage. The
    // values below are the ones the specification names in the table of the
    // performance requirements; the test names them so that a change of the
    // constants has to be a decision, not an accident.
    assert_eq!(MAX_EXTRACTED_BYTES, 2_147_483_648);
    assert_eq!(MAX_ARCHIVE_ENTRIES, 100_000);
    assert_eq!(MAX_VIEWER_FILE_SIZE, 10_485_760);
    assert_eq!(MAX_VIEWER_TEXT_SIZE, 4_194_304);
}
