# Phocommander

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust 1.95+](https://img.shields.io/badge/rust-1.95%2B-orange.svg)](https://www.rust-lang.org/tools/install)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux%20%7C%20BSD%20%7C%20Android%20%7C%20iPadOS-lightgrey.svg)](docs/REQUIREMENTS.md)
[![User guide](https://img.shields.io/badge/docs-user%20guide-informational.svg)](docs/user-guide/)

**Phocommander is a two-pane file manager with a built-in viewer, for macOS, Windows, Linux, BSD, Android and iPadOS.**

It exists because the file managers people keep open all day either cost money, cover a single operating system, or pull you out of a keyboard-driven workflow — and the one classic that covers all of it is shareware for Windows only.

---

## Status

> **Phocommander is under active development. It is not a finished file manager yet.**
>
> - **M0 — Foundation** (workspace, CI, configuration file, key resolution, threat model, project documentation, licence check): *accepted*.
> - **M1 — The working file browser** (file operations with a job queue, multiple rename, the series template, viewer tier 1, ZIP/TAR/7z archives, hardened extraction, the filter of the current directory): *in acceptance*.
> - **M2 — Maturity on macOS** (tabs, interface profiles, English and German, safe copy modes, the layouts per context): *in work*.
>
> The current state of every milestone, with the checklist that has to hold before it counts as
> reached, is in **[`docs/ROADMAP.md`](docs/ROADMAP.md)**.
>
> The binding specification is **[`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md)**. When this page and the specification disagree, the specification wins — and the disagreement is a bug in this page, not in the specification.

---

## Three rules

The product is measured against three rules that are written down as checkable requirements, not as slogans. Every later decision is judged against them.

| Rule | What it means in practice |
|---|---|
| **Free** | No licence fee, no subscription, no in-app purchase, no closed features. Every function exists in every version. The source builds without network access. |
| **Clutter-free** | No banners, no recommendation column, no message services, no account requirement, no onboarding tour. The start screen is the work surface. |
| **Ad-free** | No advertising, no tracking, no analytics, no telemetry, no usage profiles, no third-party disclosure. Local functions work without registration and without a login. |

---

## What it does / what is planned

| Target capabilities (per specification) | Planned for later milestones |
|---|---|
| Two-pane workspace: both directories visible at once, side by side or stacked (M0) | Directory comparison (M2), productive comparison and two-way synchronisation (M6) |
| Full keyboard operation, including an input line and a filter field (M2) | Search by name, size, date and content (M6) |
| File operations — copy, move, rename, delete, create directory — through a background job queue (M1) | Editor, tags and macros (M9) |
| Multiple rename with placeholders, regex, live preview and conflict jump (M1) | Audio, video and Office-document viewing (M9) |
| Built-in viewer: text, hex dump, images, archive contents, metadata (M1); PDF and syntax highlighting (M2) | Priority scheduling for background jobs (planned, no milestone assigned) |
| Archives as virtual directories: ZIP, TAR (+GZip/BZip2/XZ/Zstd), 7z — readable and writable (M1) | RAR and the exotic archive formats (deliberately out of scope, see the specification) |
| Tabs per pane, movable between panes (M2) | Windows, Linux and BSD ports (M3) |
| Native menus, native dialogs and platform-specific shortcuts (`Cmd` on macOS/iPadOS, `Ctrl` elsewhere) (M2) | Android (M4) and iPadOS (M5) |
| Overwrite is never the default — every conflict policy is explicit and configurable (M1) | Network sources: SFTP, FTP, SMB, WebDAV (M6) |

The specification deliberately does **not** promise an "80+ formats" viewer, a macro language with VBScript compatibility, or optical-media burning. Those are excluded on purpose, and the specification says why.

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
| `cargo build --workspace` | Every crate under `crates/`, the platform adapter under `platform/` and the binaries under `apps/` are compiled into `target/debug/`. A cold dependency tree means a long first build; later builds are incremental. |
| `cargo run -p phocommander-desktop` | The desktop binary starts. In M0 it reports the host platform, the primary modifier, the configuration directory and the configuration schema version, then exits — the two-panel window arrives with M1. |

**The current state of the repository, honestly:** the workspace manifest, the pinned toolchain (`rust-toolchain.toml`) and the dependency catalogue are in place, and every member that `Cargo.toml` declares exists — `cargo metadata --no-deps` resolves the workspace without an error. The first build downloads the dependency tree from crates.io (egui/eframe, winit, muda, rfd, image, …), which is why it takes noticeably longer than every later build.

What the build does **not** yet give you is a usable file manager: the program reports what exists today and stops, because a file manager that shows an empty window is worse than one that says what it is. The two panes and the operations that make it worthwhile — copy, move, rename, delete, view, archives — arrive with M1. The specification says exactly what belongs to which milestone.

The layout you are building is documented in [`docs/WHERE_TO_CHANGE.md`](docs/WHERE_TO_CHANGE.md): it answers "which file do I touch for which feature?".

---

## Your first key presses

The keyboard is the primary way to drive Phocommander — not a comfort feature. The full reference, including the context-dependent keys, is [`docs/KEYMAP.md`](docs/KEYMAP.md).

| Key | Action | Available |
|---|---|---|
| `F2` | Rename (also for multiple files) | M1 |
| `F3` | Open the built-in viewer | M1 |
| `F4` | Open the editor | M9 (planned) |
| `F5` | Copy | M1 |
| `F6` | Move | M1 |
| `F7` | Create directory | M1 |
| `F8` | Delete | M1 |
| `F9` | Menu | M2 |
| `F10` | Quit | M0 |
| `Tab` | Switch the active pane | M0 |
| `Ctrl+Enter` | Open the selection in the other pane | M1 |

On macOS and iPadOS, `Ctrl` in this table means `Cmd`. The milestone column follows the milestone table in the specification: the panel skeleton, pane switching and quitting belong to M0; copy, move, rename, delete, create directory, the viewer and the hand-over to the other pane arrive with M1; the native menu bar comes with M2; the editor is an M9 feature.

---

## Input contexts

Phocommander is operated in five input contexts. The binding is to **actions, not to devices**: the interface layer knows action identifiers and nothing about whether an action was triggered by a key, a click, a swipe, a D-pad direction or a controller trigger.

| Context | Input device | Signals | Layout principle | Milestone |
|---|---|---|---|---|
| `keyboard` | Keyboard | Keys, modifiers, text entry | Complete action set, Norton-Commander shortcuts | M0 |
| `pointer` | Mouse | Pointer, click, double click, right click, drag, scroll wheel | Two panes side by side, menu bar, icon bar, hover | M2 |
| `touch` | Finger | Tap, long press, swipe, two-finger scroll, pinch | Stacked panes, large targets, sheet dialogs, no hover | M4 |
| `dpad` | TV remote control | D-pad, OK, Back, Home, Play/Pause | One pane visible, 10-foot scaling, focus frame | M8 |
| `controller` | Gamepad, VR controller | Trigger, grip, thumbstick, A/B/X/Y, D-pad | Spatial arrangement, ray pointer, no modifiers | M7 |

On a remote control there are no function keys. The **action palette** replaces them: a searchable list of every action available in the current context, opened with the menu or the Play/Pause key and operated with the direction keys and OK. It is available in every context — on the desktop it is the "search for an action" dialog.

---

## Documentation

| Document | What it is for |
|---|---|
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | **The roadmap and the tracking**: the milestone sequence, the checklist per milestone, what has to hold before a milestone counts as reached |
| [`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md) | The binding specification: scope, architecture, security, performance gates, milestones |
| [`docs/REQUIREMENTS-IDS.md`](docs/REQUIREMENTS-IDS.md) | Requirement identifiers and where each one comes from — the reference for acceptance evidence |
| [`docs/KEYMAP.md`](docs/KEYMAP.md) | Complete keyboard reference, including per-platform variants and per-context bindings |
| [`docs/user-guide/`](docs/user-guide/) | Task-oriented user guide, German and English |
| [`docs/glossary.md`](docs/glossary.md) | Terms that carry a specific meaning in this project, German and English |
| [`docs/WHERE_TO_CHANGE.md`](docs/WHERE_TO_CHANGE.md) | "Where do I change what?" for contributors |
| [`docs/architecture/README.md`](docs/architecture/README.md) | The layers, the crate boundaries and how a change travels through them |
| [`AGENTS.md`](AGENTS.md) | Working rules for AI-assisted contributions |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to contribute: issue, branch, commit, tests, pull request |
| [`SECURITY.md`](SECURITY.md) | How to report a vulnerability |
| [`PRIVACY.md`](PRIVACY.md) | Which data is stored, where, and how to remove it |
| [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) | Contributor Covenant 2.1 |
| [`CHANGELOG.md`](CHANGELOG.md) | What changed, in Keep a Changelog format |

---

## Privacy, licence, project facts

- **No telemetry.** No analytics, no usage counters, no crash reports sent anywhere, no advertising, no cloud service, no usage profiles. The only data Phocommander stores is what you configure yourself — settings, key bindings, favourites, recently opened directories. See [`PRIVACY.md`](PRIVACY.md).
- **No cloud requirement.** Nothing is uploaded, no account is needed for local functions. If you connect to SFTP or FTP, that connection is yours and it was started by you; its credentials live in your operating system's keychain, not in the configuration file.
- **Licence: `MIT OR Apache-2.0`.** You choose. See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
- **Clean-room implementation.** Phocommander is an independent implementation. Only the *idea* of two-pane file management, an embedded viewer and a placeholder-based multiple rename is shared with the classic tools it takes inspiration from — no code, artwork, text or layout is reproduced.

## License

Licensed under either of

- Apache License, Version 2.0 — [`LICENSE-APACHE`](LICENSE-APACHE)
- MIT License — [`LICENSE-MIT`](LICENSE-MIT)

at your option.
