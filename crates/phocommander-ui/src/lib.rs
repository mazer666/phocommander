//! The user interface of Phocommander.
//!
//! WHAT: the crate that draws the two panels, the theme and the dialogs with
//!       egui.
//! WHY: egui draws its widgets itself, so the application can use one code
//!       path on all six platforms. The state of a directory list is kept
//!       outside of the immediate mode context, so that a list with a
//!       hundred thousand entries is not rebuilt in every frame.
//! BOUNDARY: this crate draws and it holds the state of the interface. It
//!       performs no file operation itself, it opens no network connection
//!       and it does not know how the platform spells a menu.
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

pub mod theme;

pub use theme::{Theme, visuals};
