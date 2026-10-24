//! The shared vocabulary of Phocommander.
//!
//! WHAT: the canonical configuration defaults ([`config`]) and the error type
//!       of the core layer ([`error`]). Nothing else lives here.
//! WHY: the workspace is flat — no crate below `crates/` may depend on a
//!       sibling. Everything that two crates have to agree on is therefore
//!       either a plain [`std`] type or a value that the composition root
//!       (`apps/phocommander-desktop`) injects. The limits are collected in
//!       one module so that exactly one place decides how large a file may
//!       become before the viewer refuses it.
//! EXAMPLE:
//! ```no_run
//! use phocommander_core::{config::MAX_VIEWER_FILE_SIZE, Error};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Error> {
//! let path = Path::new("holiday.mkv");
//! let metadata = std::fs::metadata(path).map_err(|source| Error::io(path, source))?;
//! if metadata.len() > MAX_VIEWER_FILE_SIZE {
//!     return Err(Error::too_large(path, metadata.len(), MAX_VIEWER_FILE_SIZE));
//! }
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;

pub use config::{
    MAX_ARCHIVE_ENTRIES, MAX_EXTRACTED_BYTES, MAX_VIEWER_FILE_SIZE, MAX_VIEWER_TEXT_SIZE,
};
pub use error::Error;
