//! The values a setting can take.
//!
//! WHAT: one enum per value that the configuration file restricts to a fixed
//!       set of spellings, each with the default the specification names.
//! WHY: an enum turns a typo into a message at the moment the file is read
//!       instead of into a setting that silently does nothing. The default of
//!       every value is the one the specification fixes, so a file that leaves
//!       the value out behaves like the shipped default file.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::Theme;
//!
//! assert_eq!(Theme::default().as_str(), "system");
//! ```

use serde::{Deserialize, Serialize};

/// The language of the user interface.
///
/// WHY: the default is {system}, the value the shipped default file carries.
/// The German translation is the fallback of the translation catalogue, not a
/// fixed choice of the interface: a user on an English system expects an
/// English interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// German.
    De,

    /// English.
    En,

    /// The language of the operating system.
    #[default]
    System,
}

impl Language {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::De => "de",
            Self::En => "en",
            Self::System => "system",
        }
    }
}

/// The colour scheme of the user interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// The colour scheme of the operating system.
    #[default]
    System,

    /// The light colour scheme.
    Light,

    /// The dark colour scheme.
    Dark,
}

impl Theme {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

/// The format in which file sizes are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSizeFormat {
    /// Kilobyte, megabyte, gigabyte — powers of 1024.
    #[default]
    Binary,

    /// Kilobyte, megabyte, gigabyte — powers of 1000.
    Decimal,

    /// Kilobyte, mebibyte, gibibyte — powers of 1024 with IEC names.
    Iec,
}

impl FileSizeFormat {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Binary => "binary",
            Self::Decimal => "decimal",
            Self::Iec => "iec",
        }
    }
}

/// Whether the two panels stand next to each other or below each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelLayout {
    /// Both panels are visible side by side.
    #[default]
    Horizontal,

    /// Both panels are visible one above the other.
    Vertical,
}

impl PanelLayout {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// What a right click does in the file list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RightClickAction {
    /// A context menu opens.
    #[default]
    ContextMenu,

    /// The selection of the clicked file is toggled.
    ToggleSelection,
}

impl RightClickAction {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ContextMenu => "context_menu",
            Self::ToggleSelection => "toggle_selection",
        }
    }
}

/// What happens when a file that is to be written already exists.
///
/// WHY: overwriting is never the default. A file that is silently replaced is
/// a file the user cannot get back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverwritePolicy {
    /// The user is asked.
    #[default]
    Ask,

    /// The existing file is kept, the operation continues with the next file.
    Never,

    /// The existing file is replaced.
    Always,

    /// The existing file is kept and the new file is written under a new name.
    KeepBoth,
}

impl OverwritePolicy {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Never => "never",
            Self::Always => "always",
            Self::KeepBoth => "keep_both",
        }
    }
}

/// How much the application writes into its log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Only failures are written.
    Error,

    /// Failures and warnings are written.
    #[default]
    Warn,

    /// Additional information about the operation is written.
    Info,

    /// Detailed information for the developer is written.
    Debug,

    /// Everything is written.
    Trace,
}

impl LogLevel {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

/// The input context that is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputContext {
    /// The context is recognised from the input device at run time.
    #[default]
    Auto,

    /// A physical keyboard.
    Keyboard,

    /// A mouse or a trackpad.
    Pointer,

    /// A finger on a touch screen.
    Touch,

    /// The directional pad of a television remote control.
    Dpad,

    /// A gamepad or a VR controller.
    Controller,
}

impl InputContext {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Keyboard => "keyboard",
            Self::Pointer => "pointer",
            Self::Touch => "touch",
            Self::Dpad => "dpad",
            Self::Controller => "controller",
        }
    }
}

/// Whether the signature of an SMB server is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SmbSigning {
    /// A server without signature support is refused.
    Require,

    /// The signature is used when the server supports it.
    #[default]
    Prefer,
}

impl SmbSigning {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Require => "require",
            Self::Prefer => "prefer",
        }
    }
}

/// Whether the transport encryption of an SMB server is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SmbEncryption {
    /// A server without transport encryption is refused.
    Require,

    /// The transport encryption is used when the server supports it.
    #[default]
    Prefer,
}

impl SmbEncryption {
    /// The spelling of this value in the configuration file.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Require => "require",
            Self::Prefer => "prefer",
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_shipped_default_file() {
        // WHY: a file that leaves a value out has to behave like the shipped
        // default file. The defaults are therefore part of the contract.
        assert_eq!(Language::default(), Language::System);
        assert_eq!(Theme::default(), Theme::System);
        assert_eq!(FileSizeFormat::default(), FileSizeFormat::Binary);
        assert_eq!(PanelLayout::default(), PanelLayout::Horizontal);
        assert_eq!(RightClickAction::default(), RightClickAction::ContextMenu);
        assert_eq!(OverwritePolicy::default(), OverwritePolicy::Ask);
        assert_eq!(LogLevel::default(), LogLevel::Warn);
        assert_eq!(InputContext::default(), InputContext::Auto);
        assert_eq!(SmbSigning::default(), SmbSigning::Prefer);
        assert_eq!(SmbEncryption::default(), SmbEncryption::Prefer);
    }

    #[test]
    fn every_value_has_a_unique_spelling() {
        // WHY: the spelling is the key of the value in the configuration file.
        // Two values of the same setting with the same spelling would make
        // one of them unreachable. The spellings of two different settings
        // may be equal — they live in different sections of the file.
        let groups: [&[&str]; 10] = [
            &[
                Language::De.as_str(),
                Language::En.as_str(),
                Language::System.as_str(),
            ],
            &[
                Theme::System.as_str(),
                Theme::Light.as_str(),
                Theme::Dark.as_str(),
            ],
            &[
                FileSizeFormat::Binary.as_str(),
                FileSizeFormat::Decimal.as_str(),
                FileSizeFormat::Iec.as_str(),
            ],
            &[
                PanelLayout::Horizontal.as_str(),
                PanelLayout::Vertical.as_str(),
            ],
            &[
                RightClickAction::ContextMenu.as_str(),
                RightClickAction::ToggleSelection.as_str(),
            ],
            &[
                OverwritePolicy::Ask.as_str(),
                OverwritePolicy::Never.as_str(),
                OverwritePolicy::Always.as_str(),
                OverwritePolicy::KeepBoth.as_str(),
            ],
            &[
                LogLevel::Error.as_str(),
                LogLevel::Warn.as_str(),
                LogLevel::Info.as_str(),
                LogLevel::Debug.as_str(),
                LogLevel::Trace.as_str(),
            ],
            &[
                InputContext::Auto.as_str(),
                InputContext::Keyboard.as_str(),
                InputContext::Pointer.as_str(),
                InputContext::Touch.as_str(),
                InputContext::Dpad.as_str(),
                InputContext::Controller.as_str(),
            ],
            &[SmbSigning::Require.as_str(), SmbSigning::Prefer.as_str()],
            &[
                SmbEncryption::Require.as_str(),
                SmbEncryption::Prefer.as_str(),
            ],
        ];

        for group in groups {
            let mut sorted = group.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), group.len(), "{group:?}");
        }
    }

    #[test]
    fn overwrite_is_never_the_default() {
        // WHY: a file that is silently replaced cannot be recovered. The
        // specification demands that overwriting is not the default.
        assert_ne!(OverwritePolicy::default(), OverwritePolicy::Always);
    }
}
