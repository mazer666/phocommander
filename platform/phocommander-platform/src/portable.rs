//! The decision about where Phocommander keeps its configuration.
//!
//! WHAT: the two modes of the application — installed and portable — and the
//!       function that resolves the configuration directory for each of them.
//! WHY: the portable variant runs from a USB stick and must not write into
//!       the profile of the machine it happens to run on. The decision
//!       between the two locations has to be made once, at the boundary,
//!       because everything below the platform layer works with the result
//!       and must not know which mode the application is in.
//! EXAMPLE:
//! ```no_run
//! use phocommander_platform::{config_directory, is_portable};
//!
//! match config_directory() {
//!     Ok(path) if is_portable() => println!("portable, configuration in {}", path.display()),
//!     Ok(path) => println!("installed, configuration in {}", path.display()),
//!     Err(error) => println!("no configuration directory: {error}"),
//! }
//! ```

use std::io;
use std::path::{Path, PathBuf};

use phocommander_core::Error;

/// The name of the file that marks a directory as a portable installation.
///
/// WHY: the marker is a file and not a command line argument, because the
/// user who copies the directory to a USB stick should not have to know
/// anything about it. The presence of the file is the whole declaration.
pub const PORTABLE_MARKER_FILE_NAME: &str = "portable.txt";

/// The name of the directory that holds the configuration in the portable mode.
pub const PORTABLE_CONFIG_DIRECTORY_NAME: &str = "Phocommander";

/// Returns the directory of the running executable.
///
/// WHY: the portable variant is defined relative to the executable, not
/// relative to the current working directory. A user who starts the
/// application from a shortcut in another directory still expects the
/// configuration to travel with the executable.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_platform::executable_directory;
///
/// match executable_directory() {
///     Ok(path) => println!("the executable lives in {}", path.display()),
///     Err(error) => println!("the executable directory is unknown: {error}"),
/// }
/// ```
pub fn executable_directory() -> Result<PathBuf, Error> {
    let executable = std::env::current_exe().map_err(|source| {
        Error::io(std::path::Path::new("the running executable"), source)
    })?;
    let directory = executable.parent().ok_or_else(|| {
        Error::io(
            executable.as_path(),
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "the executable has no parent directory",
            ),
        )
    })?;
    Ok(directory.to_path_buf())
}

/// Returns whether the application runs in the portable mode.
///
/// WHY: the mode is decided by the presence of the marker file beside the
/// executable. This is the only place where the decision is made; every
/// other part of the application asks for the configuration directory and
/// receives the right one.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_platform::is_portable;
///
/// if is_portable() {
///     println!("running from a removable drive");
/// }
/// ```
pub fn is_portable() -> bool {
    executable_directory()
        .map(|directory| directory.join(PORTABLE_MARKER_FILE_NAME).is_file())
        .unwrap_or(false)
}

/// Returns the directory in which Phocommander stores its configuration file.
///
/// WHY: the location follows the conventions of the operating system:
/// {~/Library/Application Support/Phocommander} on macOS,
/// {%APPDATA%\Phocommander} on Windows and {$XDG_CONFIG_HOME/phocommander}
/// on Linux and BSD. A platform that reports no such directory is an error
/// the application has to report, not a reason to guess a path.
///
/// The portable mode is the exception: there the configuration travels with
/// the executable, so that a copy on a USB stick keeps its settings when it
/// is used on another machine.
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
    if is_portable() {
        return portable_config_directory();
    }
    let base = dirs::config_dir().ok_or(Error::NoConfigDirectory)?;
    let name = match super::host_platform() {
        super::Platform::MacOs | super::Platform::Ios | super::Platform::Windows => "Phocommander",
        super::Platform::Linux | super::Platform::Bsd | super::Platform::Android => "phocommander",
    };
    Ok(base.join(name))
}

/// Returns the configuration directory of the portable mode.
///
/// WHY: the directory sits beside the executable, inside the directory that
/// the user copied. Everything the portable variant writes stays there, so
/// that nothing is left behind on a foreign machine.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_platform::portable_config_directory;
///
/// match portable_config_directory() {
///     Ok(path) => println!("the portable configuration is in {}", path.display()),
///     Err(error) => println!("the portable configuration directory is unknown: {error}"),
/// }
/// ```
pub fn portable_config_directory() -> Result<PathBuf, Error> {
    Ok(executable_directory()?.join(PORTABLE_CONFIG_DIRECTORY_NAME))
}

/// Returns the directory of the portable installation for a given executable.
///
/// WHY: the decision of the portable mode needs a test, and a test needs a
/// directory it may write into. This function takes the directory of the
/// executable as an argument, so that the test does not depend on the
/// machine it runs on.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_platform::{portable_config_directory_in, PORTABLE_MARKER_FILE_NAME};
/// use std::path::Path;
///
/// let installation = Path::new("/media/usb/Phocommander");
/// let marker = installation.join(PORTABLE_MARKER_FILE_NAME);
/// std::fs::write(&marker, "portable").expect("the marker can be written");
///
/// match portable_config_directory_in(installation) {
///     Ok(path) => println!("the portable configuration is in {}", path.display()),
///     Err(error) => println!("the portable configuration directory is unknown: {error}"),
/// }
/// ```
pub fn portable_config_directory_in(installation: &Path) -> Result<PathBuf, Error> {
    Ok(installation.join(PORTABLE_CONFIG_DIRECTORY_NAME))
}

/// Returns whether a given installation directory is a portable installation.
///
/// WHY: the same reason as for `is_portable`. The function takes the
/// directory as an argument so that the behaviour can be tested without
/// touching the directory of the running executable.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_platform::{is_portable_in, PORTABLE_MARKER_FILE_NAME};
/// use std::path::Path;
///
/// let installation = Path::new("/media/usb/Phocommander");
/// println!("portable: {}", is_portable_in(installation));
/// let _ = installation.join(PORTABLE_MARKER_FILE_NAME);
/// ```
pub fn is_portable_in(installation: &Path) -> bool {
    installation.join(PORTABLE_MARKER_FILE_NAME).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory that stands in for a portable installation on the disk.
    ///
    /// WHY: the tests must not depend on the machine they run on, and they
    /// must not write into the directory of the running executable.
    fn temporary_installation() -> (PathBuf, TempDirectory) {
        let guard = TempDirectory::new("phocommander-portable");
        let installation = guard.path().join("Phocommander");
        std::fs::create_dir_all(&installation).expect("the installation directory can be created");
        (installation, guard)
    }

    /// A directory that is removed when it goes out of scope.
    ///
    /// WHY: a test that leaves directories behind fills the disk of the
    /// machine over time, and a test that depends on a fixed path fails as
    /// soon as two runs overlap.
    struct TempDirectory {
        path: PathBuf,
    }

    impl TempDirectory {
        fn new(prefix: &str) -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
            std::fs::create_dir_all(&path).expect("the temporary directory can be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn a_directory_without_the_marker_is_not_portable() {
        // WHY: the requirement FR-22p says that the marker file decides the
        // mode, and that the installed behaviour is the default.
        // act
        let (installation, _guard) = temporary_installation();

        // assert
        assert!(!is_portable_in(&installation));
    }

    #[test]
    fn a_directory_with_the_marker_is_portable() {
        // act
        let (installation, _guard) = temporary_installation();
        std::fs::write(
            installation.join(PORTABLE_MARKER_FILE_NAME),
            "portable",
        )
        .expect("the marker can be written");

        // assert
        assert!(is_portable_in(&installation));
    }

    #[test]
    fn the_portable_configuration_sits_beside_the_executable() {
        // WHY: the requirement FR-22n says that the configuration travels
        // with the executable, so that a copy on a USB stick keeps its
        // settings on another machine.
        // act
        let (installation, _guard) = temporary_installation();

        // assert
        let configuration = portable_config_directory_in(&installation).expect("the path is known");
        assert_eq!(configuration, installation.join(PORTABLE_CONFIG_DIRECTORY_NAME));
        assert_eq!(configuration.parent(), Some(installation.as_path()));
    }

    #[test]
    fn the_portable_directory_is_inside_the_installation() {
        // WHY: the requirement FR-22o says that the portable variant writes
        // nothing outside its own directory.
        // act
        let (installation, _guard) = temporary_installation();
        let configuration = portable_config_directory_in(&installation).expect("the path is known");

        // assert
        assert!(configuration.starts_with(&installation));
    }

    #[test]
    fn the_marker_is_a_file_and_not_a_directory() {
        // WHY: a directory with the name of the marker would make the
        // detection ambiguous, and the application would then read its
        // configuration from the wrong place.
        // act
        let (installation, _guard) = temporary_installation();
        std::fs::create_dir_all(installation.join(PORTABLE_MARKER_FILE_NAME))
            .expect("the directory can be created");

        // assert
        assert!(!is_portable_in(&installation));
    }
}
