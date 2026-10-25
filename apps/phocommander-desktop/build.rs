//! The build script of the desktop application.
//!
//! WHAT: it writes the manifest of the application into the executable of
//!       the Windows build.
//! WHY: Windows limits a path to 260 characters, and only a program that
//!       declares `longPathAware` in its manifest may use the longer paths
//!       of the system. The declaration is a *resource* of the executable,
//!       and the linker takes such a resource as a file of its own — the
//!       file that this script writes (see the module `resource`). The
//!       script needs nothing but the compiler and the linker, so that the
//!       build of the Windows port does not depend on the tools of the
//!       Windows SDK.
//! EXAMPLE: `cargo build -p phocommander-desktop --target x86_64-pc-windows-msvc`

#[path = "build/resource.rs"]
mod resource;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The name of the file of the manifest.
const MANIFEST_FILE_NAME: &str = "windows.manifest";

/// The name of the binary of the application.
///
/// WHY: the resource belongs to the binary and not to the library of the
///       crate, because the library is no program that Windows starts.
const BINARY_NAME: &str = "phocommander-desktop";

/// The name of the resource file that the script writes.
const RESOURCE_FILE_NAME: &str = "phocommander.resources";

/// The name of the file of the script of the resources of the GNU toolchain.
const RESOURCE_SCRIPT_FILE_NAME: &str = "phocommander.rc";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build/resource.rs");
    println!("cargo:rerun-if-changed={MANIFEST_FILE_NAME}");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest = match read_manifest() {
        Ok(manifest) => manifest,
        Err(error) => fail(&error),
    };

    let target_environment = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    match target_environment.as_str() {
        "msvc" => embed_with_the_resource_file(&manifest),
        "gnu" => embed_with_the_resource_script(),
        other => {
            println!(
                "cargo:warning=the target environment {other} is not known; \
                 the manifest is not embedded into the program"
            );
        }
    }
}

/// Reads the manifest of the application.
///
/// WHY: the manifest is a file of the crate, and the build has to use the
///       file that is in the repository and not a copy of it.
fn read_manifest() -> Result<Vec<u8>, String> {
    let path = manifest_path();
    std::fs::read(&path)
        .map_err(|error| format!("the manifest {} could not be read: {error}", path.display()))
}

/// The path of the manifest of the application.
fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(MANIFEST_FILE_NAME)
}

/// Writes the manifest into a resource file and gives the file to the linker.
///
/// WHY: the linker of the toolchain of Microsoft takes a file of resources
///       as an input of the link, and the file carries the manifest as the
///       resource 1 of the type 24. The number of the machine of the file
///       has to match the machine of the program, or the linker refuses the
///       file.
fn embed_with_the_resource_file(manifest: &[u8]) {
    let architecture = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let Some(machine) = resource::machine_of(&architecture) else {
        println!(
            "cargo:warning=the architecture {architecture} has no known machine number; \
             the manifest is not embedded into the program"
        );
        return;
    };

    let out_directory = match env::var("OUT_DIR") {
        Ok(directory) => PathBuf::from(directory),
        Err(error) => fail(&format!(
            "the output directory of the build is unknown: {error}"
        )),
    };

    let path = out_directory.join(RESOURCE_FILE_NAME);
    let content = resource::manifest_resource(manifest, machine);
    if let Err(error) = std::fs::write(&path, content) {
        fail(&format!(
            "the resource file {} could not be written: {error}",
            path.display()
        ));
    }

    println!("cargo:rustc-link-arg-bin={BINARY_NAME}={}", path.display());
}

/// Writes the manifest into the program with the tool of the GNU toolchain.
///
/// WHY: the linker of the GNU toolchain does not read a file of resources,
///       and the tool `windres` turns a file of the script of the resources
///       into an object of the program. The step is optional: when the tool
///       is not there, the build goes on and says so, because a build that
///       stops is worse than a program without the declaration.
fn embed_with_the_resource_script() {
    let Some(tool) = find_in_the_path(&["windres", "windres.exe"]) else {
        println!(
            "cargo:warning=windres was not found; the manifest is not embedded into \
             the program of the GNU toolchain"
        );
        return;
    };

    let out_directory = match env::var("OUT_DIR") {
        Ok(directory) => PathBuf::from(directory),
        Err(error) => fail(&format!(
            "the output directory of the build is unknown: {error}"
        )),
    };

    let script = out_directory.join(RESOURCE_SCRIPT_FILE_NAME);
    let content = format!("1 RT_MANIFEST \"{MANIFEST_FILE_NAME}\"\n");
    if let Err(error) = std::fs::write(&script, content) {
        println!(
            "cargo:warning=the script of the resources {} could not be written: {error}",
            script.display()
        );
        return;
    }

    let object = out_directory.join("phocommander.resources.o");
    let directory = manifest_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let result = Command::new(&tool)
        .arg("-i")
        .arg(&script)
        .arg("-O")
        .arg("coff")
        .arg("-o")
        .arg(&object)
        .arg("--include-dir")
        .arg(&directory)
        .output();

    match result {
        Ok(output) if output.status.success() => {
            println!(
                "cargo:rustc-link-arg-bin={BINARY_NAME}={}",
                object.display()
            );
        }
        Ok(output) => {
            println!(
                "cargo:warning=windres did not write the object of the resources: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Err(error) => {
            println!("cargo:warning=windres could not be started: {error}");
        }
    }
}

/// Looks for a program in the path of the system.
///
/// WHY: the tools of the toolchains are found through the path of the
///       system, and the script of the build inherits it.
fn find_in_the_path(names: &[&str]) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for directory in env::split_paths(&path) {
        for name in names {
            let candidate = directory.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Stops the build with a message that the reader understands.
///
/// WHY: a build script that stops with a panic shows a stack trace of the
///       script, and the reason of the failure is somewhere in the middle
///       of it. A message on the standard error stream says the reason in
///       the first line.
fn fail(message: &str) -> ! {
    eprintln!("error: the build of the desktop application stops: {message}");
    std::process::exit(1);
}
