//! Reading and validating the configuration file of Phocommander.
//!
//! WHAT: the typed shape of {config/defaults.toml} ([{schema}]), the
//!       functions that read it ([{loader}]), the security rules that are in
//!       force ([{security}] and [{rule}]) and the messages about a setting
//!       that had no effect ([{notice}]).
//! WHY: the configuration file is the place where a user changes the
//!       behaviour of the application and where a contributor looks first. A
//!       typo in it must produce a clear message instead of a silently
//!       ignored setting, so unknown fields are rejected. The canonical
//!       *defaults* live in 'phocommander-core::config'; this crate may not
//!       depend on that crate — the workspace is flat — therefore the values
//!       that have a default are optional here and the composition root
//!       supplies the default.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::Config;
//!
//! # fn main() -> Result<(), phocommander_config::ConfigError> {
//! let source = "schema_version = 1\n[general]\nlanguage = \"de\"\n";
//! let config = phocommander_config::parse_toml(source)?;
//! assert_eq!(config.schema_version, 1);
//! # Ok(())
//! # }
//! ```

pub mod loader;
pub mod notice;
pub mod rule;
pub mod schema;
pub mod security;
pub mod session;
pub mod values;

pub use loader::{
    ConfigError, SUPPORTED_SCHEMA_VERSION, load_file, load_file_or_safe, parse_toml,
    parse_toml_or_safe,
};
pub use notice::{ConfigNotice, NoticeSeverity};
pub use rule::{SecurityRule, SecurityRuleId, SecurityRuleSource, SecurityRuleState};
pub use schema::{
    Advanced, Appearance, Archives, Config, General, Input, InputBindings, KeymapOverrides, Panels,
    Privacy, Remote, Security, SecurityProfile, Viewer,
};
pub use security::{EffectiveSecurity, resolve_security};
pub use session::{
    SESSION_FILE_NAME, SessionError, SessionState, load_session_state, write_session_state,
};
pub use values::{
    FileSizeFormat, InputContext, Language, LogLevel, OverwritePolicy, PanelLayout,
    RightClickAction, SmbEncryption, SmbSigning, Theme,
};
