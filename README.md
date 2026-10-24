# Phocommander

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust 1.95+](https://img.shields.io/badge/rust-1.95%2B-orange.svg)](https://www.rust-lang.org/tools/install)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux%20%7C%20BSD%20%7C%20Android%20%7C%20iPadOS-lightgrey.svg)](docs/REQUIREMENTS.md)
[![User guide](https://img.shields.io/badge/docs-user%20guide-informational.svg)](docs/user-guide/)

**Phocommander is a two-pane file manager with a built-in viewer, for macOS, Windows, Linux, BSD, Android and iPadOS.**

It exists because the file managers people keep open all day either cost money, cover a single operating system, or pull you out of a keyboard-driven workflow — and the one classic that covers all of it is shareware for Windows only.

---

## Status

> **Phocommander is under active development.**
>
> - **M0 — Foundation** (workspace, CI, configuration file, core crates, key resolution, panel skeleton): *in progress*.
> - **M1 — First usable release** (file operations with a job queue, multiple rename, viewer tier 1, ZIP/TAR/7z archives, hardened extraction): *the first release you can actually work with*.
> - Everything in this repository today is scaffolding, specification and documentation — not a finished file manager.
>
> The binding specification is **[`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md)**. When this page and the specification disagree, the specification wins.

---

## What it does / what is planned

| Target capabilities (per specification) | Planned for later milestones |
|---|---|
| Two-pane workspace: both directories visible at once, side by side or stacked | Directory comparison and two-way synchronisation (M6) |
| Full keyboard operation, including an input line and a filter field (M2) | Search by name, size, date and content (M6) |
| File operations — copy, move, rename, delete, create directory — through a background job queue (M1) | Editor, tags and macros (M7) |
| Multiple rename with placeholders, regex, live preview and conflict jump (M1) | Audio/video and Office-document viewing (M3) |
| Built-in viewer: text, hex dump, images, archive contents, metadata (M1); PDF and syntax highlighting (M2) | Priority scheduling for background jobs |
| Archives as virtual directories: ZIP, TAR (+GZip/BZip2/XZ/Zstd), 7z — readable and writable (M1) | RAR and the exotic archive formats (deliberately out of scope, see the specification) |
| Tabs per pane, movable between panes (M2) | Windows, Linux and BSD ports (M3) |
| Native menus, native dialogs and platform-specific shortcuts (`Cmd` on macOS/iPadOS, `Ctrl` elsewhere) | Android (M4) and iPadOS (M5) |
| Overwrite is never the default — every conflict policy is explicit and configurable | SFTP and FTP connections (M6) |

The specification deliberately does **not** promise an "80+ formats" viewer, a macro language with VBScript compatibility, or optical-media burning. Those are excluded on purpose, with reasons.

---

## Quickstart

Phocommander is a plain Cargo workspace. You need a Rust toolchain (see [rustup](https://www.rust-lang.org/tools/install)) and the platform's own build prerequisites — on Linux, the usual X11/Wayland and OpenGL development packages.

```bash
# Build every crate and application in the workspace.
# The first run downloads and compiles the dependency tree (egui/eframe, winit,
# muda, rfd, …) and therefore takes noticeably longer than later runs.
cargo build --workspace

# Start the desktop application (macOS, Windows, Linux, BSD).
cargo run -p phocommander-desktop
```

What happens:

| Step | What you get |
|---|---|
| `cargo build --workspace` | All crates under `crates/`, the platform adapter under `platform/` and the binaries under `apps/` are compiled into `target/debug/`. A cold dependency tree means a long first build; subsequent builds are incremental. |
| `cargo run -p phocommander-desktop` | The desktop binary starts and opens the two-pane workspace. In the current milestone the panes are a skeleton — the file operations that make it useful arrive with M1. |

The layout you are building is documented in [`docs/WHERE_TO_CHANGE.md`](docs/WHERE_TO_CHANGE.md): it answers "which file do I touch for which feature?".

---

## Your first key presses

The keyboard is the primary way to drive Phocommander — not a comfort feature.

| Key | Action | Available |
|---|---|---|
| `F2` | Rename (also for multiple files) | M1 |
| `F3` | Open the built-in viewer | M1 |
| `F4` | Open the editor | planned (M7) |
| `F5` | Copy | M1 |
| `F6` | Move | M1 |
| `F7` | Create directory | M1 |
| `F8` | Delete | M1 |
| `F9` | Menu | planned |
| `F10` | Quit | M0 |
| `Tab` | Switch the active pane | M0 |
| `Ctrl+Enter` | Open the selection in the other pane | M0 |

On macOS and iPadOS, `Ctrl` in this table means `Cmd`. The complete binding list, including the context-dependent keys, lives in [`docs/KEYMAP.md`](docs/KEYMAP.md).

The milestone column follows the roadmap in the specification: pane switching, opening the selection in the other pane and quitting belong to the M0 skeleton; copy, move, rename, delete, create directory and the viewer arrive with M1; the editor is a planned M7 feature. The menu bar is marked as planned because the roadmap does not assign it a milestone of its own.

---

## Documentation

| Document | What it is for |
|---|---|
| [`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md) | The binding specification: scope, architecture, security, performance gates, roadmap |
| [`docs/KEYMAP.md`](docs/KEYMAP.md) | Complete keyboard reference, including per-platform variants |
| [`docs/user-guide/`](docs/user-guide/) | Task-oriented user guide, German and English |
| [`docs/glossary.md`](docs/glossary.md) | Terms that carry a specific meaning in this project, German and English |
| [`docs/WHERE_TO_CHANGE.md`](docs/WHERE_TO_CHANGE.md) | "Where do I change what?" for contributors |
| [`AGENTS.md`](AGENTS.md) | Working rules for AI-assisted contributions |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to contribute: issue, branch, commit, tests, pull request |
| [`SECURITY.md`](SECURITY.md) | How to report a vulnerability |
| [`PRIVACY.md`](PRIVACY.md) | Which data is stored, where, and how to remove it |
| [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | Contributor Covenant 2.1 |

---

## Privacy, licence, project facts

- **No telemetry.** No analytics, no usage counters, no crash reports sent anywhere, no advertising, no cloud service. The only data Phocommander stores is what you configure yourself — settings, key bindings, favourites, recently opened directories. See [`PRIVACY.md`](PRIVACY.md).
- **No cloud.** Nothing is uploaded. If you connect to SFTP or FTP, that connection is yours, and its credentials live in your operating system's keychain, not in the configuration file.
- **Licence: `MIT OR Apache-2.0`.** You choose. See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
- **Clean-room implementation.** Phocommander is an independent implementation. Only the *idea* of two-pane file management, an embedded viewer and a placeholder-based multiple rename is shared with the classic tools it takes inspiration from — no code, artwork, text or layout is reproduced.

## License

Licensed under either of

- Apache License, Version 2.0 — [`LICENSE-APACHE`](LICENSE-APACHE)
- MIT License — [`LICENSE-MIT`](LICENSE-MIT)

at your option.
