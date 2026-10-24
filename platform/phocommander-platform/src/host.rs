//! The host platform and the places it uses.
//!
//! WHAT: the operating system the program runs on, the directory of the
//!       configuration file and the key that carries the "primary" meaning.
//! WHY: the six target systems differ in three ways that the rest of the
//!       application must not repeat: the modifier key, the name of the
//!       configuration directory and the answer to the question "is the system
//!       dark?". The differences are written down here once.
//! EXAMPLE:
//! ```no_run
//! use phocommander_platform::{host_platform, primary_modifier};
//!
//! println!("{} uses {}", host_platform().name(), primary_modifier());
//! ```

use std::path::PathBuf;

use phocommander_core::Error;

/// The operating system Phocommander runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
/// macOS — the first platform to be delivered.
    MacOs,

/// Windows.
    Windows,

/// Linux.
    Linux,

/// FreeBSD, NetBSD or OpenBSD.
    Bsd,

/// Android, including the television and the headset variants.
    Android,

/// iPadOS.
    Ios,
}

impl Platform {
/// The name of the platform in a message to the user.
    pub fn name(&self) -> &'static str {
        match self {
            Platform::MacOs => "macOS",
            Platform::Windows => "Windows",
            Platform::Linux => "Linux",
            Platform::Bsd => "BSD",
            Platform::Android => "Android",
            Platform::Ios => "iPadOS",
        }
    }

/// The key that carries the "primary" meaning on this platform.
///
/// WHY: the actions are the same on every platform, only the key that
/// triggers them differs. {Cmd} is the primary key on macOS and on
/// iPadOS, {Ctrl} everywhere else.
    pub fn primary_modifier(&self) -> &'static str {
        match self {
            Platform::MacOs | Platform::Ios => "Cmd",
            Platform::Windows | Platform::Linux | Platform::Bsd | Platform::Android => "Ctrl",
        }
    }
}

/// The platform this program was compiled for.
pub fn host_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(target_os = "ios") {
        Platform::Ios
    } else if cfg!(target_os = "android") {
        Platform::Android
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "freebsd")
        || cfg!(target_os = "netbsd")
        || cfg!(target_os = "openbsd")
        || cfg!(target_os = "dragonfly")
    {
        Platform::Bsd
    } else {
        Platform::Linux
    }
}

/// The key that carries the "primary" meaning on this platform.
pub fn primary_modifier() -> &'static str {
    host_platform().primary_modifier()
}

/// The directory in which Phocommander stores its configuration file.
///
/// WHY: the location follows the conventions of the operating system:
/// {~/Library/Application Support/Phocommander} on macOS,
/// {%APPDATA%\Phocommander} on Windows and {$XDG_CONFIG_HOME/phocommander}
/// on Linux and BSD. A platform that reports no such directory is an error
/// the application has to report, not a reason to guess a path.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_platform::config_directory;
///
/// match config_directory() {
///     Ok(path) => println!("configuration in {}", path.display()),
///     Err(error) => println!("no configuration directory: {error}"),
/// }
/// ```
pub fn config_directory() -> Result<PathBuf, Error> {
    let base = dirs::config_dir().ok_or(Error::NoConfigDirectory)?;
    let name = match host_platform() {
        Platform::MacOs | Platform::Ios | Platform::Windows => "Phocommander",
        Platform::Linux | Platform::Bsd | Platform::Android => "phocommander",
    };
    Ok(base.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_host_platform_is_one_of_the_six_targets() {
        // act
        let platform = host_platform();

        // assert
        // The test runs on the machine that builds the workspace, which is one
        // of the target systems. The check therefore only verifies that the
        // name and the modifier exist.
        assert!(!platform.name().is_empty());
        assert!(!platform.primary_modifier().is_empty());
    }

    #[test]
    fn macos_and_ipados_use_the_command_key() {
        // WHY: the specification fixes `Cmd` as the primary key on macOS and
        // on iPadOS.
        // act
        let macos = Platform::MacOs.primary_modifier();
        let ipados = Platform::Ios.primary_modifier();

        // assert
        assert_eq!(macos, "Cmd");
        assert_eq!(ipados, "Cmd");
    }

    #[test]
    fn every_other_platform_uses_the_control_key() {
        // act
        let windows = Platform::Windows.primary_modifier();
        let linux = Platform::Linux.primary_modifier();
        let bsd = Platform::Bsd.primary_modifier();
        let android = Platform::Android.primary_modifier();

        // assert
        assert_eq!(windows, "Ctrl");
        assert_eq!(linux, "Ctrl");
        assert_eq!(bsd, "Ctrl");
        assert_eq!(android, "Ctrl");
    }

    #[test]
    fn the_configuration_directory_is_absolute_when_it_exists() {
        // act
        let directory = config_directory();

        // assert
        if let Ok(path) = directory {
            assert!(path.is_absolute(), "{}", path.display());
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            assert!(name == "Phocommander" || name == "phocommander", "{name}");
        }
    }
}
