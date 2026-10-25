//! The drawing of the work surface.
//!
//! WHAT: the drawing of the two panels, of their file lists, of the areas
//!       around them and of the status line.
//! WHY: the drawing reads the state and never changes it — the state is
//!       changed by the commands of the shell. The file list draws only the
//!       rows that are visible, so that a directory with a hundred thousand
//!       entries costs the same in every frame as a directory with thirty
//!       entries (FR-106, NFR-01). The layout of the surface comes from the
//!       profile of the context, so that the drawing code carries no
//!       condition on the context at all (FR-106). Every button of the
//!       interface collects the action it stands for and hands it to the
//!       shell, which carries it out — the drawing never acts on its own.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::i18n::TextKey;
//!
//! assert_eq!(phocommander_ui::TextKey::PanelColumnSize, TextKey::PanelColumnSize);
//! assert_ne!(phocommander_ui::TextKey::PanelColumnSize, TextKey::PanelColumnName);
//! ```

pub mod areas;
pub mod list;
pub mod panel;

use egui::{Rect, Vec2};

use crate::commands::Request;
use crate::format::Formats;
use crate::i18n::Texts;
use crate::layout::{LayoutProfile, PanelStructure};
use crate::metrics::Metrics;
use crate::panel::PanelSide;
use crate::surface::WorkSurface;
use crate::theme::Palette;

/// The width of the line that separates two areas.
pub(crate) const SEPARATOR_WIDTH: f32 = 1.0;

/// Everything the drawing needs besides the thing that is drawn.
///
/// WHY: every part of the surface needs the same six things: the measures
///       of the platform, the colours of the theme, the texts of the
///       catalogue, the formats of the locale, the profile of the layout and
///       the place where the requests of the user are collected. Carrying
///       them as six parameters would make every signature long and
///       unreadable, and it would make it easy to mix two of them up. The
///       bundle names them once, and every drawing function reads what it
///       needs from it.
///
// BOUNDARY: the bundle is an internal helper of the drawing. It is not a
//       part of the public surface of the crate, because a caller outside the
//       crate does not draw a panel of its own.
pub(crate) struct DrawStyle<'texts> {
    /// The measures of the platform: the height of a row, the width of a
    /// scrollbar and the size of the touch targets.
    pub(crate) metrics: &'texts Metrics,

    /// The colours of the theme.
    pub(crate) palette: &'texts Palette,

    /// The texts of the catalogue in the language of the interface.
    pub(crate) texts: &'texts Texts<'texts>,

    /// The formats of the locale of the interface.
    ///
    /// WHY: the sizes and the dates of the file list are written the way the
    ///       locale of the user writes them (FR-123). The drawing therefore
    ///       asks the formats and writes no rule of a locale itself.
    pub(crate) formats: &'texts dyn Formats,

    /// The profile of the layout: which areas exist and how they are placed.
    pub(crate) profile: &'texts LayoutProfile,
}

impl<'texts> DrawStyle<'texts> {
    /// Builds the bundle.
    pub(crate) fn new(
        metrics: &'texts Metrics,
        palette: &'texts Palette,
        texts: &'texts Texts<'texts>,
        formats: &'texts dyn Formats,
        profile: &'texts LayoutProfile,
    ) -> Self {
        Self {
            metrics,
            palette,
            texts,
            formats,
            profile,
        }
    }
}

/// Draws the whole work surface.
///
/// WHY: the layout follows the context, not only the colours: two panels
/// side by side on the desktop, the panels stacked on a tablet, one panel
/// and an overlay on a television and a spatial arrangement with a
/// controller (section 6.5, FR-106). The profile answers how the surface
/// is built, and the drawing code asks it.
pub(crate) fn draw_surface(
    ui: &mut egui::Ui,
    surface: &WorkSurface,
    style: &DrawStyle<'_>,
    status: Option<&str>,
    undo_notice: Option<&str>,
    requested: &mut Vec<Request>,
) -> usize {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let metrics = style.metrics;
    let texts = style.texts;
    let profile = style.profile;

    let area = ui.max_rect();
    let row = metrics.row_height();

    let menu_height = if profile.areas().menu_bar() { row } else { 0.0 };
    let toolbar_height = if profile.areas().toolbar() { row } else { 0.0 };
    let job_height = if profile.areas().job_list() && surface.is_job_list_visible() {
        row * 3.0
    } else {
        0.0
    };
    let status_height = if profile.areas().status_bar() {
        row
    } else {
        0.0
    };
    let furniture = menu_height + toolbar_height + job_height + status_height;

    let mut y = area.min.y;

    if menu_height > 0.0 {
        let rect = Rect::from_min_size(
            egui::pos2(area.min.x, y),
            Vec2::new(area.width(), menu_height),
        );
        y += menu_height;
        areas::draw_menu_bar(ui, rect, texts, requested);
    }

    if toolbar_height > 0.0 {
        let rect = Rect::from_min_size(
            egui::pos2(area.min.x, y),
            Vec2::new(area.width(), toolbar_height),
        );
        y += toolbar_height;
        areas::draw_toolbar(ui, rect, texts, requested);
    }

    // WHY: the quick view is the third area of the surface. It stands at
    // the right of the panels and takes its width from the profile.
    let quick_width = if profile.areas().quick_view() && surface.quick_view().is_visible() {
        profile.quick_view_width()
    } else {
        0.0
    };

    let panels_height = (area.height() - furniture).max(row);
    let panels_width = (area.width() - quick_width).max(row);
    let panels_rect = Rect::from_min_size(
        egui::pos2(area.min.x, y),
        Vec2::new(panels_width, panels_height),
    );
    y += panels_height;

    let visible_rows = draw_panels(ui, surface, panels_rect, style, requested);

    if quick_width > 0.0 {
        let rect = Rect::from_min_size(
            egui::pos2(
                area.min.x + panels_width,
                area.min.y + menu_height + toolbar_height,
            ),
            Vec2::new(quick_width, panels_height),
        );
        areas::draw_quick_view(ui, rect, surface, style);
    }

    if job_height > 0.0 {
        let rect = Rect::from_min_size(
            egui::pos2(area.min.x, y),
            Vec2::new(area.width(), job_height),
        );
        y += job_height;
        areas::draw_job_list(ui, rect, surface, style);
    }

    if status_height > 0.0 {
        let rect = Rect::from_min_size(
            egui::pos2(area.min.x, y),
            Vec2::new(area.width(), status_height),
        );
        areas::draw_status_bar(ui, rect, surface, style, status, undo_notice);
    }

    visible_rows
}

/// Draws the two panels in the place the layout gives them.
///
/// WHY: the structure of the profile decides how many panels are visible
/// and how they are placed. The arrangement — next to each other or below
/// each other — is the decision of the user and is asked of the surface.
fn draw_panels(
    ui: &mut egui::Ui,
    surface: &WorkSurface,
    rect: Rect,
    style: &DrawStyle<'_>,
    requested: &mut Vec<Request>,
) -> usize {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let profile = style.profile;

    match profile.structure() {
        PanelStructure::TwoPanels => {
            match surface.arrangement() {
                crate::layout::PanelArrangement::SideBySide => {
                    let gap = profile.panel_gap();
                    let width = ((rect.width() - gap) / 2.0).max(1.0);
                    let left = Rect::from_min_size(rect.min, Vec2::new(width, rect.height()));
                    let right = Rect::from_min_size(
                        egui::pos2(rect.min.x + width + gap, rect.min.y),
                        Vec2::new(width, rect.height()),
                    );
                    let rows_left =
                        panel::draw_panel(ui, surface, PanelSide::Left, left, style, requested);
                    let rows_right =
                        panel::draw_panel(ui, surface, PanelSide::Right, right, style, requested);
                    rows_left.min(rows_right)
                }
                crate::layout::PanelArrangement::Stacked => {
                    // WHY: the panel that has the focus fills the screen and
                    // the other one lies below it. The user reaches it by
                    // scrolling, which is the gesture of a finger on a
                    // tablet (section 6.5).
                    let gap = profile.panel_gap();
                    let height = ((rect.height() - gap) / 2.0).max(1.0);
                    let active = surface.active_side();
                    let other = active.other();
                    let active_rect =
                        Rect::from_min_size(rect.min, Vec2::new(rect.width(), height));
                    let other_rect = Rect::from_min_size(
                        egui::pos2(rect.min.x, rect.min.y + height + gap),
                        Vec2::new(rect.width(), height),
                    );
                    let rows =
                        panel::draw_panel(ui, surface, active, active_rect, style, requested);
                    panel::draw_panel(ui, surface, other, other_rect, style, requested);
                    rows
                }
            }
        }
        PanelStructure::SinglePanelWithOverlay => {
            // WHY: two panels side by side are unreadable on a television.
            // The panel that has the focus fills the surface, and the other
            // one appears as an overlay when an action needs a destination.
            let active = surface.active_side();
            let rows = panel::draw_panel(ui, surface, active, rect, style, requested);
            if let Some(side) = surface.overlay() {
                let overlay = Rect::from_center_size(rect.center(), rect.size() * 0.8);
                panel::draw_panel(ui, surface, side, overlay, style, requested);
            }
            rows
        }
        PanelStructure::SpatialPanels => {
            // WHY: the panels are arranged in space rather than in a
            // window. The distance between them is the depth of the world,
            // and the focus sits on the panel the ray pointer is on.
            let gap = profile.panel_gap();
            let width = ((rect.width() - gap) / 2.0).max(1.0);
            let left = Rect::from_min_size(rect.min, Vec2::new(width, rect.height()));
            let right = Rect::from_min_size(
                egui::pos2(rect.min.x + width + gap, rect.min.y),
                Vec2::new(width, rect.height()),
            );
            let rows_left = panel::draw_panel(ui, surface, PanelSide::Left, left, style, requested);
            let rows_right =
                panel::draw_panel(ui, surface, PanelSide::Right, right, style, requested);
            rows_left.min(rows_right)
        }
    }
}
