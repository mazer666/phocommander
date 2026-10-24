//! The desktop application of Phocommander.
//!
//! WHAT: the entry point of the macOS, the Windows, the Linux and the BSD
//!       build. Milestone M0 builds the workspace, the crate boundaries and
//!       the gates; the two-panel window, the keyboard operation and the
//!       file operations arrive in milestone M1.
//! WHY: the program reports what exists today — the host platform, the
//!       directory of the configuration file and the version of the
//!       configuration schema — and stops. It does not open a window it
//!       cannot fill, because a file manager that shows nothing is worse
//!       than a program that says what it is.
//! EXAMPLE: `cargo run -p phocommander-desktop`

fn main() {
    println!("Phocommander - milestone M0: the workspace foundation.");
    println!(
        "host platform: {} (primary key: {})",
        phocommander_platform::host_platform().name(),
        phocommander_platform::primary_modifier(),
    );

    match phocommander_platform::config_directory() {
        Ok(path) => println!("configuration directory: {}", path.display()),
        Err(error) => println!("configuration directory: not available ({error})"),
    }

    println!(
        "configuration schema version: {}",
        phocommander_config::SUPPORTED_SCHEMA_VERSION,
    );
    println!("The two-panel window arrives in milestone M1.");

    std::process::exit(0);
}
