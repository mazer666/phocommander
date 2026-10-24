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
//!       what the user meant.
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

use crate::panel::{PanelSide, PanelState};

/// The two panels of the work surface.
#[derive(Debug)]
pub struct WorkSurface {
    left: PanelState,
    right: PanelState,
    active: PanelSide,
}

impl WorkSurface {
    /// The work surface with two panels.
    pub fn new(left: PathBuf, right: PathBuf) -> Self {
        Self {
            left: PanelState::new(left),
            right: PanelState::new(right),
            active: PanelSide::Left,
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::listing::{Entry, EntryKind};

    /// The work surface with three entries in the left panel.
    fn surface() -> WorkSurface {
        let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
        surface.active_mut().listing_mut().set_entries(vec![
            Entry::new("a.txt", EntryKind::File),
            Entry::new("b.txt", EntryKind::File),
        ]);
        surface
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
}
