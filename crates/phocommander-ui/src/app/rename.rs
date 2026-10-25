//! The dialog of the multiple rename in the shell.
//!
//! WHAT: the shell with an engine and with a store
//!       (`with_rename_engine`, `with_template_store`), the opening of the
//!       dialog (`open_rename_dialog`), the keys of the dialog
//!       (`rename_key`) and the drawing of the dialog
//!       (`draw_rename_dialog`).
//! WHY: the dialog is a part of the surface and not a window of its own: it
//!       belongs to the panel that has the focus, it is closed by the escape
//!       key, and it hands the rename to the application as a command with
//!       the pairs the user has seen (FR-34). The two keys of the jump list
//!       come from the binding table — F3 and Shift+F3 (FR-35) — and the
//!       other keys are the keys of the editing of a text field, the same
//!       arrangement as the filter field of a panel.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::app::WorkSurfaceApp;
//! use phocommander_ui::surface::WorkSurface;
//! use std::path::PathBuf;
//!
//! # struct Empty;
//! # impl phocommander_ui::i18n::Catalogue for Empty {
//! #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
//! #         None
//! #     }
//! # }
//! let app = WorkSurfaceApp::new(
//!     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
//!     Default::default(),
//!     Box::new(Empty),
//! );
//! assert!(app.rename_dialog().is_none());
//! ```

use crate::actions::{Action, ActionCategory};
use crate::app::{StatusMessage, WorkSurfaceApp};
use crate::commands::Command;
use crate::draw;
use crate::i18n::Texts;
use crate::keymap_view::{Binding, Key, KeyStroke};
use crate::rename::{RenameDialog, RenameEngine, RenameFile, RenameFocus, TemplateStore};

impl WorkSurfaceApp {
    /// The shell with an engine that forms the new names.
    ///
    /// WHY: the engine that forms the names lives in the crate of the
    ///       operations, and the interface crate may not depend on a sibling
    ///       crate. The composition root hands the implementation over — the
    ///       same arrangement as the catalogue of the texts and the formats
    ///       of a locale.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::surface::WorkSurface;
    /// use phocommander_ui::{RenameEngine, RenamePreview, RenameRequest};
    /// use std::path::PathBuf;
    ///
    /// # struct Engine;
    /// # impl RenameEngine for Engine {
    /// #     fn preview(&self, _request: &RenameRequest) -> RenamePreview {
    /// #         RenamePreview::empty()
    /// #     }
    /// # }
    /// # struct Empty;
    /// # impl phocommander_ui::i18n::Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     Default::default(),
    ///     Box::new(Empty),
    /// )
    /// .with_rename_engine(Box::new(Engine));
    /// assert!(app.rename_dialog().is_none());
    /// ```
    pub fn with_rename_engine(mut self, engine: Box<dyn RenameEngine>) -> Self {
        self.rename_engine = Some(engine);
        self
    }

    /// The shell with a store of the named templates.
    ///
    /// WHY: the templates are stored in a file of the configuration
    ///       directory, and the interface crate performs no file operation
    ///       of its own. The composition root hands the store over.
    pub fn with_template_store(mut self, store: Box<dyn TemplateStore>) -> Self {
        self.template_store = Some(store);
        self
    }

    /// The dialog of the multiple rename, if it is open.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::surface::WorkSurface;
    /// use std::path::PathBuf;
    ///
    /// # struct Empty;
    /// # impl phocommander_ui::i18n::Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     Default::default(),
    ///     Box::new(Empty),
    /// );
    /// assert!(app.rename_dialog().is_none());
    /// ```
    pub fn rename_dialog(&self) -> Option<&RenameDialog> {
        self.rename.as_ref()
    }

    /// The dialog of the multiple rename, if it is open.
    pub fn rename_dialog_mut(&mut self) -> Option<&mut RenameDialog> {
        self.rename.as_mut()
    }

    /// Opens the dialog of the multiple rename.
    ///
    /// WHY: the dialog works on the marked entries of the panel that has the
    ///       focus. An operation acts on the panel that has the focus, and
    ///       the transfer into the other panel is always explicit (FR-16).
    pub(super) fn open_rename_dialog(&mut self) {
        let side = self.surface.active_side();
        let files = self.rename_files();
        if files.is_empty() {
            return;
        }

        let read = self.template_store.as_ref().map(|store| store.templates());
        let templates = match read {
            Some(Ok(templates)) => templates,
            Some(Err(error)) => {
                // WHY: the message of the error reaches the user through the
                //       status line and comes from the catalogue (FR-05).
                let key = error.message_key().as_str().to_string();
                self.status = Some(StatusMessage {
                    key,
                    arguments: Vec::new(),
                });
                Vec::new()
            }
            None => Vec::new(),
        };

        self.rename = Some(RenameDialog::new(side, files, templates));
    }

    /// The files of the selection of the panel that has the focus.
    ///
    /// WHY: the marked entries are the selection. A panel without a mark
    ///       renames the entry under the cursor — the same rule as every
    ///       other operation of the panel.
    fn rename_files(&self) -> Vec<RenameFile> {
        let panel = self.surface.active();
        let directory = panel.path().to_path_buf();
        let mut files: Vec<RenameFile> = panel
            .marked_entries()
            .iter()
            .map(|entry| {
                let mut file = RenameFile::new(directory.join(entry.name.as_str()));
                file.size = entry.size;
                file.modified = entry.modified;
                file
            })
            .collect();

        if files.is_empty()
            && let Some(entry) = panel.cursor_entry()
        {
            let mut file = RenameFile::new(directory.join(entry.name.as_str()));
            file.size = entry.size;
            file.modified = entry.modified;
            files.push(file);
        }
        files
    }

    /// Carries out a key that the dialog uses.
    ///
    /// WHY: the dialog has the focus while it is open. A key that the dialog
    ///       does not use reaches neither the panel nor the tree — the dialog
    ///       is a window above the surface, and the surface waits.
    pub(super) fn rename_key(&mut self, stroke: KeyStroke) -> bool {
        if self.rename.is_none() {
            return false;
        }

        // WHY: the jump list has its own actions in the catalogue, and they
        //       are bound to F3 and Shift+F3 (FR-35). The surface with the
        //       focus decides what a key means, and the dialog has it.
        if let Some(action) = self.rename_action(stroke) {
            if let Some(mut dialog) = self.rename.take() {
                match action.id() {
                    "rename.conflict_next" => {
                        dialog.jump_to_next_conflict();
                    }
                    "rename.conflict_previous" => {
                        dialog.jump_to_previous_conflict();
                    }
                    _ => {}
                }
                self.rename = Some(dialog);
            }
            return true;
        }

        let Some(mut dialog) = self.rename.take() else {
            return false;
        };

        match stroke.key {
            Key::Escape => {
                // WHY: the escape key gives up the dialog and renames
                //       nothing.
                return true;
            }
            Key::Enter => {
                return self.rename_enter(dialog);
            }
            Key::Tab => {
                dialog.focus_next(stroke.modifiers.shift);
            }
            Key::Backspace => dialog.backspace(),
            Key::Delete => dialog.delete(),
            Key::ArrowLeft => {
                if dialog.focus().is_field() {
                    dialog.move_caret(-1);
                }
            }
            Key::ArrowRight => {
                if dialog.focus().is_field() {
                    dialog.move_caret(1);
                }
            }
            Key::Home => {
                if dialog.focus().is_field() {
                    dialog.caret_to_start();
                }
            }
            Key::End => {
                if dialog.focus().is_field() {
                    dialog.caret_to_end();
                }
            }
            Key::ArrowUp => match dialog.focus() {
                RenameFocus::Templates => dialog.move_template_cursor(-1),
                RenameFocus::Preview => dialog.move_cursor(-1),
                _ => {}
            },
            Key::ArrowDown => match dialog.focus() {
                RenameFocus::Templates => dialog.move_template_cursor(1),
                RenameFocus::Preview => dialog.move_cursor(1),
                _ => {}
            },
            Key::PageUp => {
                if dialog.focus() == RenameFocus::Preview {
                    dialog.move_cursor(-10);
                }
            }
            Key::PageDown => {
                // WHY: the focus is asked with `matches!` and not with a
                //       comparison. The comparison would be a candidate for
                //       the match, and the match would be a candidate for
                //       the comparison — the two lints want opposite things.
                //       `matches!` says the same thing and pleases both.
                if matches!(dialog.focus(), RenameFocus::Preview) {
                    dialog.move_cursor(10);
                }
            }
            _ => {}
        }

        self.rename = Some(dialog);
        true
    }

    /// Carries out the enter key of the dialog.
    ///
    /// WHY: the enter key means three things, and the part of the dialog
    ///       that has the focus decides which one it is: the name of a
    ///       template stores the template, the list of the templates applies
    ///       the template under the cursor, and the rest of the dialog
    ///       confirms the rename.
    fn rename_enter(&mut self, mut dialog: RenameDialog) -> bool {
        match dialog.focus() {
            RenameFocus::Regex => {
                // WHY: the switch is a part of the dialog, so that the
                //       keyboard reaches it as well as the mouse (FR-31).
                dialog.toggle_regex();
                self.rename = Some(dialog);
                return true;
            }
            RenameFocus::TemplateName => {
                if let Some(template) = dialog.current_template() {
                    dialog.add_template(template.clone());
                    if let Some(store) = self.template_store.as_ref()
                        && let Err(error) = store.store(&template)
                    {
                        let key = error.message_key().as_str().to_string();
                        self.status = Some(StatusMessage {
                            key,
                            arguments: Vec::new(),
                        });
                    }
                }
                self.rename = Some(dialog);
                return true;
            }
            RenameFocus::Templates => {
                let cursor = dialog.template_cursor();
                dialog.apply_template(cursor);
                self.rename = Some(dialog);
                return true;
            }
            _ => {}
        }

        // WHY: the rename runs only when the preview holds no conflict and
        //       no file that could not be named. The rename of a selection is
        //       atomic (FR-41).
        let Some(pairs) = dialog.pairs() else {
            self.rename = Some(dialog);
            return true;
        };
        let side = dialog.side();
        self.commands.push(Command::Rename { side, pairs });
        true
    }

    /// The action of the dialog that a key stroke triggers.
    fn rename_action(&self, stroke: KeyStroke) -> Option<Action> {
        Action::in_category(ActionCategory::Rename).into_iter().find(|action| {
            self.keymap.binding_for(self.context, *action) == Some(&Binding::Key(stroke))
        })
    }

    /// Draws the dialog of the multiple rename.
    ///
    /// WHY: the preview is asked for when the rule changed and not in every
    ///       frame. A frame that draws an unchanged rule costs the drawing
    ///       alone.
    pub(super) fn draw_rename_dialog(&mut self, ui: &mut egui::Ui) {
        if self.rename.is_none() {
            return;
        }

        let palette = self.palette();
        let profile = self.profile();
        let WorkSurfaceApp {
            rename,
            rename_engine,
            catalogue,
            formats,
            metrics,
            ..
        } = self;
        let texts = Texts::new(catalogue.as_ref());
        let style = draw::DrawStyle::new(metrics, &palette, &texts, formats.as_ref(), &profile);

        // WHY: the borrow of the dialog ends with the block, so that the
        //       shell can put the dialog away when the user closed the
        //       window with the mouse.
        let closed = {
            let Some(dialog) = rename.as_mut() else {
                return;
            };
            if let Some(engine) = rename_engine.as_deref() {
                dialog.update_preview(engine);
            }
            crate::rename::draw::draw_dialog(ui, dialog, &style);
            !dialog.is_window_open()
        };
        if closed {
            *rename = None;
        }
    }
}
