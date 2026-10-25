//! The drawing of the dialog of the multiple rename.
//!
//! WHAT: the window of the dialog ([`draw_dialog`]), the fields of the rule
//!       ([`draw_fields`]), the list of the stored templates
//!       ([`draw_templates`]), the list of the preview ([`draw_preview`]) and
//!       the line with the keys ([`draw_hint`]).
//! WHY: the dialog is a window above the two panels, and it is drawn with the
//!       same colours, the same measures and the same texts as the rest of
//!       the surface — a dialog that looks like a web page would be a lie
//!       about the program it belongs to. Every text of the dialog comes from
//!       the catalogue (FR-05), and the row under the cursor carries the
//!       frame of the focus, so that the place the user works at is visible
//!       (NFR-11). The drawing reads a snapshot of the fields and never holds
//!       a borrow of the dialog while it draws — the state of the dialog is
//!       changed by the keys and not by the drawing.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::rename::RenameFocus;
//!
//! assert!(RenameFocus::all().iter().any(|focus| *focus == RenameFocus::Preview));
//! ```

use egui::{Align, Color32, Layout, Rect, RichText, Stroke, Vec2};

use crate::draw::DrawStyle;
use crate::i18n::{Argument, TextKey};
use crate::layout::LayoutProfile;
use crate::rename::RenameDialog;
use crate::rename::engine::PreviewRow;
use crate::rename::focus::RenameFocus;
use crate::theme::Palette;

/// The number of the rows of the preview that are shown at the same time.
///
/// WHY: the preview of a selection of ten thousand files must not draw ten
///       thousand rows. The dialog shows a window of the list, and the cursor
///       is scrolled into it.
const VISIBLE_ROWS: usize = 12;

/// The width of the column with the old names.
const OLD_NAME_WIDTH: f32 = 200.0;

/// The width of the column with the names of the templates.
const TEMPLATE_NAME_WIDTH: f32 = 220.0;

/// The state of one field as the drawing sees it.
///
/// WHY: the drawing reads the state and never changes it. The snapshot is
///       taken before the field is drawn, so that the drawing holds no
///       borrow of the dialog while it draws.
struct FieldView {
    /// The text of the field.
    text: String,

    /// Whether the field has the focus.
    focused: bool,

    /// Whether the field carries a value that is not a number.
    invalid: bool,

    /// The text in front of the caret.
    prefix: String,
}

impl FieldView {
    /// Takes the snapshot of a field.
    fn of(dialog: &RenameDialog, focus: RenameFocus) -> Self {
        let focused = dialog.focus() == focus;
        let prefix = if focused {
            dialog.focused_text().chars().take(dialog.caret()).collect()
        } else {
            String::new()
        };
        Self {
            text: dialog.text_of(focus).to_string(),
            focused,
            invalid: dialog.is_invalid(focus),
            prefix,
        }
    }
}

/// Draws the dialog of the multiple rename.
///
/// WHY: the window is drawn above the two panels. The dialog reads the state
///       and never changes it — the state is changed by the keys, and the keys
///       are resolved by the shell.
pub(crate) fn draw_dialog(ui: &mut egui::Ui, dialog: &mut RenameDialog, style: &DrawStyle<'_>) {
    let title = style.texts.get(TextKey::RenameTitle);
    let mut open = dialog.is_window_open();
    let window = egui::Window::new(title)
        .open(&mut open)
        .default_size([680.0, 480.0]);

    // WHY: the answer of the window is not used. The dialog is closed by the
    //       shell — the escape key — and not by the frame of the window.
    let _ = window.show(ui.ctx(), |ui| {
        draw_fields(ui, dialog, style);
        ui.separator();
        draw_templates(ui, dialog, style);
        ui.separator();
        draw_preview(ui, dialog, style);
        ui.separator();
        draw_hint(ui, style);
    });

    // WHY: the user can close the window with the mouse. The flag belongs to
    //       the state of the dialog, because the frame draws the window again
    //       and would forget the click.
    dialog.set_window_open(open);
}

/// Draws the fields that carry the rule.
fn draw_fields(ui: &mut egui::Ui, dialog: &mut RenameDialog, style: &DrawStyle<'_>) {
    draw_field(
        ui,
        style,
        TextKey::RenameFormat,
        &FieldView::of(dialog, RenameFocus::Format),
    );

    let search = FieldView::of(dialog, RenameFocus::Search);
    let replacement = FieldView::of(dialog, RenameFocus::Replacement);
    ui.horizontal(|ui| {
        draw_field(ui, style, TextKey::RenameSearch, &search);
        draw_field(ui, style, TextKey::RenameReplacement, &replacement);
    });

    draw_regex(ui, dialog, style);

    let start = FieldView::of(dialog, RenameFocus::CounterStart);
    let step = FieldView::of(dialog, RenameFocus::CounterStep);
    let width = FieldView::of(dialog, RenameFocus::CounterWidth);
    ui.horizontal(|ui| {
        draw_field(ui, style, TextKey::RenameCounterStart, &start);
        draw_field(ui, style, TextKey::RenameCounterStep, &step);
        draw_field(ui, style, TextKey::RenameCounterWidth, &width);
    });

    draw_field(
        ui,
        style,
        TextKey::RenameSeries,
        &FieldView::of(dialog, RenameFocus::SeriesName),
    );
    draw_field(
        ui,
        style,
        TextKey::RenameTemplateName,
        &FieldView::of(dialog, RenameFocus::TemplateName),
    );
}

/// Draws the switch that turns the search into a regular expression.
///
/// WHY: the switch is a part of the dialog and not a widget of the drawing,
///       so that the keyboard reaches it as well as the mouse. The enter key
///       switches it when it has the focus.
fn draw_regex(ui: &mut egui::Ui, dialog: &mut RenameDialog, style: &DrawStyle<'_>) {
    let palette = style.palette;
    let profile = style.profile;
    let focused = dialog.focus() == RenameFocus::Regex;
    let mut regex = dialog.regex();

    let response = ui.checkbox(&mut regex, style.texts.get(TextKey::RenameRegex));
    if response.changed() {
        dialog.set_regex(regex);
    }
    if focused {
        // WHY: the frame of the focus is drawn around the switch with the
        //       same colour and the same width as the frame around the
        //       cursor of a file list (NFR-11).
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            Stroke::new(profile.focus_frame_width(), palette.focus_frame),
            egui::StrokeKind::Inside,
        );
    }
}

/// Draws one field of the rule.
fn draw_field(ui: &mut egui::Ui, style: &DrawStyle<'_>, label: TextKey, view: &FieldView) {
    let palette = style.palette;
    let texts = style.texts;
    let metrics = style.metrics;
    let profile = style.profile;

    let color = if view.invalid {
        palette.warning
    } else if view.text.is_empty() {
        palette.text_dimmed
    } else {
        palette.text
    };

    let height = profile.row_height();
    ui.horizontal(|ui| {
        ui.label(RichText::new(texts.get(label)).color(palette.text_dimmed));
        let width = (ui.available_width() * 0.35).clamp(120.0, 320.0);
        let response = ui.add_sized(
            [width, height],
            egui::Label::new(RichText::new(view.text.clone()).color(color)).truncate(),
        );

        if view.focused {
            // WHY: the frame of the focus is drawn around the field with the
            //       same colour and the same width as the frame around the
            //       cursor of a file list (NFR-11).
            ui.painter().rect_stroke(
                response.rect,
                0.0,
                Stroke::new(profile.focus_frame_width(), palette.focus_frame),
                egui::StrokeKind::Inside,
            );
            let caret = caret_place(ui, &view.prefix, response.rect, metrics.font_size());
            ui.painter().vline(
                caret,
                response.rect.y_range(),
                Stroke::new(1.0, palette.focus_frame),
            );
        }
    });
}

/// Draws the list of the stored templates.
fn draw_templates(ui: &mut egui::Ui, dialog: &RenameDialog, style: &DrawStyle<'_>) {
    let palette = style.palette;
    let texts = style.texts;
    let profile = style.profile;

    ui.label(RichText::new(texts.get(TextKey::RenameTemplates)).color(palette.text_dimmed));

    if dialog.templates().is_empty() {
        ui.label(RichText::new(texts.get(TextKey::RenameEmpty)).color(palette.text_dimmed));
        return;
    }

    let height = profile.row_height();
    let cursor = dialog.template_cursor();
    for (index, template) in dialog.templates().iter().enumerate() {
        let rect = Rect::from_min_size(
            egui::pos2(ui.cursor().min.x, ui.cursor().min.y),
            Vec2::new(ui.available_width(), height),
        );
        ui.advance_cursor_after_rect(rect);
        let name = template.name.clone();
        draw_row(ui, rect, palette, profile, index == cursor, |ui| {
            ui.add_sized(
                [TEMPLATE_NAME_WIDTH, height],
                egui::Label::new(RichText::new(name).color(palette.text)).truncate(),
            );
        });
    }
}

/// Draws the list of the preview.
fn draw_preview(ui: &mut egui::Ui, dialog: &mut RenameDialog, style: &DrawStyle<'_>) {
    let palette = style.palette;
    let texts = style.texts;
    let profile = style.profile;

    ui.label(RichText::new(texts.get(TextKey::RenamePreview)).color(palette.text_dimmed));

    // WHY: the reason why no name could be formed stands above the list,
    //       because it concerns the whole rule and not one file.
    let problem = dialog
        .preview()
        .and_then(|preview| preview.problem.as_ref())
        .map(|problem| {
            texts.get_with(
                problem.message_key(),
                &[Argument::named("value", problem.detail.as_str())],
            )
        });
    if let Some(problem) = problem {
        ui.label(RichText::new(problem).color(palette.warning));
    }

    if dialog.rows().is_empty() {
        ui.label(RichText::new(texts.get(TextKey::RenameEmpty)).color(palette.text_dimmed));
        return;
    }

    let height = profile.row_height();
    dialog.ensure_cursor_visible(VISIBLE_ROWS);
    let first = dialog.scroll();
    let cursor = dialog.cursor();

    for index in first..first.saturating_add(VISIBLE_ROWS) {
        let Some(row) = dialog.rows().get(index) else {
            break;
        };
        let conflict = conflict_text_of(dialog, row, texts);
        let rect = Rect::from_min_size(
            egui::pos2(ui.cursor().min.x, ui.cursor().min.y),
            Vec2::new(ui.available_width(), height),
        );
        ui.advance_cursor_after_rect(rect);
        draw_row(ui, rect, palette, profile, index == cursor, |ui| {
            draw_preview_row(ui, row, conflict.as_deref(), style, height);
        });
    }
}

/// The text of the conflict of a row of the preview.
fn conflict_text_of(
    dialog: &RenameDialog,
    row: &PreviewRow,
    texts: &crate::i18n::Texts<'_>,
) -> Option<String> {
    let preview = dialog.preview()?;
    let conflict = preview.conflicts.iter().find(|conflict| {
        conflict.path == row.source
            || row
                .target
                .as_ref()
                .is_some_and(|target| conflict.path == *target)
    })?;
    let path = conflict.path.to_string_lossy().to_string();
    Some(texts.get_with(
        conflict.message_key(),
        &[Argument::named("value", path.as_str())],
    ))
}

/// Draws one row of the preview.
fn draw_preview_row(
    ui: &mut egui::Ui,
    row: &PreviewRow,
    conflict: Option<&str>,
    style: &DrawStyle<'_>,
    height: f32,
) {
    let palette = style.palette;
    let texts = style.texts;

    ui.add_sized(
        [OLD_NAME_WIDTH, height],
        egui::Label::new(RichText::new(row.old_name()).color(palette.text_dimmed)).truncate(),
    );

    ui.label(RichText::new("->").color(palette.text_dimmed));

    let (text, color) = match (&row.name, &row.problem) {
        (Some(name), _) => (name.clone(), palette.text),
        (None, Some(problem)) => (
            texts.get_with(
                problem.message_key(),
                &[Argument::named("value", problem.detail.as_str())],
            ),
            palette.warning,
        ),
        (None, None) => (String::new(), palette.text_dimmed),
    };
    ui.add_sized(
        [ui.available_width().max(80.0), height],
        egui::Label::new(RichText::new(text).color(color)).truncate(),
    );

    if let Some(conflict) = conflict {
        ui.label(RichText::new(conflict).color(palette.warning));
    }
}

/// Draws a row of the dialog and marks it as the row under the cursor.
fn draw_row(
    ui: &mut egui::Ui,
    rect: Rect,
    palette: &Palette,
    profile: &LayoutProfile,
    is_cursor: bool,
    build: impl FnOnce(&mut egui::Ui),
) {
    if is_cursor {
        ui.painter().rect_filled(rect, 0.0, palette.cursor);
    }

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            build(ui);
        },
    );

    if is_cursor {
        // WHY: the frame of the focus is drawn after the text, so that it
        //       frames the row instead of hiding it (NFR-11).
        ui.painter().rect_stroke(
            rect,
            0.0,
            Stroke::new(profile.focus_frame_width(), palette.focus_frame),
            egui::StrokeKind::Inside,
        );
    }
}

/// Draws the line with the keys of the dialog.
fn draw_hint(ui: &mut egui::Ui, style: &DrawStyle<'_>) {
    ui.label(RichText::new(style.texts.get(TextKey::RenameHint)).color(style.palette.text_dimmed));
}

/// The place of the caret in a field.
fn caret_place(ui: &egui::Ui, prefix: &str, rect: Rect, font_size: f32) -> f32 {
    let font = egui::FontId::proportional(font_size);
    let width = ui.ctx().fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(prefix.to_string(), font, Color32::WHITE)
            .size()
            .x
    });
    rect.left() + width
}
