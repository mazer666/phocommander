//! The translation layer of the work surface.
//!
//! WHAT: the translation keys the work surface uses ([`TextKey`]), the
//!       source of the translated texts ([`Catalogue`]) and the arguments a
//!       message may carry ([`Argument`]).
//! WHY: every visible text of the interface goes through a translation key
//!       (FR-05). The surface therefore never holds a literal: it asks the
//!       catalogue for a key, and the answer is never empty — a missing
//!       translation falls back to English and then to the key itself
//!       (FR-122).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::{Catalogue, TextKey, Texts};
//!
//! # struct German;
//! # impl Catalogue for German {
//! #     fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
//! #         None
//! #     }
//! # }
//! use phocommander_ui::Argument;
//!
//! let catalogue = German;
//! let texts = Texts::new(&catalogue);
//! assert_eq!(texts.get(TextKey::CommonCancel), "common-cancel");
//! ```

/// An argument of a translated message.
///
/// WHY: a file manager needs plural forms ("1 entry" against "2 entries").
/// The count is therefore an argument of the message and not a part of the
/// text, so that the plural rule stays in the catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Argument<'a> {
    /// A number.
    Number(u64),

    /// A text.
    Text(&'a str),
}

/// The source of the translated texts of the interface.
///
/// WHY: the surface must not know how a catalogue is stored — that is the
/// business of the localisation module. The trait is the seam between the
/// two, and it is implemented by the shell that owns the catalogue.
pub trait Catalogue {
    /// The text of a key with the given arguments, or `None` when the
    /// catalogue does not know the key.
    fn lookup_with(&self, key: &str, arguments: &[Argument<'_>]) -> Option<String>;

    /// The text of a key without arguments.
    fn lookup(&self, key: &str) -> Option<String> {
        self.lookup_with(key, &[])
    }
}

/// The translation keys of the work surface.
///
/// WHY: the key is the name of a message in the catalogue. Listing the keys
/// in the code makes the visible texts of the interface findable, and the
/// gate of the workspace compares the list with the catalogues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextKey {
    /// The label of the action `action-app-menu`.
    ActionAppMenu,
    /// The label of the action `action-app-quit`.
    ActionAppQuit,
    /// The label of the action `action-palette-open`.
    ActionPaletteOpen,
    /// The label of the action `action-panel-copy`.
    ActionPanelCopy,
    /// The label of the action `action-panel-delete`.
    ActionPanelDelete,
    /// The label of the action `action-panel-drives`.
    ActionPanelDrives,
    /// The label of the action `action-panel-edit`.
    ActionPanelEdit,
    /// The label of the action `action-panel-mkdir`.
    ActionPanelMkdir,
    /// The label of the action `action-panel-move`.
    ActionPanelMove,
    /// The label of the action `action-panel-next-tab`.
    ActionPanelNextTab,
    /// The label of the action `action-panel-open-in-other`.
    ActionPanelOpenInOther,
    /// The label of the action `action-panel-parent`.
    ActionPanelParent,
    /// The label of the action `action-panel-prev-tab`.
    ActionPanelPrevTab,
    /// The label of the action `action-panel-rename`.
    ActionPanelRename,
    /// The label of the action `action-panel-reload`.
    ActionPanelReload,
    /// The label of the action `action-panel-swap`.
    ActionPanelSwap,
    /// The label of the action `action-panel-switch`.
    ActionPanelSwitch,
    /// The label of the action `action-panel-tab-1`.
    ActionPanelTab1,
    /// The label of the action `action-panel-tab-2`.
    ActionPanelTab2,
    /// The label of the action `action-panel-tab-3`.
    ActionPanelTab3,
    /// The label of the action `action-panel-tab-4`.
    ActionPanelTab4,
    /// The label of the action `action-panel-tab-5`.
    ActionPanelTab5,
    /// The label of the action `action-panel-tab-6`.
    ActionPanelTab6,
    /// The label of the action `action-panel-tab-7`.
    ActionPanelTab7,
    /// The label of the action `action-panel-tab-8`.
    ActionPanelTab8,
    /// The label of the action `action-panel-tab-9`.
    ActionPanelTab9,
    /// The label of the action `action-panel-view`.
    ActionPanelView,
    /// The label of the action `action-rename-conflict-next`.
    ActionRenameConflictNext,
    /// The label of the action `action-rename-conflict-previous`.
    ActionRenameConflictPrevious,
    /// The label of the action `action-selection-all`.
    ActionSelectionAll,
    /// The label of the action `action-selection-by-pattern`.
    ActionSelectionByPattern,
    /// The label of the action `action-selection-invert`.
    ActionSelectionInvert,
    /// The label of the action `action-selection-none`.
    ActionSelectionNone,
    /// The label of the action `action-selection-toggle`.
    ActionSelectionToggle,
    /// The label of the action `action-viewer-find-next`.
    ActionViewerFindNext,
    /// The label of the action `action-viewer-find-previous`.
    ActionViewerFindPrevious,
    /// The word "OK".
    CommonOk,
    /// The word "Cancel".
    CommonCancel,
    /// The word "Yes".
    CommonYes,
    /// The word "No".
    CommonNo,
    /// The word "Close".
    CommonClose,
    /// The title of the left panel.
    PanelTitleLeft,
    /// The title of the right panel.
    PanelTitleRight,
    /// The header of the column with the names.
    PanelColumnName,
    /// The header of the column with the sizes.
    PanelColumnSize,
    /// The header of the column with the dates.
    PanelColumnDate,
    /// The header of the column with the types.
    PanelColumnType,
    /// The name of the type of a regular file.
    PanelTypeFile,
    /// The name of the type of a directory.
    PanelTypeDirectory,
    /// The name of the type of a symbolic link.
    PanelTypeLink,
    /// The name of the type of everything else.
    PanelTypeOther,
    /// The text of a panel without entries.
    PanelEmpty,
    /// The state of a panel that waits for the next command.
    PanelStatusReady,
    /// The number of entries of a panel.
    PanelStatusEntries,
    /// The number of marked entries of a panel.
    PanelStatusSelected,
    /// The label of a tab.
    PanelTab,
    /// The name of the detailed view.
    PanelViewDetails,
    /// The name of the compact view.
    PanelViewCompact,
    /// The label of the panel that has the focus.
    PanelFocus,
    /// The title of the viewer.
    ViewerTitle,
    /// The label of the search forward in the viewer.
    ViewerFindNext,
    /// The label of the search backward in the viewer.
    ViewerFindPrevious,
    /// The text of a viewer without a file.
    ViewerNoFile,
    /// The title of the action palette.
    PaletteTitle,
    /// The placeholder of the search field of the action palette.
    PaletteSearch,
    /// The text of an action palette without a match.
    PaletteEmpty,
}

impl TextKey {
    /// The name of the message in the catalogue.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::TextKey;
    ///
    /// assert_eq!(TextKey::CommonCancel.as_str(), "common-cancel");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            TextKey::ActionAppMenu => "action-app-menu",
            TextKey::ActionAppQuit => "action-app-quit",
            TextKey::ActionPaletteOpen => "action-palette-open",
            TextKey::ActionPanelCopy => "action-panel-copy",
            TextKey::ActionPanelDelete => "action-panel-delete",
            TextKey::ActionPanelDrives => "action-panel-drives",
            TextKey::ActionPanelEdit => "action-panel-edit",
            TextKey::ActionPanelMkdir => "action-panel-mkdir",
            TextKey::ActionPanelMove => "action-panel-move",
            TextKey::ActionPanelNextTab => "action-panel-next-tab",
            TextKey::ActionPanelOpenInOther => "action-panel-open-in-other",
            TextKey::ActionPanelParent => "action-panel-parent",
            TextKey::ActionPanelPrevTab => "action-panel-prev-tab",
            TextKey::ActionPanelRename => "action-panel-rename",
            TextKey::ActionPanelReload => "action-panel-reload",
            TextKey::ActionPanelSwap => "action-panel-swap",
            TextKey::ActionPanelSwitch => "action-panel-switch",
            TextKey::ActionPanelTab1 => "action-panel-tab-1",
            TextKey::ActionPanelTab2 => "action-panel-tab-2",
            TextKey::ActionPanelTab3 => "action-panel-tab-3",
            TextKey::ActionPanelTab4 => "action-panel-tab-4",
            TextKey::ActionPanelTab5 => "action-panel-tab-5",
            TextKey::ActionPanelTab6 => "action-panel-tab-6",
            TextKey::ActionPanelTab7 => "action-panel-tab-7",
            TextKey::ActionPanelTab8 => "action-panel-tab-8",
            TextKey::ActionPanelTab9 => "action-panel-tab-9",
            TextKey::ActionPanelView => "action-panel-view",
            TextKey::ActionRenameConflictNext => "action-rename-conflict-next",
            TextKey::ActionRenameConflictPrevious => "action-rename-conflict-previous",
            TextKey::ActionSelectionAll => "action-selection-all",
            TextKey::ActionSelectionByPattern => "action-selection-by-pattern",
            TextKey::ActionSelectionInvert => "action-selection-invert",
            TextKey::ActionSelectionNone => "action-selection-none",
            TextKey::ActionSelectionToggle => "action-selection-toggle",
            TextKey::ActionViewerFindNext => "action-viewer-find-next",
            TextKey::ActionViewerFindPrevious => "action-viewer-find-previous",
            TextKey::CommonOk => "common-ok",
            TextKey::CommonCancel => "common-cancel",
            TextKey::CommonYes => "common-yes",
            TextKey::CommonNo => "common-no",
            TextKey::CommonClose => "common-close",
            TextKey::PanelTitleLeft => "panel-title-left",
            TextKey::PanelTitleRight => "panel-title-right",
            TextKey::PanelColumnName => "panel-column-name",
            TextKey::PanelColumnSize => "panel-column-size",
            TextKey::PanelColumnDate => "panel-column-date",
            TextKey::PanelColumnType => "panel-column-type",
            TextKey::PanelTypeFile => "panel-type-file",
            TextKey::PanelTypeDirectory => "panel-type-directory",
            TextKey::PanelTypeLink => "panel-type-link",
            TextKey::PanelTypeOther => "panel-type-other",
            TextKey::PanelEmpty => "panel-empty",
            TextKey::PanelStatusReady => "panel-status-ready",
            TextKey::PanelStatusEntries => "panel-status-entries",
            TextKey::PanelStatusSelected => "panel-status-selected",
            TextKey::PanelTab => "panel-tab",
            TextKey::PanelViewDetails => "panel-view-details",
            TextKey::PanelViewCompact => "panel-view-compact",
            TextKey::PanelFocus => "panel-focus",
            TextKey::ViewerTitle => "viewer-title",
            TextKey::ViewerFindNext => "viewer-find-next",
            TextKey::ViewerFindPrevious => "viewer-find-previous",
            TextKey::ViewerNoFile => "viewer-no-file",
            TextKey::PaletteTitle => "palette-title",
            TextKey::PaletteSearch => "palette-search",
            TextKey::PaletteEmpty => "palette-empty",
        }
    }

    /// Every key of the work surface, in a stable order.
    pub fn all() -> &'static [TextKey] {
        &KEYS
    }
}

/// The keys of the work surface, in a stable order.
///
/// WHY: the list is the contract between the code and the catalogue. The
/// gate of the workspace compares it with the key sets of the catalogues, so
/// that a key cannot be used without a translation.
const KEYS: [TextKey; 66] = [
    TextKey::ActionAppMenu,
    TextKey::ActionAppQuit,
    TextKey::ActionPaletteOpen,
    TextKey::ActionPanelCopy,
    TextKey::ActionPanelDelete,
    TextKey::ActionPanelDrives,
    TextKey::ActionPanelEdit,
    TextKey::ActionPanelMkdir,
    TextKey::ActionPanelMove,
    TextKey::ActionPanelNextTab,
    TextKey::ActionPanelOpenInOther,
    TextKey::ActionPanelParent,
    TextKey::ActionPanelPrevTab,
    TextKey::ActionPanelRename,
    TextKey::ActionPanelReload,
    TextKey::ActionPanelSwap,
    TextKey::ActionPanelSwitch,
    TextKey::ActionPanelTab1,
    TextKey::ActionPanelTab2,
    TextKey::ActionPanelTab3,
    TextKey::ActionPanelTab4,
    TextKey::ActionPanelTab5,
    TextKey::ActionPanelTab6,
    TextKey::ActionPanelTab7,
    TextKey::ActionPanelTab8,
    TextKey::ActionPanelTab9,
    TextKey::ActionPanelView,
    TextKey::ActionRenameConflictNext,
    TextKey::ActionRenameConflictPrevious,
    TextKey::ActionSelectionAll,
    TextKey::ActionSelectionByPattern,
    TextKey::ActionSelectionInvert,
    TextKey::ActionSelectionNone,
    TextKey::ActionSelectionToggle,
    TextKey::ActionViewerFindNext,
    TextKey::ActionViewerFindPrevious,
    TextKey::CommonOk,
    TextKey::CommonCancel,
    TextKey::CommonYes,
    TextKey::CommonNo,
    TextKey::CommonClose,
    TextKey::PanelTitleLeft,
    TextKey::PanelTitleRight,
    TextKey::PanelColumnName,
    TextKey::PanelColumnSize,
    TextKey::PanelColumnDate,
    TextKey::PanelColumnType,
    TextKey::PanelTypeFile,
    TextKey::PanelTypeDirectory,
    TextKey::PanelTypeLink,
    TextKey::PanelTypeOther,
    TextKey::PanelEmpty,
    TextKey::PanelStatusReady,
    TextKey::PanelStatusEntries,
    TextKey::PanelStatusSelected,
    TextKey::PanelTab,
    TextKey::PanelViewDetails,
    TextKey::PanelViewCompact,
    TextKey::PanelFocus,
    TextKey::ViewerTitle,
    TextKey::ViewerFindNext,
    TextKey::ViewerFindPrevious,
    TextKey::ViewerNoFile,
    TextKey::PaletteTitle,
    TextKey::PaletteSearch,
    TextKey::PaletteEmpty,
];

/// The texts of the interface, read from a catalogue.
///
/// WHY: the helper owns the promise of the project that a visible text is
/// never empty. A catalogue that does not know a key is answered with the
/// key itself, which is visible and can be diagnosed.
#[derive(Clone, Copy)]
pub struct Texts<'a> {
    catalogue: &'a dyn Catalogue,
}

impl<'a> Texts<'a> {
    /// Reads the texts from a catalogue.
    pub fn new(catalogue: &'a dyn Catalogue) -> Self {
        Self { catalogue }
    }

    /// The text of a key, or the key itself when no catalogue knows it.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::{Catalogue, TextKey, Texts};
    /// use phocommander_ui::Argument;
    ///
    /// # struct Empty;
    /// # impl Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    ///
    /// let texts = Texts::new(&Empty);
    /// assert_eq!(texts.get(TextKey::CommonOk), "common-ok");
    /// ```
    pub fn get(&self, key: TextKey) -> String {
        self.get_with(key, &[])
    }

    /// The text of a key with the given arguments, or the key itself.
    pub fn get_with(&self, key: TextKey, arguments: &[Argument<'_>]) -> String {
        match self.catalogue.lookup_with(key.as_str(), arguments) {
            Some(text) if !text.is_empty() => text,
            _ => key.as_str().to_string(),
        }
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// A catalogue that knows nothing — the worst case of the fallback.
    struct Empty;

    impl Catalogue for Empty {
        fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
            None
        }
    }

    /// A catalogue that answers every key with the key itself.
    struct Echo;

    impl Catalogue for Echo {
        fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
            Some(key.to_string())
        }
    }

    #[test]
    fn every_key_has_a_unique_name() {
        // act
        let names: BTreeSet<&str> = TextKey::all().iter().map(TextKey::as_str).collect();

        // assert
        assert_eq!(names.len(), TextKey::all().len());
    }

    #[test]
    fn a_missing_translation_falls_back_to_the_key() {
        // arrange
        let texts = Texts::new(&Empty);

        // act
        let text = texts.get(TextKey::CommonCancel);

        // assert
        assert_eq!(text, "common-cancel");
        assert!(!text.is_empty());
    }

    #[test]
    fn an_empty_translation_falls_back_to_the_key() {
        // WHY: a catalogue that answers with an empty string would draw an
        // empty label, which cannot be diagnosed.
        // arrange
        struct Blank;
        impl Catalogue for Blank {
            fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
                Some(String::new())
            }
        }
        let texts = Texts::new(&Blank);

        // act
        let text = texts.get(TextKey::CommonCancel);

        // assert
        assert_eq!(text, "common-cancel");
    }

    #[test]
    fn the_catalogue_is_asked_with_the_arguments() {
        // arrange
        struct Counting {
            calls: std::cell::Cell<usize>,
        }
        impl Catalogue for Counting {
            fn lookup_with(&self, _key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
                self.calls.set(self.calls.get() + 1);
                Some("7 entries".to_string())
            }
        }
        let counting = Counting {
            calls: std::cell::Cell::new(0),
        };
        let texts = Texts::new(&counting);

        // act
        let text = texts.get_with(TextKey::PanelStatusEntries, &[Argument::Number(7)]);

        // assert
        assert_eq!(text, "7 entries");
        assert_eq!(counting.calls.get(), 1);
    }

    #[test]
    fn the_text_of_a_catalogue_is_used_as_it_is() {
        // arrange
        let texts = Texts::new(&Echo);

        // act
        let text = texts.get(TextKey::PanelColumnName);

        // assert
        assert_eq!(text, "panel-column-name");
    }
}
