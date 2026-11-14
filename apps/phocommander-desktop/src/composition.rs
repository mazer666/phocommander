//! The composition of the desktop build.
//!
//! WHAT: the functions that build the application out of the configuration
//!       of the user ([`build_desktop_app`], [`build_application`]).
//! WHY: the application is the composition root of the workspace: it is the
//!       only crate that may see the core crate, the configuration crate, the
//!       operations crate, the viewer crate and the interface crate at the
//!       same time. The composition therefore belongs to the library and not
//!       to the binary — a composition that lives in the binary cannot be
//!       tested, and a composition that cannot be tested is a composition
//!       that drifts apart from the configuration (ADR-0001, option B).
//!
//! EXAMPLE:
//! ```no_run
//! use phocommander_config::Config;
//! use phocommander_desktop::composition::build_application;
//!
//! let app = build_application(&Config::default());
//! assert!(app.surface().visibility().show_hidden);
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::desktop::DesktopApp;
use crate::filesystem::HostFileSystem;
use crate::jobs::QueueJobSource;
use crate::language::{ConfigWatcher, InterfaceLanguage, locale_of};
use crate::preview::ViewerPreviewSource;
use crate::rename::{FileTemplateStore, OpsRenameEngine};
use crate::security::SecurityPlan;
use phocommander_config::{
    Config, CopyMode as ConfigCopyMode, RightClickAction as ConfigRightClick,
    SyncMode as ConfigSyncMode, resolve_security,
};
use phocommander_ops::copy::CopyMode as OpsCopyMode;
use phocommander_ops::queue::JobQueue;
use phocommander_platform::config_directory;
use phocommander_ui::{
    Context, KeymapView, Metrics, Platform, Theme, WorkSurface, WorkSurfaceApp,
    input::PrimaryModifier, visuals,
};
use phocommander_viewer::reader::ViewerReader;

/// Builds the shell of the interface out of the configuration.
///
/// WHY: the function builds everything that the surface needs — the surface
///       itself, the keymap, the catalogue of the texts, the formats of the
///       values, the theme and the dimensions of the platform. It is the
///       only place where these parts come together.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::Config;
/// use phocommander_desktop::composition::build_application;
///
/// let app = build_application(&Config::default());
/// assert!(app.surface().visibility().show_hidden);
/// ```
pub fn build_application(config: &Config) -> WorkSurfaceApp {
    let surface = WorkSurface::new(
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    );

    let keymap = read_keymap().unwrap_or_default();
    let metrics = Metrics::for_platform(host_platform());

    // WHY: the language of the interface is the catalogue of the texts and
    //       the formats of the values. Both belong to the language, and the
    //       shell takes both of them (FR-05, FR-123).
    let mut language = InterfaceLanguage::of(locale_of(config.general.language));

    // WHY: the user writes in the configuration file whether the hidden
    //       files and the files of the system shall be shown. The setting
    //       reaches the surface, and the surface is what the adapter asks
    //       before it reads a directory (FR-74).
    let visibility = phocommander_ui::visibility::ListingOptions {
        show_hidden: config.appearance.show_hidden_files,
        show_system: config.appearance.show_system_files,
    };

    // WHY: the user writes in the configuration file whether the right
    //       mouse button opens a menu or marks the file under the pointer.
    //       The setting reaches the shell, and the shell is what carries
    //       the action out (FR-17).
    let right_click = match config.panels.right_click_action {
        ConfigRightClick::ContextMenu => phocommander_ui::visibility::RightClickAction::ContextMenu,
        ConfigRightClick::ToggleSelection => {
            phocommander_ui::visibility::RightClickAction::ToggleSelection
        }
    };

    WorkSurfaceApp::new(surface, keymap, language.take_catalogue())
        .with_visibility(visibility)
        .with_right_click(right_click)
        .with_formats(language.take_formats())
        .with_theme(Theme::Dark)
        .with_metrics(metrics)
        .with_context(Context::Keyboard)
        // WHY: the primary key is the command key on macOS and on iPadOS
        //       and the control key everywhere else. The actions are the
        //       same on every platform; only the key that triggers them
        //       differs. Without this line the user of a mac presses
        //       Ctrl+F, the filter does not open, and the key looks
        //       broken (FR-101).
        .with_primary_modifier(primary_modifier_of(host_platform()))
        // WHY: the dialog of the multiple rename asks the engine for the new
        //       names, and the engine of the operations lives in another
        //       crate. The composition root hands the implementation over
        //       (FR-34).
        .with_rename_engine(Box::new(OpsRenameEngine::new(Box::new(
            HostFileSystem::new(),
        ))))
        .with_template_store(Box::new(FileTemplateStore::new(template_store_path())))
}

/// The colours of the application.
///
/// WHY: the function is the only place where the colours arise. The theme
///       of the frame and the palette of the interface both come from here,
///       so that they cannot drift apart. The application sets these
///       colours when it starts — without that the window shows the
///       standard colours of the framework, and the theme of the project
///       exists only in the source.
pub fn theme_visuals(system_prefers_dark: bool) -> egui::Visuals {
    visuals(Theme::Dark, system_prefers_dark)
}

/// Der Speicher, in dem die Oberfläche ihre Zustände hält.
pub type SharedSurface = Arc<WorkSurface>;

/// The path of the file of the named templates.
///
/// WHY: the templates of the user live beside the configuration file, so
///       that they are there after a restart of the program (FR-36). A
///       machine without a configuration directory has no place for them,
///       and the dialog then works without stored templates.
pub fn build_desktop_app(config: &Config) -> DesktopApp {
    let locale = locale_of(config.general.language);
    // WHY: the rules of the security section decide what a copy may do, and
    //       they reach the jobs through the plan.
    let rules = resolve_security(&config.security);
    let plan = SecurityPlan::from_effective(&rules);
    let watcher = match configuration_path() {
        Some(path) => ConfigWatcher::new(path),
        None => ConfigWatcher::without_file(),
    };

    // WHY: the waiting room holds the file system of the host, and the
    //       worker of the room carries the jobs out. The room is therefore
    //       built here, where the file system of the host lives.
    //
    //       The number of the workers is a setting of the user. The
    //       setting of zero means "as many as the machine has", and the
    //       room needs at least one worker — a room without a worker
    //       would take every job and never carry one out.
    let queue = Arc::new(JobQueue::new(
        Arc::new(HostFileSystem::new()),
        worker_count_of(config),
    ));

    // WHY: the list of the jobs reads from the same room, so that the user
    //       sees the jobs that are really running.
    let application = build_application(config)
        .with_job_source(Box::new(QueueJobSource::new(queue.clone())))
        // WHY: the quick view reads the file that the user marked, and it
        //       reads it only when the rules allow it. The reader knows the
        //       rules, and the adapter hands the content to the surface.
        .with_preview_source(Box::new(ViewerPreviewSource::new(ViewerReader::new(
            *plan.read_policy(),
            config.viewer.max_file_size.unwrap_or(10_485_760),
        ))));

    let mut app = DesktopApp::with_queue(application, locale, watcher, queue);
    // WHY: the way a file is copied is a setting of the user (FR-22b).
    app.set_copy_mode(copy_mode_of(config));
    // WHY: the way of the synchronisation is a setting of the user, and it
    //       decides whether the synchronisation only shows the plan or
    //       also brings the target into the shape of the source (FR-22b).
    app.set_sync_mode(sync_mode_of(config));
    // WHY: the rules that decide whether the copy compares the checksums
    //       and whether a link is dereferenced come from the security
    //       section of the configuration. Without them the plan would be
    //       reported and never applied — and the copy would never compare
    //       a single checksum.
    app.set_copy_rules(*plan.copy_policy());
    app
}

/// The number of the workers of the waiting room, out of the configuration file.
///
/// WHY: the setting of the user is a number, and the number zero means
///       "as many as the machine has". The room needs at least one
///       worker, because a room without a worker takes every job and
///       carries none of them out.
fn worker_count_of(config: &Config) -> usize {
    let eingestellt = config.advanced.worker_threads as usize;
    if eingestellt == 0 {
        std::thread::available_parallelism().map_or(1, |n| n.get())
    } else {
        eingestellt
    }
}

/// The way a directory is synchronised, out of the configuration file.
///
/// WHY: the configuration names the way in the spelling of the catalogue,
///       and the operations know the way in their own spelling. The two
///       meet here, in the one place that knows both. The setting of the
///       user decides whether the synchronisation only shows the plan or
///       also brings the target into the shape of the source (FR-22b).
fn sync_mode_of(config: &Config) -> OpsCopyMode {
    match config.copy.sync_mode {
        ConfigSyncMode::SyncMirror => OpsCopyMode::SyncMirror,
        ConfigSyncMode::SyncPreview => OpsCopyMode::SyncPreview,
        ConfigSyncMode::SyncUpdate => OpsCopyMode::SyncUpdate,
        ConfigSyncMode::SyncTwoWay => OpsCopyMode::SyncTwoWay,
    }
}

/// The way a file is copied, out of the configuration file.
///
/// WHY: the configuration names the way in the spelling of the catalogue,
///       and the operations know the way in their own spelling. The two
///       meet here, in the one place that knows both.
fn copy_mode_of(config: &Config) -> OpsCopyMode {
    match config.copy.mode {
        ConfigCopyMode::Quick => OpsCopyMode::Quick,
        ConfigCopyMode::Atomic => OpsCopyMode::Atomic,
        ConfigCopyMode::Verified => OpsCopyMode::Verified,
    }
}

/// Builds the application.
///
/// WHY: the function builds everything that the surface needs — the surface
///       itself, the keymap, the catalog of the texts, the formats of the
///       values, the theme and the dimensions of the platform. It is the
///       only place where these parts come together.
///
/// The path of the file of the named templates.
///
/// WHY: the templates of the user live beside the configuration file, so
///       that they are there after a restart of the program (FR-36). A
///       machine without a configuration directory has no place for them,
///       and the dialog then works without stored templates.
pub fn primary_modifier_of(platform: Platform) -> PrimaryModifier {
    match platform {
        Platform::MacOs | Platform::Ios => PrimaryModifier::Command,
        Platform::Windows | Platform::Linux | Platform::Bsd | Platform::Android => {
            PrimaryModifier::Control
        }
    }
}

/// Liest die Tastenbelegung aus der Konfigurationsdatei.
///
/// WHY: die Belegung ist eine Datei, die der Benutzer bearbeitet. Eine Datei,
///       die fehlt oder die nicht gelesen werden kann, ist kein Grund, das
///       Programm nicht zu starten: die Oberfläche startet dann ohne
///       Tastenbelegung, und die Aktionen sind über die Aktionspalette
/// The name of the configuration file in the configuration directory.
const CONFIGURATION_FILE_NAME: &str = "config.toml";

/// The name of the file of the named templates.
const TEMPLATE_STORE_FILE_NAME: &str = "rename-templates.txt";

/// The name of the file of the bindings.
const KEYMAP_FILE_NAME: &str = "keymap.toml";

fn read_keymap() -> Option<KeymapView> {
    // WHY: the defaults come first, so that a user without a file of his
    //       own still has every binding. The file overrides them.
    let view = KeymapView::with_defaults(&Default::default()).ok()?;

    let Ok(directory) = config_directory() else {
        return Some(view);
    };
    let path = directory.join(KEYMAP_FILE_NAME);
    let Ok(text) = std::fs::read_to_string(path) else {
        return Some(view);
    };
    let Some(tables) = parse_keymap(&text) else {
        return Some(view);
    };

    // WHY: a file with a broken binding must not take the whole set of the
    //       bindings away. The view of the defaults stays in force, and the
    //       file is the part that could not be read.
    match KeymapView::with_defaults(&tables) {
        Ok(ueberschrieben) => Some(ueberschrieben),
        Err(_) => Some(view),
    }
}

/// Liest die Tastenbelegung aus einer Datei mit Abschnitten.
///
/// WHY: die Datei hat einen Abschnitt je Kontext (`[input.keyboard]`) und in
///       ihm eine Zeile je Aktion. Der Parser liest beide Ebenen und legt
///       das Ergebnis in einer Tabelle ab, die die Oberfläche liest.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_desktop::composition::parse_keymap;
///
/// let tables = parse_keymap("[input.keyboard]\n\"panel.copy\" = \"F5\"\n");
/// assert!(tables.is_some());
/// ```
pub fn parse_keymap(text: &str) -> Option<BTreeMap<Context, BTreeMap<String, String>>> {
    let mut tables: BTreeMap<Context, BTreeMap<String, String>> = BTreeMap::new();
    let mut context = Context::Keyboard;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(section) = line
            .strip_prefix("[input.")
            .and_then(|s| s.strip_suffix(']'))
        {
            context = match section {
                "keyboard" => Context::Keyboard,
                "pointer" => Context::Pointer,
                "touch" => Context::Touch,
                "dpad" => Context::Dpad,
                "controller" => Context::Controller,
                _ => continue,
            };
            tables.entry(context).or_default();
            continue;
        }

        let Some((action, binding)) = line.split_once('=') else {
            continue;
        };
        let action = action.trim().trim_matches('"');
        let binding = binding.trim().trim_matches('"');
        if action.is_empty() || binding.is_empty() {
            continue;
        }
        tables
            .entry(context)
            .or_default()
            .insert(action.to_string(), binding.to_string());
    }

    if tables.is_empty() {
        None
    } else {
        Some(tables)
    }
}

/// Liest die Konfigurationsdatei, oder die sichere Konfiguration, wenn sie
/// The path of the file of the named templates.
pub fn template_store_path() -> PathBuf {
    configuration_directory()
        .map(|directory| directory.join(TEMPLATE_STORE_FILE_NAME))
        .unwrap_or_else(|| PathBuf::from(TEMPLATE_STORE_FILE_NAME))
}

/// The path of the file of the configuration.
pub fn configuration_path() -> Option<PathBuf> {
    configuration_directory().map(|directory| directory.join(CONFIGURATION_FILE_NAME))
}

/// The directory in which the files of the user live.
fn configuration_directory() -> Option<PathBuf> {
    phocommander_platform::config_directory().ok()
}

/// The platform the program runs on.
pub fn host_platform() -> Platform {
    match phocommander_platform::host_platform() {
        phocommander_platform::Platform::MacOs => Platform::MacOs,
        phocommander_platform::Platform::Ios => Platform::MacOs,
        phocommander_platform::Platform::Android => Platform::Android,
        phocommander_platform::Platform::Windows => Platform::Windows,
        phocommander_platform::Platform::Linux => Platform::Linux,
        phocommander_platform::Platform::Bsd => Platform::Bsd,
    }
}
