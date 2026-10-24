# Where to change what

The question every new contributor asks first: *which file do I touch for which
feature?*

This table is the answer. It is derived from the crate structure in
[`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md) — that specification is the binding
source, this file is the convenient index into it.

Paths are written as code, not as links, because the crates are created milestone by
milestone. A link would be dead until the file exists; a code span stays readable in
the meantime.

---

## The table

| Topic | File | Crate | How to change it |
|---|---|---|---|
| Workspace manifest, shared dependency versions | `Cargo.toml` | workspace | One version per dependency, defined once at the workspace root and inherited with `workspace = true` by the crates. Do not pin a second version inside a crate. |
| Toolchain pin | `rust-toolchain.toml` | workspace | The pinned Rust version and the components (`clippy`, `rustfmt`). Bump it in one place; never with a local `rustup override`. |
| Dependency licence and advisory policy | `deny.toml` | workspace | Licence allow-list and advisory policy for `cargo-deny`, advisory database for `cargo-audit`. The tool versions are pinned because the same config is read differently by different versions. |
| Build helper tasks | `xtask/` | `xtask` | Repository-local commands: workspace checks, crate-boundary check, release helpers. Keep it a thin wrapper around `cargo` — no product logic. |
| Default configuration template | `config/defaults.toml` | — | The commented template of the single central configuration file. Every key that a user can set must appear here with an explanation. |
| Key binding template | `config/keymap.toml` | — | The default binding per platform. Action identifiers follow the `panel.*` scheme (for example `panel.copy`). |
| Filesystem model, path types, symbols, error types | `crates/phocommander-core/src/` | `phocommander-core` | The foundation everything else builds on. Must stay free of UI dependencies — no egui, no window types, no platform APIs. New error variants belong in the crate's error type. |
| Configuration parser, schema versioning, migration | `crates/phocommander-config/src/` | `phocommander-config` | Parsing, validation and migration of the TOML file. A breaking change needs a new `schema_version` and a migration that keeps the original file as `.bak`. |
| Key resolution, platform-specific binding | `crates/phocommander-keymap/src/` | `phocommander-keymap` | Turning a key press into an action and back. The "primary" modifier is `Cmd` on macOS/iPadOS and `Ctrl` on Windows/Linux/BSD — that mapping lives here, nowhere else. |
| Copy, move, delete, job queue | `crates/phocommander-ops/src/` | `phocommander-ops` | The long-running operations. Every operation runs asynchronously, reports progress with a rate and is cancellable. Symlinks are copied as symlinks and unlinked, never dereferenced or recursively deleted. |
| Archive adapters, hardened extraction | `crates/phocommander-archive/src/` | `phocommander-archive` | Format support and the extraction hardening: iterate over entries yourself, canonicalise every target path, check it against the destination, never create symlinks from archive material, enforce the size and entry limits. |
| Viewer registry, format adapters | `crates/phocommander-viewer/src/` | `phocommander-viewer` | A new viewer format is a new implementation of the viewer trait plus one registration line — no change in the core. The viewer is read-only. |
| egui user interface, panels, theme | `crates/phocommander-ui/src/` | `phocommander-ui` | Widgets, the two-panel layout, the tab bar, the theme. The directory content lives in a state object outside the egui context; only the visible slice is drawn. |
| UI strings and translation keys | `crates/phocommander-ui/src/i18n/` | `phocommander-ui` | Every visible string goes through a translation key. German and English are both fully covered; dates, numbers and sizes follow the system language. |
| Native menu, file dialogs, trash, permissions | `platform/phocommander-platform/src/` | `phocommander-platform` | The OS boundary: native menu via `muda`, native dialogs via `rfd`, trash integration, permission handling per platform. Everything OS-specific that is not a window lives here. |
| Desktop binary | `apps/phocommander-desktop/` | `phocommander-desktop` | The macOS, Windows, Linux and BSD application. The binary wires the UI crate to the platform crate and starts the event loop. |
| Android packaging | `apps/phocommander-android/` | `phocommander-android` | Gradle project, Rust `cdylib`, SAF mode, optional full access, touch and keyboard operation. |
| iOS packaging | `apps/phocommander-ios/` | `phocommander-ios` | Xcode project, document picker, security-scoped bookmarks, split view. |
| Unit tests | `crates/<crate>/tests/`, `#[cfg(test)] mod tests` | per crate | One unit test per public function, at least. Integration tests cover the crate boundaries. |
| Property tests | `crates/<crate>/tests/` | per crate | `proptest` cases for invariants that cannot be enumerated: path resolution never escapes the destination, the rename rule is deterministic, key resolution decides deterministically. |
| Fuzz targets | `crates/<crate>/fuzz/` | per crate | Archive headers, image headers, charset detection. A fuzz run belongs to the nightly CI. |
| Performance gates | `crates/<crate>/benches/` | per crate | `criterion` benchmarks. The thresholds are the values from the performance table in the specification — the benchmark asserts them. |
| UI tests | `crates/phocommander-ui/tests/` | `phocommander-ui` | `egui_kittest` tests for deterministic keyboard operation — no pixel comparisons. |
| Specification | `docs/REQUIREMENTS.md` | — | Scope, architecture, security, performance gates, roadmap. If the code and the specification disagree, the specification is corrected first, then the code. |
| Architecture decision records | `docs/adr/` | — | Status, date, context, decision, consequences, alternatives. One ADR per decision that would be expensive to reverse. |
| User guide | `docs/user-guide/` | — | Task-oriented, German and English, one document per task area. |
| Key binding reference | `docs/KEYMAP.md` | — | The complete binding table. Change it in the same commit as the binding itself. |
| Glossary | `docs/glossary.md` | — | Terms with a specific meaning in this project, German and English. |
| Security policy | `SECURITY.md` | — | Reporting path, supported versions, what a report should contain. The threat model lives in `docs/security/THREAT_MODEL.md`. |
| Privacy policy | `PRIVACY.md` | — | Which data is stored, where, how long, how to remove it. The zero-telemetry promise lives here. |
| Contribution rules | `CONTRIBUTING.md`, `AGENTS.md` | — | The path from issue to merged pull request, and the rules that apply to automated contributions. |

---

## Finding the right crate in ten seconds

The dependency direction is fixed:

```
apps/*  →  platform/*  →  crates/*
```

Inside `crates/*` no crate may depend on a sibling crate of the same level. This is
checked mechanically — a script reads the `Cargo.toml` files and exits with code 1 on
a violation — so "it was only a small dependency" is not an argument.

Three questions decide where a change belongs:

1. **Does it need to know about pixels, widgets or events?** → `phocommander-ui`.
2. **Does it need to know about the operating system?** → `phocommander-platform`.
3. **Does it need to know about neither?** → the core crate it belongs to.

If a change fits none of them, it belongs in a new module inside the crate that owns
the surrounding responsibility — not in a new cross-dependency.

---

## Two recipes

### Adding a new archive format

1. Implement the archive adapter trait in `crates/phocommander-archive/src/` — read
   and write, no extraction.
2. Register the adapter in the registry file of the same crate.
3. Add the format to the format list in `config/defaults.toml` if the user can choose
   it.
4. Add a unit test for the adapter and an integration test that extracts a real
   archive into a temporary directory.
5. Add a negative test: a crafted archive whose entry escapes the destination must be
   rejected.
6. Update the format list in `docs/REQUIREMENTS.md` if the scope of the project
   changes.

Never call `extract()` from the underlying crate. Iterate over the entries and write
them yourself — that is the whole point of the hardening.

### Adding a new viewer format

1. Implement the viewer trait in `crates/phocommander-viewer/src/` — the result is
   read-only.
2. Register it in the registry file of the same crate.
3. Add a unit test for the detection and rendering logic.
4. If the format is parsed from untrusted bytes, add a fuzz target and a negative
   test for a malformed header.
5. Update the format table in the user guide (both languages) and the milestone
   table in `docs/REQUIREMENTS.md`.

---

## If your change is not in the table

Add it. A missing row is a documentation bug in this file — fix it in the same
commit as the change that made it necessary. The table is only useful as long as it
is complete.
