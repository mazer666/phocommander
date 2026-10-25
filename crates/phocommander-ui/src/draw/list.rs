//! The file list of a panel.
//!
//! WHAT: the places of the columns ([`ColumnLayout`]), the headers of the
//!       columns and the rows of the list.
//! WHY: the header of a column has to stand over its column, and the rows
//!       have to use the same places as the header — the layout is
//!       therefore computed once and handed to both. The rows are placed at
//!       a fixed distance from each other, so that the list has no gaps and
//!       no double lines. Only the visible slice of the list is drawn, so
//!       that a directory with a hundred thousand entries costs the same in
//!       every frame as a directory with thirty entries (FR-106).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::draw::list::column_key;
//! use phocommander_ui::metrics::{Metrics, Platform};
//! use phocommander_ui::panel::Column;
//!
//! let metrics = Metrics::for_platform(Platform::Linux);
//! assert!(metrics.row_height() > 0.0);
//! assert_eq!(column_key(Column::Name).as_str(), "panel-column-name");
//! ```

use egui::{Align, Layout, Rect, RichText, Stroke, Vec2};

use crate::i18n::TextKey;
use crate::layout::LayoutProfile;
use crate::metrics::Metrics;
use crate::panel::{Column, PanelState};
use crate::theme::Palette;

use super::DrawStyle;

/// The width that is kept free on the right for the scroll bar.
///
/// WHY: the scroll bar is drawn over the list. Without a free strip the
/// bar would lie on the text of the last column.
const SCROLLBAR_STRIP: f32 = 10.0;

/// The smallest width of the column with the names.
const MIN_NAME_WIDTH: f32 = 60.0;

/// The places of the columns of the file list.
///
/// WHY: the header of a column has to stand over its column, and the rows
/// have to use the same places as the header. The layout is therefore
/// computed once and handed to both.
pub(crate) struct ColumnLayout {
    columns: Vec<Column>,
    name_width: f32,
    size_width: f32,
    date_width: f32,
    type_width: f32,
}

impl ColumnLayout {
    /// The layout of the columns that are shown.
    pub(crate) fn new(columns: &[Column], metrics: &Metrics, width: f32) -> Self {
        let size_width = metrics.column_size_width();
        let date_width = metrics.column_date_width();
        let type_width = metrics.column_type_width();
        let fixed: f32 = columns
            .iter()
            .map(|column| match column {
                Column::Name => 0.0,
                Column::Size => size_width,
                Column::Date => date_width,
                Column::Type => type_width,
            })
            .sum();
        let usable = (width - SCROLLBAR_STRIP).max(MIN_NAME_WIDTH);

        Self {
            columns: columns.to_vec(),
            name_width: (usable - fixed).max(MIN_NAME_WIDTH),
            size_width,
            date_width,
            type_width,
        }
    }

    /// The width of a column.
    fn width_of(&self, column: Column) -> f32 {
        match column {
            Column::Name => self.name_width,
            Column::Size => self.size_width,
            Column::Date => self.date_width,
            Column::Type => self.type_width,
        }
    }

    /// The place of a column in a row of the list.
    fn cell(&self, column: Column, left: f32, top: f32, height: f32) -> Rect {
        let mut x = left;
        for candidate in &self.columns {
            if *candidate == column {
                break;
            }
            x += self.width_of(*candidate);
        }
        Rect::from_min_size(egui::pos2(x, top), Vec2::new(self.width_of(column), height))
    }
}

/// The alignment of the content of a column.
///
/// WHY: the numbers and the sizes are right-aligned, the names and the
/// dates are left-aligned. The header of a column stands at the same side
/// as the content of the column.
fn alignment_of(column: Column) -> Layout {
    match column {
        Column::Size => Layout::right_to_left(Align::Center),
        Column::Name | Column::Date | Column::Type => Layout::left_to_right(Align::Center),
    }
}

/// Draws the headers of the columns of the file list.
///
/// WHY: the headers stand over their columns and they do not scroll with
/// the list — one decision and not two. The names of the columns are
/// visible texts and go through a translation key (FR-05). The set of the
/// columns is the one the user configured (FR-17).
pub(crate) fn draw_column_headers(
    ui: &mut egui::Ui,
    rect: Rect,
    layout: &ColumnLayout,
    style: &DrawStyle<'_>,
) {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let metrics = style.metrics;
    let palette = style.palette;
    let texts = style.texts;

    let height = metrics.row_height();
    let left = rect.min.x;
    let top = rect.min.y;

    for column in &layout.columns {
        let cell = layout.cell(*column, left, top, height);
        let text = texts.get(column_key(*column));
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(cell)
                .layout(alignment_of(*column)),
            |ui| {
                ui.add_sized(
                    [cell.width(), height],
                    egui::Label::new(RichText::new(text).color(palette.text_dimmed)).truncate(),
                );
            },
        );
    }
}

/// Draws the file list of a panel.
///
/// WHY: only the rows that are visible are drawn, and every row is placed
/// at the same distance from the next one. The list therefore has neither
/// gaps nor double lines, and its cost does not depend on the number of the
/// entries of the directory (FR-106, NFR-01).
pub(crate) fn draw_list(
    ui: &mut egui::Ui,
    panel: &PanelState,
    rect: Rect,
    layout: &ColumnLayout,
    style: &DrawStyle<'_>,
) -> usize {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let metrics = style.metrics;
    let palette = style.palette;
    let texts = style.texts;
    let profile = style.profile;

    let row_height = profile.row_height();
    let visible = ((rect.height() / row_height).ceil() as usize).max(1);

    if panel.row_count() == 0 {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.label(RichText::new(texts.get(TextKey::PanelEmpty)).color(palette.text_dimmed));
        });
        return visible;
    }

    let selection = panel.selection();
    let cursor = panel.cursor();
    let first = panel.scroll_row();

    let rows = panel.visible_rows(first, visible);

    for row in &rows {
        let is_cursor = row.position() == cursor;
        let is_marked = selection.contains(row.position());

        // WHY: the rows are placed at a fixed distance from each other, so
        // that the list has no gaps and no double lines.
        let row_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, ui.cursor().min.y),
            Vec2::new(rect.width(), row_height),
        );
        ui.advance_cursor_after_rect(row_rect);

        if is_marked {
            ui.painter().rect_filled(row_rect, 0.0, palette.marked);
        }
        if is_cursor {
            ui.painter().rect_filled(row_rect, 0.0, palette.cursor);
        }

        let text_color = palette.text;
        let dimmed = palette.text_dimmed;
        let left = row_rect.left() + 4.0;

        for column in &layout.columns {
            let (text, color) = match column {
                Column::Name => (row.name().to_string(), text_color),
                Column::Size => (row.size_text(), dimmed),
                Column::Date => (row.date_text(), dimmed),
                Column::Type => (texts.get(row.type_key()), dimmed),
            };

            let cell = layout.cell(*column, left, row_rect.top(), row_height);

            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(cell)
                    .layout(alignment_of(*column)),
                |ui| {
                    ui.add_sized(
                        [cell.width(), row_height],
                        egui::Label::new(RichText::new(text).color(color)).truncate(),
                    );
                },
            );
        }

        if is_cursor {
            // WHY: the frame of the focus is drawn after the text, so that
            // it frames the entry instead of hiding it. The frame has the
            // same strength as the frame around every other element that
            // can be operated (NFR-11).
            ui.painter().rect_stroke(
                row_rect,
                0.0,
                Stroke::new(profile.focus_frame_width(), palette.focus_frame),
                egui::StrokeKind::Inside,
            );
        }
    }

    draw_scrollbar(ui, panel, rect, metrics, palette, profile);

    visible
}

/// Draws the scroll bar of a file list.
fn draw_scrollbar(
    ui: &mut egui::Ui,
    panel: &PanelState,
    rect: Rect,
    metrics: &Metrics,
    palette: &Palette,
    profile: &LayoutProfile,
) {
    let rows = panel.row_count();
    let visible = ((rect.height() / profile.row_height()).ceil() as usize).max(1);
    if rows <= visible {
        return;
    }

    let width = 6.0;
    let track = Rect::from_min_size(
        egui::pos2(rect.right() - width, rect.top()),
        egui::vec2(width, rect.height()),
    );
    ui.painter().rect_filled(track, 3.0, palette.text_dimmed);

    let height = (track.height() * visible as f32 / rows as f32).max(20.0);
    let offset = track.height() * panel.scroll_row() as f32 / rows as f32;
    let thumb = Rect::from_min_size(
        egui::pos2(track.left(), track.top() + offset),
        egui::vec2(width, height.min(track.height() - offset)),
    );
    ui.painter().rect_filled(thumb, 3.0, palette.focus_frame);
    let _ = metrics;
}

/// The translation key of the header of a column.
///
/// WHY: the header of a column is a visible text and goes through a
/// translation key (FR-05). The mapping is public, so that a caller that
/// draws a column of its own uses the same key and not one of its own.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::draw::list::column_key;
/// use phocommander_ui::panel::Column;
///
/// assert_eq!(column_key(Column::Name).as_str(), "panel-column-name");
/// ```
pub fn column_key(column: Column) -> TextKey {
    match column {
        Column::Name => TextKey::PanelColumnName,
        Column::Size => TextKey::PanelColumnSize,
        Column::Date => TextKey::PanelColumnDate,
        Column::Type => TextKey::PanelColumnType,
    }
}
