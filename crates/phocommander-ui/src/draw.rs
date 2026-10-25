//! The drawing of the work surface.
//!
//! WHAT: the drawing of the two panels, of their file lists, of the line
//!       with the filter field and of the status line.
//! WHY: the drawing reads the state and never changes it — the state is
//!       changed by the commands of the shell. The file list draws only the
//!       rows that are visible, so that a directory with a hundred thousand
//!       entries costs the same in every frame as a directory with thirty
//!       entries (FR-106, NFR-01). The filter field is a line of the panel
//!       and not a window above it, because the filter belongs to the panel
//!       (FR-74a).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::i18n::TextKey;
//!
//! assert_eq!(phocommander_ui::TextKey::PanelColumnSize, TextKey::PanelColumnSize);
//! assert_ne!(TextKey::PanelColumnSize, TextKey::PanelColumnName);
//! ```

use egui::{Align, Color32, FontId, Rect, RichText, Stroke, Ui, Vec2};

use crate::context::Context;
use crate::i18n::{Argument, TextKey, Texts};
use crate::metrics::Metrics;
use crate::panel::{Column, PanelSide, PanelState, ViewMode};
use crate::surface::WorkSurface;
use crate::theme::Palette;

/// The width of the line that separates two areas.
const SEPARATOR_WIDTH: f32 = 1.0;

/// Draws the whole work surface.
///
/// WHY: the layout follows the context, not only the colours: two panels
/// side by side on the desktop, one panel on a television and on a tablet
/// (section 6.5, FR-106).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::context::Context;
/// use phocommander_ui::panel::PanelSide;
///
/// assert!(!Context::Keyboard.shows_one_panel());
/// assert_eq!(PanelSide::Left.other(), PanelSide::Right);
/// ```
pub(crate) fn draw_surface(
    ui: &mut Ui,
    surface: &WorkSurface,
    metrics: &Metrics,
    palette: &Palette,
    texts: &Texts<'_>,
    context: Context,
    status: Option<&str>,
) -> usize {
    let status_height = metrics.row_height() + 8.0;
    let panels_height = (ui.available_height() - status_height).max(metrics.row_height());

    let mut visible_rows = 0;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.allocate_ui(Vec2::new(ui.available_width(), panels_height), |ui| {
            if context.shows_one_panel() {
                let active = surface.active_side();
                visible_rows = draw_panel(ui, surface, active, metrics, palette, texts);
            } else {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let left = draw_panel(ui, surface, PanelSide::Left, metrics, palette, texts);
                    let right = draw_panel(ui, surface, PanelSide::Right, metrics, palette, texts);
                    visible_rows = left.min(right);
                });
            }
        });

        ui.allocate_ui(Vec2::new(ui.available_width(), status_height), |ui| {
            // WHY: the line that separates the panels from the status line
            // is the same line as the one that separates the header of the
            // list from the list. One separator, one colour, one width.
            let separator = Rect::from_min_size(
                ui.min_rect().min,
                Vec2::new(ui.available_width(), SEPARATOR_WIDTH),
            );
            ui.painter().hline(
                egui::emath::Rangef::new(separator.left(), separator.right()),
                separator.top(),
                Stroke::new(SEPARATOR_WIDTH, palette.separator),
            );
            ui.advance_cursor_after_rect(separator);
            draw_status_bar(ui, surface, palette, texts, status);
        });
    });

    visible_rows
}

/// The places of the columns of the file list.
///
/// WHY: the header of a column has to stand over its column, and the rows
/// have to use the same places as the header. The layout is therefore
/// computed once and handed to both.
struct ColumnLayout {
    columns: Vec<Column>,
    name_width: f32,
    size_width: f32,
    date_width: f32,
    type_width: f32,
    width: f32,
}

impl ColumnLayout {
    /// The layout of the columns that are shown.
    fn new(columns: &[Column], metrics: &Metrics, width: f32) -> Self {
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

        Self {
            columns: columns.to_vec(),
            name_width: (width - fixed - 8.0).max(60.0),
            size_width,
            date_width,
            type_width,
            width,
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

/// Draws one panel with its path, its tabs, its filter and its file list.
fn draw_panel(
    ui: &mut Ui,
    surface: &WorkSurface,
    side: PanelSide,
    metrics: &Metrics,
    palette: &Palette,
    texts: &Texts<'_>,
) -> usize {
    let panel = surface.panel(side);
    let focused = surface.active_side() == side;

    // WHY: the panel that has the focus carries the frame of the focus —
    // the same colour and the same width as the frame around the cursor.
    // The panel without the focus carries a neutral line, so that the
    // focus is visible on the surface and not everywhere (NFR-11).
    let stroke = if focused {
        Stroke::new(metrics.focus_frame_width(), palette.focus_frame)
    } else {
        Stroke::new(SEPARATOR_WIDTH, palette.separator)
    };

    let frame = egui::Frame::new()
        .fill(palette.panel)
        .stroke(stroke)
        .inner_margin(6.0);

    frame
        .show(ui, |ui| {
            draw_panel_header(ui, panel, side, palette, texts);
            draw_filter_line(ui, panel, metrics, palette, texts);

            let shown = if panel.view() == ViewMode::Compact {
                vec![Column::Name]
            } else {
                panel.columns().to_vec()
            };
            let layout = ColumnLayout::new(&shown, metrics, ui.available_width());

            draw_column_headers(ui, panel, &layout, metrics, palette, texts);
            draw_list(ui, panel, &layout, metrics, palette, texts)
        })
        .inner
}

/// Draws the title, the path and the tabs of a panel.
fn draw_panel_header(
    ui: &mut Ui,
    panel: &PanelState,
    side: PanelSide,
    palette: &Palette,
    texts: &Texts<'_>,
) {
    ui.horizontal(|ui| {
        let title = match side {
            PanelSide::Left => TextKey::PanelTitleLeft,
            PanelSide::Right => TextKey::PanelTitleRight,
        };
        ui.label(RichText::new(texts.get(title)).strong());
        ui.label(RichText::new(panel.path().display().to_string()).color(palette.text_dimmed));
    });

    if panel.tab_count() > 1 {
        ui.horizontal(|ui| {
            for index in 0..panel.tab_count() {
                let selected = index == panel.active_tab_index();
                let label =
                    texts.get_with(TextKey::PanelTab, &[Argument::Number((index + 1) as u64)]);
                let _ = ui.selectable_label(selected, label);
            }
        });
    }

    // WHY: the line that separates the header of the panel from its
    // content is the same line as the one above the status line.
    let separator = Rect::from_min_size(
        ui.min_rect().min,
        Vec2::new(ui.available_width(), SEPARATOR_WIDTH),
    );
    ui.painter().hline(
        egui::emath::Rangef::new(separator.left(), separator.right()),
        separator.top(),
        Stroke::new(SEPARATOR_WIDTH, palette.separator),
    );
    ui.advance_cursor_after_rect(separator);
}

/// Draws the line with the filter field of a panel.
///
/// WHY: the filter belongs to the panel and not to a window above it. The
/// user types in the panel and sees at once what is left of the list
/// (FR-74a). The line shows the field, the number of the entries that pass
/// the filter (FR-74e) and the pattern language that is in use.
fn draw_filter_line(
    ui: &mut Ui,
    panel: &PanelState,
    metrics: &Metrics,
    palette: &Palette,
    texts: &Texts<'_>,
) {
    if !panel.is_filter_visible() {
        return;
    }

    let height = metrics.row_height();
    let font = FontId::proportional(metrics.font_size());
    let editing = panel.is_filter_editing();
    let empty = panel.filter_text().is_empty();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.spacing_mut().item_spacing.y = 0.0;

        ui.label(RichText::new(texts.get(TextKey::PanelFilterLabel)).color(palette.text_dimmed));

        let text = if empty {
            texts.get(TextKey::PanelFilterPlaceholder)
        } else {
            panel.filter_text().to_string()
        };
        let color = if empty {
            palette.text_dimmed
        } else {
            palette.text
        };
        let width = (ui.available_width() * 0.4).clamp(120.0, 420.0);
        let response = ui.add_sized(
            [width, height],
            egui::Label::new(RichText::new(text).color(color)).truncate(),
        );

        if editing {
            // WHY: the frame of the focus is drawn around the field, with
            // the same colour and the same width as the frame around the
            // cursor. A field that has the focus is therefore visible at a
            // glance (NFR-11).
            ui.painter().rect_stroke(
                response.rect,
                0.0,
                Stroke::new(metrics.focus_frame_width(), palette.focus_frame),
                egui::StrokeKind::Inside,
            );
            let caret = caret_place(ui, panel, response.rect, font);
            ui.painter().vline(
                caret,
                response.rect.y_range(),
                Stroke::new(SEPARATOR_WIDTH, palette.focus_frame),
            );
        }

        let matches = panel.matching_count();
        ui.label(
            RichText::new(texts.get_with(
                TextKey::PanelFilterMatches,
                &[Argument::Number(matches as u64)],
            ))
            .color(palette.text),
        );

        ui.label(RichText::new(texts.get(TextKey::PanelFilterGrammar)).color(palette.text_dimmed));
    });
}

/// The place of the caret in the filter field.
fn caret_place(ui: &Ui, panel: &PanelState, rect: Rect, font: FontId) -> f32 {
    let prefix: String = panel
        .filter_text()
        .chars()
        .take(panel.filter_caret())
        .collect();
    let width = ui
        .ctx()
        .fonts_mut(|fonts| fonts.layout_no_wrap(prefix, font, Color32::WHITE).size().x);
    rect.left() + width
}

/// Draws the headers of the columns of the file list.
///
/// WHY: the headers stand over their columns and they do not scroll with
/// the list — one decision and not two. The names of the columns are
/// visible texts and go through a translation key (FR-05). The set of the
/// columns is the one the user configured (FR-17).
fn draw_column_headers(
    ui: &mut Ui,
    panel: &PanelState,
    layout: &ColumnLayout,
    metrics: &Metrics,
    palette: &Palette,
    texts: &Texts<'_>,
) {
    if panel.view() != ViewMode::Details {
        return;
    }

    let height = metrics.row_height();
    let left = ui.min_rect().left();
    let top = ui.min_rect().top();
    let rect = Rect::from_min_size(egui::pos2(left, top), Vec2::new(layout.width, height));
    ui.advance_cursor_after_rect(rect);

    for column in &layout.columns {
        let cell = layout.cell(*column, left, top, height);
        let right_aligned = matches!(column, Column::Size);
        let alignment = if right_aligned {
            egui::Layout::right_to_left(Align::Center)
        } else {
            egui::Layout::left_to_right(Align::Center)
        };
        let text = texts.get(column_key(*column));
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(cell).layout(alignment),
            |ui| {
                ui.add_sized(
                    [cell.width(), height],
                    egui::Label::new(RichText::new(text).color(palette.text_dimmed)).truncate(),
                );
            },
        );
    }

    ui.painter().hline(
        egui::emath::Rangef::new(rect.left(), rect.right()),
        rect.bottom(),
        Stroke::new(SEPARATOR_WIDTH, palette.separator),
    );
}

/// Draws the file list of a panel.
fn draw_list(
    ui: &mut Ui,
    panel: &PanelState,
    layout: &ColumnLayout,
    metrics: &Metrics,
    palette: &Palette,
    texts: &Texts<'_>,
) -> usize {
    let row_height = metrics.row_height();
    let list_rect = ui.max_rect();
    let visible = ((list_rect.height() / row_height).ceil() as usize).max(1);

    if panel.row_count() == 0 {
        ui.label(RichText::new(texts.get(TextKey::PanelEmpty)).color(palette.text_dimmed));
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
        let rect = Rect::from_min_size(ui.min_rect().min, Vec2::new(list_rect.width(), row_height));
        ui.advance_cursor_after_rect(rect);

        if is_marked {
            ui.painter().rect_filled(rect, 0.0, palette.marked);
        }
        if is_cursor {
            ui.painter().rect_filled(rect, 0.0, palette.cursor);
        }

        let text_color = palette.text;
        let dimmed = palette.text_dimmed;
        let left = rect.left() + 4.0;

        for column in &layout.columns {
            let (text, color, right_aligned) = match column {
                Column::Name => (row.name().to_string(), text_color, false),
                Column::Size => (row.size_text(), dimmed, true),
                Column::Date => (row.date_text(), dimmed, false),
                Column::Type => (texts.get(row.type_key()), dimmed, false),
            };

            let cell = layout.cell(*column, left, rect.top(), row_height);
            let alignment = if right_aligned {
                egui::Layout::right_to_left(Align::Center)
            } else {
                egui::Layout::left_to_right(Align::Center)
            };

            ui.scope_builder(
                egui::UiBuilder::new().max_rect(cell).layout(alignment),
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
            // it frames the entry instead of hiding it.
            ui.painter().rect_stroke(
                rect,
                0.0,
                Stroke::new(metrics.focus_frame_width(), palette.focus_frame),
                egui::StrokeKind::Inside,
            );
        }
    }

    draw_scrollbar(ui, panel, list_rect, metrics, palette);

    visible
}

/// Draws the scroll bar of a file list.
fn draw_scrollbar(
    ui: &mut Ui,
    panel: &PanelState,
    list_rect: Rect,
    metrics: &Metrics,
    palette: &Palette,
) {
    let rows = panel.row_count();
    let visible = ((list_rect.height() / metrics.row_height()).ceil() as usize).max(1);
    if rows <= visible {
        return;
    }

    let width = 6.0;
    let track = Rect::from_min_size(
        egui::pos2(list_rect.right() - width, list_rect.top()),
        egui::vec2(width, list_rect.height()),
    );
    ui.painter().rect_filled(track, 3.0, palette.text_dimmed);

    let height = (track.height() * visible as f32 / rows as f32).max(20.0);
    let offset = track.height() * panel.scroll_row() as f32 / rows as f32;
    let thumb = Rect::from_min_size(
        egui::pos2(track.left(), track.top() + offset),
        egui::vec2(width, height.min(track.height() - offset)),
    );
    ui.painter().rect_filled(thumb, 3.0, palette.focus_frame);
}

/// Draws the status line.
fn draw_status_bar(
    ui: &mut Ui,
    surface: &WorkSurface,
    palette: &Palette,
    texts: &Texts<'_>,
    status: Option<&str>,
) {
    let entries = surface.active().row_count();
    let marked = surface.active().selection().len();
    let filtered = surface.active().filter().is_active();
    let total = surface.active().total_count();

    ui.horizontal(|ui| {
        let mut parts = vec![if filtered {
            texts.get_with(
                TextKey::PanelStatusFiltered,
                &[
                    Argument::Number(entries as u64),
                    Argument::Total(total as u64),
                ],
            )
        } else {
            texts.get_with(
                TextKey::PanelStatusEntries,
                &[Argument::Number(entries as u64)],
            )
        }];
        if marked > 0 {
            parts.push(texts.get_with(
                TextKey::PanelStatusSelected,
                &[Argument::Number(marked as u64)],
            ));
        }
        for part in parts {
            ui.label(RichText::new(part).color(palette.text_dimmed));
        }
        if let Some(status) = status {
            ui.label(RichText::new(status).color(palette.text));
        }
    });
}

/// The translation key of the header of a column.
///
/// WHY: the header of a column is a visible text and goes through a
/// translation key (FR-05).
pub(crate) fn column_key(column: Column) -> TextKey {
    match column {
        Column::Name => TextKey::PanelColumnName,
        Column::Size => TextKey::PanelColumnSize,
        Column::Date => TextKey::PanelColumnDate,
        Column::Type => TextKey::PanelColumnType,
    }
}
