//! The areas around the panels.
//!
//! WHAT: the menu bar, the toolbar, the tree of the directories, the quick
//!       view, the list of the jobs and the status line.
//! WHY: the areas are the furniture of the interface. They exist because
//!       the layout of the context asks for them, and they are drawn from
//!       the state of the surface — never from a condition on the context.
//!       Every button collects the action it stands for and hands it to the
//!       shell; the drawing never acts on its own.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::i18n::TextKey;
//!
//! assert_eq!(TextKey::PanelTreeTitle.as_str(), "panel-tree-title");
//! ```

use egui::{Align, Layout, Rect, RichText, Sense, Stroke, Vec2};

use crate::actions::Action;
use crate::commands::Request;
use crate::i18n::{Argument, TextKey, Texts};
use crate::panel::{PanelFocus, PanelState};
use crate::surface::WorkSurface;

use super::DrawStyle;

/// Draws the menu bar.
///
/// WHY: the menu bar is the place where every command is visible at once.
/// The groups come from the catalogue of the actions, so that a command
/// that is added to the catalogue is reachable through the menu without a
/// second change.
pub(crate) fn draw_menu_bar(
    ui: &mut egui::Ui,
    rect: Rect,
    texts: &Texts<'_>,
    requested: &mut Vec<Request>,
) {
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            for group in Action::menu_groups() {
                let title = texts.get(group.title);
                ui.menu_button(title, |ui| {
                    for identifier in group.actions {
                        let Some(action) = Action::from_id(identifier) else {
                            continue;
                        };
                        let label = texts.get(action.label());
                        if ui.button(label).clicked() {
                            requested.push(Request::Action(action));
                            ui.close();
                        }
                    }
                });
            }
        });
    });
}

/// Draws the toolbar.
///
/// WHY: the toolbar carries the commands that are used the most often. The
/// list is short on purpose: a toolbar with thirty buttons is a second
/// menu bar, and the menu bar already exists.
pub(crate) fn draw_toolbar(
    ui: &mut egui::Ui,
    rect: Rect,
    texts: &Texts<'_>,
    requested: &mut Vec<Request>,
) {
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            for identifier in Action::toolbar_actions() {
                let Some(action) = Action::from_id(identifier) else {
                    continue;
                };
                let label = texts.get(action.label());
                if ui.button(label).clicked() {
                    requested.push(Request::Action(action));
                }
            }
        },
    );
}

/// Draws the tree of the directories of a panel.
///
/// WHY: the tree shows the directories the user has opened and the
/// directories below the current path — nothing else (FR-14). The node
/// under the cursor carries the frame of the focus when the keyboard is in
/// the tree, and a click on a node is a request to open that directory.
pub(crate) fn draw_tree(
    ui: &mut egui::Ui,
    rect: Rect,
    panel: &PanelState,
    style: &DrawStyle<'_>,
    requested: &mut Vec<Request>,
) {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let metrics = style.metrics;
    let palette = style.palette;
    let texts = style.texts;
    let profile = style.profile;

    let row_height = profile.row_height();
    let tree = panel.tree();
    let focused = panel.focus() == PanelFocus::Tree;

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), row_height));
        ui.advance_cursor_after_rect(title_rect);
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(title_rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.label(RichText::new(texts.get(TextKey::PanelTreeTitle)).strong());
            },
        );

        for (index, node) in tree.nodes().iter().enumerate() {
            let row_rect = Rect::from_min_size(
                egui::pos2(rect.min.x, ui.cursor().min.y),
                Vec2::new(rect.width(), row_height),
            );
            let response = ui.allocate_rect(row_rect, Sense::click());
            if response.clicked() {
                requested.push(Request::OpenDirectory(node.path().to_path_buf()));
            }

            let indent = 6.0 + node.depth() as f32 * 12.0;
            let cell = Rect::from_min_size(
                egui::pos2(row_rect.min.x + indent, row_rect.min.y),
                Vec2::new((row_rect.width() - indent).max(1.0), row_height),
            );
            let color = if node.is_current() {
                palette.text
            } else {
                palette.text_dimmed
            };
            let text = if node.is_current() {
                RichText::new(node.name()).strong().color(color)
            } else {
                RichText::new(node.name()).color(color)
            };
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(cell)
                    .layout(Layout::left_to_right(Align::Center)),
                |ui| {
                    ui.add_sized(
                        [cell.width(), row_height],
                        egui::Label::new(text).truncate(),
                    );
                },
            );

            if focused && index == tree.cursor() {
                ui.painter().rect_stroke(
                    row_rect,
                    0.0,
                    Stroke::new(profile.focus_frame_width(), palette.focus_frame),
                    egui::StrokeKind::Inside,
                );
            }
        }
    });
    let _ = metrics;
}

/// Draws the quick view of the marked file.
///
/// WHY: the quick view shows the content of the marked file without opening
/// it (FR-15). It shows one file — the marked one — and nothing else.
pub(crate) fn draw_quick_view(
    ui: &mut egui::Ui,
    rect: Rect,
    surface: &WorkSurface,
    style: &DrawStyle<'_>,
) {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let metrics = style.metrics;
    let palette = style.palette;
    let texts = style.texts;

    let view = surface.quick_view();

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let row_height = metrics.row_height();

        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), row_height));
        ui.advance_cursor_after_rect(title_rect);
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(title_rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.label(RichText::new(texts.get(TextKey::PanelQuickViewTitle)).strong());
            },
        );

        let Some(preview) = view.preview() else {
            let empty_rect = Rect::from_min_size(
                egui::pos2(rect.min.x, title_rect.bottom()),
                Vec2::new(rect.width(), row_height),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(empty_rect), |ui| {
                ui.label(
                    RichText::new(texts.get(TextKey::PanelQuickViewEmpty))
                        .color(palette.text_dimmed),
                );
            });
            return;
        };

        let name_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, title_rect.bottom()),
            Vec2::new(rect.width(), row_height),
        );
        ui.advance_cursor_after_rect(name_rect);
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(name_rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.label(RichText::new(preview.name()).strong());
            },
        );

        if preview.is_binary() {
            let binary_rect = Rect::from_min_size(
                egui::pos2(rect.min.x, name_rect.bottom()),
                Vec2::new(rect.width(), row_height),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(binary_rect), |ui| {
                ui.label(
                    RichText::new(texts.get(TextKey::PanelQuickViewBinary))
                        .color(palette.text_dimmed),
                );
            });
            return;
        }

        let font = egui::FontId::monospace(metrics.font_size());
        let mut y = name_rect.bottom();
        for line in preview.lines() {
            let line_rect = Rect::from_min_size(
                egui::pos2(rect.min.x, y),
                Vec2::new(rect.width(), row_height),
            );
            if line_rect.bottom() > rect.bottom() {
                break;
            }
            ui.advance_cursor_after_rect(line_rect);
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(line_rect)
                    .layout(Layout::left_to_right(Align::Center)),
                |ui| {
                    ui.add_sized(
                        [line_rect.width(), row_height],
                        egui::Label::new(RichText::new(line).color(palette.text).monospace())
                            .truncate(),
                    );
                },
            );
            let _ = font;
            y = line_rect.bottom();
        }
    });
}

/// Draws the list of the jobs.
pub(crate) fn draw_job_list(
    ui: &mut egui::Ui,
    rect: Rect,
    surface: &WorkSurface,
    style: &DrawStyle<'_>,
) {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let metrics = style.metrics;
    let palette = style.palette;
    let texts = style.texts;

    let row_height = metrics.row_height();

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        let title_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), row_height));
        ui.advance_cursor_after_rect(title_rect);
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(title_rect)
                .layout(Layout::left_to_right(Align::Center)),
            |ui| {
                ui.label(RichText::new(texts.get(TextKey::PanelJobListTitle)).strong());
            },
        );

        let jobs = surface.jobs();
        if jobs.is_empty() {
            let empty_rect = Rect::from_min_size(
                egui::pos2(rect.min.x, title_rect.bottom()),
                Vec2::new(rect.width(), row_height),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(empty_rect), |ui| {
                ui.label(
                    RichText::new(texts.get(TextKey::PanelJobListEmpty)).color(palette.text_dimmed),
                );
            });
            return;
        }

        for job in jobs {
            let row_rect = Rect::from_min_size(
                egui::pos2(rect.min.x, ui.cursor().min.y),
                Vec2::new(rect.width(), row_height),
            );
            ui.advance_cursor_after_rect(row_rect);
            let kind = texts.get(job.kind().text_key());
            let state = texts.get(job.state().text_key());
            let progress = texts.get_with(
                TextKey::PanelJobProgress,
                &[
                    Argument::count(job.items_done()),
                    Argument::total(job.items_total()),
                ],
            );
            // WHY: the failure of a job is shown in the row of the job. The
            //       text comes from the catalogue, so that it speaks the
            //       language of the interface (FR-05, FR-124).
            let failure = job.error().map(|error| {
                let arguments: Vec<Argument<'_>> = error
                    .arguments()
                    .iter()
                    .map(|(name, value)| Argument::named(name, value.as_str()))
                    .collect();
                texts.get_by_key(error.key(), &arguments)
            });

            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(row_rect)
                    .layout(Layout::left_to_right(Align::Center)),
                |ui| {
                    ui.label(RichText::new(kind).color(palette.text));
                    ui.label(RichText::new(state).color(palette.text_dimmed));
                    ui.label(RichText::new(progress).color(palette.text_dimmed));
                    if let Some(failure) = failure {
                        ui.label(RichText::new(failure).color(palette.warning));
                    }
                },
            );
        }
    });
}

/// Draws the status line.
///
/// WHY: the line carries the number of the entries, the number of the
/// marked entries and the message of the last command. On a television it
/// carries the hints as well, because a remote control has no key labels
/// (section 6.5 of the specification).
pub(crate) fn draw_status_bar(
    ui: &mut egui::Ui,
    rect: Rect,
    surface: &WorkSurface,
    style: &DrawStyle<'_>,
    status: Option<&str>,
    undo_notice: Option<&str>,
) {
    // WHY: the bundle is unpacked once, so that the body of the function
    //       reads as it did before. The names are the names of the parts.
    let metrics = style.metrics;
    let palette = style.palette;
    let texts = style.texts;
    let profile = style.profile;

    let entries = surface.active().row_count();
    let marked = surface.active().selection().len();
    let filtered = surface.active().filter().is_active();
    let total = surface.active().total_count();

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            let mut parts = vec![if filtered {
                texts.get_with(
                    TextKey::PanelStatusFiltered,
                    &[
                        Argument::count(entries as u64),
                        Argument::total(total as u64),
                    ],
                )
            } else {
                texts.get_with(
                    TextKey::PanelStatusEntries,
                    &[Argument::count(entries as u64)],
                )
            }];
            if marked > 0 {
                parts.push(texts.get_with(
                    TextKey::PanelStatusSelected,
                    &[Argument::count(marked as u64)],
                ));
            }
            for part in parts {
                ui.label(RichText::new(part).color(palette.text_dimmed));
            }
            if let Some(status) = status {
                ui.label(RichText::new(status).color(palette.text));
            }
            // WHY: the notice of the undo area names the limitation of the
            // undo, and it stands in the status line as long as an undo is
            // on offer. The limitation is therefore visible and not only
            // written down (FR-26).
            if let Some(notice) = undo_notice {
                ui.label(RichText::new(notice).color(palette.text));
            }
            if profile.hints_visible() {
                ui.label(
                    RichText::new(texts.get(TextKey::PanelHintNavigation))
                        .color(palette.text_dimmed),
                );
            }
        },
    );
    let _ = metrics;
}
