//! Integration test: the configuration file that ships with the repository and
//! the schema of the configuration crate.
//!
//! WHAT: the file {config/defaults.toml} is read with the reader of the
//!       crate and compared with the schema.
//! WHY: the shipped file is the first file a user copies and the first file
//!       a contributor reads. If the reader refuses it — because it knows a
//!       field that the schema does not have, or because a value is not one
//!       of the allowed ones — every start of the application breaks before
//!       it shows anything. The file and the schema are written by different
//!       people, so the agreement between them has to be checked by a
//!       machine.
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-config --test shipped_defaults
//! ```

use phocommander_config::{Config, parse_toml};

/// The configuration file that ships with the repository.
///
/// WHY: the file is read at compile time, so that the test cannot be
/// skipped by accident because the working directory was different.
const SHIPPED_DEFAULTS: &str = include_str!("../../../config/defaults.toml");

#[test]
fn the_shipped_configuration_file_is_accepted() {
    // act
    let result = parse_toml(SHIPPED_DEFAULTS);

    // assert
    match result {
        Ok(config) => assert_eq!(config.schema_version, 1),
        Err(error) => panic!("the shipped configuration file was refused: {error}"),
    }
}

#[test]
fn the_shipped_configuration_file_holds_the_specified_values() {
    // arrange
    let config = match parse_toml(SHIPPED_DEFAULTS) {
        Ok(config) => config,
        Err(error) => panic!("the shipped configuration file was refused: {error}"),
    };

    // assert
    assert!(config.general.restore_session);
    assert_eq!(config.panels.layout.as_str(), "horizontal");
    assert_eq!(config.panels.overwrite_policy.as_str(), "ask");
    assert_eq!(config.appearance.file_size_format.as_str(), "binary");
    assert_eq!(config.privacy.log_level.as_str(), "warn");
    assert_eq!(config.input.context.as_str(), "auto");
}

#[test]
fn the_shipped_configuration_file_carries_the_viewer_and_archive_limits() {
    // arrange
    let config: Config = match parse_toml(SHIPPED_DEFAULTS) {
        Ok(config) => config,
        Err(error) => panic!("the shipped configuration file was refused: {error}"),
    };

    // assert
    assert_eq!(config.viewer.max_file_size, Some(10_485_760));
    assert_eq!(config.viewer.max_text_size, Some(4_194_304));
    assert_eq!(config.archives.max_extracted_bytes, Some(2_147_483_648));
    assert_eq!(config.archives.max_entries, Some(100_000));
}
