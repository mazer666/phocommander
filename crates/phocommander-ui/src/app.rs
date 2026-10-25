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
//! SIZE: the shell is the place where the panels, the binding table and the
//!       catalogue of the texts meet. The resolution of an event, the
//!       dispatch of the action and the drawing belong to the same flow,
//!       and a split would hide the order of the three steps. The ways in
//!       which the shell takes a language live in a module of their own, so
//!       that the shell itself stays readable.
//!
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

mod input;
mod language;

use std::path::PathBuf;

use crate::actions::Action;
use crate::commands::{Command, Request};
use crate::context::Context;
use crate::draw;
use crate::format::{Formats, PlainFormats};
use crate::i18n::{Argument, Catalogue, Texts};
use crate::input::PrimaryModifier;
use crate::jobs::JobSource;
use crate::keymap_view::KeymapView;
use crate::layout::{LayoutProfile, PanelStructure};
use crate::metrics::{Metrics, Platform};
use crate::preview::PreviewSource;
use crate::refresh::RefreshPolicy;
use crate::surface::{TransferRequest, WorkSurface};
use crate::theme::{Palette, Theme};
use crate::undo::UndoAvailability;

/// The application shell of the work surface.
pub struct WorkSurfaceApp {
    surface: WorkSurface,
    keymap: KeymapView,
    catalogue: Box<dyn Catalogue>,
    formats: Box<dyn Formats>,
    theme: Theme,
    metrics: Metrics,
    context: Context,
    primary: PrimaryModifier,
    commands: Vec<Command>,
    status: Option<StatusMessage>,
    activity: u64,
    visible_rows: usize,
    refresh_policy: RefreshPolicy,
    preview_source: Option<Box<dyn PreviewSource>>,
    job_source: Option<Box<dyn JobSource>>,
    pending_transfer: Option<PendingTransfer>,
    undo: UndoAvailability,
    /// The context of the last frame.
    ///
    /// WHY: a change of the language of the interface has to reach the screen
    ///       without a restart and without the user having to cause a
    ///       repaint (FR-124). The shell therefore keeps the context of egui
    ///       and asks it for a repaint when the language changes.
    egui_context: Option<egui::Context>,
}

/// A transfer that waits for the confirmation of the user.
///
/// WHY: on a television only one panel is visible, and the panel that
/// receives the entries appears as an overlay. The transfer starts when the
/// user confirms the destination and never before (FR-16).
#[derive(Debug)]
struct PendingTransfer {
    copy: bool,
    transfer: TransferRequest,
}

/// The message of the status line, before it is read from the catalogue.
///
/// WHY: the status line shows the message of the last command. The message
///       is kept as a key of the catalogue and as its arguments, and it is
///       read in the frame that draws it. A change of the language of the
///       interface therefore reaches the status line as well, and not only
///       the labels of the panels (FR-124).
#[derive(Debug, Clone)]
struct StatusMessage {
    /// The key of the catalogue that carries the message.
    key: String,

    /// The values that the message carries, with the names that the message
    /// of the catalogue expects.
    arguments: Vec<(&'static str, String)>,
}

impl WorkSurfaceApp {
    /// The shell with a work surface, a binding table and a catalogue.
    pub fn new(surface: WorkSurface, keymap: KeymapView, catalogue: Box<dyn Catalogue>) -> Self {
        Self {
            surface,
            keymap,
            catalogue,
            formats: Box::new(PlainFormats),
            theme: Theme::default(),
            metrics: Metrics::for_platform(Platform::default()),
            context: Context::default(),
            primary: PrimaryModifier::default(),
            commands: Vec::new(),
            status: None,
            activity: 0,
            visible_rows: 20,
            refresh_policy: RefreshPolicy::strict(),
            preview_source: None,
            job_source: None,
            pending_transfer: None,
            undo: UndoAvailability::Unavailable,
            egui_context: None,
        }
    }

    /// The shell with an undo that can be taken back.
    ///
    /// WHY: the undo of the specification is deliberately incomplete (FR-26).
    ///       The surface therefore shows the notice of the undo area as long
    ///       as an undo is on offer, and the notice names the limitation: the
    ///       undo removes the copies and restores nothing.
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// use phocommander_ui::undo::{UndoAvailability, UndoScope};
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
    /// .with_undo_availability(UndoAvailability::available(UndoScope::RemovesCopiesOnly));
    /// assert!(app.undo_availability().is_available());
    /// ```
    pub fn with_undo_availability(mut self, undo: UndoAvailability) -> Self {
        self.undo = undo;
        self
    }

    /// The undo that the surface can take back.
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
    /// assert!(!app.undo_availability().is_available());
    /// ```
    pub fn undo_availability(&self) -> &UndoAvailability {
        &self.undo
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

    /// The shell with a source for the quick view.
    ///
    /// WHY: the quick view reads the marked file through the reader of the
    /// viewer (FR-15). The interface crate may not depend on the viewer
    /// crate, so the composition root hands the reader over as the trait
    /// that the seam declares.
    pub fn with_preview_source(mut self, source: Box<dyn PreviewSource>) -> Self {
        self.preview_source = Some(source);
        self
    }

    /// The shell with a source for the list of the jobs.
    ///
    /// WHY: the list of the jobs shows what the queue of the operations
    /// does. The queue reaches the surface as a list of summaries, because
    /// the interface crate may not depend on the operations crate.
    pub fn with_job_source(mut self, source: Box<dyn JobSource>) -> Self {
        self.job_source = Some(source);
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
    ///
    /// WHY: the message is read from the catalogue in the moment of the
    ///       call, so that it speaks the language of the interface — also
    ///       after a change of that language (FR-124).
    pub fn status(&self) -> Option<String> {
        self.status
            .as_ref()
            .map(|message| self.message_of_error(&message.key, &message.arguments))
    }

    /// The number of changes of the shell.
    ///
    /// WHY: the counter is the evidence that an action was carried out. A
    /// test uses it to prove that every action of the catalogue does
    /// something.
    pub fn activity(&self) -> u64 {
        self.activity
    }

    /// The layout of the surface in the context of the shell.
    ///
    /// WHY: the profile is the answer to "how does the surface look in
    /// this context" (FR-106). The drawing asks for it and never asks for
    /// the context itself.
    pub fn profile(&self) -> LayoutProfile {
        LayoutProfile::for_context(self.context, &self.metrics)
    }

    /// Draws the work surface and resolves the input of the frame.
    pub fn update(&mut self, ui: &mut egui::Ui) {
        // WHY: the context of egui is kept, so that a change of the language
        //       of the interface can ask for a repaint (FR-124). The context
        //       is cheap to copy and it is taken once, in the first frame.
        if self.egui_context.is_none() {
            self.egui_context = Some((*ui.ctx()).clone());
        }
        // WHY: the index of the filter is rebuilt when the directory or the
        // filter changed. The check is cheap and happens before the input
        // is resolved, so that the cursor and the list always agree.
        self.surface.refresh();
        self.surface.apply_profile(&self.profile());
        self.resolve_input(ui.ctx());
        self.update_quick_view();
        self.update_jobs();
        self.draw(ui);
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
            "panel.tab.new" => self.surface.active_mut().new_tab(),
            "panel.tab.close" => {
                self.surface.active_mut().close_tab();
            }
            "panel.tab.move_to_other" => {
                self.surface.move_active_tab_to_other();
            }
            "panel.layout.toggle" => {
                self.surface.toggle_arrangement(&self.profile());
            }
            "panel.tree.toggle" => {
                self.surface.active_mut().toggle_tree();
            }
            "panel.quick_view.toggle" => {
                self.surface.quick_view_mut().toggle();
            }
            "panel.job_list.toggle" => {
                self.surface.toggle_job_list();
            }
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

    /// Carries out a request of the drawing.
    ///
    /// WHY: the drawing reads the state and never changes it. A click is
    /// therefore a request, and the shell carries it out like an action
    /// that came from the keyboard.
    fn handle_request(&mut self, request: Request) {
        self.activity += 1;
        match request {
            Request::Action(action) => self.dispatch(action),
            Request::SwitchTab(index) => {
                self.surface.active_mut().switch_tab(index);
            }
            Request::CloseTab(index) => {
                self.surface.active_mut().close_tab_at(index);
            }
            Request::OpenDirectory(path) => {
                let side = self.surface.active_side();
                self.commands.push(Command::LoadDirectory { side, path });
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

    /// Reads the message of an error from the catalogue.
    ///
    /// WHY: the status line of the surface shows the message of an error, and
    ///       the message comes from the catalogue. A catalogue that does not
    ///       know the key answers with the key itself, which is visible and
    ///       can be diagnosed — never with an empty line (FR-122).
    ///
    /// EXAMPLE:
    /// ```no_run
    /// use phocommander_ui::app::WorkSurfaceApp;
    /// ```
    fn message_of_error(&self, key: &str, arguments: &[(&'static str, String)]) -> String {
        let arguments: Vec<Argument<'_>> = arguments
            .iter()
            .map(|(name, value)| Argument::named(name, value.as_str()))
            .collect();
        self.catalogue
            .lookup_with(key, &arguments)
            .unwrap_or_else(|| key.to_string())
    }

    /// Builds the request that transfers the marked entries into the other
    /// panel.
    ///
    /// WHY: on a television only one panel is visible. The panel that
    /// receives the entries appears as an overlay there, and the user
    /// confirms the destination with the confirm key — the transfer never
    /// starts without the destination the user has seen (FR-16).
    fn request_transfer(&mut self, copy: bool) {
        let transfer = match self.surface.transfer_to_other() {
            Ok(transfer) => transfer,
            Err(error) => {
                // WHY: the message of the error is kept as a key and as its
                //       arguments, and it is read from the catalogue in the
                //       frame that draws it. The status line therefore speaks
                //       the language of the interface, also after a change of
                //       that language (FR-05, FR-124).
                self.status = Some(StatusMessage {
                    key: error.message_key().to_string(),
                    arguments: error.message_arguments(),
                });
                return;
            }
        };

        if self.profile().structure() == PanelStructure::SinglePanelWithOverlay {
            self.pending_transfer = Some(PendingTransfer { copy, transfer });
            let other = self.surface.active_side().other();
            self.surface.set_overlay(Some(other));
            return;
        }

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

    /// Reads the content of the marked file for the quick view.
    ///
    /// WHY: the quick view reads exactly one file — the marked one — and
    /// the reader of the viewer refuses the read when the rule
    /// "read_only_on_demand" is in force (FR-15, section 9.7).
    fn update_quick_view(&mut self) {
        if !self.surface.quick_view().is_visible() {
            return;
        }
        let path: Option<PathBuf> = {
            let panel = self.surface.active();
            panel
                .marked_entries()
                .first()
                .map(|entry| panel.path().join(&entry.name))
        };
        let WorkSurfaceApp {
            surface,
            preview_source,
            ..
        } = self;
        if let Some(source) = preview_source.as_deref() {
            surface.quick_view_mut().update(source, path.as_deref());
        }
    }

    /// Reads the rows of the list of the jobs.
    fn update_jobs(&mut self) {
        let WorkSurfaceApp {
            surface,
            job_source,
            ..
        } = self;
        if let Some(source) = job_source.as_deref() {
            surface.set_jobs(source.summaries());
        }
    }

    /// Draws the work surface.
    fn draw(&mut self, ui: &mut egui::Ui) {
        let palette = self.palette();
        let profile = self.profile();
        // WHY: the notice is read before the fields of the shell are
        // borrowed, so that the text of the undo area is known when the
        // surface is drawn.
        let undo_notice = self.undo.notice(self.catalogue.as_ref());
        // WHY: the message of the status line is read from the catalogue in
        //       the frame that draws it, so that it speaks the language of
        //       the interface — also after a change of that language
        //       (FR-124).
        let status = self
            .status
            .as_ref()
            .map(|message| self.message_of_error(&message.key, &message.arguments));
        let WorkSurfaceApp {
            surface,
            catalogue,
            formats,
            metrics,
            visible_rows,
            ..
        } = self;
        let texts = Texts::new(catalogue.as_ref());

        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, palette.background);

        let mut requested: Vec<Request> = Vec::new();
        // WHY: the bundle carries everything the drawing needs besides the
        //       surface itself. It is built once per frame.
        let style = draw::DrawStyle::new(metrics, &palette, &texts, formats.as_ref(), &profile);
        let rows = draw::draw_surface(
            ui,
            surface,
            &style,
            status.as_deref(),
            undo_notice.as_deref(),
            &mut requested,
        );
        *visible_rows = rows.max(1);

        for request in requested {
            self.handle_request(request);
        }
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
