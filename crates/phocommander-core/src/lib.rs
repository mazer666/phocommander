//! The shared vocabulary of Phocommander.
//!
//! WHAT: the model of the file system — the entries of a directory
//!       ([`entry`]), the interfaces a source implements ([`source`]), the
//!       local implementation of those interfaces ([`local`]), the resolution
//!       of a path against its destination directory ([`path`]), the
//!       checksum of a file ([`checksum`]), the canonical configuration
//!       defaults ([`config`]) and the error type of the core layer
//!       ([`error`]).
//! WHY: the core is the only level that every other level may use, and it
//!       knows no window, no widget and no protocol. The concept it owns is
//!       the concept of ADR-0002: the core asks a source for the content of
//!       a directory and the source answers with a list of entries — no
//!       matter whether the source is a local disk, an archive, an SFTP
//!       server or a directory the user released on iPadOS.
//! BOUNDARY: this crate performs no file operation on its own behalf and
//!       knows no operating system API beyond the standard library. The
//!       write side of a source is a second, narrower interface, because an
//!       archive can be read but not modified.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::{Entry, EntryKind, Error};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Error> {
//! let path = Path::new("holiday.mkv");
//! let metadata = std::fs::metadata(path).map_err(|source| Error::io(path, source))?;
//! if metadata.len() > phocommander_core::MAX_VIEWER_FILE_SIZE {
//!     return Err(Error::too_large(path, metadata.len(), phocommander_core::MAX_VIEWER_FILE_SIZE));
//! }
//! let entry = Entry::new("holiday.mkv", EntryKind::File).with_size(metadata.len());
//! println!("{}", entry.name);
//! # Ok(())
//! # }
//! ```

pub mod checksum;
pub mod config;
pub mod entry;
pub mod error;
pub mod local;
pub mod path;
pub mod source;

pub use checksum::{Crc32, crc32, crc32_bytes, crc32_file};
pub use config::{
    MAX_ARCHIVE_ENTRIES, MAX_EXTRACTED_BYTES, MAX_VIEWER_FILE_SIZE, MAX_VIEWER_TEXT_SIZE,
};
pub use entry::{Entry, EntryKind};
pub use error::{Error, ErrorCode, IoOperation};
pub use local::LocalSource;
pub use path::{CaseSensitivity, PathError, resolve_within};
pub use source::{MutableSource, Source};
