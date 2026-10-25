//! The application shell of the work surface.
//!
//! WHAT: the shell that draws the panels, resolves the input of the frame
//!       and hands the commands to the application ([`WorkSurfaceApp`]).
//! WHY: the shell is the only place where the three parts of the interface
//!       meet: the state of the panels, the binding table and the catalogue
//!       of the texts. It resolves an input event into an action, dispatches
//!       the action and draws the result — and it performs no file
//!       operation itself: every operation leaves the surface as a
//!       [`Command`] with its source and its destination named.
//! EXAMPLE:
//! ```no_run
//! use phocommander_ui::app::WorkSurfaceApp;
//! use phocommander_ui::surface::WorkSurface;
//! use std::path::PathBuf;
//!
//! # struct Empty;
//! # impl phocommander_ui::i18n::Catalogue for Empty {
//! #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::i18n::Argument<'_>]) -> Option<String> {
//! #         None
//! #     }
//! # }
//! let app = WorkSurfaceApp::new(
//!     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
//!     Default::default(),
//!     Box::new(Empty),
//! );
//! assert!(app.commands().is_empty());
//! ```

use std::path::PathBuf;

use crate::actions::Action;
use crate::commands::Command;
use crate::context::Context;
use crate::draw;
use crate::i18n::{Catalogue, Texts};
use crate::input::{PrimaryModifier, stroke_of};
use crate::keymap_view::{InputEvent, Key, KeyStroke, KeymapView};
use crate::metrics::{Metrics, Platform};
use crate::refresh::RefreshPolicy;
use crate::surface::WorkSurface;
use crate::theme::{Palette, Theme};

/// The application shell of the work surface.
pub struct WorkSurfaceApp {
    surface: WorkSurface,
    keymap: KeymapView,
    catalogue: Box<dyn Catalogue>,
    theme: Theme,
    metrics: Metrics,
    context: Context,
    primary: PrimaryModifier,
    commands: Vec<Command>,
    status: Option<String>,
    activity: u64,
    visible_rows: usize,
    refresh_policy: RefreshPolicy,
}

impl WorkSurfaceApp {
    /// The shell with a work surface, a binding table and a catalogue.
    pub fn new(surface: WorkSurface, keymap: KeymapView, catalogue: Box<dyn Catalogue>) -> Self {
        Self {
            surface,
            keymap,
            catalogue,
            theme: Theme::default(),
            metrics: Metrics::for_platform(Platform::default()),
            context: Context::default(),
            primary: PrimaryModifier::default(),
            commands: Vec::new(),
            status: None,
            activity: 0,
            visible_rows: 20,
            refresh_policy: RefreshPolicy::strict(),
        }
    }

    /// The shell with another rule for reading a directory again.
    ///
    /// WHY: the rule "verify_directory_before_refresh" of the security
    /// section decides whether the content of a directory is read again
    /// before a panel shows it. The value comes from the configuration, and
    /// the composition root hands it over.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::refresh::RefreshPolicy;
    /// # use phocommander_ui::{Catalogue, WorkSurface};
    /// # use phocommander_ui::keymap_view::KeymapView;
    /// # use std::path::PathBuf;
    /// # struct Empty;
    /// # impl Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::i18n::Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     KeymapView::default(),
    ///     Box::new(Empty),
    /// )
    /// .with_refresh_policy(RefreshPolicy::new(false));
    /// assert!(!app.refresh_policy().verifies_before_refresh());
    /// ```
    pub fn with_refresh_policy(mut self, refresh_policy: RefreshPolicy) -> Self {
        self.refresh_policy = refresh_policy;
        self
    }

    /// The rule for reading a directory again.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// # use phocommander_ui::{Catalogue, WorkSurface};
    /// # use phocommander_ui::keymap_view::KeymapView;
    /// # use std::path::PathBuf;
    /// # struct Empty;
    /// # impl Catalogue for Empty {
    /// #     fn lookup_with(&self, _key: &str, _arguments: &[phocommander_ui::i18n::Argument<'_>]) -> Option<String> {
    /// #         None
    /// #     }
    /// # }
    /// let app = WorkSurfaceApp::new(
    ///     WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var")),
    ///     KeymapView::default(),
    ///     Box::new(Empty),
    /// );
    /// assert!(app.refresh_policy().verifies_before_refresh());
    /// ```
    pub fn refresh_policy(&self) -> &RefreshPolicy {
        &self.refresh_policy
    }

    /// The shell with another colour scheme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// The shell with other metrics.
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// The shell with another interaction context.
    pub fn with_context(mut self, context: Context) -> Self {
        self.context = context;
        self
    }

    /// The shell with another primary modifier.
    pub fn with_primary_modifier(mut self, primary: PrimaryModifier) -> Self {
        self.primary = primary;
        self
    }

    /// The work surface.
    pub fn surface(&self) -> &WorkSurface {
        &self.surface
    }

    /// The work surface.
    pub fn surface_mut(&mut self) -> &mut WorkSurface {
        &mut self.surface
    }

    /// The commands of the shell since the last call of [`Self::take_commands`].
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Takes the commands of the shell.
    ///
    /// WHY: the shell does not carry out the commands itself. The
    /// application takes them and hands them to the operations module.
    pub fn take_commands(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.commands)
    }

    /// The last message of the status line.
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// The number of changes of the shell.
    ///
    /// WHY: the counter is the evidence that an action was carried out. A
    /// test uses it to prove that every action of the catalogue does
    /// something.
    pub fn activity(&self) -> u64 {
        self.activity
    }

    /// Draws the work surface and resolves the input of the frame.
    pub fn update(&mut self, ui: &mut egui::Ui) {
        // WHY: the index of the filter is rebuilt when the directory or the
        // filter changed. The check is cheap and happens before the input
        // is resolved, so that the cursor and the list always agree.
        self.surface.refresh();
        self.resolve_input(ui.ctx());
        self.draw(ui);
    }

    /// Resolves the input of the frame into actions and carries them out.
    fn resolve_input(&mut self, ctx: &egui::Context) {
        let events: Vec<egui::Event> = ctx.input(|input| input.events.clone());
        let primary = self.primary;

        for event in events {
            // WHY: the characters of the filter field arrive as a text
            // event and not as a key event, because the keyboard layout of
            // the user decides which characters a key produces.
            if let egui::Event::Text(text) = &event
                && self.filter_is_editing()
            {
                self.surface.active_mut().filter_insert(text);
                continue;
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

            let Some(stroke) = stroke_of(key, modifiers, primary) else {
                continue;
            };

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

            self.move_cursor(stroke);
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
    fn filter_key(&mut self, stroke: KeyStroke) -> bool {
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

    /// Carries out an action.
    ///
    /// WHY: every action of the catalogue is named here, so that an action
    /// that is added to the catalogue cannot be carried out silently. The
    /// catch-all arm reports the action as deferred, and the test of the
    /// workspace proves that every action changes something.
    fn dispatch(&mut self, action: Action) {
        self.activity += 1;
        match action.id() {
            "panel.switch" => {
                self.surface.switch_active();
                self.request_verification();
            }
            "panel.swap" => {
                self.surface.swap_panels();
                self.commands.push(Command::SwapPanels);
            }
            "panel.copy" => self.request_transfer(true),
            "panel.move" => self.request_transfer(false),
            "panel.delete" => {
                let side = self.surface.active_side();
                let entries = self.marked_paths();
                self.commands.push(Command::Delete { side, entries });
            }
            "panel.rename" => {
                let side = self.surface.active_side();
                let entries = self.marked_paths();
                self.commands.push(Command::Rename { side, entries });
            }
            "panel.mkdir" => {
                let side = self.surface.active_side();
                let path = self.surface.active().path().to_path_buf();
                self.commands.push(Command::CreateDirectory { side, path });
            }
            "panel.view" => {
                let side = self.surface.active_side();
                if let Some(entry) = self.surface.active().cursor_entry() {
                    let path = self.surface.active().path().join(&entry.name);
                    self.commands
                        .push(Command::OpenViewer { side, entry: path });
                }
            }
            "panel.open_in_other" => {
                let source = self.surface.active_side();
                let destination = source.other();
                if let Some(entry) = self.surface.active().cursor_entry() {
                    let path = self.surface.active().path().join(&entry.name);
                    self.commands.push(Command::OpenInOtherPanel {
                        source,
                        destination,
                        entry: path,
                    });
                }
            }
            "panel.reload" => {
                let side = self.surface.active_side();
                let path = self.surface.active().path().to_path_buf();
                self.commands.push(Command::LoadDirectory { side, path });
            }
            "panel.parent" => {
                let side = self.surface.active_side();
                let path = parent_of(self.surface.active().path());
                self.commands.push(Command::LoadDirectory { side, path });
            }
            "panel.drives" => self.defer(action),
            "panel.edit" => self.defer(action),
            "panel.filter" => self.surface.active_mut().open_filter(),
            "panel.next_tab" => self.surface.active_mut().next_tab(),
            "panel.prev_tab" => self.surface.active_mut().previous_tab(),
            "selection.toggle" => self.surface.active_mut().toggle_at_cursor(),
            "selection.all" => self.surface.active_mut().select_all(),
            "selection.none" => self.surface.active_mut().clear_selection(),
            "selection.invert" => self.surface.active_mut().invert_selection(),
            "selection.by_pattern" => self.defer(action),
            "app.menu" => self.commands.push(Command::Menu),
            "app.quit" => self.commands.push(Command::Quit),
            "palette.open" => self.defer(action),
            "viewer.find_next" | "viewer.find_previous" => self.defer(action),
            "rename.conflict_next" | "rename.conflict_previous" => self.defer(action),
            other => {
                // WHY: the tab bindings are the only actions whose
                // identifier carries a number. They are handled here so
                // that the catalogue can grow without a change of this
                // function.
                if let Some(number) = other
                    .strip_prefix("panel.tab.")
                    .and_then(|number| number.parse::<usize>().ok())
                {
                    self.surface
                        .active_mut()
                        .switch_tab(number.saturating_sub(1));
                } else {
                    self.defer(action);
                }
            }
        }
    }

    /// Reports an action that this milestone does not carry out.
    fn defer(&mut self, action: Action) {
        self.commands.push(Command::Deferred { action });
    }

    /// Asks the application to read the directory of the active panel again.
    ///
    /// WHY: the rule "verify_directory_before_refresh" says that the
    /// content of a directory is read again before it is shown, so that a
    /// change that was made elsewhere is noticed. The surface holds the
    /// listing and the application reads the directory; the rule decides
    /// whether the request is made at all.
    fn request_verification(&mut self) {
        if !self.refresh_policy.verifies_before_refresh() {
            return;
        }
        let side = self.surface.active_side();
        let path = self.surface.active().path().to_path_buf();
        self.commands.push(Command::LoadDirectory { side, path });
    }

    /// Builds the request that transfers the marked entries into the other
    /// panel.
    fn request_transfer(&mut self, copy: bool) {
        let transfer = match self.surface.transfer_to_other() {
            Ok(transfer) => transfer,
            Err(error) => {
                self.status = Some(error.to_string());
                return;
            }
        };

        if copy {
            self.commands.push(Command::Copy { transfer });
        } else {
            self.commands.push(Command::Move { transfer });
        }
    }

    /// The paths of the marked entries of the panel that has the focus.
    fn marked_paths(&self) -> Vec<PathBuf> {
        self.surface
            .marked_entries()
            .into_iter()
            .map(|entry| entry.path)
            .collect()
    }

    /// Moves the cursor with the keys of the navigation.
    ///
    /// WHY: the catalogue of the specification names no action for moving
    /// the cursor, and a file list that cannot be walked is not a file
    /// manager. The keys of the navigation are therefore handled by the
    /// surface itself. The deviation is reported in the handover of this
    /// milestone, and the owner decides whether the catalogue grows the
    /// actions `panel.cursor.*` or the surface keeps this behaviour.
    fn move_cursor(&mut self, stroke: KeyStroke) {
        if !stroke.modifiers.is_empty() {
            return;
        }

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
    fn visible_rows(&self) -> usize {
        self.visible_rows.max(1)
    }

    /// Draws the work surface.
    fn draw(&mut self, ui: &mut egui::Ui) {
        let palette = self.palette();
        let WorkSurfaceApp {
            surface,
            catalogue,
            metrics,
            context,
            status,
            visible_rows,
            ..
        } = self;
        let texts = Texts::new(catalogue.as_ref());

        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, palette.background);

        let rows = draw::draw_surface(
            ui,
            surface,
            metrics,
            &palette,
            &texts,
            *context,
            status.as_deref(),
        );
        *visible_rows = rows.max(1);
    }

    /// The colours of the work surface.
    fn palette(&self) -> Palette {
        match self.theme {
            Theme::Light => Palette::light(),
            Theme::Dark => Palette::dark(),
            Theme::System => match self.context {
                Context::Dpad => Palette::dark(),
                _ => Palette::light(),
            },
        }
    }
}

impl eframe::App for WorkSurfaceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.update(ui);
    }
}

/// The parent of a path.
fn parent_of(path: &std::path::Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => path.to_path_buf(),
    }
}

#[cfg(test)]
// WHY: a test may fail loudly when its arrangement is wrong. The rule
// that forbids a panic applies to the production code, and it is enforced
// there.
// WHY: the test module carries its own exception, because the file of the
// tests is a file of its own. The exception is therefore written once and
// not twice.
mod tests;
