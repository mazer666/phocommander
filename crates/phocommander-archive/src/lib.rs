//! The archive adapters of Phocommander.
//!
//! WHAT: the hardened resolution of the names an archive contains
//!       ([{entry_name}]), the rules of the file names of Windows
//!       ([{windows_name}]), the limits of an extraction ([{limits}]), the
//!       container adapters ([{formats}]) and the extraction itself
//!       ([{extract}]).
//! WHY: an archive is untrusted input, exactly like a file that comes from a
//!       network source. The name of an entry decides where the entry is
//!       written. A name that leaves the destination directory, that is an
//!       absolute path or that is a reserved device name on Windows must
//!       therefore never reach the file system. The resolution lives in one
//!       place, so that every reader has to use it and no reader can forget
//!       it.
//!
//! The extraction never calls the {extract()} of a container crate. It
//!       iterates over the entries and writes every entry itself, so that
//!       the decision about *where* something is written comes from this
//!       project and not from a dependency.
//! EXAMPLE:
//! ```no_run
//! use phocommander_archive::resolve_entry_name;
//!
//! # fn main() -> Result<(), phocommander_archive::EntryNameError> {
//! let path = resolve_entry_name("docs/readme.md")?;
//! assert_eq!(path.to_string_lossy(), "docs/readme.md");
//! assert!(resolve_entry_name("../escape.txt").is_err());
//! # Ok(())
//! # }
//! ```

pub mod entry;
pub mod entry_name;
pub mod error;
pub mod extract;
pub mod formats;
pub mod limits;
pub mod windows_name;

pub use entry::{ArchiveEntry, EntryKind};
pub use entry_name::{EntryNameError, MAX_ENTRY_NAME_BYTES, resolve_entry_name};
pub use error::{ArchiveError, LimitKind, RefusalReason};
pub use extract::{
    ConflictPolicy, EntryOutcome, ExtractionOptions, ExtractionReport, SkipReason, extract_archive,
    extract_from,
};
pub use formats::{
    ArchiveReader, ArchiveWriter, ContainerFormat, EntryVisitor, TarCompression, VisitDecision,
};
pub use limits::{
    EXTRACTED_DIRECTORY_MODE, EXTRACTED_FILE_MODE, ExtractionLimits, MAX_ARCHIVE_ENTRIES,
    MAX_EXTRACTED_BYTES, MAX_SINGLE_ENTRY_BYTES,
};
