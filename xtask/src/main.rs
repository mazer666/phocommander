//! The build helpers of Phocommander.
//!
//! WHAT: the task runner of the workspace. `cargo xtask check` runs
//!       `cargo check --workspace` and passes on its exit code.
//! WHY: several checks of this project do not belong into cargo itself — the
//!       gate for the crate boundaries, the gate for the file size, the
//!       licence check. One command that collects them makes the local run
//!       and the run in the CI identical. The exit code of the command is
//!       the exit code of the check, so that a failing check fails the build
//!       and not just prints a message.
//! EXAMPLE: `cargo xtask check`

use std::process::Command;

/// Runs the task that was named on the command line.
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let exit_code = match arguments.first().map(String::as_str) {
        Some("check") => check_workspace(),
        Some("help" | "--help" | "-h") => {
            print_usage();
            0
        }
        Some(unknown) => {
            eprintln!("xtask: unknown command: {unknown}");
            print_usage();
            2
        }
        None => {
            eprintln!("xtask: no command given");
            print_usage();
            2
        }
    };

    std::process::exit(exit_code);
}

/// Runs `cargo check --workspace` and returns its exit code.
///
/// WHY: the exit code of the check is the exit code of the task. A check that
/// fails has to fail the build, otherwise the gate can be ignored.
fn check_workspace() -> i32 {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

    match Command::new(cargo).args(["check", "--workspace"]).status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            eprintln!("xtask: cargo could not be started: {error}");
            1
        }
    }
}

/// Writes the list of the commands of this task runner.
fn print_usage() {
    println!("usage: cargo xtask <command>");
    println!();
    println!("commands:");
    println!("  check    run `cargo check --workspace` and pass on its exit code");
    println!("  help     print this text");
}
