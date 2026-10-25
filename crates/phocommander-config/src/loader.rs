//! Reading the configuration file of Phocommander.
//!
//! WHAT: the functions that turn a TOML document into a [{Config}] and the
//!       errors they report.
//! WHY: a configuration file is written by hand. Every mistake in it has to
//!       be reported with the name of the field and the reason, because the
//!       user sees nothing else. The file is read completely before it is
//!       used, so a broken file can never lead to a half-configured program.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::{load_file, ConfigError};
//!
//! # fn main() -> Result<(), ConfigError> {
//! let config = load_file("config/defaults.toml")?;
//! println!("schema version {}", config.schema_version);
//! # Ok(())
//! # }
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::notice::ConfigNotice;
use crate::schema::Config;

/// The newest version of the configuration schema this build understands.
///
/// WHY: a file that declares a higher version was written by a newer program.
/// Reading it could give the settings a wrong meaning, so the file is
/// refused and the user is told which version is needed.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// The reasons why a configuration file cannot be used.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file could not be read.
    #[error("the configuration file {path} could not be read: {source}")]
    Io {
        /// The path of the configuration file.
        path: PathBuf,
        /// The error the operating system reported.
        #[source]
        source: io::Error,
    },

    /// The file is not valid TOML or contains a field that does not exist.
    #[error("the configuration file is not valid: {source}")]
    Syntax {
        /// The error the TOML parser reported.
        #[source]
        source: toml::de::Error,
    },

    /// The file was written for a newer version of the schema.
    #[error(
        "the configuration file uses schema version {found}, \
         this build understands version {supported}"
    )]
    UnsupportedSchemaVersion {
        /// The version the file declares.
        found: u32,
        /// The newest version this build understands.
        supported: u32,
    },

    /// A value in the file cannot be used.
    #[error("the value of {field} cannot be used: {reason}")]
    InvalidValue {
        /// The name of the field, written as it is in the file.
        field: &'static str,
        /// The reason why the value was refused.
        reason: &'static str,
    },
}

/// Reads the configuration from a document in TOML.
///
/// WHY: the document is parsed into the typed schema first and checked
/// afterwards. A file with a valid but unusable value — a limit of zero, for
/// example — is refused with the name of the field instead of causing a
/// strange behaviour later.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::parse_toml;
///
/// # fn main() -> Result<(), phocommander_config::ConfigError> {
/// let config = parse_toml("schema_version = 1\n[panels]\noverwrite_policy = \"never\"\n")?;
/// assert_eq!(config.panels.overwrite_policy, phocommander_config::OverwritePolicy::Never);
/// # Ok(())
/// # }
/// ```
pub fn parse_toml(source: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(source).map_err(|source| ConfigError::Syntax { source })?;
    validate(&config)?;
    Ok(config)
}

/// Reads the configuration from a file.
///
/// WHY: the file is read completely and parsed completely before any part of
/// it is used. A file that cannot be read therefore never leads to a program
/// that runs with half of the old and half of the new settings.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::load_file;
///
/// # fn main() -> Result<(), phocommander_config::ConfigError> {
/// let config = load_file("config/defaults.toml")?;
/// println!("schema version {}", config.schema_version);
/// # Ok(())
/// # }
/// ```
pub fn load_file(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_toml(&source)
}

/// Reads the configuration and falls back to the safe configuration.
///
/// WHY: a configuration file is written by hand, and a broken file must not
/// stop the application — and it must not weaken it either. The function
/// therefore returns the configuration that is in force together with the
/// messages that say what happened. The safe configuration is the profile
/// `corporate`, in which every rule of section 9.7 is in force.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::{ConfigNotice, parse_toml_or_safe};
///
/// let (config, notices) = parse_toml_or_safe("this is not a configuration");
/// assert!(!notices.is_empty());
/// assert_eq!(notices[0].severity, phocommander_config::NoticeSeverity::Warning);
/// # let _ = config;
/// ```
pub fn parse_toml_or_safe(source: &str) -> (Config, Vec<ConfigNotice>) {
    match parse_toml(source) {
        Ok(config) => (config, Vec::new()),
        Err(error) => (
            Config::safe(),
            vec![ConfigNotice::warning(
                "the configuration file",
                format!("the file could not be used ({error}); the safe configuration is in force"),
            )],
        ),
    }
}

/// Reads the configuration file and falls back to the safe configuration.
///
/// WHY: the same rule as for a document that cannot be parsed: a file that
/// cannot be read must not weaken the application. The path is named in the
/// message, so that the user knows which file was refused.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::load_file_or_safe;
///
/// let (config, notices) = load_file_or_safe("/does/not/exist.toml");
/// assert!(!notices.is_empty());
/// # let _ = config;
/// ```
pub fn load_file_or_safe(path: impl AsRef<Path>) -> (Config, Vec<ConfigNotice>) {
    let path = path.as_ref();
    match load_file(path) {
        Ok(config) => (config, Vec::new()),
        Err(error) => (
            Config::safe(),
            vec![ConfigNotice::warning(
                path.display().to_string(),
                format!(
                    "the configuration file could not be used ({error}); \
                     the safe configuration is in force"
                ),
            )],
        ),
    }
}

/// Checks the values that the parser cannot check itself.
///
/// WHY: a syntactically correct file can still contain a value that makes the
/// program useless — a limit of zero would refuse every file, a timeout of
/// zero would abort every connection. Those values are refused here, with the
/// name of the field.
fn validate(config: &Config) -> Result<(), ConfigError> {
    if config.schema_version > SUPPORTED_SCHEMA_VERSION {
        return Err(ConfigError::UnsupportedSchemaVersion {
            found: config.schema_version,
            supported: SUPPORTED_SCHEMA_VERSION,
        });
    }

    if config.archives.max_entries == Some(0) {
        return Err(ConfigError::InvalidValue {
            field: "archives.max_entries",
            reason: "the limit must be greater than zero",
        });
    }

    if config.archives.max_extracted_bytes == Some(0) {
        return Err(ConfigError::InvalidValue {
            field: "archives.max_extracted_bytes",
            reason: "the limit must be greater than zero",
        });
    }

    if config.remote.connect_timeout_seconds == Some(0) {
        return Err(ConfigError::InvalidValue {
            field: "remote.connect_timeout_seconds",
            reason: "the time limit must be greater than zero",
        });
    }

    Ok(())
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::values::{InputContext, Language, OverwritePolicy, Theme};

    /// Returns the configuration of a document that has to be valid.
    ///
    /// WHY: the workspace forbids {expect} and {unwrap}, also in tests. A
    /// panic with the reason of the failure says more than the exit code of a
    /// test.
    fn valid(result: Result<Config, ConfigError>) -> Config {
        match result {
            Ok(config) => config,
            Err(error) => panic!("the document should be valid: {error}"),
        }
    }

    #[test]
    fn reads_an_empty_document_with_the_default_values() {
        // arrange
        let source = "schema_version = 1\n";

        // act
        let config = valid(parse_toml(source));

        // assert
        assert_eq!(config.schema_version, 1);
        assert_eq!(config.general.language, Language::System);
        assert_eq!(config.appearance.theme, Theme::System);
        assert_eq!(config.panels.overwrite_policy, OverwritePolicy::Ask);
        assert_eq!(config.input.context, InputContext::Auto);
    }

    #[test]
    fn reads_every_section_of_the_shipped_file() {
        // arrange
        let source = r#"
            schema_version = 1

            [general]
            language = "de"
            restore_session = false

            [appearance]
            theme = "dark"
            row_height = 24
            show_hidden_files = true
            show_system_files = false
            file_size_format = "iec"

            [panels]
            layout = "vertical"
            right_click_action = "toggle_selection"
            overwrite_policy = "never"

            [keymap]
            "panel.copy" = "F5"

            [viewer]
            max_file_size = 10485760
            max_text_size = 4194304

            [archives]
            max_extracted_bytes = 2147483648
            max_entries = 100000

            [privacy]
            log_level = "info"

            [advanced]
            worker_threads = 4

            [input]
            context = "dpad"

            [input.keyboard]
            "panel.copy" = "F5"

            [input.dpad]
            "panel.copy" = "palette:file.copy"

            [remote]
            connect_timeout_seconds = 15
            smb_signing = "require"
            smb_encryption = "require"
            max_remote_download_bytes = 0
        "#;

        // act
        let config = valid(parse_toml(source));

        // assert
        assert_eq!(config.general.language, Language::De);
        assert!(!config.general.restore_session);
        assert_eq!(config.appearance.theme, Theme::Dark);
        assert_eq!(config.appearance.row_height, 24);
        assert!(config.appearance.show_hidden_files);
        assert_eq!(
            config.appearance.file_size_format,
            crate::FileSizeFormat::Iec
        );
        assert_eq!(config.panels.layout, crate::PanelLayout::Vertical);
        assert_eq!(
            config.panels.right_click_action,
            crate::RightClickAction::ToggleSelection
        );
        assert_eq!(config.panels.overwrite_policy, OverwritePolicy::Never);
        assert_eq!(
            config.keymap.bindings.get("panel.copy"),
            Some(&"F5".to_string())
        );
        assert_eq!(config.viewer.max_file_size, Some(10_485_760));
        assert_eq!(config.viewer.max_text_size, Some(4_194_304));
        assert_eq!(config.archives.max_extracted_bytes, Some(2_147_483_648));
        assert_eq!(config.archives.max_entries, Some(100_000));
        assert_eq!(config.privacy.log_level, crate::LogLevel::Info);
        assert_eq!(config.advanced.worker_threads, 4);
        assert_eq!(config.input.context, InputContext::Dpad);
        assert_eq!(
            config.input.dpad.bindings.get("panel.copy"),
            Some(&"palette:file.copy".to_string())
        );
        assert_eq!(config.remote.connect_timeout_seconds, Some(15));
        assert_eq!(config.remote.smb_signing, crate::SmbSigning::Require);
        assert_eq!(config.remote.smb_encryption, crate::SmbEncryption::Require);
        assert_eq!(config.remote.max_remote_download_bytes, 0);
    }

    #[test]
    fn refuses_an_unknown_field() {
        // arrange
        let source = "schema_version = 1\n[general]\nlangauge = \"de\"\n";

        // act
        let result = parse_toml(source);

        // assert
        let error = match result {
            Ok(_) => panic!("the misspelled field was accepted"),
            Err(error) => error,
        };
        let message = error.to_string();
        assert!(message.contains("langauge"), "{message}");
    }

    #[test]
    fn refuses_an_unknown_section() {
        // arrange
        let source = "schema_version = 1\n[generl]\nlanguage = \"de\"\n";

        // act
        let result = parse_toml(source);

        // assert
        assert!(result.is_err(), "the unknown section was accepted");
    }

    #[test]
    fn refuses_a_schema_version_from_the_future() {
        // arrange
        let source = "schema_version = 2\n";

        // act
        let result = parse_toml(source);

        // assert
        let error = match result {
            Ok(_) => panic!("the newer schema version was accepted"),
            Err(error) => error,
        };
        assert!(
            matches!(
                error,
                ConfigError::UnsupportedSchemaVersion {
                    found: 2,
                    supported: 1
                }
            ),
            "{error:?}"
        );
    }

    #[test]
    fn refuses_a_limit_of_zero() {
        // arrange
        let source = "schema_version = 1\n[archives]\nmax_entries = 0\n";

        // act
        let result = parse_toml(source);

        // assert
        let error = match result {
            Ok(_) => panic!("the limit of zero was accepted"),
            Err(error) => error,
        };
        let message = error.to_string();
        assert!(message.contains("archives.max_entries"), "{message}");
    }

    #[test]
    fn refuses_a_connection_timeout_of_zero() {
        // arrange
        let source = "schema_version = 1\n[remote]\nconnect_timeout_seconds = 0\n";

        // act
        let result = parse_toml(source);

        // assert
        let error = match result {
            Ok(_) => panic!("the timeout of zero was accepted"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("remote.connect_timeout_seconds"),
            "{error}"
        );
    }

    #[test]
    fn a_broken_document_gives_the_safe_configuration() {
        // WHY: a broken file must not weaken the application. The safe
        // configuration is the one in which every rule of section 9.7 is in
        // force.
        // arrange
        let source = "this is not a configuration";

        // act
        let (config, notices) = parse_toml_or_safe(source);

        // assert
        assert_eq!(
            config.security.profile,
            crate::schema::SecurityProfile::Corporate
        );
        assert!(!config.security.remember_session);
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].severity, crate::notice::NoticeSeverity::Warning);

        let rules = crate::security::resolve_security(&config.security);
        for rule in rules.rules() {
            assert!(
                rule.is_enabled(),
                "the rule {} is not in force",
                rule.id.as_str()
            );
        }
    }

    #[test]
    fn a_file_that_cannot_be_read_gives_the_safe_configuration() {
        // arrange
        let path = std::env::temp_dir().join("phocommander-does-not-exist.toml");

        // act
        let (config, notices) = load_file_or_safe(&path);

        // assert
        assert_eq!(
            config.security.profile,
            crate::schema::SecurityProfile::Corporate
        );
        assert_eq!(notices.len(), 1);
        assert!(notices[0].message.contains("safe configuration"));
    }

    #[test]
    fn a_document_that_can_be_used_gives_no_notice() {
        // act
        let (config, notices) = parse_toml_or_safe("schema_version = 1\n");

        // assert
        assert_eq!(config.schema_version, 1);
        assert!(notices.is_empty());
    }
}
