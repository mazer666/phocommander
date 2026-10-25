# Phocommander — Architecture

**Status:** Living document · **Owner:** Rust Architect · **Date:** 24 October 2026

This document explains how Phocommander is put together: where a piece of logic lives, how the
pieces are wired to each other, and where to change something. The binding requirements are in
[`REQUIREMENTS.md`](REQUIREMENTS.md); the decisions that shape this document are in
[`adr/`](adr/).

Every diagram in this document is plain text so that it stays readable in a terminal, in a diff
and in an editor without a plugin.

---

## 1. The four layers

The workspace is four layers deep. A crate may depend on crates in the layer below it and on
nothing else. The rule is checked mechanically by `scripts/check-crate-boundaries.sh`.

```
┌──────────────────────────────────────────────────────────────────────┐
│  apps/           The binaries. The composition root.                │
│                                                                      │
│    phocommander-desktop    phocommander-android    phocommander-ios │
│                                                                      │
│    They wire the modules together and start the event loop. They    │
│    contain no logic of their own.                                    │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ may use
┌───────────────────────────────▼──────────────────────────────────────┐
│  platform/       The operating system, behind one interface.        │
│                                                                      │
│    phocommander-platform                                           │
│                                                                      │
│    Native menu, native file dialogs, the trash, the permissions,     │
│    the path of the configuration directory.                          │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ may use
┌───────────────────────────────▼──────────────────────────────────────┐
│  crates/         The portable product. No window, no protocol,      │
│                  no operating system.                                │
│                                                                      │
│    core      config    keymap    ops      archive    viewer           │
│    remote    ui        i18n                                          │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ may use
┌───────────────────────────────▼──────────────────────────────────────┐
│  (none)          The Rust standard library and third-party crates.  │
└──────────────────────────────────────────────────────────────────────┘
```

### 1.1 What each layer owes the layer above

| Layer | May reach downwards for | Must never contain |
|---|---|---|
| `apps` | Everything | Business logic, protocol handling |
| `platform` | `crates` | Product logic, protocol handling |
| `crates` | The standard library and other crates | Windows, widgets, protocols, sockets |
| third-party | — | — |

---

## 2. The module map

This is the answer to "where does this live?".

```
                        ┌──────────────────────────┐
                        │  phocommander-desktop   │
                        │  (and the mobile shells)│
                        └────────────┬─────────────┘
                                     │ wires
        ┌────────────────────────────┼────────────────────────────┐
        │                            │                            │
┌───────▼────────┐          ┌────────▼─────────┐        ┌─────────▼─────────┐
│ phocommander-  │          │ phocommander-    │        │ phocommander-     │
│ platform       │          │ ui               │        │ i18n              │
│                │          │                  │        │                   │
│ native menu    │          │ the two panels   │        │ the Fluent        │
│ file dialogs   │          │ the file list    │        │ catalogues        │
│ the trash      │          │ the theme        │        │ en, de            │
│ permissions    │          │ the action       │        │                   │
│ config paths   │          │ resolution       │        │                   │
└───────┬────────┘          └────────┬─────────┘        └─────────▲─────────┘
        │                            │                            │
        │        ┌───────────────────┼───────────────────┐        │
        │        │                   │                   │        │
        ▼        ▼                   ▼                   ▼        │
┌───────────────────────────────────────────────────────────────────────┐
│  phocommander-ops          the queue, the copy modes, the rename      │
│  ──────────────────        engine, the job cue, the editable jobs     │
└───────┬───────────────────────────────────────────────┬───────────────┘
        │                                               │
        ▼                                               ▼
┌──────────────────────┐                  ┌────────────────────────────┐
│ phocommander-core    │                  │ phocommander-archive      │
│ ──────────────────── │                  │ ──────────────────────────│
│ Entry, EntryKind     │                  │ the format adapters       │
│ Source, MutableSource│                  │ the hardened extraction   │
│ the path resolution  │                  │ the entry name resolution │
│ the error hierarchy  │                  │ the limits                │
│ the checksum         │                  │                            │
└──────────┬───────────┘                  └────────────────────────────┘
           │
           │        ┌──────────────────────┐
           │        │ phocommander-config  │  the schema, the defaults,
           │        │ phocommander-keymap  │  the profiles, the actions
           │        └──────────────────────┘
           │
           ▼
┌──────────────────────┐          ┌────────────────────────────┐
│ phocommander-remote  │          │ phocommander-viewer        │
│ ──────────────────── │          │ ──────────────────────────│
│ SFTP, FTP, SMB,      │          │ the format adapters of the │
│ WebDAV, S3           │          │ viewer: text, hex, image,   │
│ the credentials      │          │ metadata                  │
└──────────────────────┘          └────────────────────────────┘
```

### 2.1 The module table

| Module | Crate | Responsibility | Boundary — what it must not know |
|---|---|---|---|
| Model | `phocommander-core` | The directory listing, the paths, the errors | Any window, widget or operating system |
| Configuration | `phocommander-config` | The schema, the defaults, the profiles, the migration | The user interface, the file system |
| Binding | `phocommander-keymap` | The action catalogue, the binding tables, the contexts | Concrete widgets, concrete devices |
| Operations | `phocommander-ops` | The job queue, the copy modes, the rename engine, the job cue | The file system directly, the widgets |
| Archive | `phocommander-archive` | The container adapters, the hardened extraction | The user interface, the network |
| Viewer | `phocommander-viewer` | The viewer adapters, the read path | The user interface, the network |
| Remote | `phocommander-remote` | The protocol adapters, the credentials | The user interface, the archive formats |
| Presentation | `phocommander-ui` | The panels, the list, the theme, the layouts | The protocols, the job implementation |
| Localisation | `phocommander-i18n` | The catalogue, the lookup, the fallback chain | Everything but the catalogue format |
| Platform | `phocommander-platform` | The native menu, the dialogs, the trash, the rights | The product logic |
| Desktop shell | `apps/phocommander-desktop` | The binary, the event loop, the composition root | The internals of the modules it wires |
| Mobile shells | `apps/phocommander-android`, `apps/phocommander-ios` | The store packaging, the platform bridge | The internals of the modules they wire |

---

## 3. The wiring of a file operation

This is the path a key press takes, from the finger to the file. Each stage has one owner, and
that ownership is what keeps the layers apart.

```
   ┌──────────────┐
   │  The user    │  presses F5
   └──────┬───────┘
          │
          ▼
┌─────────────────────┐
│  phocommander-ui    │  the panel that has the focus reads the event
└──────────┬──────────┘
           │  a platform-neutral key event
           ▼
┌─────────────────────┐
│  phocommander-      │  the binding table of the active context
│  keymap             │  answers: which action is this?
└──────────┬──────────┘
           │  an ActionId, for example "panel.copy"
           ▼
┌─────────────────────┐
│  phocommander-ui    │  the dispatcher runs the command
└──────────┬──────────┘
           │  a job description: what, from where, to where, in which mode
           ▼
┌─────────────────────┐
│  phocommander-ops   │  the queue takes the job, the worker thread
│                     │  picks it up, the state becomes Running
└──────────┬──────────┘
           │  the operation itself, with progress and cancellation
           ▼
┌─────────────────────┐
│  phocommander-core  │  the Source or MutableSource of the panel:
│  or -remote         │  the local disk, an archive, a server
└──────────┬──────────┘
           │
           ▼
   ┌──────────────┐
   │  The file    │
   └──────────────┘
```

Two properties follow from the diagram, and they are what the diagram is for.

**The operation is testable without an interface.** The last two stages can be driven directly.
A test can copy a file without opening a window.

**The input is testable without a file system.** The first three stages can be driven with
synthetic events. A test can prove that F5 means "copy" without a single file existing.

---

## 4. The data flow of a directory listing

```
┌──────────────┐      list(path)       ┌──────────────────┐
│ The panel    │ ────────────────────► │ The Source of   │
│              │                       │ the panel        │
└──────┬───────┘                       └────────┬─────────┘
       │                                        │
       │                              a Vec<Entry>
       ▼                                        │
┌──────────────┐                                │
│ The listing  │ ◄──────────────────────────────┘
│ state        │   the entries live here, not in the egui context
│              │
│  • the path
│  • the entries, sorted
│  • the cursor
│  • the selection
└──────┬───────┘
       │
       │  the visible slice only
       ▼
┌──────────────┐
│ The drawing  │  egui describes the widget tree again in every frame
└──────────────┘
```

The listing state lives outside the egui context. That is what makes a directory with 100,000
entries affordable: the drawing step sees the visible slice, not the whole list.

---

## 5. The job queue and the job cue

```
                         ┌──────────────────────────────┐
   a command ──────────► │  the queue                    │
                         │  a Vec<Job>, ordered by        │
                         │  priority                     │
                         └───────────────┬──────────────┘
                                         │  a worker takes a job
                                         ▼
                         ┌──────────────────────────────┐
                         │  the worker                  │
                         │  Running ──► Verifying ──►   │
                         │                    Done      │
                         └───────────────┬──────────────┘
                                         │  the state of the job
                                         ▼
              ┌──────────────────────────────────────────────────┐
              │  the job cue — one state, three presentations    │
              │                                                  │
              │   ┌────────────────┐   ┌──────────────────────┐  │
              │   │ the status bar │   │ the list of jobs     │  │
              │   │ of the panel   │   │                      │  │
              │   └────────────────┘   └──────────────────────┘  │
              │   ┌────────────────┐                             │
              │   │ the marker on  │                             │
              │   │ the panel      │                             │
              │   └────────────────┘                             │
              └──────────────────────────────────────────────────┘
```

One state, three places. Three presentations of one state cannot contradict each other.

---

## 6. The interaction contexts

A context says how a particular device reaches the action identifiers.

```
        ┌────────────────────────────────────────────────────┐
        │                the action catalogue               │
        │  panel.copy, panel.view, panel.move, selection...  │
        └───────────────────────┬────────────────────────────┘
                                │
        ┌───────────┬───────────┼───────────┬───────────┐
        │           │           │           │           │
        ▼           ▼           ▼           ▼           ▼
   ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────┐
   │keyboard│ │pointer │ │ touch  │ │ dpad   │ │ controller   │
   │        │ │        │ │        │ │        │ │              │
   │ F5     │ │ menu   │ │ long   │ │palette │ │ trigger      │
   │        │ │        │ │ press  │ │        │ │              │
   └────┬───┘ └────┬───┘ └───┬────┘ └───┬────┘ └──────┬───────┘
        │          │          │        │             │
        └──────────┴──────────┴────────┴─────────────┘
                            │
                            ▼
                   the same action identifier
```

The point of the drawing is the bottom line: five different devices, one set of actions.

---

## 7. The extension points

```
        the model                     the product
        ────────                     ────────
   ┌────────────┐               ┌────────────────┐
   │  Source    │◄──────────────│ local, archive, │
   │            │               │ remote, released│
   │ Mutable    │◄──────────────│ roots           │
   │  Source    │               └────────────────┘
   └────────────┘
   ┌────────────┐               ┌────────────────┐
   │ Archive    │◄──────────────│ ZIP, TAR, 7z   │
   │  Format    │               └────────────────┘
   └────────────┘
   ┌────────────┐               ┌────────────────┐
   │ Viewer     │◄──────────────│ text, hex,     │
   │  Format    │               │ image, metadata│
   └────────────┘               └────────────────┘
   ┌────────────┐               ┌────────────────┐
   │  Action    │◄──────────────│ the catalogue  │
   └────────────┘               └────────────────┘
```

A new implementation is a new file plus a registration line. It never changes an existing
module.

---

## 8. The build and the checks

```
        a change on the branch
                  │
                  ▼
    ┌───────────────────────────────┐
    │  quality  (Linux)             │
    │    fmt · clippy · test        │
    │    boundaries · file size     │
    │    licence · advisories       │
    │    telemetry · links          │
    └───────────────┬───────────────┘
                    │
    ┌───────────────▼───────────────┐
    │  desktop  (Linux, macOS,      │
    │            Windows)            │
    │    the same gates plus the    │
    │    release build              │
    └───────────────┬───────────────┘
                    │
    ┌───────────────▼───────────────┐
    │  cross-compile                 │
    │    Android · iOS · FreeBSD ·  │
    │    NetBSD · iOS Simulator      │
    └───────────────┬───────────────┘
                    │
    ┌───────────────▼───────────────┐
    │  performance                   │
    │    the target values, measured │
    └───────────────┬───────────────┘
                    │
    ┌───────────────▼───────────────┐
    │  nightly                       │
    │    fuzzing, tests without cache│
    └───────────────────────────────┘
```

---

## 9. Where to change what

The short answer for a contributor who is about to change something.

| I want to change | The file | The crate |
|---|---|---|
| The behaviour of a file listing | `crates/phocommander-core/src/local.rs` | Model |
| The rules for a safe target path | `crates/phocommander-core/src/path.rs` | Model |
| The error codes | `crates/phocommander-core/src/error.rs` | Model |
| A setting in the configuration | `crates/phocommander-config/src/schema.rs` | Configuration |
| The default of a setting | `config/defaults.toml` | Configuration |
| The binding of a key | `config/keymap.toml` | Binding |
| The list of actions | `crates/phocommander-keymap/src/catalog.rs` | Binding |
| The copy modes | `crates/phocommander-ops/src/copy.rs` | Operations |
| The rename placeholders | `crates/phocommander-ops/src/rename.rs` | Operations |
| The state of a running job | `crates/phocommander-ops/src/job.rs` | Operations |
| The extraction rules | `crates/phocommander-archive/src/extract.rs` | Archive |
| The limits of an extraction | `crates/phocommander-archive/src/limits.rs` | Archive |
| The panels of the interface | `crates/phocommander-ui/src/panel.rs` | Presentation |
| The file list | `crates/phocommander-ui/src/listing.rs` | Presentation |
| The colours and the metrics | `crates/phocommander-ui/src/theme.rs` | Presentation |
| The formats of the numbers, the sizes, the dates and the times | `crates/phocommander-ui/src/format.rs` | Presentation |
| A translated text | `crates/phocommander-i18n/locales/*/*.ftl` | Localisation |
| The native menu | `platform/phocommander-platform/src/` | Platform |
| A new archive format | a new file in `crates/phocommander-archive/src/formats/` | Archive |
| A new network protocol | a new file in `crates/phocommander-remote/src/` | Remote |
| A new viewer format | a new file in `crates/phocommander-viewer/src/` | Viewer |
| A rename template | `config/templates/*.toml` | Operations |
| An interface profile | `config/profiles/*.toml` | Configuration |
| A requirement | `docs/REQUIREMENTS.md` | Product |
| A decision | a new file in `docs/adr/` | Architecture |

---

## 10. The rules of a good file in this project

A file that a reader can understand is not a matter of taste. The rules below are what the
project checks and what a review looks for.

**The header says three things.** Every module opens with WHAT it is, WHY it exists and an
EXAMPLE of how it is used. The WHAT names the responsibility, the WHY names the reason the
module exists at all, and the EXAMPLE shows the shape of the call.

**The name says what the thing is.** A name that describes the implementation instead of the
purpose forces the reader to look at the body to learn what it is. `JobProgress` says what it is;
`ProgressStateBuffer` says how it is built.

**One file, one responsibility.** A file that carries two responsibilities is two files. The
target value is 600 lines and the absolute maximum is 900; both are checked by
`scripts/check-file-size.sh`.

**The public surface is documented.** `missing_docs` is a lint, not a convention.

**The error says what failed.** An error carries the path, the entry or the item, so that the
user interface can name it and the log can record it.

**The test says what it proves.** A test name is a sentence about the behaviour, and the
assertion is the proof.
