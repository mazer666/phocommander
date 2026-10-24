//! The archive adapters of Phocommander.
//!
//! WHAT: the hardened resolution of the names an archive contains
//!       ([`entry_name`]). The container readers — ZIP, TAR and 7z — arrive in
//!       milestone M1.
//! WHY: an archive is untrusted input, exactly like a file that comes from a
//!       network source. The name of an entry decides where the entry is
//!       written. A name that leaves the destination directory, that is an
//!       absolute path or that is a reserved device name on Windows must
//!       therefore never reach the file system. The resolution lives in one
//!       place, so that every reader has to use it and no reader can forget
//!       it.
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

pub mod entry_name;

pub use entry_name::{EntryNameError, MAX_ENTRY_NAME_BYTES, resolve_entry_name};
