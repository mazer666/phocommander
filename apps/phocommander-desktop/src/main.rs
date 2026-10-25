//! The desktop application of Phocommander.
//!
//! WHAT: the entry point of the macOS, the Windows, the Linux and the BSD
//!       build. It reads the configuration file, resolves the security rules
//!       that are in force and shows them.
//! WHY: the program reports what exists today — the host platform, the
//!       directory of the configuration file, the version of the
//!       configuration schema and the security rules — and stops. It does not
//!       open a window it cannot fill, because a file manager that shows
//!       nothing is worse than a program that says what it is. The rules are
//!       printed because the specification (section 9.8) asks for a rule that
//!       is visible and not only switched: the user has to see what the
//!       application does on this machine.
//! EXAMPLE: `cargo run -p phocommander-desktop`

use phocommander_config::{
    Config, ConfigNotice, EffectiveSecurity, NoticeSeverity, resolve_security,
};
use phocommander_desktop::security::SecurityPlan;
use phocommander_platform::config_directory;

/// The name of the configuration file in the configuration directory.
///
/// WHY: the specification names the directory of the configuration file but
/// not the file itself. The name is written down here, in the composition
/// root, so that it exists exactly once.
const CONFIGURATION_FILE_NAME: &str = "config.toml";

fn main() {
    println!("Phocommander - milestone M0: the workspace foundation.");
    println!(
        "host platform: {} (primary key: {})",
        phocommander_platform::host_platform().name(),
        phocommander_platform::primary_modifier(),
    );

    match config_directory() {
        Ok(path) => println!("configuration directory: {}", path.display()),
        Err(error) => println!("configuration directory: not available ({error})"),
    }

    println!(
        "configuration schema version: {}",
        phocommander_config::SUPPORTED_SCHEMA_VERSION,
    );

    let (config, notices) = read_configuration();
    report_notices(&notices);

    let rules = resolve_security(&config.security);
    report_rules(&rules);

    let plan = SecurityPlan::from_effective(&rules);
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

    std::process::exit(0);
}

/// Reads the configuration file, or the safe configuration if it cannot be
/// used.
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

/// Writes the messages about the settings that had no effect into the log.
///
/// WHY: the specification (section 9.8) asks for a message in the log when a
/// setting had no effect. The log of the application is the standard error
/// stream until the window arrives with its own log view.
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

/// Prints the security rules that are in force.
///
/// WHY: the rules are visible and not only switched. Every line names the
/// rule, its state and the switch that decided it.
fn report_rules(rules: &EffectiveSecurity) {
    println!();
    println!("security profile: {}", rules.profile().as_str());
    for rule in rules.rules() {
        let state = if rule.is_enabled() { "on" } else { "off" };
        println!("  [{state}] {} — {}", rule.title(), rule.explanation());
    }
}
