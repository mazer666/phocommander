//! The bridge between the Rust core and the shells of the mobile systems.
//!
//! WHAT: the C interface that the shell of Android and the shell of
//!       iPadOS call, and the rules of the documents that they ask for
//!       ([`rules`], [`bridge`]).
//! WHY: both shells speak the language of their system — Java and Swift —
//!       and the rules of the product live in the Rust core. The bridge is
//!       the one place where the two meet, and it is one crate for both:
//!       a copy of the rules in two places drifts apart.
//!
//! BOUNDARY: this crate owns the entry points and the translation of the
//!       documents. It performs no file access and knows no system — the
//!       storage of Android and the document picker of iPadOS live in their
//!       shells.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_bridge::rules::Listing;
//!
//! let listing = Listing::of(&[("root", "Photos", true, false)]);
//! assert_eq!(listing.len(), 1);
//! ```

pub mod bridge;
pub mod rules;
