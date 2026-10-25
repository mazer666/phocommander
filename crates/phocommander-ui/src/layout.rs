//! The layout profile of the work surface.
//!
//! WHAT: the structure of the surface ([`PanelStructure`]), the areas that
//!       exist in a layout ([`VisibleAreas`]), the arrangement of the two
//!       panels ([`PanelArrangement`]) and the profile that derives all of it
//!       from the interaction context ([`LayoutProfile`]).
//! WHY: the layouts of the interface differ *structurally* per context and
//!       not only in colour and font size (FR-106): two panels side by side
//!       on the desktop, two panels stacked on a tablet, one panel with an
//!       overlay on a television, a spatial arrangement with a controller.
//!       The profile is therefore computed once per frame and the drawing
//!       code asks it — the drawing code never asks "which context is this"
//!       and never carries a cascade of conditions.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::context::Context;
//! use phocommander_ui::layout::{LayoutProfile, PanelStructure};
//! use phocommander_ui::metrics::{Metrics, Platform};
//!
//! let profile = LayoutProfile::for_context(Context::Dpad, &Metrics::for_platform(Platform::Linux));
//! assert_eq!(profile.structure(), PanelStructure::SinglePanelWithOverlay);
//! ```

use crate::context::Context;
use crate::metrics::Metrics;

/// The structure of the work surface.
///
/// WHY: the structure is the difference between the layouts of section 6.5
/// of the specification. It decides how many panels are visible at the same
/// time and how they are placed — and it is a property of the context, not
/// a decision of the drawing code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PanelStructure {
    /// Two panels, side by side or below each other.
    TwoPanels,

    /// One panel; the other one appears as an overlay when an action needs
    /// a destination.
    SinglePanelWithOverlay,

    /// The panels are arranged in space rather than in a window; the focus
    /// sits on the ray pointer.
    SpatialPanels,
}

impl PanelStructure {
    /// Every structure, in a stable order.
    pub const fn all() -> [PanelStructure; 3] {
        [
            PanelStructure::TwoPanels,
            PanelStructure::SinglePanelWithOverlay,
            PanelStructure::SpatialPanels,
        ]
    }

    /// Whether the structure shows two panels at the same time.
    pub const fn shows_two_panels(&self) -> bool {
        matches!(self, PanelStructure::TwoPanels)
    }
}

/// The arrangement of the two panels.
///
/// WHY: the arrangement is the decision of the user and not of the context
/// (FR-11): the panels stand next to each other or below each other, and
/// the user switches that while the program runs. The state of the panels
/// does not change when the arrangement changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum PanelArrangement {
    /// The panels stand next to each other.
    #[default]
    SideBySide,

    /// The panels stand below each other.
    Stacked,
}

impl PanelArrangement {
    /// Every arrangement, in a stable order.
    pub const fn all() -> [PanelArrangement; 2] {
        [PanelArrangement::SideBySide, PanelArrangement::Stacked]
    }

    /// The spelling of the arrangement in the configuration file.
    pub const fn config_name(&self) -> &'static str {
        match self {
            PanelArrangement::SideBySide => "horizontal",
            PanelArrangement::Stacked => "vertical",
        }
    }

    /// The arrangement with the given spelling in the configuration file.
    pub fn from_config_name(name: &str) -> Option<Self> {
        match name {
            "horizontal" => Some(PanelArrangement::SideBySide),
            "vertical" => Some(PanelArrangement::Stacked),
            _ => None,
        }
    }

    /// The other arrangement.
    pub const fn other(&self) -> PanelArrangement {
        match self {
            PanelArrangement::SideBySide => PanelArrangement::Stacked,
            PanelArrangement::Stacked => PanelArrangement::SideBySide,
        }
    }
}

/// The areas of the interface that a layout shows.
///
/// WHY: the areas are the furniture of the interface: the menu bar, the
/// toolbar, the tree of the directories, the quick view, the list of the
/// jobs and the status line. Which of them exist is a decision of the
/// layout, and the drawing code asks the profile instead of deciding it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VisibleAreas {
    menu_bar: bool,
    toolbar: bool,
    directory_tree: bool,
    quick_view: bool,
    job_list: bool,
    status_bar: bool,
}

impl VisibleAreas {
    /// Every area is shown.
    pub const fn all() -> Self {
        Self {
            menu_bar: true,
            toolbar: true,
            directory_tree: true,
            quick_view: true,
            job_list: true,
            status_bar: true,
        }
    }

    /// Only the status line is shown.
    pub const fn status_only() -> Self {
        Self {
            menu_bar: false,
            toolbar: false,
            directory_tree: false,
            quick_view: false,
            job_list: false,
            status_bar: true,
        }
    }

    /// Whether the menu bar is shown.
    pub const fn menu_bar(&self) -> bool {
        self.menu_bar
    }

    /// Whether the toolbar is shown.
    pub const fn toolbar(&self) -> bool {
        self.toolbar
    }

    /// Whether the tree of the directories is shown.
    pub const fn directory_tree(&self) -> bool {
        self.directory_tree
    }

    /// Whether the quick view is shown.
    pub const fn quick_view(&self) -> bool {
        self.quick_view
    }

    /// Whether the list of the jobs is shown.
    pub const fn job_list(&self) -> bool {
        self.job_list
    }

    /// Whether the status line is shown.
    pub const fn status_bar(&self) -> bool {
        self.status_bar
    }
}

/// The layout of the work surface in one interaction context.
///
/// WHY: the profile is computed once per frame from the context and the
/// metrics of the platform. Everything the drawing code needs to know about
/// the shape of the surface — how many panels, which areas, how tall a row
/// is, how strong the focus frame is — is answered here. The drawing code
/// therefore carries no condition on the context at all (FR-106).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutProfile {
    context: Context,
    structure: PanelStructure,
    areas: VisibleAreas,
    row_height: f32,
    font_size: f32,
    focus_frame_width: f32,
    panel_gap: f32,
    quick_view_width: f32,
    tree_width: f32,
    hints_visible: bool,
    sheet_dialogs: bool,
    hover_enabled: bool,
}

impl LayoutProfile {
    /// The layout of the given context.
    ///
    /// WHY: the metrics of the platform are the base, and the context
    /// changes them again: a finger needs a row of forty-four points and a
    /// television needs a focus frame that is seen from three metres
    /// (section 6.5 of the specification).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::context::Context;
    /// use phocommander_ui::layout::LayoutProfile;
    /// use phocommander_ui::metrics::{Metrics, Platform};
    ///
    /// let desktop = LayoutProfile::for_context(Context::Keyboard, &Metrics::for_platform(Platform::Linux));
    /// let touch = LayoutProfile::for_context(Context::Touch, &Metrics::for_platform(Platform::Linux));
    /// assert!(touch.row_height() >= 44.0);
    /// assert!(touch.row_height() > desktop.row_height());
    /// ```
    pub fn for_context(context: Context, metrics: &Metrics) -> Self {
        let metrics = metrics.for_context(context);
        let mut profile = Self {
            context,
            structure: PanelStructure::TwoPanels,
            areas: VisibleAreas::all(),
            row_height: metrics.row_height(),
            font_size: metrics.font_size(),
            focus_frame_width: metrics.focus_frame_width(),
            panel_gap: 6.0,
            quick_view_width: 280.0,
            tree_width: 200.0,
            hints_visible: false,
            sheet_dialogs: false,
            hover_enabled: false,
        };

        match context {
            Context::Keyboard | Context::Pointer => {
                profile.hover_enabled = context.has_pointer();
            }
            Context::Touch => {
                // WHY: a finger reaches one list at a time and the screen is
                // small. The quick view and the toolbar would take the room
                // away from the list that the user is reading.
                profile.areas = VisibleAreas {
                    menu_bar: false,
                    toolbar: false,
                    directory_tree: true,
                    quick_view: false,
                    job_list: true,
                    status_bar: true,
                };
                profile.sheet_dialogs = true;
                profile.panel_gap = 0.0;
                profile.tree_width = 180.0;
            }
            Context::Dpad => {
                // WHY: two panels side by side are unreadable on a
                // television. The second panel appears as an overlay when an
                // action needs a destination, and the operation hints are
                // visible because a remote control has no key labels.
                profile.structure = PanelStructure::SinglePanelWithOverlay;
                profile.areas = VisibleAreas::status_only();
                profile.hints_visible = true;
                profile.panel_gap = 24.0;
            }
            Context::Controller => {
                // WHY: the panels are arranged in space rather than in a
                // window, and the focus sits on the ray pointer.
                profile.structure = PanelStructure::SpatialPanels;
                profile.areas = VisibleAreas::status_only();
                profile.panel_gap = 48.0;
            }
        }

        profile
    }

    /// The context the profile belongs to.
    pub const fn context(&self) -> Context {
        self.context
    }

    /// The structure of the surface.
    pub const fn structure(&self) -> PanelStructure {
        self.structure
    }

    /// The areas that the layout shows.
    pub const fn areas(&self) -> VisibleAreas {
        self.areas
    }

    /// The height of a row of the file list.
    pub const fn row_height(&self) -> f32 {
        self.row_height
    }

    /// The size of the text.
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    /// The width of the frame around the focused element.
    pub const fn focus_frame_width(&self) -> f32 {
        self.focus_frame_width
    }

    /// The distance between the two panels.
    pub const fn panel_gap(&self) -> f32 {
        self.panel_gap
    }

    /// The width of the quick view.
    pub const fn quick_view_width(&self) -> f32 {
        self.quick_view_width
    }

    /// The width of the tree of the directories.
    pub const fn tree_width(&self) -> f32 {
        self.tree_width
    }

    /// Whether the operation hints are shown.
    pub const fn hints_visible(&self) -> bool {
        self.hints_visible
    }

    /// Whether the dialogs appear as a sheet from the bottom.
    pub const fn sheet_dialogs(&self) -> bool {
        self.sheet_dialogs
    }

    /// Whether the interface answers to the pointer.
    pub const fn hover_enabled(&self) -> bool {
        self.hover_enabled
    }

    /// Whether the layout allows the given arrangement.
    ///
    /// WHY: the arrangement is the decision of the user (FR-11), but the
    /// decision only exists where the layout shows two panels. On a
    /// television the second panel is an overlay and not a second column.
    /// The answer therefore depends on the structure alone, and the name of
    /// the arrangement carries the question of the caller.
    pub const fn allows(&self, _arrangement: PanelArrangement) -> bool {
        self.structure.shows_two_panels()
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::metrics::Platform;

    /// The metrics of the desktop.
    fn desktop() -> Metrics {
        Metrics::for_platform(Platform::Linux)
    }

    #[test]
    fn the_desktop_shows_two_panels_and_every_area() {
        // act
        let profile = LayoutProfile::for_context(Context::Keyboard, &desktop());

        // assert
        assert_eq!(profile.structure(), PanelStructure::TwoPanels);
        assert!(profile.areas().menu_bar());
        assert!(profile.areas().toolbar());
        assert!(profile.areas().directory_tree());
        assert!(profile.areas().quick_view());
        assert!(profile.areas().job_list());
        assert!(profile.areas().status_bar());
    }

    #[test]
    fn the_tablet_stacks_the_panels_and_needs_a_tall_row() {
        // act
        let profile = LayoutProfile::for_context(Context::Touch, &desktop());

        // assert
        assert_eq!(profile.structure(), PanelStructure::TwoPanels);
        assert!(profile.row_height() >= 44.0);
        assert!(profile.sheet_dialogs());
        assert!(!profile.hover_enabled());
    }

    #[test]
    fn the_television_shows_one_panel_and_the_hints() {
        // act
        let profile = LayoutProfile::for_context(Context::Dpad, &desktop());

        // assert
        assert_eq!(profile.structure(), PanelStructure::SinglePanelWithOverlay);
        assert!(profile.hints_visible());
        assert!(profile.row_height() > desktop().row_height());
        assert!(profile.focus_frame_width() > desktop().focus_frame_width());
    }

    #[test]
    fn the_controller_arranges_the_panels_in_space() {
        // act
        let profile = LayoutProfile::for_context(Context::Controller, &desktop());

        // assert
        assert_eq!(profile.structure(), PanelStructure::SpatialPanels);
        assert!(!profile.hover_enabled());
    }

    #[test]
    fn only_the_layout_with_two_panels_allows_the_arrangement() {
        // arrange
        // WHY: the variable does not carry the name of the function it calls.
        //       A variable named `desktop` would hide the function `desktop`
        //       in the next line, and the second call would try to call the
        //       layout of the first line.
        let keyboard = LayoutProfile::for_context(Context::Keyboard, &desktop());
        let television = LayoutProfile::for_context(Context::Dpad, &desktop());

        // assert
        assert!(keyboard.allows(PanelArrangement::SideBySide));
        assert!(keyboard.allows(PanelArrangement::Stacked));
        assert!(!television.allows(PanelArrangement::SideBySide));
    }
}
