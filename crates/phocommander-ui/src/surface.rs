//! The work surface: the two panels and the operations that span them.
//!
//! WHAT: the pair of panels ([`WorkSurface`]), the two panels as an
//!       operation sees them ([`PanelPair`]), an entry the user marked
//!       ([`MarkedEntry`]) and the request that transfers entries from one
//!       panel into the other ([`TransferRequest`]).
//! WHY: an operation acts on the panel that has the focus, and the
//!       transfer into the other panel is always explicit (FR-16). The
//!       surface is therefore the only place that knows both panels, and
//!       every request that spans them names its source *and* its
//!       destination. A request whose source and destination are the same
//!       panel is refused — a copy into the directory it comes from is never
//!       what the user meant. The arrangement of the two panels belongs to
//!       the surface as well, because it is a decision about both of them
//!       (FR-11).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::surface::WorkSurface;
//! use std::path::PathBuf;
//!
//! let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
//! surface.switch_active();
//! assert_eq!(surface.active_side().as_str(), "right");
//! ```

use std::path::{Path, PathBuf};

use crate::jobs::JobSummary;
use crate::layout::{LayoutProfile, PanelArrangement};
use crate::panel::{PanelSide, PanelState};
use crate::preview::QuickView;

/// The two panels of the work surface.
#[derive(Debug)]
pub struct WorkSurface {
    left: PanelState,
    right: PanelState,
    active: PanelSide,
    arrangement: PanelArrangement,
    overlay: Option<PanelSide>,
    quick_view: QuickView,
    jobs: Vec<JobSummary>,
    job_list_visible: bool,
}

impl WorkSurface {
    /// The work surface with two panels.
    pub fn new(left: PathBuf, right: PathBuf) -> Self {
        Self {
            left: PanelState::new(left),
            right: PanelState::new(right),
            active: PanelSide::Left,
            arrangement: PanelArrangement::default(),
            overlay: None,
            quick_view: QuickView::new(),
            jobs: Vec::new(),
            job_list_visible: true,
        }
    }

    /// The side of the panel that has the focus.
    pub fn active_side(&self) -> PanelSide {
        self.active
    }

    /// Gives the focus to the panel of the given side.
    pub fn set_active_side(&mut self, side: PanelSide) {
        self.active = side;
    }

    /// Gives the focus to the other panel.
    ///
    /// WHY: the panels alternate, and the operation acts on the panel that
    /// has the focus. The method therefore moves the focus and nothing else.
    pub fn switch_active(&mut self) {
        self.active = self.active.other();
    }

    /// The panel of the given side.
    pub fn panel(&self, side: PanelSide) -> &PanelState {
        match side {
            PanelSide::Left => &self.left,
            PanelSide::Right => &self.right,
        }
    }

    /// The panel of the given side.
    pub fn panel_mut(&mut self, side: PanelSide) -> &mut PanelState {
        match side {
            PanelSide::Left => &mut self.left,
            PanelSide::Right => &mut self.right,
        }
    }

    /// The panel that has the focus.
    pub fn active(&self) -> &PanelState {
        self.panel(self.active)
    }

    /// The panel that has the focus.
    pub fn active_mut(&mut self) -> &mut PanelState {
        let side = self.active;
        self.panel_mut(side)
    }

    /// The panel that does not have the focus.
    pub fn other(&self) -> &PanelState {
        self.panel(self.active.other())
    }

    /// The panel that does not have the focus.
    pub fn other_mut(&mut self) -> &mut PanelState {
        let side = self.active.other();
        self.panel_mut(side)
    }

    /// The two panels as an operation sees them.
    ///
    /// WHY: the fields are named, so that a caller cannot confuse the
    /// panel it reads from with the panel it writes into. The pair follows
    /// the focus: `active` is always the panel that has it.
    pub fn pair_mut(&mut self) -> PanelPair<'_> {
        let side = self.active;
        let (left, right) = (&mut self.left, &mut self.right);
        match side {
            PanelSide::Left => PanelPair {
                active: left,
                other: right,
            },
            PanelSide::Right => PanelPair {
                active: right,
                other: left,
            },
        }
    }

    /// Exchanges the content of the two panels.
    ///
    /// WHY: the focus stays where it is. The user swaps the directories,
    /// not the panels.
    pub fn swap_panels(&mut self) {
        std::mem::swap(&mut self.left, &mut self.right);
    }

    /// The marked entries of the panel that has the focus.
    pub fn marked_entries(&self) -> Vec<MarkedEntry> {
        let side = self.active;
        let panel = self.panel(side);
        panel
            .marked_entries()
            .into_iter()
            .map(|entry| MarkedEntry {
                side,
                path: panel.path().join(&entry.name),
                name: entry.name.clone(),
            })
            .collect()
    }

    /// The request that transfers the marked entries into the other panel.
    ///
    /// WHY: the destination is named in the request and can never be the
    /// panel the entries come from. The method is therefore the only way to
    /// move entries between the panels, and the request that it produces
    /// says where the entries go.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::surface::WorkSurface;
    /// use std::path::PathBuf;
    ///
    /// let surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
    /// let request = surface.transfer_to_other();
    /// assert!(request.is_ok());
    /// ```
    pub fn transfer_to_other(&self) -> Result<TransferRequest, PanelError> {
        let source = self.active;
        let destination = source.other();
        let entries = self
            .marked_entries()
            .into_iter()
            .map(|entry| entry.path)
            .collect();
        TransferRequest::new(source, destination, entries)
    }

    /// Moves the tab that is shown into the other panel.
    ///
    /// WHY: the move of a tab is explicit (FR-13) and it carries the whole
    /// state of the tab: the path, the order, the cursor, the marks and the
    /// filter stay with the tab. The panel the tab comes from keeps a tab
    /// of its own, so that a panel is never left empty.
    pub fn move_active_tab_to_other(&mut self) -> bool {
        let source = self.active;
        let destination = source.other();
        let index = self.panel(source).active_tab_index();
        let path = self.panel(source).active_tab().path().to_path_buf();

        let Some(tab) = self.panel_mut(source).take_tab(index) else {
            return false;
        };
        if self.panel(source).tab_count() == 0 {
            self.panel_mut(source)
                .insert_tab(crate::panel::Tab::new(path));
        }
        self.panel_mut(destination).insert_tab(tab);
        true
    }

    /// The arrangement of the two panels.
    pub fn arrangement(&self) -> PanelArrangement {
        self.arrangement
    }

    /// Changes the arrangement of the two panels.
    ///
    /// WHY: the arrangement is switched while the program runs (FR-11) and
    /// the state of the panels does not change with it. The panels keep
    /// their tabs, their paths, their cursors and their marks.
    pub fn set_arrangement(&mut self, arrangement: PanelArrangement) {
        self.arrangement = arrangement;
    }

    /// Switches the arrangement of the two panels.
    ///
    /// WHY: the method answers whether the switch happened. A layout that
    /// shows one panel has no arrangement to switch, and the caller
    /// reports that instead of pretending it worked.
    pub fn toggle_arrangement(&mut self, profile: &LayoutProfile) -> bool {
        if !profile.allows(self.arrangement) {
            return false;
        }
        self.arrangement = self.arrangement.other();
        true
    }

    /// The panel that is shown as an overlay, if one is.
    ///
    /// WHY: on a television only one panel is visible, and the second one
    /// appears as an overlay when an action needs a destination (section
    /// 6.5 of the specification). The overlay is a view of the other panel
    /// and not a change of the focus — the focus stays where the user put
    /// it.
    pub fn overlay(&self) -> Option<PanelSide> {
        self.overlay
    }

    /// Shows the panel of the given side as an overlay, or hides it.
    pub fn set_overlay(&mut self, side: Option<PanelSide>) {
        self.overlay = side;
    }

    /// Shows the other panel as an overlay, or hides the overlay.
    pub fn toggle_overlay(&mut self) {
        self.overlay = match self.overlay {
            Some(_) => None,
            None => Some(self.active.other()),
        };
    }

    /// The quick view of the marked file.
    pub fn quick_view(&self) -> &QuickView {
        &self.quick_view
    }
    /// The quick view of the marked file.
    pub fn quick_view_mut(&mut self) -> &mut QuickView {
        &mut self.quick_view
    }

    /// The rows of the list of the jobs.
    pub fn jobs(&self) -> &[JobSummary] {
        &self.jobs
    }

    /// Puts the rows of the list of the jobs into the surface.
    pub fn set_jobs(&mut self, jobs: Vec<JobSummary>) {
        self.jobs = jobs;
    }

    /// Whether the list of the jobs is shown.
    pub const fn is_job_list_visible(&self) -> bool {
        self.job_list_visible
    }

    /// Shows or hides the list of the jobs.
    ///
    /// WHY: the list of the jobs is an area of the surface and not of a
    /// panel, because the queue of the operations belongs to the program
    /// and not to a directory.
    pub fn toggle_job_list(&mut self) -> bool {
        self.job_list_visible = !self.job_list_visible;
        self.job_list_visible
    }

    /// Applies the layout of the context to the panels.
    ///
    /// WHY: the tree belongs to a panel and can be switched on and off for
    /// it alone (FR-14). The layout decides whether a tree exists at all;
    /// a panel whose tree the user has switched keeps its own decision.
    pub fn apply_profile(&mut self, profile: &LayoutProfile) {
        self.left.apply_profile(profile);
        self.right.apply_profile(profile);
    }

    /// Rebuilds the index of the filter of both panels.
    ///
    /// WHY: the index of the filter belongs to the panel and is rebuilt
    /// when the directory or the filter changed. The surface knows both
    /// panels and is therefore the place that refreshes them.
    pub fn refresh(&mut self) {
        self.left.refresh();
        self.right.refresh();
    }
}

/// The two panels as an operation sees them.
#[derive(Debug)]
pub struct PanelPair<'a> {
    /// The panel that has the focus.
    pub active: &'a mut PanelState,

    /// The panel that does not have the focus.
    pub other: &'a mut PanelState,
}

/// An entry that the user marked, with the panel it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkedEntry {
    /// The panel the entry belongs to.
    pub side: PanelSide,

    /// The path of the entry.
    pub path: PathBuf,

    /// The name of the entry.
    pub name: String,
}

/// The request that transfers entries from one panel into the other.
///
/// WHY: the source and the destination are two values of the type, and the
/// constructor refuses the request whose source and destination are the same
/// panel. A transfer into the panel it comes from is therefore not
/// expressible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequest {
    source: PanelSide,
    destination: PanelSide,
    entries: Vec<PathBuf>,
}

impl TransferRequest {
    /// Builds a request, if the source and the destination are different.
    pub fn new(
        source: PanelSide,
        destination: PanelSide,
        entries: Vec<PathBuf>,
    ) -> Result<Self, PanelError> {
        if source == destination {
            return Err(PanelError::SamePanel { side: source });
        }
        Ok(Self {
            source,
            destination,
            entries,
        })
    }

    /// The panel the entries come from.
    pub fn source(&self) -> PanelSide {
        self.source
    }

    /// The panel the entries go to.
    pub fn destination(&self) -> PanelSide {
        self.destination
    }

    /// The paths of the entries.
    pub fn entries(&self) -> &[PathBuf] {
        &self.entries
    }
}

/// The reasons why a request about the panels could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PanelError {
    /// The source and the destination of a transfer are the same panel.
    #[error(
        "the {side} panel cannot be the source and the destination of a transfer at the same time"
    )]
    SamePanel {
        /// The side that was named twice.
        side: PanelSide,
    },
}

/// The path of a panel as it is shown in the interface.
pub fn panel_path(panel: &PanelState) -> &Path {
    panel.path()
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::listing::{Entry, EntryKind};
    use crate::metrics::{Metrics, Platform};

    /// The work surface with three entries in the left panel.
    fn surface() -> WorkSurface {
        let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
        surface.active_mut().set_entries(vec![
            Entry::new("a.txt", EntryKind::File),
            Entry::new("b.txt", EntryKind::File),
        ]);
        surface
    }

    /// The layout of the desktop.
    fn desktop() -> LayoutProfile {
        LayoutProfile::for_context(Context::Keyboard, &Metrics::for_platform(Platform::Linux))
    }

    #[test]
    fn the_focus_alternates_between_the_two_panels() {
        // WHY: switching the focus moves the focus and nothing else. The
        // entries stay in the panel they were put into, which is what the
        // second assertion checks.
        // arrange
        let mut surface = surface();

        // act
        surface.switch_active();

        // assert
        assert_eq!(surface.active_side(), PanelSide::Right);
        // The panel without the focus is the left one, and it holds the
        // entries, because they were put into the left panel before the
        // switch.
        assert_eq!(surface.other().row_count(), 2);
        assert_eq!(surface.active().row_count(), 0);
    }

    #[test]
    fn a_transfer_names_the_other_panel_as_its_destination() {
        // arrange
        let mut surface = surface();
        surface.active_mut().toggle_at(0);

        // act
        let request = surface.transfer_to_other();

        // assert
        let request = match request {
            Ok(request) => request,
            Err(error) => panic!("the transfer should be possible: {error}"),
        };
        assert_eq!(request.source(), PanelSide::Left);
        assert_eq!(request.destination(), PanelSide::Right);
        assert_eq!(request.entries().len(), 1);
    }

    #[test]
    fn a_transfer_into_the_panel_itself_is_refused() {
        // act
        let result = TransferRequest::new(PanelSide::Left, PanelSide::Left, Vec::new());

        // assert
        assert!(matches!(result, Err(PanelError::SamePanel { .. })));
    }

    #[test]
    fn the_pair_follows_the_focus() {
        // arrange
        let mut surface = surface();
        surface.switch_active();

        // act
        let pair = surface.pair_mut();

        // assert
        assert_eq!(pair.active.row_count(), 0);
        assert_eq!(pair.other.row_count(), 2);
    }

    #[test]
    fn swapping_the_panels_exchanges_their_content() {
        // arrange
        let mut surface = surface();

        // act
        surface.swap_panels();

        // assert
        assert_eq!(surface.panel(PanelSide::Left).row_count(), 0);
        assert_eq!(surface.panel(PanelSide::Right).row_count(), 2);
        assert_eq!(surface.active_side(), PanelSide::Left);
    }

    #[test]
    fn the_arrangement_is_switched_without_losing_the_state() {
        // arrange
        let mut surface = surface();
        surface.active_mut().toggle_at(0);

        // act
        let switched = surface.toggle_arrangement(&desktop());

        // assert
        assert!(switched);
        assert_eq!(surface.arrangement(), PanelArrangement::Stacked);
        assert_eq!(surface.active().selection().len(), 1);
        assert_eq!(surface.active().row_count(), 2);
    }

    #[test]
    fn a_tab_that_is_moved_keeps_its_state() {
        // arrange
        let mut surface = surface();
        surface.active_mut().toggle_at(0);
        surface.active_mut().set_filter_text("a");

        // act
        let moved = surface.move_active_tab_to_other();

        // assert
        assert!(moved);
        let right = surface.panel(PanelSide::Right);
        assert_eq!(right.tab_count(), 2);
        assert_eq!(right.active_tab_index(), 1);
        assert_eq!(right.selection().len(), 1);
        assert_eq!(right.filter_text(), "a");
        // The panel the tab came from keeps a tab of its own.
        assert_eq!(surface.panel(PanelSide::Left).tab_count(), 1);
    }
}
