//! The drawing of the work surface.
//!
//! WHAT: the drawing of the two panels, of their file lists and of the
//!       status line.
//! WHY: the drawing reads the state and never changes it — the state is
//!       changed by the commands of the shell. The file list draws only the
//!       rows that are visible, so that a directory with a hundred thousand
//!       entries costs the same in every frame as a directory with thirty
//!       entries (FR-106, NFR-01).
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::i18n::TextKey;
//!
//! assert_eq!(phocommander_ui::TextKey::PanelColumnSize, TextKey::PanelColumnSize);
//! assert_ne!(TextKey::PanelColumnSize, TextKey::PanelColumnName);
//! ```

use egui::{Align, Color32, Rect, RichText, Stroke, Ui, Vec2};

use crate::context::Context;
use crate::i18n::{Argument, TextKey, Texts};
use crate::listing::Row;
use crate::metrics::Metrics;
use crate::panel::{Column, PanelSide, PanelState, ViewMode};
use crate::surface::WorkSurface;
use crate::theme::Palette;

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
            draw_status_bar(ui, surface, palette, texts, status);
        });
    });

    visible_rows
}

/// Draws one panel with its path, its tabs and its file list.
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

    let stroke = Stroke::new(
        if focused {
            metrics.focus_frame_width()
        } else {
            1.0
        },
        palette.focus_frame,
    );

    let frame = egui::Frame::new()
        .fill(palette.panel)
        .stroke(stroke)
        .inner_margin(6.0);

    frame.show(ui, |ui| {
        draw_panel_header(ui, panel, side, metrics, palette, texts);
        draw_list(ui, panel, metrics, palette, texts)
    })
    .inner
}

/// Draws the title, the path and the tabs of a panel.
fn draw_panel_header(
    ui: &mut Ui,
    panel: &PanelState,
    side: PanelSide,
    metrics: &Metrics,
    palette: &Palette,
    texts: &Texts<'_>,
) {
    ui.horizontal(|ui| {
        let title = match side {
            PanelSide::Left => TextKey::PanelTitleLeft,
            PanelSide::Right => TextKey::PanelTitleRight,
        };
        ui.label(RichText::new(texts.get(title)).strong());
        ui.label(RichText::new(panel.path().display().to_string()).color(Color32::GRAY));
    });

    if panel.tab_count() > 1 {
        ui.horizontal(|ui| {
            for index in 0..panel.tab_count() {
                let selected = index == panel.active_tab_index();
                let label = texts.get_with(
                    TextKey::PanelTab,
                    &[Argument::Number((index + 1) as u64)],
                );
                let _ = ui.selectable_label(selected, label);
            }
        });
    }

    if panel.view() == ViewMode::Details {
        // WHY: the names of the columns are visible texts and go through a
        // translation key (FR-05). The set of the columns is the one the
        // user configured (FR-17).
        ui.horizontal(|ui| {
            for column in panel.columns() {
                ui.add_sized(
                    [column_width(*column, metrics), metrics.row_height()],
                    egui::Label::new(
                        RichText::new(texts.get(column_key(*column))).color(palette.text_dimmed),
                    ),
                );
            }
        });
    }
}

/// The width of a column.
fn column_width(column: Column, metrics: &Metrics) -> f32 {
    match column {
        Column::Name => 0.0,
        Column::Size => metrics.column_size_width(),
        Column::Date => metrics.column_date_width(),
        Column::Type => metrics.column_type_width(),
    }
}

/// Draws the file list of a panel.
fn draw_list(
    ui: &mut Ui,
    panel: &PanelState,
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

    let listing = panel.listing();
    let selection = panel.selection();
    let cursor = panel.cursor();
    let columns = panel.columns().to_vec();
    let compact = panel.view() == ViewMode::Compact;
    let first = panel.scroll_row();

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
    let name_width = (list_rect.width() - fixed - 8.0).max(60.0);

    let rows: Vec<Row> = listing.visible_rows(first, visible).collect();

    for row in &rows {
        let is_cursor = row.position() == cursor;
        let is_marked = selection.contains(row.position());

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
        let mut x = rect.left() + 4.0;

        let shown: Vec<Column> = if compact {
            vec![Column::Name]
        } else {
            columns.clone()
        };

        for column in &shown {
            let (width, text, color, right_aligned) = match column {
                Column::Name => (name_width, row.name().to_string(), text_color, false),
                Column::Size => (size_width, row.size_text(), dimmed, true),
                Column::Date => (date_width, row.date_text(), dimmed, false),
                Column::Type => (type_width, texts.get(row.type_key()), dimmed, false),
            };

            let cell = Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(width, row_height));
            let layout = if right_aligned {
                egui::Layout::right_to_left(Align::Center)
            } else {
                egui::Layout::left_to_right(Align::Center)
            };

            ui.scope_builder(
                egui::UiBuilder::new().max_rect(cell).layout(layout),
                |ui| {
                    ui.add_sized(
                        [width, row_height],
                        egui::Label::new(RichText::new(text).color(color)).truncate(),
                    );
                },
            );

            x += width;
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

    ui.horizontal(|ui| {
        let mut parts = vec![texts.get_with(
            TextKey::PanelStatusEntries,
            &[Argument::Number(entries as u64)],
        )];
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
