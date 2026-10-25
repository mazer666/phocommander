//! The state of the last session.
//!
//! WHAT: the state of the two panels at the end of a session
//!       ([`SessionState`]), the name of the file that holds it
//!       ([`SESSION_FILE_NAME`]), the function that writes it
//!       ([`write_session_state`]) and the function that reads it back
//!       ([`load_session_state`]).
//! WHY: the state of the session is the one thing the application writes
//!       without the user asking for it. The rule "remember_session" decides
//!       whether it is written at all; when it is written, it goes into the
//!       configuration directory and into no other place (section 9.7 of the
//!       specification). A machine that is not the user's own must not keep
//!       a trace of the application, and the rule that switches the file off
//!       has to switch the writing off as well — not only the reading.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::{EffectiveSecurity, SessionState, write_session_state};
//! use std::path::Path;
//!
//! let state = SessionState::new("/home/user/photos", "/home/user/documents");
//! let result = write_session_state(Path::new("/tmp/config"), &state, &EffectiveSecurity::safe());
//! // The safe rules leave nothing behind, so nothing is written.
//! assert!(result.is_ok());
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::security::EffectiveSecurity;

/// The name of the file that holds the state of the last session.
///
/// WHY: the name says what the file is. It lives next to the configuration
/// file of the application, in the configuration directory.
pub const SESSION_FILE_NAME: &str = "session.toml";

/// The state of the two panels at the end of a session.
///
/// WHY: the state is small on purpose. It holds the two directories the
/// panels showed and nothing else — no history, no names of files, no
/// credentials (the rule of the interface profiles of the specification).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionState {
    /// The directory the left panel showed.
    pub left: PathBuf,

    /// The directory the right panel showed.
    pub right: PathBuf,
}

impl SessionState {
    /// Builds the state of two panels.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_config::SessionState;
    ///
    /// let state = SessionState::new("/photos", "/documents");
    /// assert_eq!(state.left.to_string_lossy(), "/photos");
    /// ```
    pub fn new(left: impl Into<PathBuf>, right: impl Into<PathBuf>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }
}

/// The failures of the session file.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// The file could not be written.
    #[error("the session file {path} could not be written: {source}")]
    Write {
        /// The path of the file that could not be written.
        path: PathBuf,
        /// The error the operating system reported.
        #[source]
        source: io::Error,
    },

    /// The file could not be read.
    #[error("the session file {path} could not be read: {source}")]
    Read {
        /// The path of the file that could not be read.
        path: PathBuf,
        /// The error the operating system reported.
        #[source]
        source: io::Error,
    },

    /// The file is not a valid session file.
    #[error("the session file {path} is not valid: {source}")]
    Syntax {
        /// The path of the file that is not valid.
        path: PathBuf,
        /// The error the parser reported.
        #[source]
        source: toml::de::Error,
    },

    /// The state could not be written as a document.
    #[error("the state of the session could not be written: {message}")]
    Serialise {
        /// What the writer reported.
        message: String,
    },
}

/// Writes the state of the session into the configuration directory.
///
/// WHY: the rule "remember_session" decides whether the file is written at
/// all. When the rule is off, the function returns without touching the
/// disk — not even the directory is created. When the rule is on, the file
/// goes into the configuration directory and into no other place, so that the
/// application writes only where the specification allows it to write.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::{EffectiveSecurity, SessionState, write_session_state};
/// use std::path::Path;
///
/// let state = SessionState::new("/photos", "/documents");
/// let rules = EffectiveSecurity::safe();
/// let written = write_session_state(Path::new("/tmp/config"), &state, &rules);
/// // The safe rules remember no session, so no file is written.
/// assert!(written.is_ok());
/// ```
pub fn write_session_state(
    directory: &Path,
    state: &SessionState,
    security: &EffectiveSecurity,
) -> Result<Option<PathBuf>, SessionError> {
    if !security.remembers_session() {
        return Ok(None);
    }

    let document = toml::to_string(state).map_err(|error| SessionError::Serialise {
        message: error.to_string(),
    })?;

    fs::create_dir_all(directory).map_err(|source| SessionError::Write {
        path: directory.to_path_buf(),
        source,
    })?;

    let path = directory.join(SESSION_FILE_NAME);
    fs::write(&path, document).map_err(|source| SessionError::Write {
        path: path.clone(),
        source,
    })?;

    Ok(Some(path))
}

/// Reads the state of the last session.
///
/// WHY: a machine that never ran the application has no session file, and
/// that is not an error. The function therefore reports "no state" with
/// `None` and reserves the error for a file that exists and cannot be used.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::load_session_state;
/// use std::path::Path;
///
/// let state = load_session_state(Path::new("/tmp/does-not-exist"));
/// assert!(state.is_ok());
/// ```
pub fn load_session_state(directory: &Path) -> Result<Option<SessionState>, SessionError> {
    let path = directory.join(SESSION_FILE_NAME);
    let document = match fs::read_to_string(&path) {
        Ok(document) => document,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(SessionError::Read {
                path: path.clone(),
                source,
            });
        }
    };

    let state = toml::from_str(&document).map_err(|source| SessionError::Syntax {
        path: path.clone(),
        source,
    })?;

    Ok(Some(state))
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::schema::{Security, SecurityProfile};
    use crate::security::resolve_security;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// The counter that makes the name of a temporary directory unique.
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Creates a directory that the test may write into.
    fn temporary_directory(label: &str) -> PathBuf {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "phocommander-session-{label}-{}-{unique}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path)
            .unwrap_or_else(|error| panic!("the directory could not be created: {error}"));
        path
    }

    /// Builds the rules of a profile with the switch `remember_session` on.
    fn rules_that_remember(profile: SecurityProfile) -> EffectiveSecurity {
        resolve_security(&Security {
            profile,
            remember_session: true,
            ..Security::default()
        })
    }

    #[test]
    fn the_state_of_two_panels_holds_the_two_directories() {
        // act
        let state = SessionState::new("/photos", "/documents");

        // assert
        assert_eq!(state.left, PathBuf::from("/photos"));
        assert_eq!(state.right, PathBuf::from("/documents"));
    }

    #[test]
    fn a_state_that_is_written_can_be_read_back() {
        // arrange
        let directory = temporary_directory("round-trip");
        let state = SessionState::new("/photos", "/documents");
        let rules = rules_that_remember(SecurityProfile::Custom);

        // act
        let written = write_session_state(&directory, &state, &rules);
        let read = load_session_state(&directory);

        // assert
        let path = match written {
            Ok(Some(path)) => path,
            other => panic!("the state should have been written: {other:?}"),
        };
        assert_eq!(path, directory.join(SESSION_FILE_NAME));
        match read {
            Ok(Some(state)) => assert_eq!(state, SessionState::new("/photos", "/documents")),
            other => panic!("the state should have been read back: {other:?}"),
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_rule_that_keeps_no_session_writes_no_file() {
        // WHY: the rule "remember_session" is switched off, so nothing is
        // written — not even inside the configuration directory.
        // arrange
        let directory = temporary_directory("not-written");
        let state = SessionState::new("/photos", "/documents");
        let rules = resolve_security(&Security {
            profile: SecurityProfile::Custom,
            remember_session: false,
            ..Security::default()
        });

        // act
        let written = write_session_state(&directory, &state, &rules);

        // assert
        match written {
            Ok(None) => {}
            other => panic!("no file should have been written: {other:?}"),
        }
        let content = fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("the directory could not be read: {error}"))
            .count();
        assert_eq!(content, 0, "the directory should be empty");
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn the_safe_rules_write_no_session() {
        // arrange
        let directory = temporary_directory("safe");
        let state = SessionState::new("/photos", "/documents");

        // act
        let written = write_session_state(&directory, &state, &EffectiveSecurity::safe());

        // assert
        match written {
            Ok(None) => {}
            other => panic!("no file should have been written: {other:?}"),
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_directory_without_a_session_file_reports_no_state() {
        // arrange
        let directory = temporary_directory("missing");

        // act
        let result = load_session_state(&directory);

        // assert
        match result {
            Ok(None) => {}
            other => panic!("the missing file should not be an error: {other:?}"),
        }
        let _ = fs::remove_dir_all(&directory);
    }
}
