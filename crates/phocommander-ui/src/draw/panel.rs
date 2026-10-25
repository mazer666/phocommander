//! The drawing of one panel.
//!
//! WHAT: the frame of a panel, its title, its path, its tabs, its filter
//!       line and its file list.
//! WHY: the panel is drawn in explicit rectangles, so that the lines of the
//!       interface sit exactly where they belong: the path, the list and
//!       the status line are separated by the same line everywhere, and the
//!       rows of the list have the same distance from each other — no gaps
//!       and no double lines (FR-106). The drawing reads the state and
//!       never changes it; a click is turned into a request and handed to
//!       the shell.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::panel::PanelSide;
//!
//! assert_eq!(PanelSide::Left.other(), PanelSide::Right);
//! ```

use egui::{Align, Layout, Rect, RichText, Stroke, Vec2};

use crate::commands::Request;
use crate::i18n::{Argument, TextKey, Texts};
use crate::layout::LayoutProfile;
use crate::metrics::Metrics;
use crate::panel::{Column, PanelSide, PanelState, ViewMode};
use crate::surface::WorkSurface;
use crate::theme::Palette;

use super::list;
use super::{DrawStyle, SEPARATOR_WIDTH};

/// The distance between the frame of a panel and its content.
const PANEL_MARGIN: f32 = 6.0;

/// Draws one panel with its path, its tabs, its filter and its file list.
///
/// WHY: the panel that has the focus carries the frame of the focus — the
/// same colour and the same width as the frame around the cursor. The
/// panel without the focus carries a neutral line, so that the focus is
/// visible on the surface and not everywhere (NFR-11).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_panel(
    ui: &mut egui::Ui,
    surface: &WorkSurface,
    side: PanelSide,
    rect: Rect,
    style: &DrawStyle<'_>,
    requested: &mut Vec<Request>,
) -> usize {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let palette = style.palette;
    let profile = style.profile;

    let panel = surface.panel(side);
    let focused = surface.active_side() == side;

    let border = if focused {
        profile.focus_frame_width()
    } else {
        SEPARATOR_WIDTH
    };
    let border_color = if focused {
        palette.focus_frame
    } else {
        palette.separator
    };

    ui.painter().rect_filled(rect, 0.0, palette.panel);
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(border, border_color),
        egui::StrokeKind::Inside,
    );

    let inset = border + PANEL_MARGIN;
    let inner = Rect::from_min_size(
        rect.min + Vec2::new(inset, inset),
        (rect.size() - Vec2::new(inset * 2.0, inset * 2.0)).max(Vec2::new(1.0, 1.0)),
    );

    // WHY: the tree stands at the left of the panel and takes its width
    // from the profile. A panel whose tree is switched off gives the whole
    // width to the list.
    let tree_width = if panel.is_tree_visible() {
        profile.tree_width().min(inner.width() * 0.4)
    } else {
        0.0
    };

    let list_rect = if tree_width > 0.0 {
        let tree_rect = Rect::from_min_size(inner.min, Vec2::new(tree_width, inner.height()));
        super::areas::draw_tree(ui, tree_rect, panel, style, requested);
        ui.painter().vline(
            tree_rect.right(),
            tree_rect.y_range(),
            Stroke::new(SEPARATOR_WIDTH, palette.separator),
        );
        Rect::from_min_size(
            egui::pos2(tree_rect.right() + PANEL_MARGIN, inner.min.y),
            Vec2::new(
                (inner.width() - tree_width - PANEL_MARGIN).max(1.0),
                inner.height(),
            ),
        )
    } else {
        inner
    };

    draw_panel_body(ui, panel, side, list_rect, style, requested)
}

/// Draws the title, the tabs, the filter line and the list of a panel.
#[allow(clippy::too_many_arguments)]
fn draw_panel_body(
    ui: &mut egui::Ui,
    panel: &PanelState,
    side: PanelSide,
    rect: Rect,
    style: &DrawStyle<'_>,
    requested: &mut Vec<Request>,
) -> usize {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let metrics = style.metrics;
    let palette = style.palette;
    let texts = style.texts;
    let profile = style.profile;

    // WHY: the bands of the panel follow each other without a gap. The
    // distance between two bands is therefore zero and not a decision of
    // the style, and the rows of the list have the same distance from each
    // other (FR-106).
    ui.spacing_mut().item_spacing.y = 0.0;

    let row_height = profile.row_height();
    let mut y = rect.min.y;

    // The title and the path of the panel.
    let title_rect = Rect::from_min_size(
        egui::pos2(rect.min.x, y),
        Vec2::new(rect.width(), row_height),
    );
    y = title_rect.bottom();
    let title = match side {
        PanelSide::Left => TextKey::PanelTitleLeft,
        PanelSide::Right => TextKey::PanelTitleRight,
    };
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(title_rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.label(RichText::new(texts.get(title)).strong());
            ui.label(RichText::new(panel.path().display().to_string()).color(palette.text_dimmed));
        },
    );
    ui.advance_cursor_after_rect(title_rect);

    // The tabs of the panel.
    if panel.tab_count() > 1 {
        let tabs_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, y),
            Vec2::new(rect.width(), row_height),
        );
        y = tabs_rect.bottom();
        draw_tabs(ui, tabs_rect, panel, texts, requested);
        ui.advance_cursor_after_rect(tabs_rect);
    }

    // The line with the filter field.
    if panel.is_filter_visible() {
        let filter_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, y),
            Vec2::new(rect.width(), row_height),
        );
        y = filter_rect.bottom();
        draw_filter_line(ui, filter_rect, panel, metrics, palette, texts, profile);
        ui.advance_cursor_after_rect(filter_rect);
    }

    // The line that separates the path from the list. It is the same line
    // as the one above the status line.
    y = draw_separator(ui, rect, y, palette);

    let shown: Vec<Column> = if panel.view() == ViewMode::Compact {
        vec![Column::Name]
    } else {
        panel.columns().to_vec()
    };
    let layout = list::ColumnLayout::new(&shown, metrics, rect.width());

    // The headers of the columns. They stand over their columns and they
    // do not scroll with the list.
    if panel.view() == ViewMode::Details {
        let header_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, y),
            Vec2::new(rect.width(), row_height),
        );
        y = header_rect.bottom();
        list::draw_column_headers(ui, header_rect, &layout, style);
        y = draw_separator(ui, rect, y, palette);
    }

    let list_rect = Rect::from_min_size(
        egui::pos2(rect.min.x, y),
        Vec2::new(rect.width(), (rect.bottom() - y).max(1.0)),
    );
    list::draw_list(ui, panel, list_rect, &layout, style)
}

/// Draws the line that separates two areas.
fn draw_separator(ui: &mut egui::Ui, rect: Rect, y: f32, palette: &Palette) -> f32 {
    ui.painter().hline(
        egui::emath::Rangef::new(rect.min.x, rect.max.x),
        y,
        Stroke::new(SEPARATOR_WIDTH, palette.separator),
    );
    y + SEPARATOR_WIDTH
}

/// Draws the strip of the tabs of a panel.
///
/// WHY: the strip shows the tab that is shown and the way into the other
/// tabs. A click on a tab is a request and not a change of the state — the
/// shell carries it out.
fn draw_tabs(
    ui: &mut egui::Ui,
    rect: Rect,
    panel: &PanelState,
    texts: &Texts<'_>,
    requested: &mut Vec<Request>,
) {
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            for index in 0..panel.tab_count() {
                let selected = index == panel.active_tab_index();
                let label =
                    texts.get_with(TextKey::PanelTab, &[Argument::Number((index + 1) as u64)]);
                let response = ui.selectable_label(selected, label);
                if response.clicked() {
                    requested.push(Request::SwitchTab(index));
                }
                let close = ui.add(egui::Button::new(
                    RichText::new(texts.get(TextKey::PanelTabClose)).small(),
                ));
                if close.clicked() {
                    requested.push(Request::CloseTab(index));
                }
            }
        },
    );
}

/// Draws the line with the filter field of a panel.
///
/// WHY: the filter belongs to the panel and not to a window above it. The
/// user types in the panel and sees at once what is left of the list
/// (FR-74a). The line shows the field, the number of the entries that pass
/// the filter (FR-74e) and the pattern language that is in use.
fn draw_filter_line(
    ui: &mut egui::Ui,
    rect: Rect,
    panel: &PanelState,
    metrics: &Metrics,
    palette: &Palette,
    texts: &Texts<'_>,
    profile: &LayoutProfile,
) {
    let height = profile.row_height();
    let font = egui::FontId::proportional(metrics.font_size());
    let editing = panel.is_filter_editing();
    let empty = panel.filter_text().is_empty();

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.label(
                RichText::new(texts.get(TextKey::PanelFilterLabel)).color(palette.text_dimmed),
            );

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
                // WHY: the frame of the focus is drawn around the field,
                // with the same colour and the same width as the frame
                // around the cursor. A field that has the focus is therefore
                // visible at a glance (NFR-11).
                ui.painter().rect_stroke(
                    response.rect,
                    0.0,
                    Stroke::new(profile.focus_frame_width(), palette.focus_frame),
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

            ui.label(
                RichText::new(texts.get(TextKey::PanelFilterGrammar)).color(palette.text_dimmed),
            );
        },
    );
}

/// The place of the caret in the filter field.
fn caret_place(ui: &egui::Ui, panel: &PanelState, rect: Rect, font: egui::FontId) -> f32 {
    let prefix: String = panel
        .filter_text()
        .chars()
        .take(panel.filter_caret())
        .collect();
    let width = ui.ctx().fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(prefix, font, egui::Color32::WHITE)
            .size()
            .x
    });
    rect.left() + width
}
