# Architecture — layers and crate boundaries

Phocommander is one Cargo workspace with a fixed dependency direction. This page shows
the layers, what each crate owns and where a change belongs. The binding source is
[`docs/REQUIREMENTS.md`](../REQUIREMENTS.md), section *Systemarchitektur*.

---

## The one-line version

```
apps/*  →  platform/*  →  crates/*
```

Everything that is product logic lives in `crates/*`. Everything that knows about the
operating system lives in `platform/*`. Everything that starts a process lives in
`apps/*`. Nothing below a layer may depend on the layer above it, and no crate under
`crates/*` may depend on a sibling under `crates/*`.

---

## The diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│ apps/                                                              │
│                                                                      │
│   phocommander-desktop ── macOS · Windows · Linux · BSD             │
│   phocommander-android ── Gradle project, Rust cdylib, SAF          │
│   phocommander-ios     ── Xcode project, document picker, bookmarks │
│                                                                      │
│   The binaries. They wire the layers together and start the event   │
│   loop. No product logic lives here.                                 │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ depends on
┌───────────────────────────────▼──────────────────────────────────────┐
│ platform/                                                           │
│                                                                      │
│   phocommander-platform ── native menu (muda), native dialogs (rfd), │
│                              trash, permissions, keychain access     │
│                                                                      │
│   The OS boundary. Everything that is not a window but differs       │
│   between operating systems belongs here.                            │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ depends on
┌───────────────────────────────▼──────────────────────────────────────┐
│ crates/                                                             │
│                                                                      │
│   phocommander-core     ── filesystem model, paths, symbols, errors │
│   phocommander-config   ── configuration parser, schema versioning   │
│   phocommander-keymap   ── action catalogue, resolution, contexts    │
│   phocommander-ops      ── copy, move, delete, job queue            │
│   phocommander-archive  ── archive adapters, hardened extraction    │
│   phocommander-viewer   ── viewer registry, format adapters         │
│   phocommander-remote   ── network sources: SFTP, FTP, SMB, WebDAV  │
│   phocommander-ui       ── egui interface, panels, theme            │
│                                                                      │
│   No arrows between them. Each crate is usable on its own.          │
└──────────────────────────────────────────────────────────────────────┘
```

---

## What each crate owns

| Crate | Owns | Never contains |
|---|---|---|
| `phocommander-core` | The filesystem model: path types, entry metadata, symbols, the error hierarchy | Windows, widgets, platform APIs — no UI dependencies at all |
| `phocommander-config` | Parsing, validation and migration of the central TOML file, `schema_version` handling | Business logic of the subsystems it configures |
| `phocommander-keymap` | The action catalogue, the resolution of a physical key to an action ID, the per-context defaults, the primary modifier (`Cmd` on macOS/iPadOS, `Ctrl` elsewhere) | Anything about how the actions are carried out |
| `phocommander-ops` | The long-running operations: copy, move, delete, the job queue with progress, rate and cancellation | Direct user-interface calls |
| `phocommander-archive` | The archive adapters and the extraction hardening: iterate over the entries yourself, canonicalise every target path, check it against the destination, never create symlinks from archive material, enforce the size limits | Calls to `extract()` of the underlying crates |
| `phocommander-viewer` | The viewer registry and the format adapters; the result is read-only | Writing to the file that is being viewed |
| `phocommander-remote` | The network sources: SFTP, FTP, SMB, WebDAV, S3 — each of them a *source*, not a separate feature area | Protocol details in the core |
| `phocommander-ui` | The egui surface: widgets, the two-pane layout, the tab bar, the theme, the translation keys | Knowledge of the operating system beyond the traits it is given |
| `phocommander-platform` | Native menu, native file dialogs, trash integration, permission handling, keychain access | Product logic |
| `phocommander-desktop` | The desktop binary: wires the UI crate to the platform crate, starts the event loop | Anything that could live in a crate |

---

## The three rules

**1. The dependency direction is fixed.** `apps/*` → `platform/*` → `crates/*`. The
check is mechanical: a script reads the `Cargo.toml` files and exits with code 1 on a
violation. A negative test — build a violation, watch the gate go red, undo it, watch
it go green — proves that the gate has teeth.

**2. No sibling dependencies.** Two crates under `crates/*` may not depend on each
other, not even for a helper function. The shared part belongs in `phocommander-core`,
which is the crate that exists for exactly that purpose.

**3. The core stays free of the UI.** `phocommander-core` knows no window, no crate of
the UI stack and no platform-specific API. That is what makes it testable on every
target platform — and it is the only way to measure the test coverage honestly.

---

## How a change travels

```
  key press  ──▶  phocommander-keymap  ──▶  action ID
                                                    │
                    ┌───────────────────────────────┤
                    ▼                               ▼
         phocommander-ops                   phocommander-ui
         (copy, move, delete)               (what the user sees)
                    │
                    ▼
         phocommander-core
         (paths, entries, errors)
```

The surface layer knows action IDs and nothing else. It never asks whether an action
came from a key press, a click, a swipe, a D-pad direction or a controller trigger —
that separation is what allows the same program to run on six platforms and in five
input contexts.

---

## The extension points

Three registries make the program grow without the code base collecting in one place:

| Extension point | Trait | A new format means |
|---|---|---|
| Viewer | The viewer trait in `phocommander-viewer` | One implementation plus one registration line. The result stays read-only. |
| Archive | The archive adapter trait in `phocommander-archive` | One implementation plus one registration line. The extraction hardening stays where it is. |
| Source | The filesystem abstraction in `phocommander-core` | One implementation that delivers a directory listing — a local directory, an archive, a remote server and a user-granted root all look the same to the core. |

The last one is the reason the core stays small: the network sources of M6 are not a
separate feature area with their own special cases, they are another source of the
same abstraction.

---

## Where to change what

The detailed table — topic, file, crate, instruction — is in
[`docs/WHERE_TO_CHANGE.md`](../WHERE_TO_CHANGE.md).

## Further reading

- [`docs/REQUIREMENTS.md`](../REQUIREMENTS.md) — the binding specification
- [`docs/KEYMAP.md`](../KEYMAP.md) — the action catalogue and its bindings
- [`docs/glossary.md`](../glossary.md) — the terms of the architecture
