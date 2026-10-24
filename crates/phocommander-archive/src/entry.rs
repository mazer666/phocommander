//! The entries of an archive, as the container formats report them.
//!
//! WHAT: the kind of an entry, its name, its size, the mode bits it carries
//!       and, for a link, the target it points to.
//! WHY: the adapters of the container formats differ in their details — the
//!       ZIP format keeps the mode in the external attributes, the TAR format
//!       in the header, the 7z format has no mode at all. The extraction must
//!       not know about those differences, so every adapter translates its
//!       format into this one model.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::{ArchiveEntry, EntryKind};
//!
//! let entry = ArchiveEntry {
//!     name: "docs/readme.md".to_string(),
//!     kind: EntryKind::File,
//!     size: 42,
//!     mode: Some(0o100_644),
//!     link_target: None,
//!     name_is_lossy: false,
//! };
//! assert!(entry.is_file());
//! ```

/// What an entry of an archive is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
/// A regular file with content.
    File,

/// A directory.
    Directory,

/// A symbolic link.
    Symlink,

/// Something else: a hard link, a device node, a named pipe.
    Other,
}

/// One entry of an archive, as the container format reports it.
///
/// WHY: every field is untrusted. The name decides where the entry is
///       written, the size decides how much space it may take, the mode bits
///       decide the permissions of the file — and all three are chosen by
///       whoever built the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
/// The name of the entry, as the container carries it.
    pub name: String,

/// Whether the name could be decoded without a loss.
///
/// WHY: a container can carry a name that is not valid UTF-8. Such a
///       name is shown in the report with a replacement character, but
///       the entry is refused instead of being written under a name that
///       the archive does not carry.
    pub name_is_lossy: bool,

/// What the entry is.
    pub kind: EntryKind,

/// The number of bytes the entry announces.
    pub size: u64,

/// The mode bits the entry carries, if the format has them.
    pub mode: Option<u32>,

/// The target of the link, if the entry is a link.
    pub link_target: Option<String>,
}

impl ArchiveEntry {
/// Whether the entry is a regular file.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::{ArchiveEntry, EntryKind};
///
/// # let entry = ArchiveEntry {
/// #     name: "a.txt".to_string(),
/// #     name_is_lossy: false,
/// #     kind: EntryKind::File,
/// #     size: 0,
/// #     mode: None,
/// #     link_target: None,
/// # };
/// assert!(entry.is_file());
/// ```
    pub const fn is_file(&self) -> bool {
        matches!(self.kind, EntryKind::File)
    }

/// Whether the entry is a directory.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::{ArchiveEntry, EntryKind};
///
/// # let entry = ArchiveEntry {
/// #     name: "docs".to_string(),
/// #     name_is_lossy: false,
/// #     kind: EntryKind::Directory,
/// #     size: 0,
/// #     mode: None,
/// #     link_target: None,
/// # };
/// assert!(entry.is_directory());
/// ```
    pub const fn is_directory(&self) -> bool {
        matches!(self.kind, EntryKind::Directory)
    }

/// Whether the entry is a symbolic link.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_archive::{ArchiveEntry, EntryKind};
///
/// # let entry = ArchiveEntry {
/// #     name: "link".to_string(),
/// #     name_is_lossy: false,
/// #     kind: EntryKind::Symlink,
/// #     size: 0,
/// #     mode: None,
/// #     link_target: Some("../outside".to_string()),
/// # };
/// assert!(entry.is_symlink());
/// ```
    pub const fn is_symlink(&self) -> bool {
        matches!(self.kind, EntryKind::Symlink)
    }
}
