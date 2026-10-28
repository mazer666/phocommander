//! The translation keys of the work surface.
//!
//! WHAT: the keys of the messages of the interface ([`TextKey`]).
//! WHY: the key is the name of a message in the catalogue. Listing the keys
//!       in the code makes the visible texts of the interface findable, and
//!       the gate of the workspace compares the list with the catalogues —
//!       so that a key cannot be used without a translation.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::TextKey;
//!
//! assert_eq!(TextKey::CommonCancel.as_str(), "common-cancel");
//! ```

/// The translation keys of the work surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextKey {
    /// The label of the action `action-app-menu`.
    ActionAppMenu,
    /// The interface in German.
    ActionAppLanguageDe,
    /// The interface in English.
    ActionAppLanguageEn,
    /// The interface in French.
    ActionAppLanguageFr,
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
    /// The label of the action `action-panel-filter`.
    ActionPanelFilter,
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
    /// The label of the action `action-panel-tab-new`.
    ActionPanelTabNew,
    /// The label of the action `action-panel-tab-close`.
    ActionPanelTabClose,
    /// The label of the action `action-panel-tab-move-to-other`.
    ActionPanelTabMoveToOther,
    /// The label of the action `action-panel-layout-toggle`.
    ActionPanelLayoutToggle,
    /// The label of the action `action-panel-tree-toggle`.
    ActionPanelTreeToggle,
    /// The label of the action `action-panel-quick-view-toggle`.
    ActionPanelQuickViewToggle,
    /// The label of the action `action-panel-job-list-toggle`.
    ActionPanelJobListToggle,
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
    /// The number of the entries that pass the filter and the number of the
    /// entries of the directory.
    PanelStatusFiltered,
    /// The number of marked entries of a panel.
    PanelStatusSelected,
    /// The label of the line with the filter field.
    PanelFilterLabel,
    /// The text of an empty filter field.
    PanelFilterPlaceholder,
    /// The number of the entries that pass the filter.
    PanelFilterMatches,
    /// The hint at the pattern language of the filter.
    PanelFilterGrammar,
    /// The label of a tab.
    PanelTab,
    /// The button that closes a tab.
    PanelTabClose,
    /// The name of the detailed view.
    PanelViewDetails,
    /// The name of the compact view.
    PanelViewCompact,
    /// The label of the panel that has the focus.
    PanelFocus,
    /// The title of the tree of the directories.
    PanelTreeTitle,
    /// The title of the quick view.
    PanelQuickViewTitle,
    /// The text of a quick view without a file.
    PanelQuickViewEmpty,
    /// The text of a quick view of a file that is not text.
    PanelQuickViewBinary,
    /// The title of the list of the jobs.
    PanelJobListTitle,
    /// The text of a list of jobs without a job.
    PanelJobListEmpty,
    /// The progress of a job.
    PanelJobProgress,
    /// The rate of a job, in bytes per second.
    PanelJobRate,
    /// The remaining time of a job, in seconds.
    PanelJobEtaSeconds,
    /// The remaining time of a job, in minutes.
    PanelJobEtaMinutes,
    /// The name of the arrangement with the panels next to each other.
    PanelLayoutSideBySide,
    /// The name of the arrangement with the panels below each other.
    PanelLayoutStacked,
    /// The hint that names the keys of the navigation.
    PanelHintNavigation,
    /// The title of the menu with the file operations.
    MenuFile,
    /// The title of the menu with the commands of the panel.
    MenuPanel,
    /// The title of the menu with the commands of the view.
    MenuView,
    /// The title of the menu with the commands of the application.
    MenuApplication,
    /// The state of a job that waits for a worker.
    JobStateQueued,
    /// The state of a job that works.
    JobStateRunning,
    /// The state of a job that compares the source and the destination.
    JobStateVerifying,
    /// The state of a job that waits.
    JobStateWaiting,
    /// The state of a job that finished without an error.
    JobStateDone,
    /// The state of a job that finished with errors.
    JobStateFailed,
    /// The state of a job that was cancelled.
    JobStateCancelled,
    /// The kind of a job that copies.
    JobKindCopy,
    /// The kind of a job that moves.
    JobKindMove,
    /// The kind of a job that deletes.
    JobKindDelete,
    /// The kind of a job that renames.
    JobKindRename,
    /// The kind of a job that synchronises.
    JobKindSync,
    /// The kind of a job that creates a directory.
    JobKindCreateDirectory,
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
    /// The title of the dialog of the multiple rename.
    RenameTitle,
    /// The label of the field with the format of the new name.
    RenameFormat,
    /// The label of the field with the text that is searched.
    RenameSearch,
    /// The label of the field with the text that replaces it.
    RenameReplacement,
    /// The label of the switch that turns the search into a pattern.
    RenameRegex,
    /// The label of the field with the first number of the counter.
    RenameCounterStart,
    /// The label of the field with the step of the counter.
    RenameCounterStep,
    /// The label of the field with the width of the counter.
    RenameCounterWidth,
    /// The label of the field with the name of the series.
    RenameSeries,
    /// The label of the field with the name of a new template.
    RenameTemplateName,
    /// The label of the list of the stored templates.
    RenameTemplates,
    /// The label of the list of the preview.
    RenamePreview,
    /// The text of the dialog without files and without templates.
    RenameEmpty,
    /// The line with the keys of the dialog.
    RenameHint,
    /// The reason why a rule without a format names no file.
    RenameProblemEmptyFormat,
    /// The reason why the value of a placeholder is not known.
    RenameProblemUnknownValue,
    /// The reason why a pattern is not a valid regular expression.
    RenameProblemInvalidPattern,
    /// The reason why a name cannot be used as a file name.
    RenameProblemInvalidName,
    /// The reason why the target of a rename is occupied.
    RenameConflictTargetExists,
    /// The reason why two files of a selection want the same name.
    RenameConflictDuplicateTarget,
    /// The reason why the file that is to be renamed is missing.
    RenameConflictMissingSource,
    /// The reason why a template could not be stored.
    RenameTemplateStoreFailed,
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
            TextKey::ActionAppLanguageDe => "action-app-language-de",
            TextKey::ActionAppLanguageEn => "action-app-language-en",
            TextKey::ActionAppLanguageFr => "action-app-language-fr",
            TextKey::ActionAppQuit => "action-app-quit",
            TextKey::ActionPaletteOpen => "action-palette-open",
            TextKey::ActionPanelCopy => "action-panel-copy",
            TextKey::ActionPanelDelete => "action-panel-delete",
            TextKey::ActionPanelDrives => "action-panel-drives",
            TextKey::ActionPanelEdit => "action-panel-edit",
            TextKey::ActionPanelFilter => "action-panel-filter",
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
            TextKey::ActionPanelTabNew => "action-panel-tab-new",
            TextKey::ActionPanelTabClose => "action-panel-tab-close",
            TextKey::ActionPanelTabMoveToOther => "action-panel-tab-move-to-other",
            TextKey::ActionPanelLayoutToggle => "action-panel-layout-toggle",
            TextKey::ActionPanelTreeToggle => "action-panel-tree-toggle",
            TextKey::ActionPanelQuickViewToggle => "action-panel-quick-view-toggle",
            TextKey::ActionPanelJobListToggle => "action-panel-job-list-toggle",
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
            TextKey::PanelStatusFiltered => "panel-status-filtered",
            TextKey::PanelStatusSelected => "panel-status-selected",
            TextKey::PanelFilterLabel => "panel-filter-label",
            TextKey::PanelFilterPlaceholder => "panel-filter-placeholder",
            TextKey::PanelFilterMatches => "panel-filter-matches",
            TextKey::PanelFilterGrammar => "panel-filter-grammar",
            TextKey::PanelTab => "panel-tab",
            TextKey::PanelTabClose => "panel-tab-close",
            TextKey::PanelViewDetails => "panel-view-details",
            TextKey::PanelViewCompact => "panel-view-compact",
            TextKey::PanelFocus => "panel-focus",
            TextKey::PanelTreeTitle => "panel-tree-title",
            TextKey::PanelQuickViewTitle => "panel-quick-view-title",
            TextKey::PanelQuickViewEmpty => "panel-quick-view-empty",
            TextKey::PanelQuickViewBinary => "panel-quick-view-binary",
            TextKey::PanelJobListTitle => "panel-job-list-title",
            TextKey::PanelJobListEmpty => "panel-job-list-empty",
            TextKey::PanelJobProgress => "panel-job-progress",
            TextKey::PanelJobRate => "panel-job-rate",
            TextKey::PanelJobEtaSeconds => "panel-job-eta-seconds",
            TextKey::PanelJobEtaMinutes => "panel-job-eta-minutes",
            TextKey::PanelLayoutSideBySide => "panel-layout-side-by-side",
            TextKey::PanelLayoutStacked => "panel-layout-stacked",
            TextKey::PanelHintNavigation => "panel-hint-navigation",
            TextKey::MenuFile => "menu-file",
            TextKey::MenuPanel => "menu-panel",
            TextKey::MenuView => "menu-view",
            TextKey::MenuApplication => "menu-application",
            TextKey::JobStateQueued => "job-state-queued",
            TextKey::JobStateRunning => "job-state-running",
            TextKey::JobStateVerifying => "job-state-verifying",
            TextKey::JobStateWaiting => "job-state-waiting",
            TextKey::JobStateDone => "job-state-done",
            TextKey::JobStateFailed => "job-state-failed",
            TextKey::JobStateCancelled => "job-state-cancelled",
            TextKey::JobKindCopy => "job-kind-copy",
            TextKey::JobKindMove => "job-kind-move",
            TextKey::JobKindDelete => "job-kind-delete",
            TextKey::JobKindRename => "job-kind-rename",
            TextKey::JobKindSync => "job-kind-sync",
            TextKey::JobKindCreateDirectory => "job-kind-create-directory",
            TextKey::ViewerTitle => "viewer-title",
            TextKey::ViewerFindNext => "viewer-find-next",
            TextKey::ViewerFindPrevious => "viewer-find-previous",
            TextKey::ViewerNoFile => "viewer-no-file",
            TextKey::PaletteTitle => "palette-title",
            TextKey::PaletteSearch => "palette-search",
            TextKey::PaletteEmpty => "palette-empty",
            TextKey::RenameTitle => "rename-title",
            TextKey::RenameFormat => "rename-format",
            TextKey::RenameSearch => "rename-search",
            TextKey::RenameReplacement => "rename-replacement",
            TextKey::RenameRegex => "rename-regex",
            TextKey::RenameCounterStart => "rename-counter-start",
            TextKey::RenameCounterStep => "rename-counter-step",
            TextKey::RenameCounterWidth => "rename-counter-width",
            TextKey::RenameSeries => "rename-series",
            TextKey::RenameTemplateName => "rename-template-name",
            TextKey::RenameTemplates => "rename-templates",
            TextKey::RenamePreview => "rename-preview",
            TextKey::RenameEmpty => "rename-empty",
            TextKey::RenameHint => "rename-hint",
            TextKey::RenameProblemEmptyFormat => "rename-problem-empty-format",
            TextKey::RenameProblemUnknownValue => "rename-problem-unknown-value",
            TextKey::RenameProblemInvalidPattern => "rename-problem-invalid-pattern",
            TextKey::RenameProblemInvalidName => "rename-problem-invalid-name",
            TextKey::RenameConflictTargetExists => "rename-conflict-target-exists",
            TextKey::RenameConflictDuplicateTarget => "rename-conflict-duplicate-target",
            TextKey::RenameConflictMissingSource => "rename-conflict-missing-source",
            TextKey::RenameTemplateStoreFailed => "rename-template-store-failed",
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
const KEYS: [TextKey; 132] = [
    TextKey::ActionAppMenu,
    TextKey::ActionAppLanguageDe,
    TextKey::ActionAppLanguageEn,
    TextKey::ActionAppLanguageFr,
    TextKey::ActionAppQuit,
    TextKey::ActionPaletteOpen,
    TextKey::ActionPanelCopy,
    TextKey::ActionPanelDelete,
    TextKey::ActionPanelDrives,
    TextKey::ActionPanelEdit,
    TextKey::ActionPanelFilter,
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
    TextKey::ActionPanelTabNew,
    TextKey::ActionPanelTabClose,
    TextKey::ActionPanelTabMoveToOther,
    TextKey::ActionPanelLayoutToggle,
    TextKey::ActionPanelTreeToggle,
    TextKey::ActionPanelQuickViewToggle,
    TextKey::ActionPanelJobListToggle,
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
    TextKey::PanelStatusFiltered,
    TextKey::PanelStatusSelected,
    TextKey::PanelFilterLabel,
    TextKey::PanelFilterPlaceholder,
    TextKey::PanelFilterMatches,
    TextKey::PanelFilterGrammar,
    TextKey::PanelTab,
    TextKey::PanelTabClose,
    TextKey::PanelViewDetails,
    TextKey::PanelViewCompact,
    TextKey::PanelFocus,
    TextKey::PanelTreeTitle,
    TextKey::PanelQuickViewTitle,
    TextKey::PanelQuickViewEmpty,
    TextKey::PanelQuickViewBinary,
    TextKey::PanelJobListTitle,
    TextKey::PanelJobListEmpty,
    TextKey::PanelJobProgress,
    TextKey::PanelLayoutSideBySide,
    TextKey::PanelLayoutStacked,
    TextKey::PanelHintNavigation,
    TextKey::MenuFile,
    TextKey::MenuPanel,
    TextKey::MenuView,
    TextKey::MenuApplication,
    TextKey::JobStateQueued,
    TextKey::JobStateRunning,
    TextKey::JobStateVerifying,
    TextKey::JobStateWaiting,
    TextKey::JobStateDone,
    TextKey::JobStateFailed,
    TextKey::JobStateCancelled,
    TextKey::JobKindCopy,
    TextKey::JobKindMove,
    TextKey::JobKindDelete,
    TextKey::JobKindRename,
    TextKey::JobKindSync,
    TextKey::JobKindCreateDirectory,
    TextKey::ViewerTitle,
    TextKey::ViewerFindNext,
    TextKey::ViewerFindPrevious,
    TextKey::ViewerNoFile,
    TextKey::PaletteTitle,
    TextKey::PaletteSearch,
    TextKey::PaletteEmpty,
    TextKey::RenameTitle,
    TextKey::RenameFormat,
    TextKey::RenameSearch,
    TextKey::RenameReplacement,
    TextKey::RenameRegex,
    TextKey::RenameCounterStart,
    TextKey::RenameCounterStep,
    TextKey::RenameCounterWidth,
    TextKey::RenameSeries,
    TextKey::RenameTemplateName,
    TextKey::RenameTemplates,
    TextKey::RenamePreview,
    TextKey::RenameEmpty,
    TextKey::RenameHint,
    TextKey::RenameProblemEmptyFormat,
    TextKey::RenameProblemUnknownValue,
    TextKey::RenameProblemInvalidPattern,
    TextKey::RenameProblemInvalidName,
    TextKey::RenameConflictTargetExists,
    TextKey::RenameConflictDuplicateTarget,
    TextKey::RenameConflictMissingSource,
    TextKey::RenameTemplateStoreFailed,
];
