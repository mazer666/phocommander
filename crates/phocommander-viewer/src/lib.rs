//! The file viewer of Phocommander.
//!
//! WHAT: the crate that shows the content of a file inside the application —
//!       text, hexadecimal dump, images, the content of an archive and the
//!       metadata of a file.
//! WHY: the viewer keeps the user inside the application instead of sending
//!       them to another program. The formats are registered through a trait:
//!       a new format is a new implementation plus one registration line, and
//!       no change in the core.
//! BOUNDARY: this crate reads files and produces something that can be drawn.
//!       It does not draw, it does not write files and it knows no platform.
//!       It reads a file only when somebody asked for it — the reader
//!       ([{reader}]) is the place where that promise of the specification
//!       (section 9.7) is kept.
//!
//! MILESTONE: the crate is created in M0 so that the boundary exists from the
//! first day. The registry and the formats of stage 1 — text, hexadecimal
//! dump, images, archive content and metadata — arrive in M1, PDF and syntax
//! highlighting in M2, audio and video in M3.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_viewer::policy::ReadPolicy;
//! use phocommander_viewer::reader::ViewerReader;
//! use std::path::Path;
//!
//! // The reader of the viewer opens the file the user asked for.
//! let reader = ViewerReader::new(ReadPolicy::strict(), 10_485_760);
//! let _ = reader.open(Path::new("/notes.txt"));
//! ```

pub mod content;
pub mod policy;
pub mod reader;

pub use content::OpenedFile;
pub use policy::{ReadPolicy, ReadTrigger};
pub use reader::{ViewerError, ViewerReader};
