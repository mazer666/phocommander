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
//!
//! MILESTONE: the crate is created in M0 so that the boundary exists from the
//! first day. The registry and the formats of stage 1 — text, hexadecimal
//! dump, images, archive content and metadata — arrive in M1, PDF and syntax
//! highlighting in M2, audio and video in M3.
//!
//! EXAMPLE:
//! ```no_run
//! // Milestone M1 will look like this:
//! // let content = phocommander_viewer::show(&path)?;
//! ```
