//! The resolution of the input of a frame.
//!
//! WHAT: the reading of the events of the frame, the handling of the keys
//!       of the filter field and the navigation with the keys.
//! WHY: the input layer translates the physical input into actions and the
//!       surface carries them out. The keys that belong to the field of the
//!       filter never reach the actions of the panel, and the keys of the
//!       navigation move the cursor in the place the keyboard is in — the
//!       list of the entries or the tree of the directories.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::keymap_view::{Key, KeyStroke};
//!
//! let stroke = KeyStroke::new(Key::ArrowDown);
//! assert_eq!(stroke.key, Key::ArrowDown);
//! ```

use std::path::Path;

use crate::app::WorkSurfaceApp;
use crate::commands::Command;
use crate::keymap_view::{InputEvent, Key, KeyStroke};
use crate::panel::PanelFocus;

impl WorkSurfaceApp {
    /// Resolves the input of the frame into actions and carries them out.
    pub(super) fn resolve_input(&mut self, ctx: &egui::Context) {
        let events: Vec<egui::Event> = ctx.input(|input| input.events.clone());
        let primary = self.primary;

        for event in events {
            // WHY: the characters of the filter field arrive as a text
            // event and not as a key event, because the keyboard layout of
            // the user decides which characters a key produces.
            if let egui::Event::Text(text) = &event {
                if self.rename.is_some() {
                    // WHY: the dialog has the focus while it is open. The
                    //       text belongs to the field of the dialog that has
                    //       the focus.
                    if let Some(dialog) = self.rename.as_mut() {
                        dialog.insert(text);
                    }
                    continue;
                }
                if self.filter_is_editing() {
                    self.surface.active_mut().filter_insert(text);
                    continue;
                }
            }

            let egui::Event::Key {
                key,
                modifiers,
                pressed: true,
                repeat: false,
                ..
            } = event
            else {
                continue;
            };

            let Some(stroke) = crate::input::stroke_of(key, modifiers, primary) else {
                continue;
            };

            // WHY: the dialog of the multiple rename is a window above the
            //       surface. While it is open, a key that it does not use
            //       reaches neither the panel nor the tree.
            if self.rename.is_some() {
                self.rename_key(stroke);
                continue;
            }

            // WHY: the field that has the focus takes the keys that belong
            // to it. The escape key empties the field and hides it, the
            // enter key confirms the filter and keeps it (FR-74a).
            if self.filter_is_editing() && self.filter_key(stroke) {
                continue;
            }

            let action = self.keymap.resolve(self.context, &InputEvent::Key(stroke));
            if let Some(action) = action {
                self.dispatch(action);
                continue;
            }

            self.handle_key(stroke);
        }
    }

    /// Whether the panel that has the focus is editing its filter.
    fn filter_is_editing(&self) -> bool {
        self.surface.active().is_filter_editing()
    }

    /// Carries out a key stroke in the filter field.
    ///
    /// WHY: while the field is being edited, the keys that belong to the
    /// field must not reach the actions of the panel. The method answers
    /// whether it has used the key.
    pub(crate) fn filter_key(&mut self, stroke: KeyStroke) -> bool {
        let panel = self.surface.active_mut();
        match stroke.key {
            Key::Escape => {
                panel.cancel_filter();
                true
            }
            Key::Enter => {
                panel.confirm_filter();
                true
            }
            Key::Backspace => {
                panel.filter_backspace();
                true
            }
            Key::Delete => {
                panel.filter_delete();
                true
            }
            Key::ArrowLeft => {
                panel.filter_move_caret(-1);
                true
            }
            Key::ArrowRight => {
                panel.filter_move_caret(1);
                true
            }
            Key::Home => {
                panel.filter_caret_to_start();
                true
            }
            Key::End => {
                panel.filter_caret_to_end();
                true
            }
            _ => false,
        }
    }

    /// Carries out a key that no action of the catalogue is bound to.
    ///
    /// WHY: the catalogue of the specification names no action for moving
    /// the cursor, and a file list that cannot be walked is not a file
    /// manager. The keys of the navigation are therefore handled by the
    /// shell itself. The deviation is reported in the handover of this
    /// milestone, and the owner decides whether the catalogue grows the
    /// actions `panel.cursor.*` or the surface keeps this behaviour.
    fn handle_key(&mut self, stroke: KeyStroke) {
        if !stroke.modifiers.is_empty() {
            return;
        }

        // WHY: the overlay of a television waits for the confirmation of the
        // user. The transfer starts when the user confirms and it never
        // starts without the destination the user has seen (FR-16).
        if self.pending_transfer.is_some() {
            match stroke.key {
                Key::Enter => {
                    self.confirm_pending_transfer();
                    return;
                }
                Key::Escape => {
                    self.cancel_pending_transfer();
                    return;
                }
                _ => return,
            }
        }

        match stroke.key {
            Key::Escape => {
                self.surface.active_mut().set_focus(PanelFocus::List);
                return;
            }
            Key::Enter => {
                self.open_the_focused_directory();
                return;
            }
            _ => {}
        }

        if self.surface.active().focus() == PanelFocus::Tree {
            let delta = match stroke.key {
                Key::ArrowUp => -1,
                Key::ArrowDown => 1,
                _ => return,
            };
            self.surface.active_mut().tree_mut().move_cursor(delta);
            return;
        }

        self.move_cursor(stroke);
    }

    /// Opens the directory the tree points at.
    ///
    /// WHY: the tree is a way into a directory, and the jump is explicit —
    /// the user presses the confirm key on the node he wants (FR-14).
    fn open_the_focused_directory(&mut self) {
        let side = self.surface.active_side();
        let path = self
            .surface
            .active()
            .tree()
            .cursor_path()
            .map(Path::to_path_buf);
        if let Some(path) = path {
            self.commands.push(Command::LoadDirectory { side, path });
        }
    }

    /// Carries out the transfer that waits for the confirmation.
    fn confirm_pending_transfer(&mut self) {
        let Some(pending) = self.pending_transfer.take() else {
            return;
        };
        self.surface.set_overlay(None);
        if pending.copy {
            self.commands.push(Command::Copy {
                transfer: pending.transfer,
            });
        } else {
            self.commands.push(Command::Move {
                transfer: pending.transfer,
            });
        }
    }

    /// Gives up the transfer that waits for the confirmation.
    fn cancel_pending_transfer(&mut self) {
        self.pending_transfer = None;
        self.surface.set_overlay(None);
    }

    /// Moves the cursor with the keys of the navigation.
    fn move_cursor(&mut self, stroke: KeyStroke) {
        let rows: isize = match stroke.key {
            Key::ArrowUp => -1,
            Key::ArrowDown => 1,
            Key::PageUp => -10,
            Key::PageDown => 10,
            Key::Home => return self.move_cursor_to(0),
            Key::End => return self.move_cursor_to_last(),
            _ => return,
        };

        let visible = self.visible_rows();
        let panel = self.surface.active_mut();
        panel.move_cursor(rows);
        panel.ensure_cursor_visible(visible);
    }

    /// Moves the cursor to the first entry.
    fn move_cursor_to(&mut self, row: usize) {
        let visible = self.visible_rows();
        let panel = self.surface.active_mut();
        panel.set_cursor_row(row);
        panel.ensure_cursor_visible(visible);
    }

    /// Moves the cursor to the last entry.
    fn move_cursor_to_last(&mut self) {
        let last = self.surface.active().row_count().saturating_sub(1);
        self.move_cursor_to(last);
    }

    /// The number of rows that fit on the screen.
    ///
    /// WHY: the number is known after the drawing, because the height of the
    /// list is known then. The shell keeps the number of the last frame and
    /// uses it to scroll the cursor into the window.
    pub(super) fn visible_rows(&self) -> usize {
        self.visible_rows.max(1)
    }
}
