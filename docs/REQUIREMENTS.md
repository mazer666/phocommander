# Phocommander — Requirements and Technical Specification

**Version:** 2.0 · **Status:** Draft for owner review · **Supersedes:** 1.1 · **Date:** 24 October 2026
**Repository:** `github.com/mazer666/phocommander` (public) · **Licence:** `MIT OR Apache-2.0`
**Language of this document:** English. The user interface ships in English and German first.

---

## 1. Purpose and Scope

Phocommander is a dual-pane file manager with an integrated viewer, built in Rust, for macOS,
Android, iPadOS, Windows, Linux and BSD. This document is the binding specification for the
project: it defines the product, its architecture, its organisational plan, its milestone
sequence, and the acceptance rules that allow every piece of work to be signed off.

This document supersedes version 1.1. Version 1.1 defined the product concept and the platform
matrix. Version 2.0 adds the technical plan, the organisational plan, the modular decomposition,
the configurable interface model, the internationalisation plan, and the acceptance framework that
turns the document into something work can be measured against.

### 1.1 Scope

In scope: the desktop core, the shared Rust library used by all six targets, the platform
shells, the archive and viewer subsystems, the remote source subsystem, the input model across
five interaction contexts, the internationalisation framework, and the project organisation that
produces and verifies all of it.

Out of scope for the first release cycle: third-party cloud storage with vendor-specific
sync semantics, a scripting engine compatible with VBScript, optical media burning, a binary
plugin interface modelled on the reference product's add-in system, and an immersive virtual
reality port. The architecture reserves a place for each of these; none of them is part of the
committed delivery.

### 1.2 How to read this document

| Section | Contains | Read it when |
|---|---|---|
| 2 | Product principles, non-negotiable | You are about to make a design decision |
| 3 | Product definition and reference analysis | You need to know what the product is |
| 4 | Requirement catalogue, `FR-xx` and `NFR-xx` | You implement or verify a feature |
| 5 | Technical plan: architecture, modules, data flows | You design or change a module |
| 6 | Input model and interaction contexts | You work on navigation or an input device |
| 7 | Interface configuration model | You design the settings surface |
| 8 | Internationalisation plan | You add or change a user-visible string |
| 9 | Security and privacy plan | You touch untrusted input or the network |
| 10 | Test and quality strategy | You add or change a test |
| 11. | Milestone plan M0–M9 | You plan work |
| 12 | Organisational plan: roles, flow of work, review | You join the project or take a role |
| 13 | Acceptance rules and gates | You accept or reject a milestone |
| 14 | Decisions taken and open questions | You need the current state of play |

Requirement identifiers are stable. A requirement never changes its meaning; when it does, the
change is recorded in section 14 with the date and the reason. Acceptance criteria reference
requirement identifiers, so a traceable chain runs from specification to test to acceptance
document.

---

## 2. Product Principles

Three principles define the product. They are not slogans; they are constraints that every
design decision is measured against, and several of them are enforced mechanically by the build.

### 2.1 Free

No licence fee, no subscription, no in-app purchase, no feature behind a paywall. Every feature
that exists in one build exists in every build. No charge for security updates. The build does not
phone home to check a licence.

Enforced by: a CI check that fails the build if any dependency from the telemetry or analytics
category is referenced, and a test that proves the process opens no network connection during
start-up. The licence of every dependency is checked by `cargo deny` on every build.

### 2.2 Clutter-free

No banners, no recommendation columns, no news feeds, no mandatory account, no onboarding tour
that insists on itself. The start screen is the work surface, not a landing page. No advertising
inside the application, no sponsored entries in search or connection lists. No setting that has
no purpose.

Enforced by: a screenshot comparison of the start view per context, and a check that the
connection list contains only entries the user created.

### 2.3 Advertising-free

No advertising, no tracking, no analytics, no telemetry, no user profiles, no disclosure to
third parties, no affiliate links, no sponsored placements. Local functions work without
registration and without a login. The source code shows it: there is no network request that the
user did not initiate.

Enforced by: the promise in `PRIVACY.md`, the zero-telemetry evidence in the security
documentation, and the dependency check described in 2.1.

### 2.4 The principles are testable

A principle that cannot be falsified is a slogan. Each of the three principles above therefore
names the mechanism that enforces it, and the mechanism is part of the acceptance criteria of the
milestone in which it first becomes relevant.

---

## 3. Product Definition

### 3.1 Vision

Phocommander is for people who spend their working day deliberately organising, comparing,
renaming and inspecting files — in a terminal, in an IDE, in a photo workflow, in a backup job.
The target user finds the platform file manager too distracting, too slow, or too unclear.

Three capabilities form the product core:

**Both sides visible.** Source and destination are on screen at the same time. Before a copy, the
user can see where the file will go, and two directories can be compared directly without leaving
the tool.

**The keyboard is the primary path.** The complete operation is possible without a mouse. This is
not a convenience feature; it is the reason dual-pane managers have survived for decades. A
command line and a filter field accelerate navigation and selection by an order of magnitude
compared with pointer-only operation.

**The viewer is in the window.** Files are inspected without launching an external application.
For logs, screenshots, configurations and code fragments this saves the application-switch, and
it is the reason both reference products ship an integrated viewer.

### 3.2 Reference analysis

The reference product defines the functional scope. SpeedCommander provides the behavioural
model: dual pane with a configurable arrangement, tabbed directory windows, an integrated
multi-format viewer, thirteen archive formats, FTP/FTPS/SFTP, folder synchronisation, multiple
rename, file search, an integrated command line, macros and an add-in interface [1][2][3]. Its
interface is dense and keyboard-driven, and the density is a deliberate choice.

Forklift provides the quality reference on macOS: native menus, Finder-like appearance, Quick
Look, tags, forward and backward navigation, Time Machine restoration [5][38]. It is regarded as
the most capable file management utility for macOS, but it costs money and does not cover
iOS or iPadOS.

Phocommander takes the operating model from the first and the craft from the second. The goal is
not to copy either product but to combine their respective strengths in one free, cross-platform
tool.

The three products are compared along the capabilities that decide daily work.

| Capability | SpeedCommander [2][3] | Forklift [5] | Phocommander target |
|---|---|---|---|
| Platforms | Windows 7–11 | macOS | macOS, Android, iPadOS, Windows, Linux, BSD |
| Window model | Dual pane, above or below | Dual pane, also single | Dual pane, above or below, tabbed |
| Viewer | 80+ formats | Quick Look, PDF, text | Text, image, PDF, hex, archive, metadata, staged |
| Archive formats | 13 | ZIP and others | ZIP, TAR with compression, 7z |
| Multiple rename | Placeholders, conflict jump F3 | Filters, parameters, presets | Placeholders, regex, preview, conflict jump |
| Remote access | FTP, FTPS, SFTP | FTP, SFTP, SMB, WebDAV, cloud | SFTP, FTP, SMB, WebDAV, S3-compatible |
| Operation | Purely keyboard-driven | Keyboard and pointer | Purely keyboard-driven, pointer optional |
| Licence | Shareware | Commercial | MIT OR Apache-2.0 |

The gap the comparison shows is the one the project intends to close.: the operating idea and
keyboard orientation of one product, the integration quality of the other, plus a free licence
and six platforms that neither of them reaches.

### 3.3 Clean room declaration

Phocommander is an independent implementation. The only inputs are publicly documented
observations of the functional scope of the classic dual-pane file manager lineage and of the
quasi-standard key binding that has existed since the 1980s [4][36]. No code, image, icon, text
or layout of the original vendor is taken, mirrored or reproduced.

Only the *idea* of a function is taken: two synchronously visible directory windows, an embedded
viewer, a multiple-rename dialog with placeholders. The concrete implementation, the interaction
model, the interface design and the code base are original. In software engineering, a clean room
is a development process in which the specification is written without knowledge of the original
implementation — that is the process used here [2][4].

The reference product's feature names are used only to describe scope in this document. They are
not used as identifiers in the code, the configuration files or the user interface. Phocommander
uses its own vocabulary, defined in `docs/glossary.md`.

---

## 4. Requirement Catalogue

Every requirement has an identifier, a milestone, a priority and a verification method. The
priority scale is **Must** (required for the milestone to be accepted), **Should** (required
unless a documented reason exists), **Could** (nice to have, may be deferred with a note).

### 4.1 Product and principles

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-01 | The application is free of charge, has no paid tier, and no feature is gated | M0 | Must | Dependency check plus licence review |
| FR-02 | The application starts without an account, a login or a network connection | M0 | Must | Start-up test that asserts no socket is opened |
| FR-03 | No telemetry, analytics, advertising or crash reporting is collected | M0 | Must | Dependency check plus code review |
| FR-04 | The start view contains no banner, no promotional area and no onboarding overlay | M0 | Must | Screenshot review in the acceptance document |
| FR-05 | Every user-visible string is externalised and translatable | M2b | Must | Gate that fails on a literal string in the UI layer |
| FR-06 | The product ships under `MIT OR Apache-2.0` with no proprietary component | M0 | Must | `cargo deny` plus licence review |

### 4.2 Dual-pane work surface

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-10 | Two independent directory panels are shown side by side or stacked | M1 | Must | Manual test plus keyboard test |
| FR-11 | The arrangement can be switched at run time without a restart | M2 | Should | Manual test |
| FR-12 | Each panel owns its tabs, and a tab holds its own path, sorting and view state | M2 | Must | State model unit test |
| FR-13 | Tabs can be moved between the two panels | M2 | Should | Manual test |
| FR-14 | A directory tree can be toggled per panel | M2 | Should | Manual test |
| FR-15 | A third area shows a preview of the marked file without opening it | M2 | Should | Manual test |
| FR-16 | Operations act on the active panel; transfer to the other panel is always explicit | M1 | Must | State model test that asserts the target panel is never implied |
| FR-17 | The column set is configurable: name, size, date, type | M1 | Must | Configuration test plus manual test |
| FR-18 | The sort order is configurable and remembered per tab | M1 | Must | Configuration test |

### 4.3 File operations

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-20 | Copy, move, rename, delete and create-directory are available | M1 | Must | Unit test plus manual test |
| FR-21 | Every operation runs in a background job; the interface stays responsive | M1 | Must | Test that asserts the UI thread is not blocked |
| FR-22 | The conflict policy is configurable: ask, never, always, keep both | M1 | Must | Unit test per policy |
| FR-22a | The copy mode is selectable: quick, verified, atomic, sync-mirror, sync-update, sync-two-way, sync-preview | M1 | Must | Unit test per mode |
| FR-22b | The default copy mode is `verified` and the default synchronisation mode is `sync-preview` | M1 | Must | Configuration test |
| FR-22c | The `verified` mode compares the checksum of source and destination after the copy and reports a mismatch as a job error | M1 | Must | Unit test with a corrupted destination |
| FR-22d | The `atomic` mode writes to a temporary file next to the destination, verifies it, and renames it into place | M2 | Must | Integration test plus a failed-verification case |
| FR-22e | `sync-mirror` is the only mode that deletes, and the deletion appears in the plan before it runs | M2 | Must | Unit test on the plan |
| FR-22f | Every synchronisation produces a plan with the state per entry, and no data moves before the plan is confirmed | M2 | Must | Unit test on the plan |
| FR-22g | The job cue shows the state, the progress, the rate, the estimate and the current item | M1 | Must | Manual test plus state model test |
| FR-22h | The verification phase has its own progress and is visible as a separate state | M1 | Must | Manual test |
| FR-22i | A queued job can be reprioritised, retargeted, and its conflict and verification modes changed | M1 | Must | Unit test per editable field |
| FR-22j | A running job can be paused, resumed and cancelled, and a change takes effect at the next item boundary | M1 | Must | Unit test with a controlled source |
| FR-22k | The failed items of a job can be retried on their own | M2 | Should | Unit test |
| FR-22l | The job list can be reordered, filtered by state and inspected | M2 | Should | Manual test |
| FR-22m | The Windows build ships in a portable variant that runs from a USB stick without installation | M3 | Must | Manual test on a removable drive |
| FR-22n | The portable variant keeps its configuration next to the executable, not in the user profile | M3 | Must | Unit test of the path resolution |
| FR-22o | The portable variant writes no file outside its own directory except the documents the user explicitly opens | M3 | Must | Test that the portable directory is the only place written |
| FR-22p | The installed and the portable variant share one executable; the behaviour is selected by the presence of a marker file | M3 | Must | Manual test of both modes |
| FR-22q | The portable variant carries no activation, no licence check and no network call at start-up | M3 | Must | Test that the start-up opens no socket |
| FR-23 | Overwrite is not the default | M1 | Must | Configuration test |
| FR-24 | Every job reports progress with a rate and can be cancelled | M1 | Must | Unit test on the job queue |
| FR-25 | A job collects its errors instead of stopping at the first one | M1 | Should | Unit test with a partially failing copy |
| FR-26 | Undo exists for copy operations and is labelled as partial | M2 | Should | Manual test plus documentation review |
| FR-27 | The trash is the default path for deletion | M1 | Must | Manual test |
| FR-28 | The path length limit of the platform is detected and reported in plain language | M3 | Must | Manual test on Windows [35] |

### 4.4 Multiple rename

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-30 | The rename dialog offers placeholders for name, extension, parent, counter, date, time, size, short checksum, random value and case conversion | M1 | Must | Unit test per placeholder |
| FR-31 | The dialog offers search-and-replace, optionally with regular expressions | M1 | Must | Unit test |
| FR-32 | The dialog offers a counter with start value, step and zero-padded width | M1 | Must | Unit test |
| FR-33 | The rename result is deterministic: the same input always yields the same output | M1 | Must | Property test |
| FR-34 | The dialog shows a live preview of every new name before the rename runs | M1 | Must | Manual test |
| FR-35 | Conflicts are marked and listed in a conflict jump list reachable with F3 and Shift+F3 | M1 | Must | Unit test plus manual test |
| FR-36 | Named templates can be stored and re-applied | M1 | Must | Configuration test |
| FR-37 | The series episode template detects season and episode in an existing name and offers them as pre-filled values | M1 | Must | Unit test per input pattern |
| FR-38 | The series name defaults to the parent directory, is always visible and can be overridden | M1 | Must | Unit test plus manual test |
| FR-39 | The season and episode numbers are zero-padded, and the width grows automatically when the selection contains a three-digit number | M1 | Must | Unit test |
| FR-40 | Characters that the target platform forbids are reported before the rename and a replacement is suggested; nothing is changed silently | M1 | Must | Unit test |
| FR-41 | The rename is atomic per selection: either every file is renamed or none is | M2 | Should | Unit test with a failing case in the middle |

### 4.5 Viewer

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-50 | The viewer opens in an additional area without leaving the application | M1 | Must | Manual test |
| FR-51 | Text is shown with character set detection for UTF-8 and UTF-16 | M1 | Must | Unit test per encoding |
| FR-52 | A hex dump shows offset, hex and printable ASCII | M1 | Must | Unit test |
| FR-53 | Common image formats are displayed | M1 | Must | Unit test per format |
| FR-54 | The contents of an archive can be browsed and a single entry extracted | M1 | Must | Integration test |
| FR-55 | File metadata is shown: size, type, timestamps, checksum | M1 | Must | Unit test |
| FR-56 | PDF text and page navigation are supported | M2 | Should | Manual test |
| FR-57 | Syntax highlighting is supported | M2 | Should | Manual test |
| FR-58 | Audio and video metadata are shown with a still frame | M9 | Could | Manual test |
| FR-59 | The viewer is read-only; editing is a separate later feature | M1 | Must | Code review |
| FR-60 | The number of formats that are actually supported is stated truthfully in the about dialog | M2 | Must | Documentation review |

### 4.6 Archives

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-65 | An archive appears as a normal entry and opens as a virtual directory | M1 | Must | Integration test |
| FR-66 | Entries can be listed, viewed and copied to the other panel without extracting the archive first | M1 | Must | Integration test |
| FR-67 | ZIP, TAR with GZip, BZip2, XZ and Zstd, and 7z are supported | M1 | Must | Integration test per format |
| FR-68 | Archives can be created, including encrypted | M1 | Must | Integration test |
| FR-69 | RAR and the exotic formats are not supported | M0 | Must | Requirement, verified by the absence of a dependency |
| FR-70 | Every extraction goes through the hardened path resolution | M1 | Must | Negative security test |
| FR-71 | An archive can be tested for integrity | M2 | Should | Integration test |

### 4.7 Search, compare, synchronise

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-74a | The current directory can be filtered while typing, without a dialog | M1 | Must | Manual test plus unit test of the matcher |
| FR-74b | The filter matches the name with a pattern language: `*` and `?` as wildcards, `~` for a case-insensitive match, and a plain text that matches anywhere in the name | M1 | Must | Unit test per pattern form |
| FR-74c | The filter can be limited to files, to directories or to both | M1 | Must | Unit test per kind |
| FR-74d | The filter can search deeper than the current directory, with a configurable depth | M1 | Should | Integration test |
| FR-74e | The number of matching entries is shown while the filter is typed, and the list updates live | M1 | Must | Manual test |
| FR-74f | The filter state belongs to the panel and survives a directory change, so the user does not have to retype it | M1 | Should | State model test |
| FR-74g | The filter is searchable by content in the same field, using an explicit prefix, so that the name filter stays fast | M6 | Should | Integration test |
| FR-75 | The search matches name, size, date and content | M6 | Must | Integration test |
| FR-76 | Text, hex and binary search are supported | M6 | Should | Integration test |
| FR-77 | Boolean operators AND, OR, NOT are supported | M6 | Could | Integration test |
| FR-78 | The content search runs in the background with a growing result list | M6 | Must | Manual test |
| FR-79 | Two directories are compared for content, date and size | M2 | Must | Integration test |
| FR-80 | The synchronisation transfers differences in both directions from a shown plan | M6 | Must | Integration test |

### 4.8 Remote sources

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-85 | A remote source behaves like a local directory in every operation | M6 | Must | Integration test with an in-process server |
| FR-86 | SFTP, FTP, SMB and WebDAV are supported | M6 | Must | Integration test per protocol |
| FR-87 | S3-compatible targets are supported | M7 | Must | Integration test |
| FR-88 | Cloud sign-in uses OAuth 2.0 directly against the provider | M7 | Must | Manual test |
| FR-89 | The application operates no server of its own | M7 | Must | Architecture review plus dependency review |
| FR-90 | SMB 1 and NTLMv1 are not implemented | M6 | Must | Negative test against a server that offers only SMB 1 |
| FR-91 | SMB signing and SMB 3 transport encryption are required where the server supports them | M6 | Must | Integration test against a server with and without signing |
| FR-92 | The WebDAV client is part of the project, not a copyleft dependency | M6 | Must | `cargo deny` plus code review |
| FR-93 | Every network operation has a timeout | M6 | Must | Integration test against a server that stalls |
| FR-94 | The response size is bounded independently of the content length the server announces | M6 | Must | Integration test |
| FR-95 | A server that offers an older protocol version is refused, not silently accepted | M6 | Must | Integration test |

### 4.9 Input model

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-100 | Every operation is bound to a named action, not to a physical key | M1 | Must | Unit test on the action catalogue |
| FR-101 | The key binding, the pointer, the touch gesture, the D-pad and the controller each bind to the same actions | M1 | Must | Unit test per context — umgesetzt in M4, siehe `docs/acceptance/M4-ACCEPTANCE.md` |
| FR-102 | The action palette lists every action that is reachable in the current context | M8 | Must | Reachability protocol |
| FR-103 | The action palette is fully operable with a D-pad and the confirm key | M8 | Must | Manual test |
| FR-104 | Focus and selection are separate states in the D-pad and controller contexts | M4 | Must | State model test |
| FR-105 | The multi-selection on a remote control uses an explicit selection mode | M8 | Must | Manual test |
| FR-106 | The interface adapts structurally per context, not only in colour and font size | M2 | Must | Screenshot review per context |
| FR-107 | Every action reachable with the keyboard is reachable with the D-pad | M8 | Must | Reachability protocol |

### 4.10 Interface configuration

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-110 | The user chooses between a clean interface and a full interface | M2 | Must | Manual test |
| FR-140 | The clean interface shows only the essential commands | M2 | Must | Screenshot review |
| FR-141 | The full interface shows every command and every panel | M2 | Must | Screenshot review |
| FR-142 | The choice can be made per platform and per device context | M2 | Should | Configuration test |
| FR-143 | Every action can be shown or hidden individually | M2 | Should | Configuration test |
| FR-144 | The interface profile can be exported and imported | M2 | Should | Round-trip test |
| FR-145 | The interface profile contains no data beyond the settings it configures | M2 | Must | Documentation review |

### 4.11 Internationalisation

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| FR-120 | English and German are complete at the release of M2 | M2b | Must | Gate that compares the key sets of both catalogues |
| FR-121 | The catalogue key set is identical in every language | M2b | Must | Gate comparing the catalogues |
| FR-122 | A missing translation falls back to English and then to the key, never to an empty string | M2b | Must | Unit test |
| FR-123 | Numbers, sizes, dates and times follow the system locale | M2b | Must | Unit test |
| FR-124 | The language can be changed at run time without a restart | M2b | Must | Manual test |
| FR-125 | Adding a language requires no change to the source code | M2b | Must | Documentation review |
| FR-126 | The interface language and the content language are independent | M2b | Should | Manual test |

### 4.12 Non-functional requirements

| ID | Requirement | Milestone | Priority | Verification |
|---|---|---|---|---|
| NFR-01 | A directory of 10,000 entries is on screen within 100 ms | M2 | Must | Benchmark |
| NFR-02 | Switching between loaded directories takes less than 16 ms | M2 | Must | Benchmark |
| NFR-03 | A key press produces a visible reaction within 50 ms | M2 | Must | Benchmark |
| NFR-04 | The application is usable within 800 ms of start on macOS | M2 | Must | Measurement |
| NFR-05 | Idle memory with two panels open stays below 120 MB | M2 | Must | Measurement |
| NFR-06 | A sequential copy reaches at least 80 % of the sequential read and write bandwidth | M2 | Must | Benchmark |
| NFR-07 | A name search across 100,000 files completes within 30 s | M6 | Must | Benchmark |
| NFR-08 | Listing an archive with 5,000 entries takes less than 500 ms | M2 | Must | Benchmark |
| NFR-09 | No operation that can take longer than 50 ms blocks the interface | M1 | Must | Test on the job queue |
| NFR-09a | The verification of a copy raises the read volume to at most twice the copied volume | M1 | Must | Benchmark |
| NFR-09b | The job cue updates at most ten times per second, and never blocks the interface | M1 | Must | Unit test on the update rate |
| NFR-10 | The whole operation is operable with the keyboard alone | M2 | Must | Keyboard-only test protocol |
| NFR-11 | Text contrast is at least 4.5:1, focus frames at least 3:1 | M2 | Must | Computed contrast |
| NFR-12 | `prefers-reduced-motion` is respected | M2 | Must | Manual test |
| NFR-13 | A screen reader can traverse the interface through AccessKit | M2 | Must | Manual test with a screen reader |
| NFR-14 | No source file exceeds the target value of 600 lines; the absolute maximum is 900 lines | M0 | Must | `scripts/check-file-size.sh` |
| NFR-15 | No `unsafe`, no `unwrap()` and no `expect()` on a reachable path | M0 | Must | Workspace lints plus clippy |
| NFR-16 | Every public function carries a WHAT, WHY, EXAMPLE doc comment | M0 | Must | `missing_docs` lint |
| NFR-17 | Every public function has at least one unit test | M1 | Must | Coverage review |
| NFR-17a | Every module carries a WHAT, WHY and EXAMPLE header that names its responsibility and its boundary | M0 | Must | `missing_docs` lint plus review |
| NFR-17b | Every type, function and variable carries a name that states what it is, not what it is made of | M1 | Must | Review |
| NFR-17c | Every crate and every module relationship is documented in a diagram, so that a reader can see where something lives and how it is wired | M2 | Must | Review of `docs/ARCHITECTURE.md` |
| NFR-18 | No dependency carries a copyleft licence | M0 | Must | `cargo deny` |
| NFR-19 | No dependency carries a known vulnerability | M0 | Must | `cargo audit` |
| NFR-20 | The crate dependency direction is `apps` → `platform` → `crates` | M0 | Must | `scripts/check-crate-boundaries.sh` with a negative test |
| NFR-21 | The build is reproducible from the committed lock file | M0 | Must | `cargo build --locked` |
| NFR-22 | The application is fully operable with the keyboard, the pointer and the touch | M2 | Must | Per-context test protocol |
| NFR-23 | The application never runs with elevated privileges and never asks for them | M2 | Must | Code review plus a test that the process does not request elevation |
| NFR-24 | The application writes only inside its own configuration directory and the directories the user explicitly opens | M2 | Must | Test that traces every write |
| NFR-25 | The application opens no network connection unless the user configured a remote source and triggers it | M2 | Must | Test that the start-up opens no socket |
| NFR-26 | The application is signed on Windows and macOS, and the signature is verified before the release is published | M3 | Must | Verification of the release artefact |
| NFR-27 | The application requests no permission that it does not use | M3 | Must | Manifest review against the documented use |
| NFR-28 | The application makes no change to the security configuration of the machine it runs on | M3 | Must | Code review |
| NFR-29 | The rules of section 9.7 are configurable: the user decides which of them apply | M2 | Must | Configuration test per rule |
| NFR-30 | The configuration carries a `corporate` profile that enables every rule of section 9.7 at once | M2 | Must | Configuration test |
| NFR-31 | A rule that is switched off is never executed, and the switch is visible in the settings | M2 | Must | Unit test plus manual test |
| NFR-32 | Three guarantees cannot be switched off: the absence of telemetry, the absence of an elevation request and the absence of a background index | M2 | Must | Test that the attempt to configure them has no effect |

The non-functional requirements are the acceptance conditions of the milestones above. Every one
of them is measured rather than estimated, and a value that was not measured is reported as not
measured.


---

## 5. Technical Plan

### 5.1 Layer model

The workspace is organised in four layers with a strictly enforced dependency direction. Every
crate belongs to exactly one layer, and a crate may only depend on crates in the layer below it
or on the same layer through an explicitly permitted exception.

```
┌─────────────────────────────────────────────────────────────────┐
│ apps/                                                            │
│   phocommander-desktop   phocommander-android   phocommander-ios │
│   → binaries and shells, composition root, platform wiring       │
├─────────────────────────────────────────────────────────────────┤
│ platform/                                                        │
│   phocommander-platform                                          │
│   → OS integration: native menus, file dialogs, trash, rights    │
├─────────────────────────────────────────────────────────────────┤
│ crates/                                                          │
│   core  config  keymap  ops  archive  viewer  remote  ui  i18n   │
│   → the portable product: model, logic, adapters, presentation   │
├─────────────────────────────────────────────────────────────────┤
│ (no layer)                                                        │
│   → only the Rust standard library and third-party crates        │
└─────────────────────────────────────────────────────────────────┘
```

The rule is checked by `scripts/check-crate-boundaries.sh`, which reads the `Cargo.toml` files and
fails the build with exit code 1 on a violation. The rule was proven effective by a negative
test during M0: a deliberately added dependency from `phocommander-core` to `phocommander-ui`
made the gate fail, and removing it made the gate pass again.

### 5.2 Module catalogue

The product is decomposed into twelve modules. The table states, for each, what it owns, what it
must not know, and the requirement IDs it carries.

| Module | Crate | Owns | Must not know about | Carries |
|---|---|---|---|---|
| **Model** | `phocommander-core` | `Entry`, `EntryKind`, path handling, the error hierarchy | Any window, widget or operating system | FR-16, FR-17, NFR-01 |
| **Configuration** | `phocommander-config` | Schema, defaults, loading, migration, profiles | The user interface, the file system | FR-110 – FR-145 |
| **Binding** | `phocommander-keymap` | The action catalogue, the binding tables, the context model | Concrete widgets, concrete input devices | FR-100 – FR-107 |
| **Operations** | `phocommander-ops` | The job queue, copy, move, delete, rename rules | The file system directly, the widgets | FR-20 – FR-41, NFR-09 |
| **Archive** | `phocommander-archive` | Format adapters, entry listing, hardened extraction | The user interface, the network | FR-65 – FR-71 |
| **Viewer** | `phocommander-viewer` | Format adapters for the viewer, the read path | The user interface, the network | FR-50 – FR-60 |
| **Remote** | `phocommander-remote` | SFTP, FTP, SMB, WebDAV, S3 adapters, credentials | The user interface, the archive format | FR-85 – FR-95 |
| **Presentation** | `phocommander-ui` | Panels, widgets, theme, layout per context | The protocols, the job implementation | FR-10 – FR-18, FR-106 |
| **Localisation** | `phocommander-i18n` | The catalogue, the lookup, the fallback chain | Everything except the catalogue format | FR-120 – FR-126 |
| **Platform** | `phocommander-platform` | Native menu, file dialog, trash, permissions | The product logic | FR-04, FR-27, FR-28 |
| **Desktop shell** | `apps/phocommander-desktop` | The binary, the event loop, the composition root | The internals of the modules it wires | FR-01 – FR-06 |
| **Mobile shells** | `apps/phocommander-android`, `apps/phocommander-ios` | The store packaging, the platform bridge | The internals of the modules they wire | FR-87 – FR-89 |

### 5.3 Module contracts

A module contract is what a module promises to its callers. Contracts are narrow on purpose: a
contract that promises too much is the reason a codebase grows in the wrong direction.

**The model module promises** a directory listing and nothing more. It answers the question "what
is in this directory" and returns entries that carry a name, a kind, and — where the source can
answer it — a size and a timestamp.

**The operations module promises** that a job completes, reports progress and can be cancelled.
It does not promise that the operation succeeds: a job that copied 400 of 500 files and failed on
the other 100 is a completed job with errors.

**The archive module promises** that an extracted entry never leaves the destination directory.
This promise is enforced by a negative test, not by inspection.

**The presentation module promises** that it never blocks. Every slow operation is a job; the
presentation layer only ever reads a snapshot of the job state.

**The remote module promises** that every operation terminates. A source that stops answering
produces an error after the configured timeout, never a hanging job.

**The localisation module promises** that a string is never empty. A missing key falls back to
English and then to the key itself.

### 5.4 Extension points

Four things in this product can be extended without touching an existing module. Each extension
point is a trait plus a registration, and the documentation states what a new implementation must
provide.

Each extension point is a trait plus a registration. The table states what a new
implementation has to provide.

| Extension point | Trait | Where it lives | What a new implementation provides | Requirement |
|---|---|---|---|---|
| File source | `Source` | `phocommander-core` | A way to list a directory and report its errors | FR-85 |
| Writable source | `MutableSource` | `phocommander-core` | Copy, move, rename, delete within a source | FR-20 |
| Archive format | `ArchiveFormat` | `phocommander-archive` | Listing, reading, writing entries of one container format | FR-67 |
| Viewer format | `ViewerFormat` | `phocommander-viewer` | Rendering one file kind into the viewer area | FR-51 |
| Rename template | TOML file | `config/templates/` | A name pattern, a detection rule, a counter definition | FR-36 |
| Action | `Action` | `phocommander-keymap` | An identifier, a label, a category, a default binding per context | FR-100 |
| Interface profile | TOML file | `config/profiles/` | A set of visible commands and a panel arrangement | FR-110 |

The rule for all of them is the same: a new implementation is a new file plus a registration
line. It never requires a change to an existing module, and it never requires a change to the
model.

### 5.5 Data flow

The path from a key press to a file being copied runs through five stages. Each stage has one
owner, and the ownership is what keeps the layers apart.

```
1. Input            the platform delivers a key, a click, a gesture or a button
       │              → phocommander-platform normalises it into a platform-neutral event
       ▼
2. Binding          the event is resolved against the binding table of the active context
       │              → phocommander-keymap produces an ActionId
       ▼
3. Command          the ActionId is dispatched to the command layer
       │              → phocommander-ui or a headless dispatcher executes it
       ▼
4. Operation        the command builds a job and submits it
       │              → phocommander-ops owns the queue, the progress and the cancellation
       ▼
5. Source           the job performs the work through the source interface
                      → local, archive or remote, depending on the panel
```

Two properties follow from this and are worth stating because they are what the diagram is for.
First, the operation is testable without a user interface: stage 4 and stage 5 can be driven
directly. Second, the input path is testable without a file system: stage 1 to stage 3 can be
driven with synthetic events.

### 5.6 Concurrency model

| Concern | Owner | Rule |
|---|---|---|
| The interface thread | `phocommander-ui` | Never blocks. Never performs I/O. |
| The job queue | `phocommander-ops` | Thread-safe. The interface reads a snapshot. |
| Worker threads | `phocommander-ops` | Bounded by `advanced.worker_threads`, default the core count |
| Directory listing | The source | Runs on a worker, returns a snapshot |
| File watching | `notify` | One watcher per open panel, coalesced |
| Progress reporting | `phocommander-ops` | At most ten updates per second, to avoid flooding the interface |

The rule that everything slow is a job has one consequence worth naming: the panel state is
updated from the job, never the other way round. A job that is cancelled mid-copy leaves the
destination in a partially written state, and the interface reports that honestly rather than
rolling back silently.

### 5.7 Error model

The error hierarchy follows ADR-0003. Every error carries a stable numeric code so that it can
be referred to in documentation, in a bug report and in a test without quoting prose.

| Layer | Error type | Carries |
|---|---|---|
| Model | `phocommander_core::Error` | A code, a message, an optional path |
| Operations | `phocommander_ops::JobError` | The failing item, the reason, whether the job continues |
| Archive | `phocommander_archive::ArchiveError` | The entry, the reason, whether the extraction was refused |
| Remote | Per-source error type, converted at the boundary | The protocol, the reason |

The conversion happens at the boundary: the core turns a source error into its own error, so that
the presentation layer never sees a protocol-specific type. The presentation layer shows a
message from the localisation catalogue, keyed by the error code, not by the error text.

### 5.8 Build and dependency policy

| Rule | Value | Reason |
|---|---|---|
| Rust edition | 2024 | Current stable |
| Minimum supported Rust version | 1.95 | Fixed in `rust-toolchain.toml` |
| Lock file | Committed | The build resolves to identical versions everywhere |
| Copyleft dependencies | Forbidden | Would pull the whole product under a copyleft licence |
| Dependencies with a C component | Avoided | Cross-compilation for Android and iOS |
| `unsafe` | Forbidden by lint | Reviewability |
| `unwrap()`, `expect()` | Forbidden on reachable paths | A panic in a file operation is a poor user experience |
| Source file length | Target value 600 lines, absolute maximum 900 lines | Forces separation of concerns while leaving room for a documented, coherent file |
| Dependency additions | Require a licence check and a rationale in the pull request | Keeps the licence policy enforceable |

Together these rules are what makes the quality gates enforceable rather than aspirational: each
one has a script, and each script has a negative test that proves it fails when it should.


---

## 6. Input Model and Interaction Contexts

### 6.1 The action model

Every operable function of Phocommander is a named action with a stable identifier. The input
layer translates physical input into action identifiers. The presentation layer knows only action
identifiers and never knows whether an action was triggered by a key, a click, a gesture, a D-pad
button or a controller trigger.

This separation is not decoration. It is the precondition for the same application working with a
keyboard on a laptop, with fingers on a tablet, with a remote control on a television and with a
controller in a headset — without four separate interfaces.

The action catalogue lives in `config/keymap.toml` and is documented in `docs/KEYMAP.md`. An
action carries an identifier, a label key, a category and a default binding per context.

### 6.2 The interaction contexts

A context describes how a particular input device reaches the action identifiers. The bindings
are stored per context, so a user sees a different set on the laptop than on the television.

| Context | Device | Available signals | Layout principle |
|---|---|---|---|
| `keyboard` | Keyboard | Keys, modifiers, text input | Full binding set, Norton Commander function keys |
| `pointer` | Mouse | Pointer, click, double click, right click, drag, scroll wheel | Two panels side by side, menu bar, toolbar, hover |
| `touch` | Finger | Tap, long press, swipe, two-finger scroll, pinch | Stacked panels, large targets, sheet dialogs, no hover |
| `dpad` | TV remote | D-pad, confirm, back, home, play/pause | One panel visible, 10-foot scaling, focus frame |
| `controller` | Gamepad, VR controller | Trigger, grip, thumbstick, A/B/X/Y, D-pad | Spatial arrangement, ray pointer, no modifiers |

### 6.3 The action palette

The central conflict between the interaction model and a remote control: the Norton Commander
function keys `F2` to `F10` carry the main operations, and on a Fire TV or Google TV remote those
keys do not exist. Only a D-pad, confirm, back and home are available. A VR controller has no
function keys either.

The solution is the **action palette**: a searchable list of every action that is reachable in the
current context. In the `dpad` context it opens from the menu or play/pause key and is fully
operable with the D-pad and the confirm key. It replaces the function key row functionally, and it
is available in every context — on the desktop as "find action".

The palette filters by context. An action that requires a held modifier key does not appear in the
`dpad` context, because it could not be triggered there. Showing it would be clutter, and clutter
is what the product principles forbid.

### 6.4 Focus and selection as separate states

On platforms without a pointer there is no hover. Focus therefore has to be a visible, permanent
state rather than a side effect of pointer position.

In the `dpad` and `controller` contexts, focus and selection are kept apart:

| State | Meaning | Changed by |
|---|---|---|
| Focus | The entry the next action applies to | Navigation |
| Selection | The marked entries an action applies to | Explicit selection commands |

Navigating moves the focus without changing the selection. Selecting changes the selection
without losing the focus. A remote control has no key combination for multi-selection, so the
`dpad` context offers an explicit selection mode in which confirm toggles the mark and the
direction keys move the focus. The mode is visible and can be left at any time.

### 6.5 Adaptive layouts per context

The layouts differ structurally, not only in colour and font size.

**Native integration.** Four axes carry the platform character, and they are what separates
Phocommander from a web form in a window:

| Axis | Implementation | Source |
|---|---|---|
| Menu | A real native menu through `muda` [32]; on macOS in the system menu bar, on Windows as an `HMENU` in the window, on Linux and BSD through GTK | [32][34] |
| Keyboard | Platform modifiers: `Cmd` on macOS and iPadOS, `Ctrl` on Windows, Linux and BSD. The actions stay identical, only the binding varies | [36] |
| Dialogs | Native file dialogs through `rfd`, so permission prompts and file type filters look and behave like the system's | [26] |
| Metrics and theme | Platform-specific row heights, font sizes, focus frames, colour palettes and sidebar depth, so the interface does not look like a web application | [12] |

This combination yields a dense, keyboard-oriented tool with system conventions. It does not
yield the pixel accuracy of a native AppKit program, and the specification states that openly,
because it is one of the open design decisions of the project.

**`keyboard` and `pointer`.** Both panels are visible at the same time. Source and destination
are graspable at a glance, which is the core of the concept.

**`touch`.** The panels are stacked: one file list fills the screen, the second lies below and is
reached by a swipe or a switching control. Row heights are at least 44 points. Dialogs appear as
a sheet from the bottom, not as a centred box. Hover effects are absent, because hover does not
exist.

**`dpad`.** One panel is visible. Two simultaneous panels would be unreadable on a television.
The second panel appears as an overlay when an action needs a destination. Type sizes are designed
for a viewing distance of about three metres, focus frames are markedly stronger than on the
desktop. The operation hints are visible, because a remote control has no key labels.

**`controller`.** The panels are arranged spatially rather than in a window. The focus sits on
the ray pointer, not on the row.

### 6.6 Meta Quest as an Android configuration

Horizon OS is an Android operating system [18][44]. A Horizon OS application is an ordinary
Android program that runs as a **flat panel** in space; touch gestures such as tap, swipe, scroll
and pinch are mapped by the system to controller and hand-tracking input [19][45].

The consequence is a recommendation: **no immersive VR port**. The Android version gains a
`controller` interaction context, and Quest operates the application as a panel. This avoids an
entire OpenXR and Vulkan frontend that would bring no value to a file manager. The `openxr` crate
is available under `MIT OR Apache-2.0` [28][46] and is held in reserve for a possible spatial
variant later, but it is not planned for M0 through M9.

On Android, the platform constraints are the same as on any other Android device: scoped storage
applies [41], and full access to the shared storage is a special permission that has to be
declared in the Play Console and explained to the user [40][42]. The same applies on Horizon OS,
which shares the Android permission model.

The limitation is stated honestly: in panel mode the operation is limited to gestures and
controller actions. A file manager lives on a keyboard and a fine pointer, and a panel in space has
neither. Quest is therefore planned as an **interaction context on the Android version**, not as
an independent target.

### 6.7 Fire TV and Google TV

On a television the D-pad is the primary navigation [14][40]. Google Play tests television
applications for whether the **entire functionality is reachable with a five-way D-pad and the
back key** [15]. Further requirements:

- The application must not depend on a menu key of the remote to reach interface controls [15].
- Back navigates backwards and leaves the application without an endless loop [14][15].
- `android.hardware.touchscreen` is declared as not required, `CATEGORY_LEANBACK_LAUNCHER` is set,
  and a 320×180 pixel banner is provided.
- The pointer is never the primary means of operation.

For Phocommander this means the `dpad` context is a **complete** way of operating, not a fallback.
Every action that is reachable with the keyboard must be reachable through the action palette in
the `dpad` context. The acceptance milestone M8 carries a protocol that checks exactly this.

### 6.8 Where the input comes from

`winit` delivers keyboard, mouse and touch events on the desktop, but gamepad events only on [6][39][48]
Redox — on Linux neither under X11 nor under Wayland [39][48]. The `dpad` and `controller` contexts
therefore **do not** take their input from `winit`; they take it from the platform shell. On
Android and iPadOS the D-pad arrives as ordinary key events and needs no special path. On the
desktop, controller support is a separate task that binds gamepads through a platform-native
library rather than through `winit`.


### 6.9 Safe copy and synchronisation modes

A copy is not one operation but a family of them, and the difference between them is the
difference between a fast copy and a copy that can be trusted. The user chooses the mode; the
default is the safe one, because the failure of a copy is discovered far too late otherwise.

| Mode | What it does | Cost | When it applies |
|---|---|---|---|
| `quick` | Byte copy, no verification | The speed of the medium | Throwaway data, a known-good source |
| `verified` | Byte copy, then a checksum of source and destination compared | Twice the read volume | The default for everything |
| `atomic` | Write to a temporary file next to the destination, verify, then rename into place | The same as `verified`, plus one rename | Files that a running program may read |
| `sync-mirror` | The destination becomes an exact copy of the source: files that are missing are copied, files that differ are overwritten, files that are extra in the destination are deleted | The full comparison | Maintaining a mirror |
| `sync-update` | One-way: the destination is brought up to date with the source, but nothing is deleted | The full comparison | Keeping a backup current |
| `sync-two-way` | The newer version wins in both directions, determined by timestamp and then by checksum | The full comparison | Two machines that both change | 
| `sync-preview` | No data is moved. The comparison runs and produces a plan that the user confirms | The full comparison | Before any destructive synchronisation |

The distinction between `sync-update` and `sync-two-way` is the distinction between a backup and
a partnership, and it is the user's decision, never the tool's. `sync-mirror` is the only mode
that deletes, and it requires the deletion to be listed in the preview before it runs.

The comparison behind every synchronisation mode is the same and produces the same plan: per
entry, the state is `identical`, `different`, `missing-in-destination` or `extra-in-destination`,
together with the size and the timestamp of both sides and the checksum where one was computed.
The plan is what the user confirms, and it is the same document that the acceptance of a
synchronisation refers to.

The modes are configurable per operation in the settings and selectable per operation in the
dialog. The default is `verified` for a copy and `sync-preview` for a synchronisation, so that
the destructive case always requires an explicit act.

### 6.10 The job cue

A running operation must be visible without hunting for it, and its state must be readable at a
glance. The **job cue** is the permanent, always-present indicator that carries this information.

| Element | Content |
|---|---|
| State | Queued, running, verifying, waiting, done, failed, cancelled |
| Progress | Completed bytes against total bytes, and the number of items |
| Rate | Current throughput, smoothed over a short window |
| Estimate | The remaining time, derived from the rate |
| Current item | The name of the item being worked on |
| Verification | A second phase with its own progress when the mode includes verification |

The cue appears in three places, all driven by the same state: in the status bar of the active
panel, in a list of all running operations, and as a marker on the panel that owns the
operation. The user therefore sees the state whether they are working in the other panel, in the
viewer, or in a dialog.

The cue is deliberately plain. It shows a state, a number and a rate. It does not offer a
progress bar with a percentage that has no meaning for a directory tree, and it does not
celebrate completion.

### 6.11 Editable jobs

A job that has been submitted is not frozen. As long as it has not finished, the user can change
it — because a queue of twenty copy operations often contains one with the wrong target or the
wrong mode, and discovering that only after the last one has run is the worst possible moment.

| Action | Allowed while | Effect |
|---|---|---|
| Change the priority | Queued | The job moves within the queue |
| Change the conflict policy | Queued | The job uses the new policy when it starts |
| Change the verification mode | Queued | The job verifies with the new mode |
| Change the destination | Queued | The job targets the new destination |
| Pause and resume | Running | The job stops and continues at the same item |
| Cancel | Queued or running | The job ends, and what it has already done stays done |
| Retry the failed items | Failed | Only the failed items run again |

A change to a running job takes effect at the next item boundary, never in the middle of a file
being written. The change is recorded in the job's history, so that an interrupted and corrected
operation remains traceable.

The jobs are edited in the job list, which is the same list the cue refers to. The list is a
first-class surface, not a notification area: it can be reordered, filtered by state and
inspected.

---

## 7. Interface Configuration Model

The user decides how much interface they see. Two extremes exist, and everything between them is
reachable.

### 7.1 The two extremes

| Profile | What it shows | Who it is for |
|---|---|---|
| `clean` | The panel pair, the path, the file list, the command line. Nothing else | Someone who knows the keys and wants nothing between them and the files |
| `full` | The panel pair, the menu bar, the toolbar, the directory tree, the quick view, the job list, the status bar, every command | Someone who arrived from a desktop file manager and expects the furniture |

The choice is not a hidden preference. It is a visible setting with a one-line description of
what it changes.

### 7.2 The profile system

A profile is a named set of decisions:

```toml
# Interface profile — an example.
name = "clean"
description = "Only the two panels and the command line."

[panels]
# Which auxiliary areas exist in this profile.
show_toolbar = false
show_directory_tree = false
show_quick_view = false
show_job_list = false
show_status_bar = true
show_menu_bar = false

[commands]
# How commands are presented. "hidden" removes them from every menu and
# palette; "palette" keeps them reachable but out of sight.
visibility = "palette"

[display]
# Per-command overrides. A command that is visible by default can be
# hidden, and a command that is hidden by default can be made visible.
hidden = ["panel.tree.toggle", "panel.view.tabs"]
```

The profile is a normal TOML file in the configuration directory. The built-in profiles `clean`
and `full` ship with the application; the user can create their own, and can export and import
them. A profile contains nothing but interface decisions — no paths, no history, no credentials.

### 7.3 Resolution order

The interface resolves what to show in a fixed order, so that the answer is predictable:

1. The built-in default for the active context.
2. The profile selected in the configuration.
3. The per-command overrides of that profile.
4. The per-platform override, if one exists.

The last step is what allows the same profile to behave differently on a television and on a
desktop without the user maintaining two files: the platform override can, for example, force the
menu bar on for a pointer context where the menu bar is the expected affordance.

### 7.4 What the clean profile does not remove

The `clean` profile removes furniture, never capability. Every action stays reachable through the
action palette. A clean interface is a smaller interface, not a reduced product. This is the
difference between a profile and a feature gate, and it is what keeps principle 2.1 intact.

### 7.5 The portable variant

The Windows build ships in a portable variant, in the tradition of the reference product, which
offers a portable edition for technicians who move between machines [50]. The variant runs from a
USB stick without installation and behaves like an installed copy.

**One executable, two modes.** The portable variant is not a second build. It is the same
executable, and the mode is decided by the presence of a marker file — `portable.txt` — next to
the executable. Without the marker the application behaves like an installed copy and keeps its
configuration in the user profile. With the marker it keeps its configuration next to the
executable.

**The configuration follows the executable.** In portable mode the settings, the key bindings,
the favourites and the interface profiles live in a `Phocommander/` directory beside the
executable. The rule is the same as for the local source of the model: the path is resolved once,
at the boundary, and everything below the platform layer works with the result.

**Nothing is written outside the directory.** The portable variant writes only inside its own
directory, with two exceptions that the user causes: the documents the user explicitly opens and
the trash of the system, when the user chooses to delete rather than to move to the trash. A
technician who carries the stick on a foreign machine leaves nothing behind.

**No activation.** This is the practical difference from the reference product, which binds a
licence file to the USB stick through its activation server [50]. Phocommander is free and has no
licence server, so a portable copy needs no preparation beyond copying the directory. That is the
whole point of the variant, and it is worth stating plainly: the portable copy is as complete as
the installed one.

**What the variant is not.** It is not a different product with fewer functions. Every function
of the installed variant is available, because there is only one code path.

### 7.6 Configurability without complexity

The configuration surface is deliberately small. It has four groups: appearance, panels, commands
and input. Anything that would need a fifth group has to earn its place by being something a user
actually asks for.

---

## 8. Internationalisation Plan

The internationalisation is the milestone **M2b** of the milestone plan. The roadmap lists the
points that have to hold, and the release Beta 1 depends on it.

### 8.1 Languages and order

English and German are the two languages of the first release. Both must be complete at the
release of M2. Adding a third language is a matter of adding a catalogue directory; it requires no
change to the source code (FR-125).

### 8.2 The catalogue format

The catalogue is Mozilla Fluent. The choice has three reasons that matter for this project:
Fluent keeps the linguistic complexity in the translation files rather than in the code, it has
native support for plurals, which a file manager needs for "1 file" versus "2 files", and it is
the most widely adopted localisation system for Rust applications.

| Component | Crate | Version | Licence |
|---|---|---|---|
| Fluent runtime | `fluent` | 0.17 | MIT OR Apache-2.0 |
| Bundle handling | `fluent-bundle` | 0.16 | MIT OR Apache-2.0 |
| Embedding | `rust-embed` | 8.13 | MIT OR Apache-2.0 |
| Locale detection | `sys-locale` | 0.3 | MIT OR Apache-2.0 |
| Unicode locale identifiers | `unic-langid` | 0.9 | MIT OR Apache-2.0 |
| Plural rules, number and date formatting | `icu` | 2.4 | MIT OR Apache-2.0 |

The catalogue lives in `crates/phocommander-i18n/locales/<locale>/<domain>.ftl`, split by domain
so that a translator can work on the file manager without touching the viewer or the settings.

### 8.3 The lookup chain

A lookup walks a fixed chain and never returns an empty string:

```
requested locale → language fallback chain → English → the key itself
```

The last step matters. A missing translation must be *visible* as the key, never as an empty
label, because an empty button in the interface cannot be diagnosed and a visible key can (FR-122).

### 8.4 What is and is not translated

| Translated | Not translated |
|---|---|
| All interface labels and messages | File and directory names |
| All error messages | The contents of files |
| All help texts | Protocol names such as SMB or WebDAV |
| Date, time and number formats | The action identifiers |
| Plural forms | The keymap, which is stable across languages |

The action identifiers stay in English even in a German interface, because they are keys in the
configuration file and in the documentation, and a configuration file that changes meaning with
the interface language is a configuration file that cannot be shared between colleagues.

### 8.5 Testing the localisation

Three gates protect the localisation:

| Gate | What it checks | When it fails |
|---|---|---|
| Key parity | Every locale has exactly the same key set | A translation is missing a key or has an extra one |
| Fallback | A missing key resolves to English, then to the key | The lookup chain is broken |
| Literal check | The user interface layer contains no literal strings | A string was hard-coded instead of externalised |

The first two are unit tests over the catalogues. The third is a script that scans the user
interface crate for string literals. Together they make the claim "English and German are complete"
verifiable rather than asserted.

---

## 9. Security and Privacy Plan

### 9.1 Trust boundary

The trust boundary runs between the process and the file system. Everything that arrives from a
file system, from an archive or from a network source is untrusted input. The attacker is
someone who sends the user a file, who modifies a file the user opens, or who controls a server
the user connects to.

The full treatment lives in `docs/security/THREAT_MODEL.md`. This section states the rules that
bind the implementation.

### 9.2 The hardening rules

| # | Rule | Enforcement point | Negative test |
|---|---|---|---|
| H-01 | `zip` at least 9.0.5 [22][23][24][25] | `Cargo.lock` | Version check in the build [22] |
| H-02 | No `extract()` from any crate; entries are written by us | `phocommander-archive` | `tests/security/path_traversal.rs` |
| H-03 | Every destination path is canonicalised and checked against the destination directory | `phocommander-archive` | `tests/security/path_traversal.rs` |
| H-04 | Symlinks are never created from archive material | `phocommander-archive` | `tests/security/path_traversal.rs` |
| H-05 | SUID and SGID bits are never carried over | `phocommander-archive` | `tests/security/path_traversal.rs` |
| H-06 | Limits on total size, entry count and single entry size | `phocommander-config`, `phocommander-archive` | `tests/security/decompression_bomb.rs` |
| H-07 | SMB 1 and NTLMv1 are not implemented | `phocommander-remote` | Integration test against a server offering only SMB 1 |
| H-08 | SMB signing and SMB 3 encryption are required where the server supports them | `phocommander-remote` | Integration test with and without signing |
| H-09 | TLS certificates are verified strictly; no opt-out | `phocommander-remote` | Integration test against a self-signed server |
| H-10 | Every network operation has a timeout | `phocommander-remote` | Integration test against a stalling server |
| H-11 | The response size is bounded independently of the announced content length | `phocommander-remote` | Integration test with an oversized answer |
| H-12 | Credentials live in the system keyring, never in the configuration file | `phocommander-remote` | Code review plus configuration test |

### 9.3 Accepted residual risks

The Rust standard library offers no atomic file operation with path semantics. The time between
a check and the use of a checked path therefore cannot be closed completely. The measure that
reduces the window is a descriptor-based access with `openat` semantics via `rustix` or
`cap-std`, or the repetition of the check immediately before the access. The remaining window is
named in the threat model rather than hidden. The same applies to the recursive copy, which cannot
be made race-free without a descriptor walk.

### 9.4 Secure deletion

The standard is NIST SP 800-88 Revision 2 with its levels *Clear*, *Purge* and *Destroy* [19]. The
historical DoD 5220.22-M three-pass standard is not used: it was removed from NISPOM in 2006
because multiple passes add no protection on modern media, and overwriting is not reliable on
flash storage because wear levelling and over-provisioning redistribute the cells [19][20]. The
trash is the default path for deletion. "Secure delete" is an explicit additional option whose
effect cannot be guaranteed on flash media — and that limitation is stated in the interface, not
only in the manual.

### 9.5 Privacy

No telemetry, no analytics, no advertising, no profiles, no crash reports, no third-party
disclosure. The application stores only what the user configured: settings, key bindings,
favourites, recently opened directories. Credentials for remote sources live in the system keyring.
On platforms without a keyring, the application falls back to its own sandbox and says so.

For a network connection, only the directory data the user requested leaves the machine. No
identifier, no profile, no usage data is sent.

### 9.6 Dependency policy

Every dependency is checked for its licence by `cargo deny` and for known vulnerabilities by
`cargo audit` on every build. The tool versions are pinned, because `cargo deny` reads the same
configuration differently depending on its version — an unpinned version produced a false alarm in
the Vocument project, where the message blamed the configuration instead of the problem [27].

Copyleft dependencies are excluded. This is why the WebDAV client is written as part of the
project instead of taken from `fast-dav-rs`, which is LGPL-3.0 [26]: a statically linked binary
under LGPL creates obligations that conflict with the licence of the project.


### 9.7 Security in a managed corporate environment

A file manager with full read access is, by its nature, the tool that a security department looks
at first. This section does not pretend that the problem can be designed away. It states what
Phocommander does so that the application is not a liability, and what it deliberately does not do.

**The three places where a file manager becomes visible.** On a managed Windows machine, the
application shows up in three independent places, and a good tool has to answer all three.

| Place | What the organisation sees | What Phocommander does |
|---|---|---|
| Application control | Windows Defender Application Control or AppLocker decides which programs may start. A policy in enforce mode blocks everything that is not explicitly allowed [52][53] | The application is signed, so that a publisher rule can admit it. It never asks for elevation, so it never triggers the "run as administrator" branch that policies watch for [54] |
| Data loss prevention | An endpoint agent intercepts reads, writes, copies and moves, and reports the file names, the paths and the timestamps [55][56] | The application reads only what the user opens. It does not scan, index, preview or fingerprint the disk in the background |
| Portable media control | A policy often allows a removable drive but blocks unknown executables on it, because a stick can carry a tool that copies data out [57] | The portable variant is one signed executable. It writes its configuration into its own directory, so it leaves nothing on the machine it runs on |

**The rules that follow from this.**

*Never elevate.* Phocommander never requests administrator rights and never restarts itself as
administrator. A file manager that needs elevation for a single path is a file manager that has
to be run elevated for everything, and an elevated file manager is a much larger risk than the
problem it solves. Where an operation genuinely fails because of a permission, the application
says so in plain language and lets the user decide.

*Never scan what was not asked for.* The application does not index the disk, does not build a
search index in the background, does not calculate checksums of files the user has not opened and
does not preview directories in advance. Every file access is a consequence of a user action:
opening a directory, selecting a file, starting a copy, applying a filter. This is a privacy
requirement and at the same time the requirement that keeps the application out of the
data-at-rest scan of a DLP agent [56].

*Never write outside the own directory.* Without a user action, the application writes only into
its configuration directory. It creates no temporary files in the directories of the user, it does
not write a log file next to the files it copies, and it does not touch the registry or the
system configuration. The portable variant goes further and writes nothing outside its own
directory at all (section 7.5).

*Stay quiet.* The application opens no network connection at start-up, sends nothing anywhere and
contains no update check. An update is a download the user starts. A file manager that phones
home is a file manager that a network policy has to look at, and the user has to justify.

*Be signed.* On Windows and macOS the released binary is signed, and the signature is verified
before a release is published. A signed binary can be admitted by a publisher rule in a
corporate policy [52]. An unsigned one cannot, and it will be blocked by every managed machine
that runs an application control policy.

*Ask for nothing it does not use.* The application requests no permission it does not need, and
the manifest of every platform is reviewed against the documented use of every permission. A
file manager needs no camera, no microphone, no location and no contacts. On Android it asks for
the file access it needs and for nothing else; the special permission for the whole storage is
offered as an explicit, optional extension and is declared in the Play Console [18][42].

**What the application does not do, and will not do.**

It does not attempt to work around a corporate policy. No elevation, no service installation, no
registry change, no scheduled task, no driver, no shell extension. If a policy blocks it, the
user reads the reason in the event log of the policy and asks the responsible department. A tool
that tries to be clever here is the reason a tool gets blocked.

It does not run in the background. There is no resident process, no indexer, no watcher that runs
after the window is closed. The file watcher of an open panel exists only while the panel is
open.

It does not read files in order to classify them. The application has no idea what a file
contains until the user opens it in the viewer, and even then it reads the bytes the format
requires.

**The honest limit.** Phocommander cannot make itself invisible to a security department, and
it does not try to. A file manager that can read everything is by definition a tool that a
security department wants to know about. What the application can do is make the answer easy:
it is signed, it runs without elevation, it reads only what the user asks for, it writes only
into its own directory, it talks to nobody, and it says plainly what it is doing. A colleague
can read the source and verify every one of these claims.


---

### 9.8 Configurable security rules

The rules of section 9.7 are not carved in stone. They are the default, and the user may adjust
them to the machine and to the policy in force. A rule that is switched off is simply not
executed, and the switch is visible in the settings rather than hidden in a build flag.

**The profile.** The configuration carries a profile for this group:

| Profile | What it does | Who it is for |
|---|---|---|
| `standard` | The rules of section 9.7, with the convenience switches that do not weaken a guarantee | A private machine |
| `corporate` | Every rule of section 9.7 enabled, including the ones that cost convenience | A managed machine, where the policy has to be provable |
| `custom` | Whatever the user set | A user who knows the machine |

**The individual switches.**

| Switch | Default | What it changes |
|---|---|---|
| `verify_directory_before_refresh` | on | The content of a directory is read again before it is shown, so that a change made elsewhere is noticed |
| `background_index` | off | The application keeps no index. It never reads a file that the user has not opened |
| `remember_session` | on | The last directories are written at the end. Switching it off leaves nothing behind |
| `read_only_on_demand` | on | The viewer reads a file only when the user opens it in the viewer |
| `checksum_on_copy` | on | The verified copy mode compares the checksums of source and destination |
| `follow_symlinks` | off | A symlink is copied as a symlink and never followed |

**The three guarantees that are not configurable.** Three rules cannot be switched off, because
switching them off would break a promise the product makes:

1. There is no telemetry. There is nothing to switch off, because there is nothing.
2. The application never asks for elevation. A user who wants an elevated operation runs the
   operation in a terminal; the file manager does not become a privileged process.
3. There is no background index. The application never reads a file that the user has not asked
   for.

The attempt to configure one of these in the configuration file has no effect and produces a
message in the log, so that a configuration that was carried over from another machine does not
silently weaken the guarantee.

**The point of the switch.** A tool that is used on a private machine and a tool that is used
on a managed machine do not have to behave differently in their code. They have the same code and
a different setting. The settings that a corporate policy would want to enforce are the ones the
`corporate` profile switches on, and the user who wants to see for himself what the application
does with a file can switch them off and read the source.

---

## 10. Test and Quality Strategy

### 10.1 The test layers

| Layer | What it covers | Tool | Count rule |
|---|---|---|---|
| Unit | Pure logic: path resolution, key resolution, rename rules, sorting, size formatting | Built-in test harness | Every public function has at least one |
| Integration | The boundaries between crates: a copy over a temporary directory, an extraction with malicious entries, a configuration file with an unknown field | Built-in test harness | One per crate boundary |
| Property | Invariants that cannot be enumerated: the path resolution never leaves the destination, the rename rule is deterministic, the key resolution decides deterministically | `proptest` | One per invariant |
| Fuzz | The parsers: archive headers, image headers, character set detection | `cargo-fuzz` | One target per parser family, nightly |
| Interface | The keyboard operation of the interface | `egui_kittest` | One per interaction context |
| Security | The hardening rules H-01 to H-12 | Built-in test harness | One negative test per rule |
| Performance | The values of the non-functional requirements | `criterion` | One benchmark per target value |
| Platform | The build on every target platform | `cargo` | Build check per target |

### 10.2 The negative test rule

A positive test proves that a defence is not triggered. It does not prove that the defence works.
For every hardening rule there is a test that executes the actual attack and demonstrates that the
defence holds. A rule without a negative test is not implemented, it is only intended.

### 10.3 The build matrix

| Job | What it does | When it blocks |
|---|---|---|
| `quality` | Format, clippy, tests, crate boundaries, file size, licence, advisory, telemetry | Every change |
| `desktop` | The same six gates plus the release build on Linux, macOS and Windows | Every change |
| `cross-compile` | Build for Android, iOS, FreeBSD, NetBSD and the iOS Simulator | Every change |
| `performance` | The benchmarks of the non-functional requirements | Before a milestone is accepted |
| `nightly` | Longer fuzzing and a test run without cache | Does not block; reported |

### 10.4 The build cache rule

Two cargo processes must never write to the same target directory. The failure mode is subtle: the
compiler finds two versions of the same crate and reports a missing dependency that does not
exist. The project therefore uses one lock file, one cargo home per build context, and one target
directory per concurrent build.

### 10.5 What "best in class" means here

The quality bar is not a slogan but a list of gates that fail the build:

1. The format check is not allowed to be skipped.
2. Clippy runs with `-D warnings`, and a lint is never silenced with an `#[allow]` unless the reason is written down in an ADR.
3. Every public function has a doc comment and a test.
4. Every hardening rule has a negative test.
5. The dependency direction is checked mechanically, with a negative test proving the check works.
6. No file exceeds the length limit.
7. No copyleft dependency enters the tree.
8. No known vulnerability remains unreported.
9. The performance targets are measured, not estimated.
10. Every milestone ships with an acceptance document that contains the measured evidence.

---

## 11. Milestone Plan

### 11.1 The rules of the sequence

**The core comes before the surface.** Every milestone delivers a working, testable layer in the
Rust core first. The presentation layer binds to it afterwards. A presentation feature without
core support is not started.

**Every milestone ends with a pushed state on `main`.** Progress is visible at all times, and no
work accumulates on a side branch waiting for a merge window.

**No milestone counts as reached without evidence.** A milestone without a green test run and
without an acceptance document is not reached, however convincing the description sounds.

**Questions are asked, not guessed.** Where a decision is missing, it is put to the owner and the
answer is recorded in the document that needs it.

### 11.2 The milestones

#### M0 — Foundation

**Goal.** A repository that builds, checks and tests itself on three operating systems, with the
specification as the binding basis for everything after it.

The work is divided into packages small enough to be finished in one session.

| Package | Content | Module |
|---|---|---|
| M0.1 | Cargo workspace, twelve crates, pinned dependency catalogue, committed lock file | Model, all |
| M0.2 | Workspace lints, `deny.toml`, the licence policy | All |
| M0.3 | The five CI jobs, the seven check scripts | All |
| M0.4 | The configuration schema, the key binding catalogue, the rename templates | Configuration, Binding |
| M0.5 | The requirement catalogue and the acceptance criteria | All |
| M0.6 | The threat model, `SECURITY.md`, `PRIVACY.md`, the governance documents | All |
| M0.7 | The glossary, the user guide in both languages, the architecture overview | All |
| M0.8 | The ADRs for the interface profiles and for the safe copy modes | Architect |

**Definition of done.** Build, test, format, clippy, licence, advisory, boundary, file-size and
telemetry checks all pass; the acceptance document M0 exists with the measured evidence; the
threat model names a negative test for every hardening rule.

**Status at the time of writing.** Complete. The acceptance document records 129 tests passing with
no failure and no skipped test, a green licence check, and a proven negative test of the boundary
gate. The acceptance was granted with conditions, and the conditions were closed: the CI matrix
now runs all six gates, the cross-compilation no longer tolerates failure, the iOS targets were
added, and the performance gate became a job of its own.

#### M1 — The working file browser

**Goal.** A functional dual-pane file browser on macOS that copies, moves, renames, inspects and
opens archives.

The packages below build the working file browser in the order of their dependency.

| Package | Content | Module |
|---|---|---|
| M1.1 | The directory listing model, the `Source` and `MutableSource` traits, the error hierarchy | Model |
| M1.2 | The job queue with progress, cancellation and error collection | Operations |
| M1.3 | The panel state model: two panels, tabs, cursor, selection, sorting | Presentation |
| M1.4 | The action catalogue and the resolution of an event into an action | Binding |
| M1.5 | Copy, move, delete with the conflict policies | Operations |
| M1.6 | The rename engine with the placeholders, the rules and the conflict jump list | Operations |
| M1.7 | The series episode template with detection, numbering and the preview | Operations |
| M1.8 | The viewer stage 1: text, hex, images, archive contents, metadata | Viewer |
| M1.9 | The archive adapters for ZIP, TAR and 7z, with the hardened extraction | Archive |
| M1.10 | The negative security tests for H-02 to H-06 | Archive |

**Definition of done.** A user can browse, select, copy, move, rename with a template, inspect a
file and open an archive on macOS, all of it with the keyboard. The negative tests for the
extraction hardening exist and pass.

**Dependencies.** M0.

#### M2 — Maturity on macOS

**Goal.** An application that can replace a desktop file manager in daily use, in both languages.

**Work packages.** Tabs and the directory tree, the filter field, the command line and its
history, the directory comparison, the favourites and the file container, viewer stage 2 with PDF
and syntax highlighting, the native menu bar, the context menus, the interface profiles with the
clean and the full variant, the performance gates.

**Definition of done.** Every action is reachable with the keyboard, the interface exists in a
clean and a full variant, and the performance targets of NFR-01 to NFR-06 are measured and met.

**Dependencies.** M1.

#### M2b — Internationalisation

**Goal.** The interface in two languages — English and German — of equal quality, with localised
formatting and a change of the language at run time.

**Why this is a milestone of its own.** The completeness of the languages is a property of the
product and not a detail of the presentation. A release that is only half translated is worse than
a release in one language, and that has to be decided at the level of the milestone and not at the
level of the translation file.

**Work packages.** The catalogue with an identical key set in both languages, the lookup chain,
the run-time change of the language, the localised formatting of numbers, sizes, dates and times,
the third language as the proof that no code change is needed, the user documentation in both
languages, the store texts in both languages.

**Definition of done.** No text is hard-coded in the interface layer, English and German have the
same key set, a missing key never yields an empty text, the language can be changed at run time
without a restart, the formats follow the system, a third language arrives without a change to the
source, and the user documentation exists in both languages.

**Dependencies.** M2.

#### M3 — The other desktop platforms

**Goal.** The same application on Windows, Linux and BSD, built from the same code.

**Work packages.** The Windows port with the long path manifest and the path length messaging, the
Linux port with the Wayland path and the X11 fallback, the FreeBSD and NetBSD builds, the
packaging for the three platforms.

**Definition of done.** The CI matrix builds and tests on all three, and the manual test protocol
for the platform-specific behaviour has run.

**Dependencies.** M2b.

#### M4 — Android

**Goal.** An Android version in the store.

**Work packages.** The Gradle project and the Rust `cdylib`, the Storage Access Framework mode
with the optional full access as a declared extension, the touch interaction context, the
adaptive layouts, the packaging and the store declaration.

**Definition of done.** The application runs on an Android device, browses the roots the user
released, and the store declaration for the full access is prepared.

**Dependencies.** M3.

#### M5 — iPadOS

**Goal.** An iPadOS version in the App Store.

**Work packages.** The Xcode project, the document picker, the security-scoped bookmarks, the
split view, the Apple keyboard support, the packaging and the store submission.

**Definition of done.** The application runs on an iPad, holds the released roots across restarts,
and handles a revoked permission without failing.

**Dependencies.** M4.

#### M6 — Remote sources

**Goal.** Remote sources behave exactly like local directories.

**Work packages.** The connection manager, the SFTP, FTP, SMB and WebDAV adapters, the WebDAV
client written for this project, the keyring integration, the hardening of the network attack
surface with the negative tests H-07 to H-11, the synchronisation, the content search.

**Definition of done.** A copy from an SFTP server to a local directory works with the same
commands as a local copy, and the network hardening rules have their negative tests.

**Dependencies.** M5.

#### M7 — Cloud and controllers

**Goal.** Cloud access without a server of our own, and operation with a controller.

**Work packages.** The OAuth 2.0 flow against the providers, the S3 adapter, the `controller`
interaction context on Android, the controller binding on the desktop, the connection manager
extensions.

**Definition of done.** A user signs in to a provider, browses the account and copies a file, and
no request passes through a server of ours.

**Dependencies.** M6.

#### M8 — Television

**Goal.** A Fire TV and Google TV version that passes the store review.

**Work packages.** The `dpad` interaction context, the action palette in its D-pad operation, the
10-foot layout, the Leanback manifest, the reachability protocol, the store submission.

**Definition of done.** Every action that the keyboard can reach is reachable with the D-pad,
proven by the reachability protocol, and the store review passes.

**Dependencies.** M7.

#### M9 — The remaining capabilities

**Goal.** The extensions of the functional scope.

**Work packages.** Viewer stage 3 with audio and video, the text editor, the tags, the macros.

**Definition of done.** The features work, are documented in both languages, and the acceptance
document carries their evidence.

**Dependencies.** M8.

### 11.3 The risk register

| Risk | Likelihood | Impact | Countermeasure | Owner |
|---|---|---|---|---|
| The file system abstraction turns out to be too narrow | medium | high | ADR-0002 decides the interface before M1; the network sources of M6 are played through in M1 as a test implementation | Architect |
| The action palette becomes too powerful and unreadable | medium | medium | The palette shows only context-reachable actions; the filter rule is fixed in M1 and checked in M8 | UI Engineer |
| The television context fails the store review | medium | high | M8 is a milestone with its own protocol, not a late addition to M4; the reachability list is kept from M1 | Product |
| A dependency carries an unfavourable licence | low | medium | `deny.toml` enforces the licence check; a copyleft dependency fails the gate | Security |
| The cross-compilation fails on system libraries | high | high | `smb2` and `russh` are pure Rust; libraries with a C component are excluded | Architect |
| The iPadOS sandbox limits the scope more than expected | high | medium | The limitation is in the specification and is stated in the M5 document rather than discovered by users | Product |
| The file length rule slows the work down | medium | low | The 400-line limit forces clean separation; that is its purpose, not an obstacle | Architect |
| The build fails in the continuous integration and not locally | medium | medium | The continuous integration build is the gate; locally only development happens | Quality |
| The localisation falls behind the feature work | medium | medium | The key parity gate fails the build, so an incomplete translation cannot merge | Product |

The highest risk is the file system abstraction, and it is the only one rated high on both axes.
It is therefore decided in an ADR before the first line of code and tested in M1 against a test
implementation of a remote source, rather than surfacing in M6.

### 11.4 The rule for going back

When a milestone reveals that a fundamental assumption does not hold, the symptom is not patched.
The affected milestone is stopped, the assumption is corrected in an ADR or in the specification,
and the affected part is redone. This costs days and saves a feature that rests on a false
assumption.

---

## 12. Organisational Plan

### 12.1 The roles

| Role | Owns | Definition of done |
|---|---|---|
| **Product Manager** | Requirement identifiers, scope decisions, acceptance criteria, the configuration schema | The requirement catalogue has no gaps and no duplicates; the acceptance criteria are measurable |
| **Rust Architect** | Module boundaries, the public API, the ADRs, the workspace integrity | The boundary check is green, the ADRs are written, clippy and format are clean |
| **Core Engineer** | The model, the job queue, the rename engine, the archive adapters | Every public function has a test, no stubs remain |
| **UI Engineer** | The panels, the keyboard operation, the theme, the native menus, the layouts per context | The application is operable without a pointer, the contrast is measured |
| **Security and Privacy** | The threat model, the hardening of the extraction, the permissions, the privacy statements | The negative tests prove every hardening, `SECURITY.md` and `PRIVACY.md` exist |
| **Quality Engineer** | The test coverage, the build gate, the acceptance evidence, the performance measurements | The gate is proven effective, the acceptance documents contain real output |
| **Documentation Lead** | The user documentation, the glossary, `AGENTS.md`, the governance documents | No dead link, the glossary is complete, both languages are covered |

### 12.2 The flow of work

```
Requirement      the catalogue states the identifiers and the acceptance criteria
     │
     ▼
Decision         an ADR for every decision that is not obvious
     │
     ▼
Implementation   the module owner writes the code and the colocated tests
     │
     ▼
Verification     an independent verifier re-runs the commands and attacks the result
     │
     ▼
Acceptance       the milestone owner writes the acceptance document with the evidence
     │
     ▼
Release          the push to main, the tag, the release notes
```

The separation between implementation and verification is the point of the arrangement. The
verifier does not read the implementer's summary and conclude from it; the verifier re-runs the
commands, attacks the result, and returns the exact gap if something is missing.

### 12.3 The rules that bind everyone

1. Nothing is claimed that has not been measured. The evidence is the command output.
2. No stubs, no dummy returns, no `todo!()` presented as a result.
3. No test is skipped or commented out to make a build green.
4. No commit, tag or release is produced by a worker. The team lead commits and merges.
5. The repository content stays in English; the handover document to the owner is written in
   German.
6. The specification belongs to the owner. A worker proposes a change, the owner decides.
7. A question without an answer is asked. It is never guessed.

### 12.4 The work plan for a milestone

Each milestone is planned as a table of work packages with an owner, a definition of done and a
verification method. The plan is written before the work begins, and it is the same table that
the acceptance document later refers to.

A work package is small enough to be finished within one session. A package that cannot be
finished is split, because an unfinished package cannot be reviewed.

### 12.5 The agent operating model

The work is carried out by AI agents with defined roles. Two findings from the practice of this
project shaped the model:

**Parallel build processes collide.** Several agents that build at the same time corrupt the
shared cargo cache, and the resulting error names a crate that does exist. Every concurrent build
therefore gets its own cargo home and its own target directory, and the lock file is committed so
that all of them resolve the same versions.

**The plan engine and the workers have different reliability.** The plan engine worked well for
the planning and the review matrix, but its workers stalled for hours in this environment without
writing a file. Long writing tasks are therefore handed to subordinate agents that receive a
complete brief, and the team lead builds and tests centrally afterwards.

The general rule that follows: **the central build is the truth.** A worker that cannot build
must not report success, and the team lead always builds and tests the integrated result
before pushing.

### 12.6 The documentation landscape

| Document | Purpose | Owner |
|---|---|---|
| `docs/REQUIREMENTS.md` | This document. The binding specification | Product Manager |
| `docs/ROADMAP.md` | The roadmap: the milestone sequence, the checklist per milestone, the tracking | Product Manager |
| `docs/ARCHITECTURE.md` | The module boundaries, the data flows, the extension points, with the diagrams of the wiring | Architect |
| `docs/adr/` | One document per decision that is not obvious. Six decisions exist: the workspace and the module boundaries, the file system abstraction, the error hierarchy, the archive extraction security, the interface profiles, the safe copy modes | Architect |
| `docs/WHERE_TO_CHANGE.md` | "Where do I change this?" for contributors | Documentation Lead |
| `docs/KEYMAP.md` | Every action, its binding, its context | Documentation Lead |
| `docs/glossary.md` | The terms with a specific meaning in this project | Documentation Lead |
| `docs/security/THREAT_MODEL.md` | The trust boundary, the hardening, the residual risks | Security |
| `docs/acceptance/` | One acceptance document per milestone | Quality Engineer |
| `docs/reviews/` | The reviews and the protocols | Quality Engineer |
| `docs/user-guide/` | The user documentation, German and English | Documentation Lead |
| `AGENTS.md` | The working rules for the agents | Documentation Lead |

Every document in this table has a named owner, and the owner is responsible for keeping it
current. A document that is not maintained is worse than a missing document, because it is
trusted.


---

## 13. Acceptance Rules and Gates

### 13.1 The structure of an acceptance document

Every milestone acceptance follows the same structure, in this order:

**Functionality.** Which functions of the milestone state exist, and how do they behave? Each line
names the requirement identifier and an observed behaviour, not a claim.

**Technique.** Which architectural decisions are implemented, which module boundaries were
respected, which tools ran through?

**Performance.** Which target values were measured, with which tool, with which result? A target
value that was not measured is reported as "not measured", never as "met".

**Guidance.** How does the milestone look, and how is it operated? The keyboard operation is
reported separately, because it is the primary path.

**Security and privacy.** Which hardenings are effective, and **which negative test proves it**? A
positive test proves nothing about a defence.

**Interoperability.** Which foreign systems were tested: archives written by other tools, network
shares from Samba, WebDAV from Nextcloud?

**Known limitations.** What does not work, and why? This list is mandatory and must not be empty
as long as there is anything to list.

**Decision.** Accepted, accepted with conditions, or rejected. The decision names the person and
the date.

### 13.2 The evidence rule

Evidence is a command output, a test name with a green result, or an extract from an acceptance
document. "It works" is not evidence. "It should pass" is not evidence. A coverage percentage
without a measurement is not evidence.

A test that could not be executed is reported as **NOT RUN with the exact reason**. A NOT RUN with
a reason is legitimate and honest. A pretended PASS is not.

### 13.3 The gates

| Gate | When | What it checks | What happens when it is red |
|---|---|---|---|
| Formatting | Every change | `cargo fmt --check` | The build stops |
| Static analysis | Every change | `cargo clippy --all-targets -- -D warnings` | The build stops |
| Licence check | Every change | `cargo deny check` | The build stops |
| Vulnerability check | Every change | `cargo audit` | The build stops |
| Module boundaries | Every change | `scripts/check-crate-boundaries.sh` | The build stops |
| File size | Every change | `scripts/check-file-size.sh` | The build stops |
| Telemetry check | Every change | `scripts/check-no-telemetry.sh` | The build stops |
| Link check | Every change | `scripts/check-docs-links.py` | The build stops |
| Test run | Every change | `cargo test --workspace` | The build stops |
| Platform build | Before a milestone is accepted | Linux, macOS, Windows, FreeBSD, NetBSD, Android, iOS | The milestone is not reached |
| Performance | Before a milestone is accepted | The target values, measured with criterion | The milestone is not reached |
| Security evidence | Before a milestone is accepted | The negative test for every hardening in scope | The milestone is not reached |
| Localisation parity | Before M2 and every later release | The key sets of the catalogues are identical | The milestone is not reached |
| Reachability | Before M8 | Every action is reachable in the D-pad context | The milestone is not reached |

No gate is weakened to make the build green. A lint is not silenced with an `#[allow]` without a
reason in an ADR. A check is not removed from the continuous integration. A test is not
deactivated.

### 13.4 The evidence index

Under `docs/acceptance/` lie the acceptance documents, and under `docs/acceptance/logs/` the
command output they refer to. The documents are numbered and dated, so that the history of the
project remains traceable. A requirement that has no acceptance criterion and no test is not
finished, however clearly it is described.

### 13.5 The requirement trace

Every requirement in section 4 carries a milestone and a verification method. The requirement
catalogue in `docs/REQUIREMENTS-IDS.md` expands this into the full matrix with the source
section, so that a requirement can be traced forwards to its acceptance criterion and backwards
to the sentence in this document that asked for it.

---

## 14. Decisions and Open Questions

### 14.1 Decisions taken

The following decisions were taken by the owner on 24 October 2026. They are binding. A departure
requires a new decision, recorded here with the date and the reason.

| ID | Question | Decision | Rationale |
|---|---|---|---|
| E1 | How strict is the platform adaptation? | egui as the shared interface, with a native menu bar, native dialogs, platform shortcuts and platform metrics | One code path, six platforms, and the pattern is already proven in the Photocraft project |
| E2 | How broad is the archive support? | ZIP, TAR with five compressions, 7z. RAR and the exotic formats are excluded | RAR requires a licence from the WinRAR owners, which is not obtained |
| E3 | How strict is the security hardening? | Strict. No `extract()`, own canonicalised path resolution, no symlinks from archive material, size limits, `cargo deny` and `cargo audit` as hard gates | The cost is paid once and it protects against the most severe error class in file managers |
| E4 | Is the macOS app sandbox used? [43][47] | No. Distribution without a sandbox, with notarisation, hardened runtime and a first-start page that explains Full Disk Access | A sandboxed file manager loses its basic function. The price is that the user grants the permission, which the application neither forces nor circumvents |
| E5 | How many viewer formats at the start? | Stage 1 complete. Text, hex, images, archive contents and metadata from M1, PDF and syntax highlighting from M2, audio and video from M9 | An honest list of ten formats that work is a better position than the claim of eighty |
| E6 | What is the order of the network milestones? [13] | SMB and WebDAV first. SFTP and FTP follow in M6 together with the connection manager | Both are used in the home network, and WebDAV covers Nextcloud, ownCloud and most NAS systems |
| E7 | How strict is the SMB hardening? [16][17][21] | Strict. SMB 1 and NTLMv1 are not implemented; signing and transport encryption are required; a server without them is refused | The alternative is a comfortable but insecure behaviour |
| E8 | How far does the Quest support go? | A `controller` interaction context on the Android version. No immersive VR port | Horizon OS is Android, so the controller operation comes without an OpenXR frontend. The limitation is stated in the documentation |
| E9 | Is the remote control operation a full version or an add-on? | A full version | Google Play checks the entire functionality with a five-way D-pad. A reduced version would also be a paid compromise of the principle "free" |
| E10 | Which interface framework? [7][8][9][10][11][12][33][34][37] | egui and eframe 0.37 | MIT OR Apache-2.0, all six targets, and the same generation is already in use in the sister project. Slint was rejected because its licence is `GPL-3.0-only OR LicenseRef-Slint-Royalty-free` and the royalty-free variant demands a visible attribution and excludes embedded use [8][9][10]; iced was rejected because mobile support is still open since April 2020 [12] |
| E11 | Which localisation system? [29][30][31] | Mozilla Fluent with `fluent`, `rust-embed` and `icu` | Native plural support, complexity in the catalogue rather than in the code, and a widely adopted system for Rust |
| E12 | How is the interface adapted to the user's taste? | Two profiles, `clean` and `full`, with per-command overrides, exportable and importable | The user chooses how much furniture they see, without a feature gate |
| E13 | Which placeholders does the rename engine use? | Both dialects. Square brackets in the dialog, curly braces in the named templates | The dialog stays simple, the templates get the expressive power, and the separation is documented |

### 14.2 Open questions for the implementation

These points do not block the start of the work, but they need an answer before the affected code
is written.

**The reference for the series name.** Whether the series name is taken from the parent
directory or entered for every series is a question of habit. The proposal: the directory name is
the default, it is always visible and can be overridden. The decision is made when real series
are processed the first time.

**The scope of the action palette.** Whether the palette shows only the actions that work in the
current context, or all actions with a note on the unreachable ones. The proposal: only the
working ones. A palette that constantly shows entries which do nothing is exactly the clutter the
product principles forbid.

**The size limit for a network answer.** The proposal is a fixed reading budget. The exact value
follows from the smallest sensible target platform and is measured in M6 rather than guessed.

**The placeholder dialect in the rename dialog.** Square brackets or curly braces in the dialog
itself. The proposal is square brackets for the generic dialog, because they are what the
documentation already uses and they read clearly in a compact field.

---

## 15. Source Register

[1] SpeedCommander — product page and feature overview: https://www.speedproject.de/
[2] SpeedCommander — features in detail: https://www.speedproject.de/sc/funktionen/
[3] SpeedCommander (English) — feature description: https://www.speedproject.com/sc/
[4] SpeedCommander — Wikipedia: https://de.wikipedia.org/wiki/SpeedCommander
[5] Forklift 3 review: https://smartwebsetup.com/forklift-3-review/
[6] winit — platform support: https://rust-windowing.github.io/winit/winit/index.html
[7] egui on crates.io: https://crates.io/crates/egui
[8] slint on crates.io: https://crates.io/crates/slint
[9] Slint — licensing FAQ: https://github.com/slint-ui/slint/blob/master/FAQ.md
[10] Slint — licence file: https://github.com/slint-ui/slint/blob/master/LICENSE.md
[11] Rust GUI ecosystem — decision basis: https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/20-how-to-build.md
[12] iced — mobile ticket #302, status 2026: https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/01-iced.md
[13] Nextcloud — WebDAV basic APIs (PROPFIND, PUT, MKCOL): https://docs.nextcloud.com/server/latest/developer_manual/client_apis/WebDAV/basic.html
[14] Android TV — navigation with the D-pad: https://developer.android.com/training/tv/get-started/navigation
[15] Android TV — TV application quality requirements (TV-DP, TV-DM, TV-DB, TV-ML, TV-MT): https://developer.android.com/develop/adaptive-apps/quality-guidelines/tv-app-quality
[16] TrueNAS — SMB service, NTLMv1 removal by Microsoft: https://www.truenas.com/docs/scale/26/systemsettings/services/smbservice/
[17] Microsoft — SMB interception defence, SMB 1 removal, signing requirement: https://learn.microsoft.com/en-us/windows-server/storage/file-server/smb-interception-defense
[18] Meta — Horizon OS basics (Android based): https://developers.meta.com/vr/essentials/quick-start/
[19] Meta — platforms and tools (flat panels, gesture mapping): https://developers.meta.com/vr/discover/platforms/
[20] NIST SP 800-88 Rev. 2 — Guidelines for Media Sanitization: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-88r2.pdf
[21] Berkeley — secure deletion guideline: https://security.berkeley.edu/secure-deletion-guideline
[22] RUSTSEC-2026-0369 — zip: Unix permissions through symlinks: https://rustsec.org/advisories/RUSTSEC-2026-0369.html
[23] RUSTSEC-2026-0367 — zip: preservation of SUID/SGID bits: https://rustsec.org/advisories/RUSTSEC-2026-0367.html
[24] RUSTSEC-2026-0368 — zip: symlinks outside the destination directory: https://rustsec.org/advisories/RUSTSEC-2026-0368.html
[25] RUSTSEC-2025-0168 — zip: incorrect path canonicalisation: https://rustsec.org/advisories/RUSTSEC-2025-0168.html
[26] crates.io — package register (versions and licences of the named crates): https://crates.io/
[27] `zip` crate — `ZipFile::enclosed_name`: https://docs.rs/zip/latest/zip/read/struct.ZipFile.html
[28] openxr — Rust bindings for OpenXR: https://crates.io/crates/openxr
[29] fluent — Rust localisation runtime: https://crates.io/crates/fluent
[30] icu — Unicode locale data, number and date formatting: https://crates.io/crates/icu
[31] rust-i18n — comparison of the localisation approaches: https://better-i18n.com/en/blog/rust-i18n-guide/
[32] muda — native menus for Windows, macOS, Linux and BSD: https://crates.io/crates/muda
[33] Photocraft — workspace manifest: https://github.com/storytold/photocraft/blob/main/Cargo.toml
[34] Photocraft — abstraction of the native menu: https://github.com/storytold/photocraft/blob/v0.12.0/crates/ui-egui/src/native_menu.rs
[35] Microsoft — maximum path length limitation: https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation
[36] Double Commander — key binding as the quasi-standard reference: https://github.com/networm/doublecmd/blob/main/shortcuts.scf
[37] Rust GUI ecosystem — Tauri in detail: https://github.com/GoldStrikeArch/rust-gui-desktop-ecosystem-state/blob/main/report/04-tauri.md
[38] The best macOS file managers of 2026: https://www.compsmag.com/best/mac-file-managers/
[39] winit — platform and input support matrix: https://docs.rs/crate/winit/latest/source/FEATURES.md
[40] Android — manage all files: https://developer.android.com/training/data-storage/manage-all-files
[41] Android — scoped storage: https://source.android.com/docs/core/storage/scoped
[42] Google Play — policy on `MANAGE_EXTERNAL_STORAGE`: https://support.google.com/googleplay/android-developer/answer/10467955?hl=en
[43] Apple — access to directories (UIDocumentPickerViewController): https://developer.apple.com/documentation/uikit/providing-access-to-directories
[44] Meta — Horizon OS quick start: https://developers.meta.com/vr/essentials/quick-start/
[45] Meta — platforms and tools: https://developers.meta.com/vr/discover/platforms/
[46] openxr — Rust bindings, licence: https://crates.io/crates/openxr
[47] Apple — file access from the macOS app sandbox: https://developer.apple.com/tutorials/data/documentation/security/accessing-files-from-the-macos-app-sandbox.md
[48] RustSec advisory database: https://rustsec.org/advisories/
[49] SpeedCommander — activation on a USB stick, the portable edition: https://www.speedproject.com/help/activation-on-usb-stick/
[50] Gizmodo — SpeedCommander, the portable build for technicians: https://gizmodo.com/download/speedcommander
[52] Microsoft — Windows Defender Application Control: https://learn.microsoft.com/en-us/windows/security/application-security/application-control/windows-defender-application-control/wdac
[53] Microsoft — AppLocker, the rules and the enforcement modes: https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/applocker/administer-applocker
[54] Microsoft — App Control for Business, the overview: https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/appcontrol
[55] Palo Alto Networks — Endpoint DLP, how it monitors the file operations of an endpoint: https://docs.paloaltonetworks.com/enterprise-dlp/administration/configure-enterprise-dlp/endpoint-dlp/how-does-endpoint-dlp-work
[56] Palo Alto Networks — Enterprise DLP, the data in use and the data at rest: https://www.paloaltonetworks.com/sase/enterprise-data-loss-prevention
[57] Australian Cyber Security Centre — application control, the accepted rule types: https://learn.microsoft.com/en-us/compliance/anz/e8-app-control
