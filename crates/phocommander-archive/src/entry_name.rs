//! The hardened resolution of the names an archive contains.
//!
//! WHAT: turns the untrusted name of an archive entry into a relative path
//!       that is guaranteed to stay inside the destination directory.
//! WHY: the name of an entry decides where the entry is written. The historic
//!       attacks — the "zip slip" and its successors, the advisories
//!       RUSTSEC-2025-0168, RUSTSEC-2026-0367, RUSTSEC-2026-0368 and
//!       RUSTSEC-2026-0369 — all use the name of an entry to write outside of
//!       the directory the user chose. The rules below therefore refuse
//!       everything that could leave the destination: absolute paths, drive
//!       letters, the parent segments, control characters, alternate data
//!       the characters and the endings that Windows forbids, and the
//!       reserved device names of Windows.
//!
//! The resolution is a pure function. The same name always produces the
//! same result, and a name that is refused always produces the same
//! reason and the same suggestion. A user who reads the message of a
//! refusal can therefore rely on it.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::resolve_entry_name;
//!
//! # fn main() -> Result<(), phocommander_archive::EntryNameError> {
//! // a normal name is kept as it is
//! let path = resolve_entry_name("photos/2026/sunset.jpg")?;
//! assert_eq!(path.components().count(), 3);
//!
//! // a name that would leave the destination is refused
//! assert!(resolve_entry_name("../../etc/passwd").is_err());
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;

use crate::windows_name;

/// The longest name of an archive entry that is accepted, in bytes.
///
/// WHY: a name that is far longer than any file system accepts is either an
///       attack or a mistake. Refusing it early keeps the error message clear.
pub const MAX_ENTRY_NAME_BYTES: usize = 4096;

/// The reasons why the name of an archive entry was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EntryNameError {
/// The name was empty.
    #[error("the archive entry has an empty name")]
    Empty,

/// The name was an absolute path or carried a drive letter.
    #[error("the archive entry name {name:?} is an absolute path")]
    Absolute {
/// The name that was refused.
        name: String,
    },

/// The name tried to leave the destination directory.
    #[error("the archive entry name {name:?} leaves the destination directory")]
    Traversal {
/// The name that was refused.
        name: String,
    },

/// The name contained a control character.
    #[error("the archive entry name {name:?} contains a control character")]
    ControlCharacter {
/// The name that was refused.
        name: String,
    },

/// The name was a reserved device name of Windows.
    #[error("the archive entry name {name:?} is a reserved device name of Windows")]
    ReservedName {
/// The name that was refused.
        name: String,
    },

/// The name contained a colon, which Windows reads as an alternate data
/// stream of another file.
    #[error("the archive entry name {name:?} contains a colon, which Windows reads as an alternate data stream; the name {suggestion:?} would work on every platform")]
    AlternateDataStream {
/// The name that was refused.
        name: String,
/// The name that would work on every platform.
        suggestion: String,
    },

/// The name contained a character that Windows forbids in a file name.
    #[error("the archive entry name {name:?} contains the character {character:?}, which Windows forbids in a file name; the name {suggestion:?} would work on every platform")]
    ForbiddenCharacter {
/// The name that was refused.
        name: String,
/// The character that caused the refusal.
        character: char,
/// The name that would work on every platform.
        suggestion: String,
    },

/// The name ended in a dot or in a space, which Windows removes silently.
    #[error("the archive entry name {name:?} ends in a dot or a space, which Windows removes silently; the name {suggestion:?} would work on every platform")]
    ForbiddenEnding {
/// The name that was refused.
        name: String,
/// The name that would work on every platform.
        suggestion: String,
    },

/// The name was longer than [{MAX_ENTRY_NAME_BYTES}].
    #[error("the archive entry name is {length} bytes long, the limit is {limit} bytes")]
    TooLong {
/// The length of the name in bytes.
        length: usize,
/// The limit that was exceeded.
        limit: usize,
    },
}

/// Resolves the name of an archive entry to a relative path.
///
/// WHY: the result is always relative and never contains a parent segment, so
///       that joining it with the destination directory cannot leave the
///       destination. The function is the only place where an entry name is
///       turned into a path, so a reader cannot resolve a name in a laxer
///       way. It is pure: the same name always produces the same result.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::resolve_entry_name;
///
/// # fn main() -> Result<(), phocommander_archive::EntryNameError> {
/// let path = resolve_entry_name("docs/readme.md")?;
/// assert!(path.is_relative());
/// assert_eq!(path.file_name().map(|name| name.to_string_lossy().to_string()), Some("readme.md".to_string()));
/// # Ok(())
/// # }
/// ```
pub fn resolve_entry_name(name: &str) -> Result<PathBuf, EntryNameError> {
    if name.is_empty() {
        return Err(EntryNameError::Empty);
    }

    if name.len() > MAX_ENTRY_NAME_BYTES {
        return Err(EntryNameError::TooLong {
            length: name.len(),
            limit: MAX_ENTRY_NAME_BYTES,
        });
    }

    if name.chars().any(char::is_control) {
        return Err(EntryNameError::ControlCharacter {
            name: name.to_string(),
        });
    }

    if name.starts_with('/') || name.starts_with('\\') || has_drive_letter(name) {
        return Err(EntryNameError::Absolute {
            name: name.to_string(),
        });
    }

    let mut path = PathBuf::new();
    for segment in name.split(['/', '\\']) {
        // An empty segment comes from a trailing or a doubled separator, a
        // single dot from a path that points at itself. Both carry no
        // information and are dropped.
        if segment.is_empty() || segment == "." {
            continue;
        }

        if segment == ".." {
            return Err(EntryNameError::Traversal {
                name: name.to_string(),
            });
        }

        if segment.contains(':') {
            return Err(EntryNameError::AlternateDataStream {
                name: name.to_string(),
                suggestion: windows_name::suggest_replacement(name),
            });
        }

        if is_reserved_device_name(segment) {
            return Err(EntryNameError::ReservedName {
                name: name.to_string(),
            });
        }

        if let Some(character) = windows_name::first_forbidden_character(segment) {
            return Err(EntryNameError::ForbiddenCharacter {
                name: name.to_string(),
                character,
                suggestion: windows_name::suggest_replacement(name),
            });
        }

        if windows_name::has_forbidden_ending(segment) {
            return Err(EntryNameError::ForbiddenEnding {
                name: name.to_string(),
                suggestion: windows_name::suggest_replacement(name),
            });
        }

        path.push(segment);
    }

    if path.as_os_str().is_empty() {
        return Err(EntryNameError::Empty);
    }

    Ok(path)
}

/// Whether the name starts with a drive letter such as {C:}.
fn has_drive_letter(name: &str) -> bool {
    let mut characters = name.chars();
    match (characters.next(), characters.next()) {
        (Some(letter), Some(':')) => letter.is_ascii_alphabetic(),
        _ => false,
    }
}

/// Whether the segment is a reserved device name of Windows.
///
/// WHY: Windows refuses to create a file called {CON} or {NUL}, and it does
///       so in a way that silently redirects the write. An archive that
///       contains such a name would therefore write somewhere the user did
///       not choose.
fn is_reserved_device_name(segment: &str) -> bool {
    let stem = segment
        .split('.')
        .next()
        .unwrap_or(segment)
        .to_ascii_uppercase();
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") {
        return true;
    }

    match stem.as_bytes() {
        [b'C', b'O', b'M', digit] | [b'L', b'P', b'T', digit] => {
            matches!(digit, b'1'..=b'9')
        }
        _ => false,
    }
}
