//! The Android library of Phocommander.
//!
//! WHAT: the Rust library that the Gradle project of the Android application
//!       loads. Milestone M0 contains the crate and this architecture note;
//!       the Gradle project arrives in M4.
//! WHY: Android is the target system with the most divergent access rules,
//!       and they decide what the application can offer at all:
//!
//! * **SAF mode (the default).** The Storage Access Framework grants access
//!   to the directories the user selects, without any special permission.
//!   The application works as a document browser in this mode: it shows the
//!   content of the granted roots, copies, moves and opens files there.
//! * **Optional full access.** `MANAGE_EXTERNAL_STORAGE` gives the
//!   application access to the whole storage of the device. Google allows this
//!   for file managers, but it requires a declaration in the manifest, an
//!   explanation in the store entry, a review by the Play Console and an
//!   explicit request to the user. The application therefore works without
//!   the permission and asks for it only when the user wants a directory that
//!   SAF does not offer.
//!
//! The two modes are a property of the *source*, not of the user interface:
//! the same panel code lists a SAF root and a full-access directory.
//!
//! BOUNDARY: this library is the bridge between the Rust core and the Java
//!       or Kotlin shell. It owns the JNI entry points and nothing else; the
//!       file access itself lives in the platform adapter.
//!
//! MILESTONE: M4 — Gradle project, SAF mode, optional full access, the touch
//! and the keyboard context, the adaptive layouts.
//!
//! EXAMPLE:
//! ```no_run
//! // Milestone M4 will look like this:
//! // phocommander_android::list_saf_root(&tree_uri)?;
//! ```
