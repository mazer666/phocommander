//! The user interface of Phocommander.
//!
//! WHAT: the crate that draws the two panels, the theme and the dialogs.
//! WHY: egui draws its widgets itself, so the application can use one code
//!       path on all six platforms. The state of a directory list is kept
//!       outside of the immediate mode context, so that a list with a
//!       hundred thousand entries is not rebuilt in every frame.
//! BOUNDARY: this crate draws and it holds the state of the interface. It
//!       performs no file operation itself, it opens no network connection
//!       and it does not know how the platform spells a menu. Every operation
//!       leaves the surface as a [`Command`] with its source and its
//!       destination named.
//!
//! MILESTONE: the crate is created in M0, the theme in M0 as well. The two
//! panels, the keyboard operation and the action palette arrive in M1 and
//! M2.
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::{visuals, Theme};
//!
//! let visuals = visuals(Theme::Dark, false);
//! assert!(visuals.dark_mode);
//! ```

pub mod actions;
pub mod app;
pub mod commands;
pub mod context;
pub mod draw;
pub mod filter;
pub mod i18n;
pub mod input;
pub mod keymap_view;
pub mod listing;
pub mod metrics;
pub mod panel;
pub mod refresh;
pub mod surface;
pub mod theme;

pub use actions::{Action, ActionCategory};
pub use app::WorkSurfaceApp;
pub use commands::Command;
pub use context::Context;
pub use filter::{Filter, FilterDepth, FilterField, FilterKind, FilterPattern};
pub use i18n::{Argument, Catalogue, TextKey, Texts};
pub use input::PrimaryModifier;
pub use keymap_view::{Binding, BindingError, InputEvent, Key, KeyStroke, KeymapView, Modifiers};
pub use listing::{Entry, EntryKind, ListingState, Sort, SortDirection, SortKey};
pub use metrics::{Metrics, Platform};
pub use panel::{Column, PanelSide, PanelState, Selection, Tab, ViewMode};
pub use refresh::RefreshPolicy;
pub use surface::{MarkedEntry, PanelError, PanelPair, TransferRequest, WorkSurface};
pub use theme::{Palette, Theme, contrast_ratio, visuals};
