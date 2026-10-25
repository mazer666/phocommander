//! The action catalogue of the user interface.
//!
//! WHAT: the commands the work surface can carry out ([{Action}]) and the
//!       group they belong to ([{ActionCategory}]).
//! WHY: the interface never asks which key was pressed, it asks which
//!       command was requested (FR-100). The catalogue is therefore the
//!       contract between the input layer, the configuration file and the
//!       surface — and the identifiers are the ones of
//!       {config/keymap.toml}, so that a binding in the file and a command
//!       in the code can never drift apart.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::{Action, ActionCategory};
//!
//! let copy = Action::from_id("panel.copy").expect("the catalogue holds the action");
//! assert_eq!(copy.id(), "panel.copy");
//! assert_eq!(copy.category(), ActionCategory::File);
//! ```

use crate::i18n::TextKey;

/// The group an action belongs to.
///
/// WHY: the group decides where the action appears in the menus and in the
/// action palette. The groups are the ones the specification names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActionCategory {
    /// The operations on the files: rename, copy, move, delete.
    File,

    /// The commands that open or show something.
    View,

    /// The commands that move the focus and change the place.
    Navigate,

    /// The commands that mark and unmark entries.
    Selection,

    /// The commands of the application itself.
    Application,

    /// The commands of the multiple rename dialog.
    Rename,

    /// The commands of the action palette.
    Palette,
}

impl ActionCategory {
    /// Every category, in a stable order.
    pub const fn all() -> [ActionCategory; 7] {
        [
            ActionCategory::File,
            ActionCategory::View,
            ActionCategory::Navigate,
            ActionCategory::Selection,
            ActionCategory::Application,
            ActionCategory::Rename,
            ActionCategory::Palette,
        ]
    }
}

/// One command of the work surface.
///
/// WHY: the order of the catalogue is the order of {config/keymap.toml} and
/// it decides every conflict: when two actions of one context are bound to
/// the same key, the action that comes first here wins. The order is
/// therefore part of the contract and not an accident of the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Action {
    id: &'static str,
    category: ActionCategory,
    label: TextKey,
}

impl Action {
    /// The identifier of the action, as it is written in the configuration
    /// file and in the documentation.
    pub const fn id(&self) -> &'static str {
        self.id
    }

    /// The group the action belongs to.
    pub const fn category(&self) -> ActionCategory {
        self.category
    }

    /// The translation key of the label of the action.
    ///
    /// WHY: the label of an action is a visible text, and a visible text
    /// goes through a translation key (FR-05). The key is part of the action
    /// so that no caller can invent a label of its own.
    pub const fn label(&self) -> TextKey {
        self.label
    }

    /// The action with the given identifier.
    ///
    /// WHY: the identifier comes from the configuration file and from the
    /// input layer. An identifier that the surface does not know is {None},
    /// so that a binding to an unknown action is reported instead of
    /// silently doing nothing.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::Action;
    ///
    /// assert!(Action::from_id("panel.copy").is_some());
    /// assert!(Action::from_id("panel.nonsense").is_none());
    /// ```
    pub fn from_id(id: &str) -> Option<Action> {
        Action::all().iter().copied().find(|action| action.id == id)
    }

    /// Every action of the catalogue, in the order of the configuration file.
    pub fn all() -> &'static [Action] {
        &ACTIONS
    }

    /// The actions of one category, in the order of the catalogue.
    pub fn in_category(category: ActionCategory) -> impl Iterator<Item = Action> {
        Action::all()
            .iter()
            .copied()
            .filter(move |action| action.category == category)
    }
}

/// The catalogue of the actions.
///
/// WHY: the identifiers are the ones of {config/keymap.toml} and of
/// {docs/KEYMAP.md}. The gate of the integration tests compares the three
/// descriptions, so that a new command cannot appear in the code without
/// appearing in the documentation.
const ACTIONS: [Action; 44] = [
    Action {
        id: "panel.rename",
        category: ActionCategory::File,
        label: TextKey::ActionPanelRename,
    },
    Action {
        id: "panel.view",
        category: ActionCategory::View,
        label: TextKey::ActionPanelView,
    },
    Action {
        id: "panel.filter",
        category: ActionCategory::View,
        label: TextKey::ActionPanelFilter,
    },
    Action {
        id: "panel.edit",
        category: ActionCategory::File,
        label: TextKey::ActionPanelEdit,
    },
    Action {
        id: "panel.copy",
        category: ActionCategory::File,
        label: TextKey::ActionPanelCopy,
    },
    Action {
        id: "panel.move",
        category: ActionCategory::File,
        label: TextKey::ActionPanelMove,
    },
    Action {
        id: "panel.mkdir",
        category: ActionCategory::File,
        label: TextKey::ActionPanelMkdir,
    },
    Action {
        id: "panel.delete",
        category: ActionCategory::File,
        label: TextKey::ActionPanelDelete,
    },
    Action {
        id: "app.menu",
        category: ActionCategory::Application,
        label: TextKey::ActionAppMenu,
    },
    Action {
        id: "app.quit",
        category: ActionCategory::Application,
        label: TextKey::ActionAppQuit,
    },
    Action {
        id: "panel.switch",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelSwitch,
    },
    Action {
        id: "panel.reload",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelReload,
    },
    Action {
        id: "panel.swap",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelSwap,
    },
    Action {
        id: "panel.open_in_other",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelOpenInOther,
    },
    Action {
        id: "panel.tab.1",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab1,
    },
    Action {
        id: "panel.tab.2",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab2,
    },
    Action {
        id: "panel.tab.3",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab3,
    },
    Action {
        id: "panel.tab.4",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab4,
    },
    Action {
        id: "panel.tab.5",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab5,
    },
    Action {
        id: "panel.tab.6",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab6,
    },
    Action {
        id: "panel.tab.7",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab7,
    },
    Action {
        id: "panel.tab.8",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab8,
    },
    Action {
        id: "panel.tab.9",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTab9,
    },
    Action {
        id: "panel.next_tab",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelNextTab,
    },
    Action {
        id: "panel.prev_tab",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelPrevTab,
    },
    Action {
        id: "panel.tab.new",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTabNew,
    },
    Action {
        id: "panel.tab.close",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTabClose,
    },
    Action {
        id: "panel.tab.move_to_other",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelTabMoveToOther,
    },
    Action {
        id: "panel.layout.toggle",
        category: ActionCategory::View,
        label: TextKey::ActionPanelLayoutToggle,
    },
    Action {
        id: "panel.tree.toggle",
        category: ActionCategory::View,
        label: TextKey::ActionPanelTreeToggle,
    },
    Action {
        id: "panel.quick_view.toggle",
        category: ActionCategory::View,
        label: TextKey::ActionPanelQuickViewToggle,
    },
    Action {
        id: "panel.job_list.toggle",
        category: ActionCategory::View,
        label: TextKey::ActionPanelJobListToggle,
    },
    Action {
        id: "selection.toggle",
        category: ActionCategory::Selection,
        label: TextKey::ActionSelectionToggle,
    },
    Action {
        id: "selection.all",
        category: ActionCategory::Selection,
        label: TextKey::ActionSelectionAll,
    },
    Action {
        id: "selection.none",
        category: ActionCategory::Selection,
        label: TextKey::ActionSelectionNone,
    },
    Action {
        id: "selection.invert",
        category: ActionCategory::Selection,
        label: TextKey::ActionSelectionInvert,
    },
    Action {
        id: "selection.by_pattern",
        category: ActionCategory::Selection,
        label: TextKey::ActionSelectionByPattern,
    },
    Action {
        id: "panel.drives",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelDrives,
    },
    Action {
        id: "panel.parent",
        category: ActionCategory::Navigate,
        label: TextKey::ActionPanelParent,
    },
    Action {
        id: "viewer.find_next",
        category: ActionCategory::View,
        label: TextKey::ActionViewerFindNext,
    },
    Action {
        id: "viewer.find_previous",
        category: ActionCategory::View,
        label: TextKey::ActionViewerFindPrevious,
    },
    Action {
        id: "rename.conflict_next",
        category: ActionCategory::Rename,
        label: TextKey::ActionRenameConflictNext,
    },
    Action {
        id: "rename.conflict_previous",
        category: ActionCategory::Rename,
        label: TextKey::ActionRenameConflictPrevious,
    },
    Action {
        id: "palette.open",
        category: ActionCategory::Palette,
        label: TextKey::ActionPaletteOpen,
    },
];

/// A group of the menu bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuGroup {
    /// The title of the group.
    pub title: TextKey,

    /// The identifiers of the actions of the group, in the order in which
    /// they are shown.
    pub actions: &'static [&'static str],
}

/// The groups of the menu bar.
///
/// WHY: the menu bar is the place where every command is visible at once.
/// The groups are the ones of the reference product — the file operations,
/// the commands of the panel, the commands of the view and the commands of
/// the application — so that the user finds a command in the place he
/// expects it. Every entry names an action of the catalogue, and the
/// catalogue therefore stays the single place where a command exists.
const MENU_GROUPS: [MenuGroup; 4] = [
    MenuGroup {
        title: TextKey::MenuFile,
        actions: &[
            "panel.rename",
            "panel.view",
            "panel.edit",
            "panel.copy",
            "panel.move",
            "panel.mkdir",
            "panel.delete",
        ],
    },
    MenuGroup {
        title: TextKey::MenuPanel,
        actions: &[
            "panel.switch",
            "panel.reload",
            "panel.swap",
            "panel.open_in_other",
            "panel.tab.new",
            "panel.tab.close",
            "panel.tab.move_to_other",
        ],
    },
    MenuGroup {
        title: TextKey::MenuView,
        actions: &[
            "panel.filter",
            "panel.tree.toggle",
            "panel.quick_view.toggle",
            "panel.job_list.toggle",
            "panel.layout.toggle",
        ],
    },
    MenuGroup {
        title: TextKey::MenuApplication,
        actions: &["app.menu", "app.quit"],
    },
];

/// The actions of the toolbar, in the order in which they are shown.
///
/// WHY: the toolbar carries the commands that are used the most often. The
/// list is short on purpose: a toolbar with thirty buttons is a second
/// menu bar, and the menu bar already exists.
const TOOLBAR_ACTIONS: [&str; 9] = [
    "panel.view",
    "panel.copy",
    "panel.move",
    "panel.mkdir",
    "panel.rename",
    "panel.delete",
    "panel.tab.new",
    "panel.tree.toggle",
    "panel.quick_view.toggle",
];

impl Action {
    /// The groups of the menu bar.
    ///
    /// WHY: the menu bar is built from the catalogue and not from a list of
    /// labels of its own. A command that is added to the catalogue is
    /// therefore reachable through the menu without a second change.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::Action;
    ///
    /// assert!(!Action::menu_groups().is_empty());
    /// ```
    pub fn menu_groups() -> &'static [MenuGroup] {
        &MENU_GROUPS
    }

    /// The actions of the toolbar, in the order in which they are shown.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::Action;
    ///
    /// assert!(!Action::toolbar_actions().is_empty());
    /// ```
    pub fn toolbar_actions() -> &'static [&'static str] {
        &TOOLBAR_ACTIONS
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_identifier_is_unique() {
        // act
        let identifiers: BTreeSet<&str> = Action::all().iter().map(|action| action.id()).collect();

        // assert
        assert_eq!(identifiers.len(), Action::all().len());
    }

    #[test]
    fn finds_an_action_by_its_identifier() {
        // act
        let found = Action::from_id("panel.copy");
        let missing = Action::from_id("panel.nonsense");

        // assert
        assert_eq!(found.map(|action| action.id()), Some("panel.copy"));
        assert!(missing.is_none());
    }

    #[test]
    fn every_action_belongs_to_exactly_one_category() {
        // act
        let counted: usize = ActionCategory::all()
            .iter()
            .map(|category| Action::in_category(*category).count())
            .sum();

        // assert
        assert_eq!(counted, Action::all().len());
    }

    #[test]
    fn the_catalogue_holds_the_actions_of_the_configuration_file() {
        // WHY: the identifiers of the catalogue are the identifiers of
        // `config/keymap.toml`. The list below is that file, and the
        // integration test of the workspace compares it with the file
        // itself.
        // act
        let identifiers: Vec<&str> = Action::all().iter().map(|action| action.id()).collect();

        // assert
        assert_eq!(identifiers.len(), 44);
        assert!(identifiers.contains(&"palette.open"));
        assert!(identifiers.contains(&"panel.tab.9"));
        assert!(identifiers.contains(&"rename.conflict_previous"));
        assert!(identifiers.contains(&"panel.filter"));
        assert!(identifiers.contains(&"panel.tab.new"));
        assert!(identifiers.contains(&"panel.tab.close"));
        assert!(identifiers.contains(&"panel.tab.move_to_other"));
        assert!(identifiers.contains(&"panel.layout.toggle"));
        assert!(identifiers.contains(&"panel.tree.toggle"));
        assert!(identifiers.contains(&"panel.quick_view.toggle"));
        assert!(identifiers.contains(&"panel.job_list.toggle"));
    }

    #[test]
    fn every_entry_of_the_menu_and_of_the_toolbar_names_an_action() {
        // WHY: a menu entry that names an action the catalogue does not
        // hold would be a button that does nothing.
        // act
        let mut named: Vec<&str> = Vec::new();
        for group in Action::menu_groups() {
            named.extend(group.actions.iter().copied());
        }
        named.extend(Action::toolbar_actions().iter().copied());

        // assert
        for identifier in named {
            assert!(
                Action::from_id(identifier).is_some(),
                "the catalogue does not hold the action {identifier}"
            );
        }
    }

    #[test]
    fn every_action_has_a_label_key() {
        // act
        let keys: BTreeSet<&str> = Action::all()
            .iter()
            .map(|action| action.label().as_str())
            .collect();

        // assert
        assert_eq!(keys.len(), Action::all().len());
    }
}
