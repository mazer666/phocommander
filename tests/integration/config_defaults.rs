//! Integration test: the shipped configuration file and the configuration reader.
//!
//! WHAT: the file `config/defaults.toml` that ships with the repository is
//!       read through `phocommander_config`, and the values of the file are
//!       compared with the values the specification names.
//! WHY:  the configuration file is the contract between the application and its
//!       user, and it is the file a user edits first. A file that the shipped
//!       reader refuses would break every start of the application; a file
//!       whose values differ from the specification would change the behaviour
//!       of the product silently. This test crosses the crate boundary on
//!       purpose: it reads the real file from the repository with the real
//!       reader of the real crate.
//!
//!       The second part of the file covers the third example of the test
//!       strategy of the specification: "Laden einer Konfigurationsdatei mit
//!       unbekanntem Feld" — a configuration file with an unknown field must
//!       be refused, with the name of the field in the message.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test config_defaults
//! ```

use std::fs;

use phocommander_config::{
    ConfigError, FileSizeFormat, InputContext, Language, LogLevel, OverwritePolicy, PanelLayout,
    RightClickAction, SmbEncryption, SmbSigning, Theme, load_file, parse_toml,
};
use phocommander_testkit::{TempTree, repository_root};

/// The path of the configuration file that ships with the repository.
fn shipped_defaults() -> std::path::PathBuf {
    repository_root().join("config").join("defaults.toml")
}

#[test]
fn the_shipped_configuration_file_is_accepted() {
    // arrange
    let path = shipped_defaults();

    // act
    let result = load_file(&path);

    // assert
    match result {
        Ok(_) => {}
        Err(error) => panic!("the shipped configuration file {path:?} was refused: {error}"),
    }
}

#[test]
fn the_shipped_configuration_file_holds_the_values_of_the_specification() {
    // arrange
    let path = shipped_defaults();

    // act
    let config = match load_file(&path) {
        Ok(config) => config,
        Err(error) => panic!("the shipped configuration file {path:?} was refused: {error}"),
    };

    // assert — general
    assert_eq!(config.schema_version, 1);
    assert_eq!(config.general.language, Language::System);
    assert!(config.general.restore_session);

    // assert — appearance
    assert_eq!(config.appearance.theme, Theme::System);
    assert_eq!(config.appearance.row_height, 0);
    assert!(!config.appearance.show_hidden_files);
    assert!(!config.appearance.show_system_files);
    assert_eq!(config.appearance.file_size_format, FileSizeFormat::Binary);

    // assert — panels. Overwriting is never the default, that is a product
    // decision of the specification, not a default that may drift.
    assert_eq!(config.panels.layout, PanelLayout::Horizontal);
    assert_eq!(
        config.panels.right_click_action,
        RightClickAction::ContextMenu
    );
    assert_eq!(config.panels.overwrite_policy, OverwritePolicy::Ask);

    // assert — keymap overrides
    assert_eq!(
        config.keymap.bindings.get("panel.copy"),
        Some(&"F5".to_string())
    );
    assert_eq!(
        config.keymap.bindings.get("panel.move"),
        Some(&"F6".to_string())
    );

    // assert — viewer
    assert_eq!(config.viewer.max_file_size, Some(10_485_760));
    assert_eq!(config.viewer.max_text_size, Some(4_194_304));

    // assert — archives
    assert_eq!(config.archives.max_extracted_bytes, Some(2_147_483_648));
    assert_eq!(config.archives.max_entries, Some(100_000));

    // assert — privacy and advanced
    assert_eq!(config.privacy.log_level, LogLevel::Warn);
    assert_eq!(config.advanced.worker_threads, 0);

    // assert — input
    assert_eq!(config.input.context, InputContext::Auto);

    // assert — remote
    assert_eq!(config.remote.connect_timeout_seconds, Some(15));
    assert_eq!(config.remote.smb_signing, SmbSigning::Prefer);
    assert_eq!(config.remote.smb_encryption, SmbEncryption::Prefer);
    assert_eq!(config.remote.max_remote_download_bytes, 0);
}

#[test]
fn a_configuration_file_with_an_unknown_field_is_refused() {
    // arrange — the misspelled field of the specification example
    let source = "schema_version = 1\n[general]\nlangauge = \"de\"\n";

    // act
    let result = parse_toml(source);

    // assert — the error names the field, otherwise the user cannot fix it
    let error = match result {
        Ok(_) => panic!("the misspelled field was accepted"),
        Err(error) => error,
    };
    assert!(
        matches!(error, ConfigError::Syntax { .. }),
        "the unknown field did not produce a syntax error: {error:?}"
    );
    let message = error.to_string();
    assert!(message.contains("langauge"), "{message}");
}

#[test]
fn a_configuration_file_that_cannot_be_read_reports_the_path() {
    // arrange
    let tree = match TempTree::new("config-missing") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let missing = tree.path().join("does-not-exist.toml");

    // act
    let result = load_file(&missing);

    // assert
    let error = match result {
        Ok(_) => panic!("a missing file was read"),
        Err(error) => error,
    };
    assert!(matches!(error, ConfigError::Io { .. }), "{error:?}");
    let message = error.to_string();
    assert!(message.contains("does-not-exist.toml"), "{message}");
}

#[test]
fn a_configuration_file_written_by_the_user_is_read_from_the_disk() {
    // arrange — a file in a temporary directory, written by the test
    let tree = match TempTree::new("config-user") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let path = tree.path().join("phocommander.toml");
    let source = "schema_version = 1\n[panels]\noverwrite_policy = \"never\"\n";
    if let Err(error) = fs::write(&path, source) {
        panic!("the configuration file {path:?} could not be written: {error}");
    }

    // act
    let result = load_file(&path);

    // assert
    let config = match result {
        Ok(config) => config,
        Err(error) => panic!("the configuration file {path:?} was refused: {error}"),
    };
    assert_eq!(config.panels.overwrite_policy, OverwritePolicy::Never);
}
