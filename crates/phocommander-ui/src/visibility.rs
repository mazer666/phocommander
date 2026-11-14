//! What the user wants to see of a directory.
//!
//! WHAT: the options that decide whether the hidden files and the files of
//!       the system stand in a panel ([`ListingOptions`]).
//! WHY: the user writes in the configuration file whether the hidden files
//!       and the files of the system shall be shown. A setting that nobody
//!       reads is worse than no setting: the user writes the value, sees
//!       nothing change, and concludes that the program is broken. The
//!       setting reaches the reading of the directory and decides there
//!       which entries the panel gets (FR-17, FR-74).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::visibility::ListingOptions;
//!
//! let options = ListingOptions::ORDINARY;
//! assert!(!options.accepts(".hidden"));
//! assert!(options.accepts("notes.txt"));
//! ```

/// What the user wants to see of a directory.
///
/// WHY: the user writes in the configuration file whether the hidden files
///       and the files of the system shall be shown. A setting that nobody
///       reads is worse than no setting: the user writes the value, sees
///       nothing change, and concludes that the program is broken. The
///       setting reaches the reading of the directory and decides there
///       which entries the panel gets (FR-17, FR-74).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListingOptions {
    /// Whether the entries whose name begins with a point are shown.
    pub show_hidden: bool,

    /// Whether the files of the system are shown.
    pub show_system: bool,
}

impl ListingOptions {
    /// The options of a user who wants to see what is there.
    pub const ALL: ListingOptions = ListingOptions {
        show_hidden: true,
        show_system: true,
    };

    /// The options of a user who wants to see the ordinary files only.
    pub const ORDINARY: ListingOptions = ListingOptions {
        show_hidden: false,
        show_system: false,
    };

    /// Whether the entry with the given name belongs in the panel.
    ///
    /// WHY: a hidden file is a file whose name begins with a point — that
    ///       is the rule on Unix and on macOS, and the rule of Windows for
    ///       the names that the shell hides. A file of the system is a file
    ///       that the system writes for itself and that the user never
    ///       made: the list of them is short and it is the same on every
    ///       platform.
    pub fn accepts(&self, name: &str) -> bool {
        if is_file_of_the_system(name) {
            return self.show_system;
        }
        if name.starts_with('.') {
            return self.show_hidden;
        }
        true
    }
}

/// The names of the files that the system writes for itself.
///
/// WHY: the list is the same on every platform, and a file of the system
///       is noise in a panel. The names are the ones that the systems
///       really write.
const FILES_OF_THE_SYSTEM: [&str; 4] = [".DS_Store", "Thumbs.db", "desktop.ini", ".localized"];

/// Whether the entry with the given name is a file of the system.
pub fn is_file_of_the_system(name: &str) -> bool {
    FILES_OF_THE_SYSTEM.contains(&name)
}

/// What the right mouse button does.
///
/// WHY: the user writes in the configuration file whether the right mouse
///       button opens a menu or marks the file under the pointer. The
///       setting reaches the surface, and the surface is what the shell
///       asks before it carries the action out (FR-17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RightClickAction {
    /// A menu of the application opens.
    ///
    /// WHY: the menu of the application is a native menu of the system, and
    ///       it belongs to the milestone of the platform integration
    ///       (FR-28). Until then the shell says that the menu comes later.
    #[default]
    ContextMenu,

    /// The file under the pointer is marked, or unmarked.
    ToggleSelection,
}
