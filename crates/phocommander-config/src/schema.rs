//! The typed shape of the configuration file.
//!
//! WHAT: one struct per section of {config/defaults.toml}, exactly as the
//!       specification writes it.
//! WHY: the file is the contract between the application and its user. The
//!       structs here are that contract in Rust: a section that is renamed in
//!       the file has to be renamed here, and a value that the file does not
//!       contain is {Option} so that the default from
//!       'phocommander-core::config' can take its place.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::Config;
//!
//! # fn main() -> Result<(), phocommander_config::ConfigError> {
//! let config = phocommander_config::parse_toml("schema_version = 1\n")?;
//! assert!(config.viewer.max_file_size.is_none());
//! # Ok(())
//! # }
//! ```

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::values::{
    FileSizeFormat, InputContext, Language, LogLevel, OverwritePolicy, PanelLayout,
    RightClickAction, SmbEncryption, SmbSigning, Theme,
};

/// The complete configuration of Phocommander.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
/// The version of the schema the file was written for.
///
/// WHY: an incompatible change raises this number. A file with a higher
/// number than the program understands is refused instead of being read
/// with a wrong meaning.
    pub schema_version: u32,

/// The general behaviour of the application.
    #[serde(default)]
    pub general: General,

/// The appearance of the user interface.
    #[serde(default)]
    pub appearance: Appearance,

/// The behaviour of the two panels.
    #[serde(default)]
    pub panels: Panels,

/// Overrides of the platform default key bindings.
    #[serde(default)]
    pub keymap: KeymapOverrides,

/// The limits of the file viewer.
    #[serde(default)]
    pub viewer: Viewer,

/// The limits of the archive adapter.
    #[serde(default)]
    pub archives: Archives,

/// The privacy settings.
    #[serde(default)]
    pub privacy: Privacy,

/// The advanced settings.
    #[serde(default)]
    pub advanced: Advanced,

/// The {[security]} section.
    #[serde(default)]
    pub security: Security,

/// The input contexts and their key bindings.
    #[serde(default)]
    pub input: Input,

/// The behaviour of the network sources.
    #[serde(default)]
    pub remote: Remote,
}

/// The {[general]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct General {
/// The language of the user interface.
    #[serde(default)]
    pub language: Language,

/// Whether the last opened directories are restored at start.
    #[serde(default)]
    pub restore_session: bool,
}

/// The {[appearance]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
/// The colour scheme of the user interface.
    #[serde(default)]
    pub theme: Theme,

/// The height of a row in the file list, in points.
///
/// WHY: zero means that the platform decides the height. A fixed height
/// that is too small would cut off the file names.
    #[serde(default)]
    pub row_height: u32,

/// Whether hidden files are listed.
    #[serde(default)]
    pub show_hidden_files: bool,

/// Whether system files are listed.
    #[serde(default)]
    pub show_system_files: bool,

/// The format in which file sizes are shown.
    #[serde(default)]
    pub file_size_format: FileSizeFormat,
}

/// The {[panels]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Panels {
/// Whether the two panels stand next to each other or below each other.
    #[serde(default)]
    pub layout: PanelLayout,

/// What a right click does.
    #[serde(default)]
    pub right_click_action: RightClickAction,

/// What happens when a file that is to be written already exists.
    #[serde(default)]
    pub overwrite_policy: OverwritePolicy,
}

/// The {[keymap]} section of the configuration file.
///
/// WHY: the section is a plain table of action identifiers because an action
/// identifier is open: a new action may be added without changing this
/// struct. The identifier itself is checked by the keymap crate, which owns
/// the action catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct KeymapOverrides {
/// The binding of an action, by action identifier.
    pub bindings: BTreeMap<String, String>,
}

/// The {[viewer]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Viewer {
/// The largest file the viewer loads into memory, in bytes.
    #[serde(default)]
    pub max_file_size: Option<u64>,

/// The largest file the viewer reads as text, in bytes.
    #[serde(default)]
    pub max_text_size: Option<u64>,
}

/// The {[archives]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Archives {
/// The largest total size an extraction may write, in bytes.
    #[serde(default)]
    pub max_extracted_bytes: Option<u64>,

/// The largest number of entries an archive may contain.
    #[serde(default)]
    pub max_entries: Option<u64>,
}

/// The {[privacy]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Privacy {
/// How much the application writes into its log.
    #[serde(default)]
    pub log_level: LogLevel,
}

/// The {[advanced]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Advanced {
/// The number of worker threads for background operations.
///
/// WHY: zero means that the program uses as many threads as the machine
/// has cores.
    #[serde(default)]
    pub worker_threads: u32,
}

/// The {[security]} section of the configuration file.
///
/// WHY: the security rules of the specification (sections 9.7 and 9.8) are
/// not carved in stone. They are the default, and the user may adjust them
/// to the machine and to the policy in force. Three guarantees are not part
/// of this structure, because they cannot be switched off.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Security {
/// The profile that decides which rules apply.
    #[serde(default)]
    pub profile: SecurityProfile,

/// Read the content of a directory again before it is shown.
    ///
    /// WHY: a change that was made elsewhere is noticed. On a network
    /// drive this costs round trips, which is why it is a switch.
    #[serde(default = "default_true")]
    pub verify_directory_before_refresh: bool,

/// Keep no index.
    ///
    /// WHY: the application never reads a file that the user has not
    /// opened. The field exists so that the value is visible; the
    /// guarantee itself does not depend on it.
    #[serde(default)]
    pub background_index: bool,

/// Write the last directories at the end of a session.
    ///
    /// WHY: switching this off leaves nothing behind on a machine that
    /// is not the user's own.
    #[serde(default = "default_true")]
    pub remember_session: bool,

/// The viewer reads a file only when the user opens it.
    #[serde(default = "default_true")]
    pub read_only_on_demand: bool,

/// Compare the checksums of source and destination in the verified copy.
    #[serde(default = "default_true")]
    pub checksum_on_copy: bool,

/// Copy a symbolic link as a symbolic link and never follow it.
    #[serde(default)]
    pub follow_symlinks: bool,
}

/// The profile that decides which security rules apply.
///
/// WHY: three profiles cover the practical cases without asking the user to
/// understand every switch. "standard" is for a private machine,
/// "corporate" for a managed one, and "custom" for a user who knows the
/// machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecurityProfile {
/// The rules of section 9.7 with the convenience switches.
    #[default]
    Standard,

/// Every rule of section 9.7, for a managed machine.
    Corporate,

/// Exactly what the switches of the section say.
    Custom,
}

/// Returns the value of a switch that is on by default.
fn default_true() -> bool {
    true
}

/// The {[input]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
/// The input context that is active.
    #[serde(default)]
    pub context: InputContext,

/// The bindings of the keyboard context.
    #[serde(default)]
    pub keyboard: InputBindings,

/// The bindings of the pointer context.
    #[serde(default)]
    pub pointer: InputBindings,

/// The bindings of the touch context.
    #[serde(default)]
    pub touch: InputBindings,

/// The bindings of the television remote control context.
    #[serde(default)]
    pub dpad: InputBindings,

/// The bindings of the gamepad context.
    #[serde(default)]
    pub controller: InputBindings,
}

/// The key bindings of one input context.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct InputBindings {
/// The binding of an action, by action identifier.
    pub bindings: BTreeMap<String, String>,
}

/// The {[remote]} section of the configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Remote {
/// The time limit for establishing a connection, in seconds.
    #[serde(default)]
    pub connect_timeout_seconds: Option<u64>,

/// Whether the signature of an SMB server is required.
    #[serde(default)]
    pub smb_signing: SmbSigning,

/// Whether the transport encryption of an SMB server is required.
    #[serde(default)]
    pub smb_encryption: SmbEncryption,

/// The largest file that may be downloaded from a remote source, in bytes.
///
/// WHY: zero means that there is no limit. The default is zero because the
/// limit is only sensible for sources the user trusts.
    #[serde(default)]
    pub max_remote_download_bytes: u64,
}
