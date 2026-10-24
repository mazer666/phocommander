//! Reading and validating the configuration file of Phocommander.
//!
//! WHAT: the typed shape of `config/defaults.toml` ([`schema`]) and the
//!       functions that read it ([`loader`]).
//! WHY: the configuration file is the place where a user changes the
//!       behaviour of the application and where a contributor looks first. A
//!       typo in it must produce a clear message instead of a silently
//!       ignored setting, so unknown fields are rejected. The canonical
//!       *defaults* live in `phocommander-core::config`; this crate may not
//!       depend on that crate — the workspace is flat — therefore the values
//!       that have a default are optional here and the composition root
//!       (`apps/phocommander-desktop`) supplies the default.
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
pub mod schema;
pub mod values;

pub use loader::{load_file, parse_toml, ConfigError, SUPPORTED_SCHEMA_VERSION};
pub use schema::{
    Advanced, Appearance, Archives, Config, General, Input, InputBindings, KeymapOverrides,
    Panels, Privacy, Remote, Viewer,
};
pub use values::{
    FileSizeFormat, InputContext, Language, LogLevel, OverwritePolicy, PanelLayout,
    RightClickAction, SmbEncryption, SmbSigning, Theme,
};
