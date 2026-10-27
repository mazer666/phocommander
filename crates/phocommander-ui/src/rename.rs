//! The dialog of the multiple rename.
//!
//! WHAT: the state of the dialog ([`RenameDialog`]), the parts of it that
//!       have the focus ([`RenameFocus`]), the rule that the dialog edits
//!       ([`RenameRule`]), the seam to the engine that forms the new names
//!       ([`RenameEngine`]) and the named templates ([`NamedTemplate`],
//!       [`TemplateStore`]).
//! WHY: the dialog shows a live preview of every new name before the rename
//!       runs (FR-34), it marks the conflicts and offers them as a jump list
//!       (FR-35), and it stores the named templates (FR-36). The engine that
//!       forms the names lives in the crate of the operations, and the
//!       interface crate may not depend on a sibling crate of the same level.
//!       The seam is therefore a trait, and the composition root hands the
//!       implementation over — the same arrangement as the texts of the
//!       catalogue and the formats of a locale.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::PanelSide;
//! use phocommander_ui::rename::{RenameDialog, RenameFile};
//!
//! let dialog = RenameDialog::new(
//!     PanelSide::Left,
//!     vec![RenameFile::new("/photos/sunset.jpg")],
//!     Vec::new(),
//! );
//! assert_eq!(dialog.files().len(), 1);
//! assert!(dialog.preview().is_none());
//! ```

pub mod dialect;
pub(crate) mod dialog;
pub(crate) mod draw;
pub mod engine;
pub(crate) mod fields;
pub(crate) mod focus;
pub mod rule;
pub mod template;

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests;

pub use dialect::Dialect;
pub use dialog::RenameDialog;
pub use engine::{
    Conflict, ConflictKind, PreviewProblem, PreviewRow, ProblemKind, RenameEngine, RenameFile,
    RenamePair, RenamePreview, RenameRequest,
};
pub use focus::RenameFocus;
pub use rule::{CounterOrder, CounterRule, RenameRule, ReplaceRule, SeriesRule};
pub use template::{
    NamedTemplate, TemplateStore, TemplateStoreError, decode_templates, encode_templates,
};
