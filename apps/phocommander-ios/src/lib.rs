//! The iPadOS library of Phocommander.
//!
//! WHAT: the Rust library that the Xcode project of the iPadOS application
//!       loads. Milestone M0 contains the crate and this architecture note;
//!       the Xcode project arrives in M5.
//! WHY: iPadOS is fully sandboxed, and the access model decides what the
//!       application can offer at all:
//!
//! * **Document picker.** `UIDocumentPickerViewController` returns the
//!   directories the user selects, together with security-scoped URLs. The
//!   permission covers the content of the selected directory recursively.
//! * **Security-scoped bookmarks.** A lasting permission is stored as a
//!   bookmark. The user can revoke it at any time in the settings, and the
//!   application has to handle that error cleanly instead of pretending that
//!   the directory is still there.
//!
//! The consequence is written down here and must not be forgotten later: the
//! iPadOS version is **not** a file system browser. It is a viewer and an
//! organiser for the root directories the user has explicitly released. The
//! two-panel surface stays useful in the split view, with touch for the
//! selection and the keyboard as the main way of operating.
//!
//! BOUNDARY: this library is the bridge between the Rust core and the
//!       Objective-C or Swift shell. It owns the FFI entry points and
//!       nothing else; the file access itself lives in the platform adapter.
//!
//! MILESTONE: M5 — Xcode project, document picker, security-scoped bookmarks,
//!       split view, support for the Apple keyboard.
//!
//! EXAMPLE:
//! ```no_run
//! // Milestone M5 will look like this:
//! // phocommander_ios::resolve_bookmark(&data)?;
//! ```
