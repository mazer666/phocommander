//! The desktop application of Phocommander.
//!
//! WHAT: the entry point of the macOS, the Windows, the Linux and the BSD
//!       build. It reads the configuration file, resolves the security rules
//!       that are in force, builds the surface and opens the window.
//! WHY: the program is the composition root of the workspace — the one place
//!       that may see the core crate, the configuration crate, the operations
//!       crate, the viewer crate and the interface crate at the same time
//!       (ADR-0001, option B). Everything that has to be put together is put
//!       together here and nowhere else.
//! EXAMPLE: `cargo run -p phocommander-desktop`

use std::collections::BTreeMap;
use std::sync::Arc;

use phocommander_config::{
    Config, ConfigNotice, EffectiveSecurity, Language, NoticeSeverity, resolve_security,
};
use phocommander_desktop::catalogue::TranslatorCatalogue;
use phocommander_desktop::security::SecurityPlan;
use phocommander_i18n::{Locale, Translator};
use phocommander_platform::config_directory;
use phocommander_ui::{
    Context, KeymapView, Metrics, Platform, Theme, WorkSurface, WorkSurfaceApp, visuals,
};

/// Der Name der Konfigurationsdatei im Konfigurationsverzeichnis.
///
/// WHY: die Spezifikation nennt das Verzeichnis der Konfigurationsdatei, aber
///       nicht die Datei selbst. Der Name steht hier, in der Wurzel der
///       Zusammenstellung, damit er genau einmal existiert.
const CONFIGURATION_FILE_NAME: &str = "config.toml";

/// Der Name der Tastenbelegung in der Konfigurationsdatei.
const KEYMAP_FILE_NAME: &str = "keymap.toml";

fn main() -> eframe::Result<()> {
    let (config, notices) = read_configuration();
    report_notices(&notices);

    let rules = resolve_security(&config.security);
    report_rules(&rules);

    let plan = SecurityPlan::from_effective(&rules);
    report_plan(&plan);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title("Phocommander"),
        ..Default::default()
    };

    eframe::run_native(
        "Phocommander",
        options,
        Box::new(|_creation| Ok(Box::new(build_application(&config)))),
    )
}

/// Baut die Anwendung.
///
/// WHY: die Funktion baut alles zusammen, was die Oberfläche braucht — die
///       Oberfläche selbst, die Tastenbelegung, den Katalog der Texte, das
///       Thema und die Maße der Plattform. Sie ist die einzige Stelle, an der
///       diese Teile zusammenkommen.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::Config;
/// use phocommander_desktop::build_application;
///
/// let app = build_application(&Config::safe());
/// // WHY: die Anwendung ist gebaut und kann gezeichnet werden.
/// let _ = app;
/// ```
pub fn build_application(config: &Config) -> WorkSurfaceApp {
    let surface = WorkSurface::new(
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    );

    let keymap = read_keymap().unwrap_or_default();
    let catalogue = TranslatorCatalogue::new(build_translator(config));
    let metrics = Metrics::for_platform(host_platform());

    WorkSurfaceApp::new(surface, keymap, Box::new(catalogue))
        .with_theme(Theme::Dark)
        .with_metrics(metrics)
        .with_context(Context::Keyboard)
}

/// Baut den Übersetzer der Oberfläche.
///
/// WHY: die Sprache der Oberfläche folgt der Einstellung der
///       Konfigurationsdatei. Eine Sprache, die es nicht gibt, führt zu
///       Englisch, weil Englisch der letzte Schritt der Kette ist.
fn build_translator(config: &Config) -> Translator {
    // WHY: the language of the interface follows the setting of the
    //       configuration file. A language that the product does not ship
    //       leads to English, because English is the last step of the chain
    //       of the lookup.
    let locale = match config.general.language {
        Language::De => Locale::of("de"),
        Language::En => Locale::english(),
        Language::System => Locale::from_system(),
    };

    Translator::new(locale)
        .unwrap_or_else(|_| Translator::new(Locale::english()).unwrap_or_else(|_| unreachable!()))
}

/// The platform of the machine, as the interface names it.
///
/// WHY: the two crates of the platform and of the interface name the
///       platforms the same way. The conversion is written down here, so
///       that the interface does not have to know the platform crate.
fn host_platform() -> Platform {
    match phocommander_platform::host_platform() {
        phocommander_platform::Platform::MacOs => Platform::MacOs,
        phocommander_platform::Platform::Ios => Platform::MacOs,
        phocommander_platform::Platform::Android => Platform::Android,
        phocommander_platform::Platform::Windows => Platform::Windows,
        phocommander_platform::Platform::Linux => Platform::Linux,
        phocommander_platform::Platform::Bsd => Platform::Bsd,
    }
}

/// Liest die Tastenbelegung aus der Konfigurationsdatei.
///
/// WHY: die Belegung ist eine Datei, die der Benutzer bearbeitet. Eine Datei,
///       die fehlt oder die nicht gelesen werden kann, ist kein Grund, das
///       Programm nicht zu starten: die Oberfläche startet dann ohne
///       Tastenbelegung, und die Aktionen sind über die Aktionspalette
///       erreichbar.
fn read_keymap() -> Option<KeymapView> {
    let directory = config_directory().ok()?;
    let path = directory.join(KEYMAP_FILE_NAME);
    let text = std::fs::read_to_string(path).ok()?;

    let tables = parse_keymap(&text)?;
    KeymapView::from_context_tables(&tables).ok()
}

/// Liest die Tastenbelegung aus einer Datei mit Abschnitten.
///
/// WHY: die Datei hat einen Abschnitt je Kontext (`[input.keyboard]`) und in
///       ihm eine Zeile je Aktion. Der Parser liest beide Ebenen und legt
///       das Ergebnis in einer Tabelle ab, die die Oberfläche liest.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_desktop::parse_keymap;
///
/// let tables = parse_keymap("[input.keyboard]\n\"panel.copy\" = \"F5\"\n");
/// assert_eq!(tables.len(), 1);
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
/// nicht benutzt werden kann.
fn read_configuration() -> (Config, Vec<ConfigNotice>) {
    let directory = match config_directory() {
        Ok(directory) => directory,
        Err(error) => {
            return (
                Config::safe(),
                vec![ConfigNotice::warning(
                    "the configuration directory",
                    format!(
                        "the directory could not be found ({error}); \
                         the safe configuration is in force"
                    ),
                )],
            );
        }
    };

    let path = directory.join(CONFIGURATION_FILE_NAME);
    if !path.exists() {
        return (
            Config::safe(),
            vec![ConfigNotice::warning(
                path.display().to_string(),
                "the configuration file does not exist; the safe configuration is in force",
            )],
        );
    }

    phocommander_config::load_file_or_safe(path)
}

/// Schreibt die Meldungen über die Einstellungen, die keine Wirkung hatten,
/// in das Protokoll.
///
/// WHY: die Spezifikation (Abschnitt 9.8) verlangt eine Meldung im Protokoll,
///       wenn eine Einstellung keine Wirkung hatte. Das Protokoll der
///       Anwendung ist der Standardfehlerstrom, bis das Fenster ein eigenes
///       Protokoll hat.
fn report_notices(notices: &[ConfigNotice]) {
    for notice in notices {
        match notice.severity {
            NoticeSeverity::Information => {
                println!("notice: {}: {}", notice.field, notice.message);
            }
            NoticeSeverity::Warning => {
                eprintln!("warning: {}: {}", notice.field, notice.message);
            }
        }
    }
}

/// Druckt die Sicherheitsregeln, die in Kraft sind.
///
/// WHY: die Regeln sind sichtbar und nicht nur geschaltet. Jede Zeile nennt
///       die Regel, ihren Zustand und den Schalter, der entschieden hat.
fn report_rules(rules: &EffectiveSecurity) {
    println!();
    println!("security profile: {}", rules.profile().as_str());
    for rule in rules.rules() {
        let state = if rule.is_enabled() { "on" } else { "off" };
        println!("  [{state}] {} — {}", rule.title(), rule.explanation());
    }
}

/// Druckt den Plan der Teilsysteme.
fn report_plan(plan: &SecurityPlan) {
    println!();
    println!("the plan of the subsystems:");
    println!(
        "  copy      : checksums compared {}, symbolic links followed {}",
        plan.copy_policy().verifies_checksums(),
        plan.copy_policy().follows_symlinks(),
    );
    println!(
        "  viewer    : reads only what the user opened {}",
        plan.read_policy().reads_only_on_demand(),
    );
    println!(
        "  interface : directory read again before it is shown {}",
        plan.refresh_policy().verifies_before_refresh(),
    );
}

/// Der Baustein der Anwendung, der die Fehler des Rahmens meldet.
///
/// WHY: der Rahmen von `eframe` verlangt eine `Result`. Diese Kapselung
///       übersetzt einen Fehler in eine Meldung, die der Benutzer lesen kann.
#[derive(Debug)]
pub struct StartupError(pub String);

impl std::fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "the application could not be started: {}",
            self.0
        )
    }
}

impl std::error::Error for StartupError {}

/// Die Farben der Oberfläche.
///
/// WHY: die Funktion ist die einzige Stelle, an der die Farben entstehen. Das
///       Thema des Rahmens und die Palette der Oberfläche kommen beide von
///       hier, damit sie nicht auseinanderlaufen können.
pub fn theme_visuals(system_prefers_dark: bool) -> egui::Visuals {
    visuals(Theme::Dark, system_prefers_dark)
}

/// Der Speicher, in dem die Oberfläche ihre Zustände hält.
pub type SharedSurface = Arc<WorkSurface>;

#[cfg(test)]
// WHY: die Tests der Zusammenstellung liegen in einer Datei für sich, damit
//       die Wurzel der Zusammenstellung lesbar bleibt.
mod tests;
