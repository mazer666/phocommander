//! The desktop application of Phocommander.
//!
//! WHAT: the crate of the desktop build — the translation of the
//!       configuration into the rules of the subsystems ([{security}]).
//! WHY: the application is the composition root of the workspace: it is the
//!       only crate that may see the core crate, the configuration crate, the
//!       operations crate, the viewer crate and the interface crate at the
//!       same time, because no crate below `crates/` may depend on a sibling
//!       (ADR-0001, option B). The translation of the configuration into the
//!       rules of the subsystems therefore belongs here and nowhere else.
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::EffectiveSecurity;
//! use phocommander_desktop::security::SecurityPlan;
//!
//! let rules = EffectiveSecurity::safe();
//! let plan = SecurityPlan::from_effective(&rules);
//! assert!(plan.copy_policy().verifies_checksums());
//! ```

pub mod security;

pub use security::SecurityPlan;
