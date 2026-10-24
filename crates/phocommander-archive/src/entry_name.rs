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
//!       letters, `..` segments, control characters, alternate data streams
//!       and the reserved device names of Windows.
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

/// The longest name of an archive entry that is accepted, in bytes.
///
/// WHY: a name that is far longer than any file system accepts is either an
/// attack or a mistake. Refusing it early keeps the error message clear.
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

    /// The name was a reserved device name or an alternate data stream.
    #[error("the archive entry name {name:?} is not a valid file name on every platform")]
    ReservedName {
        /// The name that was refused.
        name: String,
    },

    /// The name was longer than [`MAX_ENTRY_NAME_BYTES`].
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
/// WHY: the result is always relative and never contains a `..` segment, so
/// that joining it with the destination directory cannot leave the
/// destination. The function is the only place where an entry name is turned
/// into a path, so a reader cannot resolve a name in a laxer way.
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
            return Err(EntryNameError::ReservedName {
                name: name.to_string(),
            });
        }

        if is_reserved_device_name(segment) {
            return Err(EntryNameError::ReservedName {
                name: name.to_string(),
            });
        }

        path.push(segment);
    }

    if path.as_os_str().is_empty() {
        return Err(EntryNameError::Empty);
    }

    Ok(path)
}

/// Whether the name starts with a drive letter such as `C:`.
fn has_drive_letter(name: &str) -> bool {
    let mut characters = name.chars();
    match (characters.next(), characters.next()) {
        (Some(letter), Some(':')) => letter.is_ascii_alphabetic(),
        _ => false,
    }
}

/// Whether the segment is a reserved device name of Windows.
///
/// WHY: Windows refuses to create a file called `CON` or `NUL`, and it does
/// so in a way that silently redirects the write. An archive that contains
/// such a name would therefore write somewhere the user did not choose.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_a_normal_name() {
        // act
        let path = match resolve_entry_name("readme.md") {
            Ok(path) => path,
            Err(error) => panic!("the name should be accepted: {error}"),
        };

        // assert
        assert_eq!(path.to_string_lossy(), "readme.md");
        assert!(path.is_relative());
    }

    #[test]
    fn keeps_the_directories_of_a_name() {
        // act
        let path = match resolve_entry_name("photos/2026/sunset.jpg") {
            Ok(path) => path,
            Err(error) => panic!("the name should be accepted: {error}"),
        };

        // assert
        assert_eq!(path.components().count(), 3);
        assert_eq!(path.to_string_lossy(), "photos/2026/sunset.jpg");
    }

    #[test]
    fn keeps_a_name_with_non_latin_characters() {
        // act
        let path = match resolve_entry_name("übung/Übersicht.md") {
            Ok(path) => path,
            Err(error) => panic!("the name should be accepted: {error}"),
        };

        // assert
        assert_eq!(path.to_string_lossy(), "übung/Übersicht.md");
    }

    #[test]
    fn drops_a_trailing_separator_and_a_current_directory() {
        // act
        let path = match resolve_entry_name("docs/./readme.md/") {
            Ok(path) => path,
            Err(error) => panic!("the name should be accepted: {error}"),
        };

        // assert
        assert_eq!(path.to_string_lossy(), "docs/readme.md");
    }

    #[test]
    fn refuses_an_empty_name() {
        // act
        let result = resolve_entry_name("");

        // assert
        assert_eq!(result.err(), Some(EntryNameError::Empty));
    }

    #[test]
    fn refuses_a_name_that_leaves_the_destination() {
        // WHY: this is the attack the hardening exists for.
        // act
        let names = [
            "../escape.txt",
            "..",
            "docs/../../escape.txt",
            "..\\escape.txt",
            "docs\\..\\..\\escape.txt",
        ];

        // assert
        for name in names {
            let result = resolve_entry_name(name);
            assert!(
                matches!(result, Err(EntryNameError::Traversal { .. })),
                "{name} was accepted"
            );
        }
    }

    #[test]
    fn refuses_an_absolute_name() {
        // act
        let names = [
            "/etc/passwd",
            "\\windows\\system32",
            "C:\\Windows\\evil.dll",
            "d:evil.txt",
        ];

        // assert
        for name in names {
            let result = resolve_entry_name(name);
            assert!(
                matches!(result, Err(EntryNameError::Absolute { .. })),
                "{name} was accepted"
            );
        }
    }

    #[test]
    fn refuses_a_name_with_a_control_character() {
        // act
        let result = resolve_entry_name("evil\u{0}.txt");

        // assert
        assert!(
            matches!(result, Err(EntryNameError::ControlCharacter { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn refuses_a_reserved_device_name() {
        // act
        let names = ["CON", "nul.txt", "docs/PRN", "COM1", "LPT9.log"];

        // assert
        for name in names {
            let result = resolve_entry_name(name);
            assert!(
                matches!(result, Err(EntryNameError::ReservedName { .. })),
                "{name} was accepted"
            );
        }
    }

    #[test]
    fn refuses_an_alternate_data_stream() {
        // act
        let result = resolve_entry_name("readme.txt:hidden");

        // assert
        assert!(
            matches!(result, Err(EntryNameError::ReservedName { .. })),
            "{result:?}"
        );
    }

    #[test]
    fn refuses_a_name_that_is_too_long() {
        // arrange
        let name = "a".repeat(MAX_ENTRY_NAME_BYTES + 1);

        // act
        let result = resolve_entry_name(&name);

        // assert
        assert_eq!(
            result.err(),
            Some(EntryNameError::TooLong {
                length: MAX_ENTRY_NAME_BYTES + 1,
                limit: MAX_ENTRY_NAME_BYTES,
            })
        );
    }

    #[test]
    fn accepts_a_name_of_the_maximum_length() {
        // arrange
        let name = "a".repeat(MAX_ENTRY_NAME_BYTES);

        // act
        let result = resolve_entry_name(&name);

        // assert
        assert!(result.is_ok());
    }
}
