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
    CopyMode, FileSizeFormat, InputContext, Language, LogLevel, OverwritePolicy, PanelLayout,
    RightClickAction, SmbEncryption, SmbSigning, SyncMode, Theme,
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

    /// The ways of copying and of synchronising.
    #[serde(default)]
    pub copy: Copy,

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

impl Config {
    /// The configuration that is in force when the file cannot be used.
    ///
    /// WHY: a broken configuration file must not weaken the application. The
    /// safe configuration is the one of the profile `corporate`, in which
    /// every rule of section 9.7 is in force. The limits of the other
    /// sections are the ones of the core crate, which the composition root
    /// supplies when the file does not name a value.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::{Config, SecurityProfile};
    ///
    /// let safe = Config::safe();
    /// assert_eq!(safe.security.profile, SecurityProfile::Corporate);
    /// assert!(!safe.security.remember_session);
    /// ```
    pub fn safe() -> Self {
        Self {
            security: Security::safe(),
            ..Self::default()
        }
    }
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

/// The {[copy]} section of the configuration file.
///
/// WHY: the way a file is copied and the way a directory is synchronised
///       are settings of the user, and they stand in the file like every
///       other setting (FR-22b).
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Copy {
    /// The way a file is copied.
    #[serde(default)]
    pub mode: CopyMode,

    /// The way a directory is synchronised.
    #[serde(default)]
    pub sync_mode: SyncMode,
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
/// of the profile and of the switches, because they cannot be switched off:
/// the fields [`Security::telemetry`] and [`Security::elevation`] exist so
/// that a file which carries them can be read at all — their value has no
/// effect and produces a message in the log — and the guarantee of the
/// background index is decided by the code.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Security {
    /// The profile that decides which rules apply.
    #[serde(default)]
    pub profile: SecurityProfile,

    /// Read the content of a directory again before it is shown.
    ///
    /// WHY: a change that was made elsewhere is noticed. On a network
    /// drive this costs round trips, which is why it is a switch.
    #[serde(default = "default_verify_directory_before_refresh")]
    pub verify_directory_before_refresh: bool,

    /// Keep no index.
    ///
    /// WHY: the application never reads a file that the user has not
    /// opened. The field exists so that the value is visible; the
    /// guarantee itself does not depend on it. A file that sets the value
    /// to `true` is read, the value is ignored and a message is written to
    /// the log.
    #[serde(default = "default_background_index")]
    pub background_index: bool,

    /// Write the last directories at the end of a session.
    ///
    /// WHY: switching this off leaves nothing behind on a machine that
    /// is not the user's own.
    #[serde(default = "default_remember_session")]
    pub remember_session: bool,

    /// The viewer reads a file only when the user opens it.
    #[serde(default = "default_read_only_on_demand")]
    pub read_only_on_demand: bool,

    /// Compare the checksums of source and destination in the verified copy.
    #[serde(default = "default_checksum_on_copy")]
    pub checksum_on_copy: bool,

    /// Follow a symbolic link during a copy.
    ///
    /// WHY: the switch names the deviation from the rule of the
    /// specification — "a symbolic link is copied as a symbolic link and is
    /// never dereferenced" — and not the rule itself. The safe value is
    /// therefore `false`, and the rule is in force exactly when the switch
    /// is off.
    #[serde(default = "default_follow_symlinks")]
    pub follow_symlinks: bool,

    /// The application sends telemetry.
    ///
    /// WHY: there is no telemetry, so there is nothing to switch off. The
    /// field exists so that a configuration file which carries the setting
    /// can be read; the value has no effect and produces a message in the
    /// log.
    #[serde(default)]
    pub telemetry: bool,

    /// The application asks for administrator rights.
    ///
    /// WHY: the same as the field above. The application never asks for
    /// elevation; the attempt to configure the guarantee is reported in the
    /// log and has no effect.
    #[serde(default)]
    pub elevation: bool,
}

impl Default for Security {
    /// The defaults of the section, as the shipped file writes them.
    ///
    /// WHY: the derived default of a structure of `bool` is `false` for
    /// every field, and that is not the default of the file. The default is
    /// therefore written out, so that an empty document and a missing
    /// section produce the same result.
    fn default() -> Self {
        Self {
            profile: SecurityProfile::default(),
            verify_directory_before_refresh:
                crate::security::DEFAULT_VERIFY_DIRECTORY_BEFORE_REFRESH,
            background_index: crate::security::DEFAULT_BACKGROUND_INDEX,
            remember_session: crate::security::DEFAULT_REMEMBER_SESSION,
            read_only_on_demand: crate::security::DEFAULT_READ_ONLY_ON_DEMAND,
            checksum_on_copy: crate::security::DEFAULT_CHECKSUM_ON_COPY,
            follow_symlinks: crate::security::DEFAULT_FOLLOW_SYMLINKS,
            telemetry: false,
            elevation: false,
        }
    }
}

impl Security {
    /// The settings that are in force when the configuration file cannot be
    /// used.
    ///
    /// WHY: a broken file must not weaken the application. The safe set is
    /// the profile `corporate`, because it is the set in which every rule of
    /// section 9.7 is in force.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::schema::{Security, SecurityProfile};
    ///
    /// let safe = Security::safe();
    /// assert_eq!(safe.profile, SecurityProfile::Corporate);
    /// assert!(!safe.remember_session);
    /// ```
    pub fn safe() -> Self {
        Self {
            profile: SecurityProfile::Corporate,
            remember_session: false,
            ..Self::default()
        }
    }
}

/// The profile that decides which security rules apply.
///
/// WHY: three profiles cover the practical cases without asking the user to
/// understand every switch. "standard" is for a private machine,
/// "corporate" for a managed one, and "custom" for a user who knows the
/// machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
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

impl SecurityProfile {
    /// The name of the profile, as the configuration file writes it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::schema::SecurityProfile;
    ///
    /// assert_eq!(SecurityProfile::Corporate.as_str(), "corporate");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            SecurityProfile::Standard => "standard",
            SecurityProfile::Corporate => "corporate",
            SecurityProfile::Custom => "custom",
        }
    }
}

/// Returns the default of the switch `verify_directory_before_refresh`.
fn default_verify_directory_before_refresh() -> bool {
    crate::security::DEFAULT_VERIFY_DIRECTORY_BEFORE_REFRESH
}

/// Returns the default of the switch `background_index`.
fn default_background_index() -> bool {
    crate::security::DEFAULT_BACKGROUND_INDEX
}

/// Returns the default of the switch `remember_session`.
fn default_remember_session() -> bool {
    crate::security::DEFAULT_REMEMBER_SESSION
}

/// Returns the default of the switch `read_only_on_demand`.
fn default_read_only_on_demand() -> bool {
    crate::security::DEFAULT_READ_ONLY_ON_DEMAND
}

/// Returns the default of the switch `checksum_on_copy`.
fn default_checksum_on_copy() -> bool {
    crate::security::DEFAULT_CHECKSUM_ON_COPY
}

/// Returns the default of the switch `follow_symlinks`.
fn default_follow_symlinks() -> bool {
    crate::security::DEFAULT_FOLLOW_SYMLINKS
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

/// The default of the largest answer of a remote source, in bytes.
///
/// WHY: sixty-four megabytes. A directory listing with a hundred thousand
///       entries fits into it, a file that the viewer opens is far smaller,
///       and the memory of a phone holds it. The user may set another value
///       in the settings, and zero turns the limit off.
pub const DEFAULT_MAX_REMOTE_DOWNLOAD_BYTES: u64 = 67_108_864;

/// The default of the largest answer of a remote source.
const fn default_max_remote_download_bytes() -> u64 {
    DEFAULT_MAX_REMOTE_DOWNLOAD_BYTES
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

    /// The largest answer that may be read from a remote source, in bytes.
    ///
    /// WHY: a remote source is the one place where the size of an answer
    ///       is decided by somebody else. The default is sixty-four
    ///       megabytes: a directory listing with a hundred thousand entries
    ///       fits into it, a file that the viewer opens is far smaller, and
    ///       the memory of a phone holds it. The user may set another value
    ///       in the settings, and zero turns the limit off.
    #[serde(default = "default_max_remote_download_bytes")]
    pub max_remote_download_bytes: u64,
}
