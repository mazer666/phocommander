//! The trash of the operating system.
//!
//! WHAT: the function that moves a file into the trash of the system
//!       ([`move_to_trash`]).
//! WHY: the deletion of a file is the one operation of a file manager that
//! can destroy data that the user cannot get back. The specification
//! therefore names the trash as the default path of the deletion (FR-27),
//! and the trash of the system is the place where the user can still undo
//! the deletion.
//! The three systems spell the trash differently: macOS keeps it in the
//! home directory and knows it through the file manager of the system,
//! Windows knows it through the shell, and Linux follows the specification
//! of the freedesktop. The crate `trash` speaks all three, and the
//! difference stays in this module — the crates above know one function
//! and no `cfg(target_os)`.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_platform::trash::move_to_trash;
//!
//! let ergebnis = move_to_trash(std::path::Path::new("/tmp/ein Beispiel.txt"));
//! if let Err(error) = ergebnis {
//!     eprintln!("the file could not be moved into the trash: {error}");
//! }
//! ```

use std::path::Path;

/// The reason why a file did not reach the trash.
#[derive(Debug)]
pub enum TrashError {
    /// The system refused the file.
    ///
    /// WHY: the reason of the system is kept as it is. It names the cause —
    /// a missing permission, a read-only medium, a file that is in use —
    /// and the user has to see it in the words of the system.
    System(String),
}

impl std::fmt::Display for TrashError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System(message) => write!(
                formatter,
                "the file could not be moved into the trash: {message}"
            ),
        }
    }
}

impl std::error::Error for TrashError {}

/// Moves the file into the trash of the operating system.
///
/// WHY: the deletion of a file goes into the trash and not into the void.
///       The user can undo it there, and the file keeps its data until the
///       trash is emptied (FR-27).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_platform::trash::move_to_trash;
///
/// let ergebnis = move_to_trash(std::path::Path::new("/tmp/ein Beispiel.txt"));
/// assert!(ergebnis.is_ok() || ergebnis.is_err());
/// ```
pub fn move_to_trash(path: &Path) -> Result<(), TrashError> {
    trash::delete(path).map_err(|error| TrashError::System(error.to_string()))
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;

    #[test]
    fn a_file_that_does_not_exist_cannot_reach_the_trash() {
        // arrange
        let pfad = std::env::temp_dir().join("phocommander-trash-probe-does-not-exist.txt");

        // act
        let ergebnis = move_to_trash(&pfad);

        // assert
        // WHY: the trash of the system reports the reason and does not
        //       pretend that the file is there.
        assert!(ergebnis.is_err());
    }

    #[test]
    fn a_file_that_is_there_reaches_the_trash() {
        // arrange
        let pfad = std::env::temp_dir().join("phocommander-trash-probe.txt");
        std::fs::write(&pfad, b"payload")
            .unwrap_or_else(|error| panic!("the file should have been written: {error}"));

        // act
        let ergebnis = move_to_trash(&pfad);

        // assert
        // WHY: the file is not there any more — it went into the trash.
        assert!(ergebnis.is_ok(), "{ergebnis:?}");
        assert!(!pfad.exists());
    }
}
