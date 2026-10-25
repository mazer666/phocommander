//! Integration test: the security rules of the configuration file and their
//! effect on the file system.
//!
//! WHAT: the security section of the configuration file is read through
//!       `phocommander_config`, resolved into the rules that are in force
//!       and put to work: the state of the session is written or not written,
//!       and the guarantees that no switch can break are checked.
//! WHY: the security rules are the part of the configuration that a user
//!       trusts. A rule that is only a switch in a file is a promise that
//!       nothing keeps. This test crosses the crate boundary on purpose: it
//!       reads a real file from the disk, resolves it with the real reader
//!       and watches the real file system for the result.
//!
//! EXAMPLE:
//! ```text
//! cargo test -p phocommander-integration-tests --test security_profiles
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use phocommander_config::{
    NoticeSeverity, SecurityProfile, SecurityRuleId, SessionState, load_file, resolve_security,
    write_session_state,
};
use phocommander_testkit::{TempTree, repository_root};

/// The path of the configuration file that ships with the repository.
fn shipped_defaults() -> PathBuf {
    repository_root().join("config").join("defaults.toml")
}

/// Returns every file below the directory, relative to it and sorted.
fn files_below(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    collect(root, root, &mut found);
    found.sort();
    found
}

/// Adds every file below the directory to the list.
fn collect(root: &Path, directory: &Path, found: &mut Vec<String>) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, found);
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        found.push(relative);
    }
}

#[test]
fn the_shipped_file_carries_the_security_section() {
    // arrange
    let path = shipped_defaults();

    // act
    let config = match load_file(&path) {
        Ok(config) => config,
        Err(error) => panic!("the shipped configuration file {path:?} was refused: {error}"),
    };

    // assert
    assert_eq!(config.security.profile, SecurityProfile::Standard);
    assert!(config.security.verify_directory_before_refresh);
    assert!(!config.security.background_index);
    assert!(config.security.remember_session);
    assert!(config.security.read_only_on_demand);
    assert!(config.security.checksum_on_copy);
    assert!(!config.security.follow_symlinks);
    assert!(!config.security.telemetry);
    assert!(!config.security.elevation);
}

#[test]
fn the_shipped_file_produces_the_rules_of_the_profile_standard() {
    // arrange
    let path = shipped_defaults();

    // act
    let config = match load_file(&path) {
        Ok(config) => config,
        Err(error) => panic!("the shipped configuration file {path:?} was refused: {error}"),
    };
    let rules = resolve_security(&config.security);

    // assert
    assert_eq!(rules.profile(), SecurityProfile::Standard);
    assert!(rules.is_enabled(SecurityRuleId::ReadOnlyOnDemand));
    assert!(rules.is_enabled(SecurityRuleId::VerifyDirectoryBeforeRefresh));
    assert!(rules.is_enabled(SecurityRuleId::ChecksumOnCopy));
    assert!(rules.is_enabled(SecurityRuleId::SymlinksAreNeverFollowed));
    assert!(rules.is_enabled(SecurityRuleId::NoTelemetry));
    assert!(rules.is_enabled(SecurityRuleId::NoElevation));
    assert!(rules.is_enabled(SecurityRuleId::NoBackgroundIndex));
    assert!(!rules.is_enabled(SecurityRuleId::LeavesNothingBehind));
    assert!(rules.notices().is_empty());
}

#[test]
fn a_configuration_that_keeps_no_session_writes_no_file_at_all() {
    // WHY: the rule "remember_session" is switched off, so the application
    // leaves nothing behind — neither inside the configuration directory nor
    // outside of it.
    // arrange
    let tree = match TempTree::new("session-off") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let configuration = tree.path().join("configuration");
    let state = SessionState::new("/photos", "/documents");

    let document = "schema_version = 1\n\
                    [security]\n\
                    profile = \"custom\"\n\
                    remember_session = false\n";
    let file = tree.write_file("configuration/config.toml", document.as_bytes());
    let config = match load_file(&file) {
        Ok(config) => config,
        Err(error) => panic!("the configuration file {file:?} was refused: {error}"),
    };
    let rules = resolve_security(&config.security);

    // act
    let written = write_session_state(&configuration, &state, &rules);

    // assert
    match written {
        Ok(None) => {}
        other => panic!("no file should have been written: {other:?}"),
    }
    let found = files_below(tree.path());
    assert_eq!(
        found,
        vec!["configuration/config.toml".to_string()],
        "the application wrote something: {found:?}"
    );
}

#[test]
fn a_configuration_that_remembers_the_session_writes_only_inside_the_configuration_directory() {
    // arrange
    let tree = match TempTree::new("session-on") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let configuration = tree.path().join("configuration");
    let state = SessionState::new("/photos", "/documents");

    let document = "schema_version = 1\n\
                    [security]\n\
                    profile = \"custom\"\n\
                    remember_session = true\n";
    let file = tree.write_file("configuration/config.toml", document.as_bytes());
    let config = match load_file(&file) {
        Ok(config) => config,
        Err(error) => panic!("the configuration file {file:?} was refused: {error}"),
    };
    let rules = resolve_security(&config.security);

    // act
    let written = write_session_state(&configuration, &state, &rules);

    // assert
    let path = match written {
        Ok(Some(path)) => path,
        other => panic!("the state should have been written: {other:?}"),
    };
    assert!(path.starts_with(&configuration), "{path:?}");
    let found = files_below(tree.path());
    assert_eq!(
        found,
        vec![
            "configuration/config.toml".to_string(),
            "configuration/session.toml".to_string(),
        ],
        "the application wrote something else: {found:?}"
    );
}

#[test]
fn a_file_that_tries_to_break_a_guarantee_changes_nothing() {
    // WHY: the three guarantees of NFR-32 cannot be switched off. The file
    // is read, the values are ignored and every attempt is reported.
    // arrange
    let tree = match TempTree::new("guarantees") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let document = "schema_version = 1\n\
                    [security]\n\
                    profile = \"custom\"\n\
                    background_index = true\n\
                    telemetry = true\n\
                    elevation = true\n";
    let file = tree.write_file("configuration/config.toml", document.as_bytes());

    // act
    let config = match load_file(&file) {
        Ok(config) => config,
        Err(error) => panic!("the configuration file {file:?} was refused: {error}"),
    };
    let rules = resolve_security(&config.security);

    // assert
    assert!(!rules.keeps_background_index());
    assert!(!rules.sends_telemetry());
    assert!(!rules.requests_elevation());
    assert!(rules.is_enabled(SecurityRuleId::NoBackgroundIndex));
    assert!(rules.is_enabled(SecurityRuleId::NoTelemetry));
    assert!(rules.is_enabled(SecurityRuleId::NoElevation));

    let warnings: Vec<&str> = rules
        .notices()
        .iter()
        .filter(|notice| notice.severity == NoticeSeverity::Warning)
        .map(|notice| notice.field.as_str())
        .collect();
    assert_eq!(
        warnings,
        vec![
            "security.telemetry",
            "security.elevation",
            "security.background_index"
        ]
    );
}

#[test]
fn the_profile_corporate_enables_every_rule_in_a_real_file() {
    // arrange
    let tree = match TempTree::new("corporate") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let document = "schema_version = 1\n\
                    [security]\n\
                    profile = \"corporate\"\n\
                    remember_session = true\n";
    let file = tree.write_file("configuration/config.toml", document.as_bytes());

    // act
    let config = match load_file(&file) {
        Ok(config) => config,
        Err(error) => panic!("the configuration file {file:?} was refused: {error}"),
    };
    let rules = resolve_security(&config.security);

    // assert
    for rule in rules.rules() {
        assert!(
            rule.is_enabled(),
            "the rule {} is not in force",
            rule.id.as_str()
        );
    }
    assert!(!rules.remembers_session());
    let notice = rules
        .notices()
        .iter()
        .find(|notice| notice.field == "security.remember_session");
    assert!(notice.is_some(), "{:?}", rules.notices());
}

#[test]
fn a_broken_file_leaves_the_application_in_the_safe_mode() {
    // arrange
    let tree = match TempTree::new("broken") {
        Ok(tree) => tree,
        Err(error) => panic!("the temporary directory could not be created: {error}"),
    };
    let file = tree.write_file("configuration/config.toml", b"this is not a configuration");

    // act
    let (config, notices) = phocommander_config::load_file_or_safe(&file);
    let rules = resolve_security(&config.security);

    // assert
    assert_eq!(rules.profile(), SecurityProfile::Corporate);
    for rule in rules.rules() {
        assert!(
            rule.is_enabled(),
            "the rule {} is not in force",
            rule.id.as_str()
        );
    }
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].severity, NoticeSeverity::Warning);
}
