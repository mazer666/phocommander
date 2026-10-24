//! The platform adapters of Phocommander.
//!
//! WHAT: the differences between the six target systems — the host platform,
//!       the directory of the configuration file, the key that carries the
//!       "primary" meaning, and the decision between the installed and the
//!       portable mode.
//! WHY: macOS uses {Cmd} where Windows, Linux and BSD use {Ctrl}, and the
//!       configuration file lives in a different directory on every system.
//!       Collecting the differences here keeps the crates above free of
//!       {cfg(target_os)} and testable on every platform.
//! BOUNDARY: this crate asks the operating system and answers the
//!       application. It draws nothing, it reads no configuration file and it
//!       performs no file operation.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_platform::host_platform;
//!
//! println!("{}", host_platform().name());
//! ```

pub mod host;
pub mod portable;

pub use host::{Platform, host_platform, primary_modifier};
pub use portable::{
    PORTABLE_CONFIG_DIRECTORY_NAME, PORTABLE_MARKER_FILE_NAME, config_directory,
    executable_directory, is_portable, is_portable_in, portable_config_directory,
    portable_config_directory_in,
};
