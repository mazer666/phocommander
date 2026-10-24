//! The resolution of a target path against the directory it belongs to.
//!
//! WHAT: the functions that decide where a file may be written inside a
//!       destination directory — [`resolve_within`], [`normalize_relative`]
//!       and [`is_within`] — together with the two policies they need,
//!       [`CaseSensitivity`] and the error type [`PathError`].
//! WHY: a name that comes from outside — from an archive, from a server, from
//!       a file system the user released on another device — decides where a
//!       file is written. A name that leaves the destination directory is
//!       the oldest attack in this field, and the defence has to live in one
//!       place, otherwise one call site can forget it. The resolution is
//!       *canonicalised*: it asks the file system what the path really is,
//!       so that a symbolic link in the destination cannot be used to write
//!       outside of it.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::path::{CaseSensitivity, resolve_within};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), phocommander_core::path::PathError> {
//! let destination = Path::new("/tmp/photos");
//! let target = resolve_within(destination, Path::new("2026/sunset.jpg"), CaseSensitivity::Sensitive)?;
//! assert!(target.starts_with(destination));
//!
//! // A name that tries to leave the destination is refused.
//! assert!(resolve_within(destination, Path::new("../escape.txt"), CaseSensitivity::Sensitive).is_err());
//! # Ok(())
//! # }
//! ```

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::{Component, Path, PathBuf};

/// How a file system compares two names that differ only in their case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CaseSensitivity {
    /// The file system tells `Report.txt` and `report.txt` apart.
    ///
    /// WHY: Linux and the BSDs behave this way, and the core cannot know
    /// what the file system of the machine it runs on does.
    #[default]
    Sensitive,

    /// The file system treats `Report.txt` and `report.txt` as one name.
    ///
    /// WHY: macOS and Windows behave this way by default. A name that
    /// differs only in the case is the same name there, so a path that
    /// leaves the destination is detected with the same comparison.
    Insensitive,
}

/// The reason why a path was refused.
#[derive(Debug, thiserror::Error)]
pub enum PathError {
    /// The path was absolute where a relative one was required.
    #[error("the path {path} is absolute, a path inside the destination is required")]
    Absolute {
        /// The path that was refused.
        path: PathBuf,
    },

    /// The path tried to leave the destination directory.
    #[error("the path {path} leaves the destination directory")]
    Traversal {
        /// The path that was refused.
        path: PathBuf,
    },

    /// The name is not valid on every target platform.
    #[error("the name {name} is not valid on every target platform")]
    Reserved {
        /// The name that was refused.
        name: String,
    },

    /// The file system refused the operation.
    #[error("the file system refused the operation for {path}: {source}")]
    Io {
        /// The path the operation was working on.
        path: PathBuf,
        /// The error the operating system reported.
        #[source]
        source: io::Error,
    },
}

impl PathError {
    /// The stable code of the failure, as the table of ADR-0003 names it.
    ///
    /// WHY: the user interface matches on the code and shows the message of
    /// its translation catalogue. It must never parse the text of the error.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_core::error::ErrorCode;
    /// use phocommander_core::path::PathError;
    /// use std::path::Path;
    ///
    /// let error = PathError::Traversal { path: Path::new("../escape.txt").to_path_buf() };
    /// assert_eq!(error.code(), ErrorCode::PathTraversal);
    /// ```
    pub fn code(&self) -> crate::error::ErrorCode {
        match self {
            PathError::Absolute { .. } => crate::error::ErrorCode::PathAbsolute,
            PathError::Traversal { .. } => crate::error::ErrorCode::PathTraversal,
            PathError::Reserved { .. } => crate::error::ErrorCode::PathReserved,
            PathError::Io { .. } => crate::error::ErrorCode::IoRead,
        }
    }
}

/// The characters that Windows does not allow in a file name.
///
/// WHY: a name that Windows refuses cannot be created there, and a
/// synchronisation between a Linux disk and a Windows disk has to notice
/// that before it starts, not after it copied a thousand files.
pub const WINDOWS_FORBIDDEN_CHARACTERS: [char; 9] =
    ['<', '>', ':', '"', '/', '\\', '?', '*', '|'];

/// Resolves `relative` against `base` and proves that the result stays inside.
///
/// WHY: this is the function the extraction hardening and the copy
/// operations call before they write anything. It refuses the three ways a
/// path can leave the destination — an absolute path, a `..` segment and a
/// symbolic link — and it resolves the destination itself, so that a link in
/// the destination cannot be used as a way out either.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::path::{CaseSensitivity, resolve_within};
/// use std::ffi::OsStr;
/// use std::path::Path;
///
/// # fn main() -> Result<(), phocommander_core::path::PathError> {
/// let base = Path::new("/tmp/destination");
/// let target = resolve_within(base, Path::new("docs/readme.md"), CaseSensitivity::Sensitive)?;
/// assert_eq!(target.file_name().and_then(OsStr::to_str), Some("readme.md"));
/// # Ok(())
/// # }
/// ```
pub fn resolve_within(
    base: &Path,
    relative: &Path,
    case: CaseSensitivity,
) -> Result<PathBuf, PathError> {
    let normalized = normalize_relative(relative)?;

    let canonical_base = base.canonicalize().map_err(|source| PathError::Io {
        path: base.to_path_buf(),
        source,
    })?;

    let candidate = base.join(&normalized);

    // The deepest directory of the target that exists decides where the
    // write really lands. Everything below it does not exist yet and cannot
    // be a way out.
    let (existing, tail) = split_at_existing(&candidate);
    let canonical_existing = existing.canonicalize().map_err(|source| PathError::Io {
        path: existing.clone(),
        source,
    })?;

    if !is_within(&canonical_existing, &canonical_base, case) {
        return Err(PathError::Traversal {
            path: relative.to_path_buf(),
        });
    }

    // A target that exists and is a link would be written *through* the
    // link. That is the way a destination directory is used to escape it,
    // so the link is refused instead of followed.
    if is_symbolic_link(&candidate) {
        return Err(PathError::Traversal {
            path: relative.to_path_buf(),
        });
    }

    Ok(canonical_existing.join(tail))
}

/// Removes the `.` and the `..` segments of a relative path.
///
/// WHY: a name that arrives from outside may contain segments that carry no
/// information (`./docs`) or that point outwards (`../escape`). The result of
/// this function never contains a `..` segment, so that joining it with a
/// destination cannot leave the destination.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::path::normalize_relative;
/// use std::path::Path;
///
/// # fn main() -> Result<(), phocommander_core::path::PathError> {
/// let normalized = normalize_relative(Path::new("./docs/../readme.md"))?;
/// assert_eq!(normalized.to_string_lossy(), "readme.md");
/// # Ok(())
/// # }
/// ```
pub fn normalize_relative(relative: &Path) -> Result<PathBuf, PathError> {
    if relative.is_absolute() {
        return Err(PathError::Absolute {
            path: relative.to_path_buf(),
        });
    }

    let mut normalized = PathBuf::new();

    for component in relative.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                return Err(PathError::Absolute {
                    path: relative.to_path_buf(),
                });
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(PathError::Traversal {
                        path: relative.to_path_buf(),
                    });
                }
            }
            Component::Normal(part) => {
                let name = part.to_string_lossy().to_string();
                if has_drive_letter(&name) {
                    return Err(PathError::Absolute {
                        path: relative.to_path_buf(),
                    });
                }
                if is_reserved_device_name(&name) {
                    return Err(PathError::Reserved { name });
                }
                if name.chars().any(|character| {
                    WINDOWS_FORBIDDEN_CHARACTERS.contains(&character) || character.is_control()
                }) {
                    return Err(PathError::Reserved { name });
                }
                normalized.push(part);
            }
        }
    }

    Ok(normalized)
}

/// Whether `candidate` lies inside `base`.
///
/// WHY: the comparison has to follow the case policy of the file system.
/// On a case-insensitive file system a path that differs from the base only
/// in its case is still inside it, and a path that shares a prefix but
/// crosses a directory boundary is not.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::path::{CaseSensitivity, is_within};
/// use std::path::Path;
///
/// let base = Path::new("/tmp/destination");
/// assert!(is_within(Path::new("/tmp/destination/a.txt"), base, CaseSensitivity::Sensitive));
/// assert!(!is_within(Path::new("/tmp/destination-other/a.txt"), base, CaseSensitivity::Sensitive));
/// ```
pub fn is_within(candidate: &Path, base: &Path, case: CaseSensitivity) -> bool {
    match case {
        CaseSensitivity::Sensitive => candidate.starts_with(base),
        CaseSensitivity::Insensitive => {
            let candidate_text = candidate.to_string_lossy().to_lowercase();
            let base_text = base.to_string_lossy().to_lowercase();
            Path::new(&candidate_text).starts_with(Path::new(&base_text))
        }
    }
}

/// Whether two names are the same name on a file system with this policy.
///
/// WHY: a rename that only changes the case of a name is a conflict on a
/// case-insensitive file system and not a conflict on a case-sensitive one.
/// The operations ask this question before they report a conflict.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::path::{CaseSensitivity, same_name};
///
/// assert!(same_name("Report.txt", "report.txt", CaseSensitivity::Insensitive));
/// assert!(!same_name("Report.txt", "report.txt", CaseSensitivity::Sensitive));
/// ```
pub fn same_name(left: &str, right: &str, case: CaseSensitivity) -> bool {
    match case {
        CaseSensitivity::Sensitive => left == right,
        CaseSensitivity::Insensitive => left.to_lowercase() == right.to_lowercase(),
    }
}

/// Whether the name is a device name that Windows reserves.
///
/// WHY: Windows refuses to create a file called `CON` or `NUL`, and it does
/// so in a way that silently redirects the write. A name that is reserved on
/// one platform has to be reported before the operation, not after it.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_core::path::is_reserved_device_name;
///
/// assert!(is_reserved_device_name("CON"));
/// assert!(is_reserved_device_name("nul.txt"));
/// assert!(!is_reserved_device_name("concert.txt"));
/// ```
pub fn is_reserved_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
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

/// Whether the name starts with a drive letter such as `C:`.
fn has_drive_letter(name: &str) -> bool {
    let mut characters = name.chars();
    match (characters.next(), characters.next()) {
        (Some(letter), Some(':')) => letter.is_ascii_alphabetic(),
        _ => false,
    }
}

/// Whether the path exists, even if it is a broken symbolic link.
fn path_exists(path: &Path) -> bool {
    path.symlink_metadata().is_ok()
}

/// Whether the path is a symbolic link.
fn is_symbolic_link(path: &Path) -> bool {
    match path.symlink_metadata() {
        Ok(metadata) => metadata.file_type().is_symlink(),
        Err(_) => false,
    }
}

/// Splits a path into the deepest part that exists and the part that does not.
///
/// WHY: the part that exists is the part that can point somewhere else. The
/// part that does not exist yet is created by the write itself and cannot
/// leave the destination.
fn split_at_existing(path: &Path) -> (PathBuf, PathBuf) {
    let mut existing = path.to_path_buf();
    let mut missing: Vec<OsString> = Vec::new();

    loop {
        if path_exists(&existing) {
            break;
        }
        let Some(name) = existing.file_name().map(OsStr::to_os_string) else {
            break;
        };
        missing.push(name);
        let Some(parent) = existing.parent().map(Path::to_path_buf) else {
            break;
        };
        existing = parent;
    }

    let mut tail = existing.clone();
    for name in missing.iter().rev() {
        tail.push(name);
    }

    let relative_tail = tail
        .strip_prefix(&existing)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| PathBuf::new());

    (existing, relative_tail)
}

#[cfg(test)]
mod tests;
