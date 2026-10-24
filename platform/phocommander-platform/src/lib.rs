//! The platform adapters of Phocommander.
//!
//! WHAT: the differences between the six target systems — the host platform,
//!       the directory of the configuration file, the key that carries the
//!       "primary" meaning.
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

pub use host::{Platform, config_directory, host_platform, primary_modifier};
