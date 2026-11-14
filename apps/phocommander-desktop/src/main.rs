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

use phocommander_config::resolve_security;
use phocommander_config::{Config, ConfigNotice, EffectiveSecurity, NoticeSeverity};
use phocommander_desktop::composition::{build_desktop_app, theme_visuals};
use phocommander_desktop::security::SecurityPlan;
use phocommander_platform::config_directory;

/// Der Name der Konfigurationsdatei im Konfigurationsverzeichnis.
///
/// WHY: die Spezifikation nennt das Verzeichnis der Konfigurationsdatei, aber
///       nicht die Datei selbst. Der Name steht hier, in der Wurzel der
///       Zusammenstellung, damit er genau einmal existiert.
const CONFIGURATION_FILE_NAME: &str = "config.toml";

fn main() -> eframe::Result<()> {
    let (config, notices) = read_configuration();
    report_notices(&notices);

    let rules = resolve_security(&config.security);
    report_rules(&rules);

    let plan = SecurityPlan::from_effective(&rules);
    report_plan(&plan);

    // WHY: the display server is printed, because it decides the path of the
    //       window. On Linux the application uses Wayland when the machine
    //       offers it, and X11 otherwise (M3).
    println!(
        "display server: {}",
        phocommander_desktop::linux::display_server().as_str()
    );

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
        Box::new(|creation| {
            // WHY: the colours of the application are built in one place,
            //       and that place is the only one that may build them.
            //       Without this line the window shows the standard colours
            //       of the framework, and the theme of the project exists
            //       only in the source.
            creation.egui_ctx.set_visuals(theme_visuals(true));
            Ok(Box::new(build_desktop_app(&config)))
        }),
    )
}

/// Builds the application of the desktop build.
///
/// WHY: the application of the desktop build holds the shell of the interface
///       and the watcher of the configuration file beside it. The watcher is
///       what lets the language of the interface change while the program
///       runs — the user writes the choice into the configuration file, and
///       the interface follows (FR-124).
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_config::Config;
/// use phocommander_desktop::build_desktop_app;
///
/// let app = build_desktop_app(&Config::safe());
/// // WHY: die Anwendung ist gebaut und kann gezeichnet werden.
/// let _ = app;
/// ```
///     primary_modifier_of(Platform::Linux),
///     PrimaryModifier::Control
/// );
/// ```
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
#[cfg(test)]
// WHY: die Tests der Zusammenstellung liegen in einer Datei für sich, damit
//       die Wurzel der Zusammenstellung lesbar bleibt.
mod tests;
