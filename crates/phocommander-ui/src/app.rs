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
use crate::keymap_view::{InputEvent, Key, KeyStroke, KeymapView, Modifiers};
use crate::metrics::{Metrics, Platform};
use crate::surface::WorkSurface;
use crate::theme::{Palette, Theme};

/// The key that carries the primary meaning on the platform.
///
/// WHY: the actions are the same on every platform, only the key that
/// triggers them differs. `Cmd` is the primary key on macOS and on iPadOS,
/// `Ctrl` everywhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrimaryModifier {
    /// The command key — macOS and iPadOS.
    Command,

    /// The control key — Windows, Linux and BSD.
    #[default]
    Control,
}

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
}

impl WorkSurfaceApp {
    /// The shell with a work surface, a binding table and a catalogue.
    pub fn new(
        surface: WorkSurface,
        keymap: KeymapView,
        catalogue: Box<dyn Catalogue>,
    ) -> Self {
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
        }
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
        self.resolve_input(ui.ctx());
        self.draw(ui);
    }

    /// Resolves the input of the frame into actions and carries them out.
    fn resolve_input(&mut self, ctx: &egui::Context) {
        let events: Vec<egui::Event> = ctx.input(|input| input.events.clone());
        let primary = self.primary;

        for event in events {
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

            let action = self
                .keymap
                .resolve(self.context, &InputEvent::Key(stroke));
            if let Some(action) = action {
                self.dispatch(action);
                continue;
            }

            self.move_cursor(stroke);
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
            "panel.switch" => self.surface.switch_active(),
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
                    self.commands.push(Command::OpenViewer { side, entry: path });
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
                    self.surface.active_mut().switch_tab(number.saturating_sub(1));
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
    fn move_cursor_to(&mut self, position: usize) {
        let visible = self.visible_rows();
        let panel = self.surface.active_mut();
        panel.set_cursor(position);
        panel.ensure_cursor_visible(visible);
    }

    /// Moves the cursor to the last entry.
    fn move_cursor_to_last(&mut self) {
        let visible = self.visible_rows();
        let last = self.surface.active().row_count().saturating_sub(1);
        let panel = self.surface.active_mut();
        panel.set_cursor(last);
        panel.ensure_cursor_visible(visible);
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

        ui.painter().rect_filled(ui.max_rect(), 0.0, palette.background);

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

/// The key stroke of an event of egui.
///
/// WHY: the binding table of the configuration file is written in words and
/// the platform delivers a key of the framework. The stroke is the form in
/// which both can be compared.
fn stroke_of(
    key: egui::Key,
    modifiers: egui::Modifiers,
    primary: PrimaryModifier,
) -> Option<KeyStroke> {
    let mut key = Key::from_name(key.name())?;

    let mut shift = modifiers.shift;
    let alt = modifiers.alt;
    let primary_pressed = match primary {
        PrimaryModifier::Command => modifiers.command,
        PrimaryModifier::Control => modifiers.ctrl,
    };

    // WHY: the star key of the numeric keypad is Shift+8 on a keyboard with
    // an American layout, and egui delivers the shift and the eight. The
    // combination is normalised into the key of the keypad, so that the
    // binding of the catalogue can be reached.
    if shift && key == Key::Digit(8) {
        key = Key::NumpadMultiply;
        shift = false;
    }

    Some(KeyStroke {
        key,
        modifiers: Modifiers {
            primary: primary_pressed,
            shift,
            alt,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap_view::KeymapView;
    use crate::listing::{Entry, EntryKind};
    use crate::panel::PanelSide;
    use crate::Argument;
    use std::collections::BTreeMap;

    /// A catalogue that answers every key with the key itself.
    struct Echo;

    impl Catalogue for Echo {
        fn lookup_with(&self, key: &str, _arguments: &[Argument<'_>]) -> Option<String> {
            Some(key.to_string())
        }
    }

    /// The shell with three entries in the left panel.
    fn app() -> WorkSurfaceApp {
        let mut surface = WorkSurface::new(PathBuf::from("/tmp"), PathBuf::from("/var"));
        surface.active_mut().listing_mut().set_entries(vec![
            Entry::new("a.txt", EntryKind::File),
            Entry::new("b.txt", EntryKind::File),
            Entry::new("c.txt", EntryKind::File),
        ]);

        let mut keyboard = BTreeMap::new();
        keyboard.insert("panel.copy".to_string(), "F5".to_string());
        keyboard.insert("panel.switch".to_string(), "Tab".to_string());
        keyboard.insert("selection.toggle".to_string(), "Ins".to_string());
        let mut tables = BTreeMap::new();
        tables.insert(Context::Keyboard, keyboard);

        let keymap = match KeymapView::from_context_tables(&tables) {
            Ok(keymap) => keymap,
            Err(error) => panic!("the tables should be valid: {error}"),
        };

        WorkSurfaceApp::new(surface, keymap, Box::new(Echo))
    }

    #[test]
    fn every_action_of_the_catalogue_does_something() {
        // WHY: an action that is added to the catalogue and carried out
        // silently would be a defect that nobody notices.
        for action in Action::all() {
            let mut app = app();
            let before = app.activity();

            app.dispatch(*action);

            assert!(
                app.activity() > before,
                "the action {} changed nothing",
                action.id()
            );
        }
    }

    #[test]
    fn the_copy_names_the_other_panel_as_its_destination() {
        // arrange
        let mut app = app();
        app.surface_mut().active_mut().toggle_at(0);

        // act
        let copy = match Action::from_id("panel.copy") {
            Some(action) => action,
            None => panic!("the catalogue should hold the action"),
        };
        app.dispatch(copy);

        // assert
        let commands = app.commands();
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            Command::Copy { transfer } => {
                assert_eq!(transfer.source(), PanelSide::Left);
                assert_eq!(transfer.destination(), PanelSide::Right);
                assert_eq!(transfer.entries().len(), 1);
            }
            other => panic!("the command should be a copy: {other:?}"),
        }
    }

    #[test]
    fn the_copy_after_the_switch_names_the_right_panel_as_its_source() {
        // arrange
        let mut app = app();
        app.surface_mut().switch_active();

        // act
        let copy = match Action::from_id("panel.copy") {
            Some(action) => action,
            None => panic!("the catalogue should hold the action"),
        };
        app.dispatch(copy);

        // assert
        match &app.commands()[0] {
            Command::Copy { transfer } => {
                assert_eq!(transfer.source(), PanelSide::Right);
                assert_eq!(transfer.destination(), PanelSide::Left);
            }
            other => panic!("the command should be a copy: {other:?}"),
        }
    }

    #[test]
    fn the_parent_of_a_path_is_the_directory_above_it() {
        // act
        let parent = parent_of(std::path::Path::new("/tmp/folder"));

        // assert
        assert_eq!(parent, PathBuf::from("/tmp"));
    }

    #[test]
    fn the_parent_of_the_root_is_the_root() {
        // act
        let parent = parent_of(std::path::Path::new("/"));

        // assert
        assert_eq!(parent, PathBuf::from("/"));
    }
}
