//! The commands the work surface asks the application to carry out.
//!
//! WHAT: the requests of the interface ([`Command`]).
//! WHY: the surface draws and it holds the state of the interface. It
//!       performs no file operation itself — the operations belong to the
//!       operations module, and the file system belongs to the source. The
//!       surface therefore hands a request to the shell, and the request
//!       names its source and its destination explicitly (FR-16).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::commands::Command;
//!
//! let command = Command::Quit;
//! assert_eq!(command, Command::Quit);
//! ```

use std::path::PathBuf;

use crate::actions::Action;
use crate::panel::PanelSide;
use crate::rename::RenamePair;
use crate::surface::TransferRequest;

/// A request of the drawing to the shell.
///
/// WHY: the drawing reads the state and never changes it. A click on a
/// button, on a tab or on a node of the tree is therefore a request and
/// not a change — the shell carries it out, exactly as it carries out an
/// action that came from the keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Carry out an action of the catalogue.
    Action(Action),

    /// Show another tab of the panel that has the focus.
    SwitchTab(usize),

    /// Close a tab of the panel that has the focus.
    CloseTab(usize),

    /// Open the directory the user chose in the tree.
    OpenDirectory(PathBuf),
}

/// A request of the interface to the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Copy the entries of one panel into the other panel.
    Copy {
        /// The transfer, with its source and its destination.
        transfer: TransferRequest,
    },

    /// Move the entries of one panel into the other panel.
    Move {
        /// The transfer, with its source and its destination.
        transfer: TransferRequest,
    },

    /// Delete the marked entries of a panel.
    Delete {
        /// The panel the entries belong to.
        side: PanelSide,

        /// The paths of the entries.
        entries: Vec<PathBuf>,
    },

    /// Rename the files of a panel.
    ///
    /// WHY: the command carries the pairs and not only the paths of the
    ///       files. The dialog has already shown the new names to the user
    ///       (FR-34), and the application has to rename exactly the names
    ///       that the user has seen. A command that carried only the paths
    ///       would leave the application to form the names a second time —
    ///       and a second answer can be a different one.
    Rename {
        /// The panel the files belong to.
        side: PanelSide,

        /// The pairs of the rename: the path the file carries now and the
        /// path it receives.
        pairs: Vec<RenamePair>,
    },

    /// Create a directory in a panel.
    CreateDirectory {
        /// The panel the directory belongs to.
        side: PanelSide,

        /// The path of the new directory.
        path: PathBuf,
    },

    /// Open an entry in the other panel.
    OpenInOtherPanel {
        /// The panel the entry belongs to.
        source: PanelSide,

        /// The panel the entry is opened in.
        destination: PanelSide,

        /// The path of the entry.
        entry: PathBuf,
    },

    /// Open an entry in the viewer.
    OpenViewer {
        /// The panel the entry belongs to.
        side: PanelSide,

        /// The path of the entry.
        entry: PathBuf,
    },

    /// Read the directory of a panel.
    LoadDirectory {
        /// The panel whose directory is read.
        side: PanelSide,

        /// The path of the directory.
        path: PathBuf,
    },

    /// Exchange the content of the two panels.
    SwapPanels,

    /// Open the menu of the application.
    Menu,

    /// Leave the application.
    Quit,

    /// The action belongs to a later milestone.
    ///
    /// WHY: the catalogue of the specification holds more actions than this
    /// milestone carries out — the drive list, the pattern selection, the
    /// action palette and the dialogs of the viewer and of the rename. The
    /// surface reports them instead of pretending that they happened.
    Deferred {
        /// The action that was requested.
        action: Action,
    },
}

impl Command {
    /// The action that caused the command, if it was caused by one.
    pub fn action(&self) -> Option<Action> {
        match self {
            Command::Deferred { action } => Some(*action),
            _ => None,
        }
    }
}
