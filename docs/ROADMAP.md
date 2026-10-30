# Phocommander — Roadmap

**Status:** Living document · **Owner:** Product Manager · **Last updated:** 27.10.2026
**Binding basis:** [`REQUIREMENTS.md`](REQUIREMENTS.md) version 2.0

This document is the **roadmap and the tracking** of the project. It says which milestone comes
next, what has to be in it, and which checkpoints have to hold before it counts as reached.

The separation to the other documents is strict:

| Question | Document |
|---|---|
| What must the product be able to do? | [`REQUIREMENTS.md`](REQUIREMENTS.md) — the requirements FR-xx and NFR-xx |
| How is every detail verified? | [`ACCEPTANCE-CRITERIA.md`](ACCEPTANCE-CRITERIA.md) — 121 acceptance criteria with "How to Verify" and "Pass Condition" |
| Which requirement belongs to which milestone? | [`REQUIREMENTS-IDS.md`](REQUIREMENTS-IDS.md) — the catalogue with 154 rows |
| **How do I know where I stand?** | **this document** |
| Was a milestone accepted? | [`acceptance/`](acceptance/) — one document per milestone with the evidence |

## How this roadmap is read

Every milestone has a **checklist**. The checklist is the binding condition: a milestone counts as
reached only when every point is fulfilled **and evidenced**. A point without evidence is not
fulfilled.

The three columns of the checklist are:

| Column | Meaning |
|---|---|
| **Point** | What has to hold |
| **Evidence** | Where the evidence comes from — a test, a command, a document |
| **Status** | `open`, `in work`, `fulfilled`, `NOT RUN` (with the reason) |

The status column is maintained by hand and updated at the end of every milestone. It is the only
place where the current state stands.

## The rules that hold for every milestone

These points hold in **every** milestone, without exception. They are not negotiable and they are
checked at every acceptance.

| No. | Point | Evidence |
|---|---|---|
| R-01 | The workspace builds without an error | `cargo build --workspace --all-targets --locked` |
| R-02 | All tests run through, none is skipped | `cargo test --workspace --locked` |
| R-03 | The formatting is clean | `cargo fmt --all --check` |
| R-04 | The static analysis is without a warning | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| R-05 | No dependency with a copyleft licence | `cargo deny check` |
| R-06 | No known vulnerability in the list of dependencies | `cargo audit` |
| R-07 | The direction of the dependencies is kept | `scripts/check-crate-boundaries.sh` |
| R-08 | No source file above 600 lines (target value) respectively 900 (maximum) | `scripts/check-file-size.sh` |
| R-09 | No telemetry or analytics dependency | `scripts/check-no-telemetry.sh` |
| R-10 | No dead link in the documentation | `scripts/check-docs-links.py` |
| R-11 | The requirement identifiers of the specification and of the catalogue are identical | `scripts/check-requirement-ids.py` |
| R-12 | The action identifiers of the keymap, of the code and of the documentation are identical | `scripts/check-action-ids.py` and `scripts/check-keymap-consistency.py` |
| R-13 | The shipped configuration agrees with the schema | `scripts/check-config-schema.py` |
| R-14 | The interface profiles keep their values | `scripts/check-interface-profiles.py` |
| R-15 | Every gate proves its effect: red on a break, green after the repair | `scripts/check-gates-negative.sh` |
| R-16 | Every public function has a test | The coverage matrix of the acceptance |
| R-17 | Every hardening rule has a negative test | `tests/security/` |
| R-18 | No `unwrap()` and no `expect()` in the production code, no `unsafe` | The lints of the workspace plus Clippy |
| R-19 | Every module carries a doc comment in the WHAT/WHY/EXAMPLE style | The `missing_docs` lint |
| R-20 | The acceptance of the milestone lies under `docs/acceptance/` | The file exists, the decision is documented |
| R-21 | The state lies on `main` in GitHub | A commit on `main` |
| R-22 | The documentation of the project is English — the user guide, the glossary and the archive of the handovers are the exceptions | `scripts/check-docs-language.py` |

---

## The overview

| Milestone | Content | State |
|---|---|---|
| **M0** — Foundation | Workspace, CI, configuration, threat model, documentation | **accepted** |
| **M1** — The working file browser | File system model, job queue, rename, viewer stage 1, archives | **in review** — needs the manual protocol on a real macOS machine |
| **M2** — Maturity on macOS | Tabs, profiles, safe copy modes, layouts per context | **accepted** |
| **M2b** — Internationalisation | English and German complete, localised formatting, change of the language at run time | **in work** — the third language and the shop files are open |
| **M3** — The other desktop platforms | Windows, Linux, BSD | **accepted with two open points** — the manual protocol and the signing |
| **M4** — Android | SAF, touch context, store | **in work** — the seam of the input stands |
| **M5** — iPadOS | Document picker, bookmarks | open |
| **M6** — Remote sources | SFTP, FTP, SMB, WebDAV | open |
| **M7** — Cloud and controllers | OAuth, S3, controller context | open |
| **M8** — Television | D-pad, action palette, ten feet | open |
| **M9** — The remaining capabilities | Viewer stage 3, editor, tags, macros | open |

---

## M0 — Foundation

**Goal.** A repository that builds, checks and tests itself on three operating systems, with the
specification as the binding basis for everything that follows.

| Point | Evidence | Status |
|---|---|---|
| Cargo workspace with all crates, pinned dependencies, committed lock file | `Cargo.toml`, `Cargo.lock` | fulfilled |
| The twelve common rules R-01 to R-22 | all gates | fulfilled |
| The five CI jobs: quality, desktop matrix, cross compilation, performance, nightly run | `.github/workflows/ci.yml` | fulfilled |
| The versions of the tools are pinned and verified in the workflow | `ci.yml`, `docs/REQUIREMENTS.md` 5.8 | fulfilled |
| `deny.toml` allows exclusively permissive licences, no GPL and no AGPL | `deny.toml` | fulfilled |
| The configuration file with the defaults, the key bindings, the rename templates | `config/` | fulfilled |
| The threat model with the residual risk and the naming of the TOCTOU gap | `docs/security/THREAT_MODEL.md` | fulfilled |
| The negative tests for the zip slip, the symlink escape, the SUID/SGID bits and the decompression bomb | `tests/security/` | fulfilled |
| The acceptance with the evidence under `docs/acceptance/logs/` | `docs/acceptance/M0-ACCEPTANCE.md` | fulfilled |

**Result.** 129 tests green, no test skipped, the conditions of the acceptance closed.

---

## M1 — The working file browser

**Goal.** A functional two-pane file browser on macOS that copies, moves, renames, views and opens
archives.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | in work — measured again at the end |
| The file system model with `Entry`, `EntryKind`, `Source`, `MutableSource` | `phocommander-core` | fulfilled |
| The job queue with progress, rate, cancellation and the collection of the failures | `phocommander-ops` | fulfilled |
| The rename engine with the placeholders, the replacing and the counter | `phocommander-ops` | in work — the tests are measured again |
| The series template: the detection, the season with two and three digits, the Windows characters | `config/templates/series.toml` | fulfilled — with a test for the two-digit season |
| The jump list of the conflicts with F3 and Shift+F3 | `phocommander-ops` | fulfilled |
| The viewer stage 1: text, hex, images, archive contents, metadata | `phocommander-viewer` | fulfilled |
| The archive adapters for ZIP, TAR with five compressions and 7z | `phocommander-archive` | fulfilled |
| The hardened extraction: canonicalised paths, no symlinks from archive material, no SUID/SGID bits | `phocommander-archive`, `tests/security/` | fulfilled |
| The filtering of the current directory with the pattern language and the live count of the hits | `phocommander-ui` | fulfilled |
| The determinism invariant of the rename, checked with a property test | `phocommander-ops` | fulfilled |
| The property gate for the frame: 10,000 and 100,000 entries | `crates/phocommander-ui/tests/` | fulfilled |
| The eighteen common rules of the security check, each with a negative test | `tests/security/` | fulfilled |

**Open.** The acceptance of M1 exists, but it was measured while the M2 agents worked in parallel
and therefore says **"not accepted"**. It is rewritten on the quiet tree once M2 is done.

---

## M2 — Maturity on macOS

**Goal.** An application that can replace a file browser in everyday use, with an interface that
adapts itself to the context. The languages follow in **M2b**.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | **fulfilled** — 924 tests, 34 gates, all green |
| The tabs with a state of their own per tab, movable between the panels | `phocommander-ui` | **fulfilled** |
| The switch of the arrangement without a restart | `phocommander-ui` | **fulfilled** |
| The directory tree per panel, without a recursive read | `phocommander-ui` | **fulfilled** |
| The quick view of the marked entry | `phocommander-ui` | **fulfilled** |
| The structurally different layouts per context | `phocommander-ui` | **fulfilled** |
| The interface is tidy: the same distances, one dividing line, a visible focus | `phocommander-ui` | **fulfilled** |
| The atomic copy mode: temporary file, check, rename, always clean up | `phocommander-ops` | **fulfilled** |
| The synchronisation plan with the state per entry, without a plan no movement | `phocommander-ops` | **fulfilled** |
| The atomic rename: all files or none | `phocommander-ops` | **fulfilled** |
| The retry of the failed entries only | `phocommander-ops` | **fulfilled** |
| The comparison of directories by content, date and size | `phocommander-ops` | **fulfilled** |
| The undo, with the visible limitation | `phocommander-ops` | **fulfilled** |
| The check of the archive without unpacking it | `phocommander-archive` | **fulfilled** |
| The interface profiles `clean` and `full` are effective, not only described | `config/profiles/` | **fulfilled** |
| Every action switchable on its own: `visible`, `palette`, `hidden` | `phocommander-ui` | **fulfilled** |
| A profile contains nothing but decisions of the surface | `scripts/check-interface-profiles.py` | **fulfilled** |
| The about dialog names the number of the formats that are really supported | `phocommander-viewer` | open |
| The performance goals NFR-01 to NFR-08 are measured | `benches/`, `docs/acceptance/logs/` | open |

## M2b — Internationalisation

**Goal.** The interface in **two languages** — English and German — with the same quality, with
localised formatting and without a single hardcoded string in the source.

This milestone is a **state of its own** and not a part of M2, because the completeness of the
languages is a property of the product that has to be accepted separately and kept separately. A
release that is only half translated is worse than a release in one language.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | open — measured at the end |
| **No hardcoded text in the interface** — every visible word goes through a key | `scripts/check-i18n-hardcoded.py` | fulfilled |
| English and German are complete, with an **identical set of keys** | `scripts/check-i18n-keys.py` | fulfilled — 158 keys in three languages |
| The chain of the language lookup: requested → language chain → English → **the key itself** | `phocommander-i18n`, a test per level | fulfilled |
| A missing key yields **never** an empty text | Test | fulfilled |
| The language is switchable at run time, without a restart; the interface updates itself at once | Test | **fulfilled** — the actions `app.language.de`, `app.language.en` and `app.language.fr` change it while the program runs, and the choice is written into the configuration file |
| Numbers, sizes, dates and times follow the system of the language | `phocommander-i18n`, `icu` | fulfilled |
| **A third language comes without a change of the code** — only a directory of catalogues | `locales/fr/` | fulfilled |
| The interface language and the language of the file names are independent | Test | open |
| The language files are separated by topic, so that a translator does not read the whole code | `locales/<language>/*.ftl` | fulfilled |
| The user documentation exists in both languages | `docs/user-guide/` | fulfilled |
| The screenshot and text files of the stores are at hand in both languages | `docs/guides/` | open |

**Result.** An interface that is equivalent in German and in English, in which no text is
hardcoded and in which a new language needs a single file.


---

## M3 — The other desktop platforms

**Goal.** The same application on Windows, Linux and BSD, out of the same code base.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | **fulfilled** — 1277 tests, 38 gates |
| The Windows port with the `longPathAware` manifest and the plain-language message on the path length | `apps/`, `docs/` | **fulfilled** — the first Windows build is the proof |
| The Linux port with the Wayland path and the fallback to X11 | `apps/` | **fulfilled** — winit decides, the decision is visible and tested |
| The FreeBSD and NetBSD builds | The CI matrix | **fulfilled** — a job of its own with the cross toolchain |
| The packaging for the three platforms | `docs/guides/` | **fulfilled** — MSIX, portable, Flatpak, AppImage, deb/rpm/PKGBUILD/APKBUILD, Ports and pkgsrc |
| The manual test protocol for the platform-specific behaviour | `docs/acceptance/` | **open** — needs a Windows and a BSD machine |
| The first real Windows build, the release signing and the notarisation | `docs/acceptance/M3-ACCEPTANCE.md` | **open** — the certificates come from the owner |

---

## M4 — Android

**Goal.** An Android version that can go into the store.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | open — 1744 tests, 15 consistency gates, 42 of 42 negative runs green |
| The seam of the input: keyboard, pointer, finger, remote control and gamepad bind to the same actions | `phocommander-ui`, `config/keymap.toml` | **fulfilled** |
| The default bindings in the code, so that the file of the user only overrides them | `keymap_view/defaults.rs` | **fulfilled** |
| The Gradle project and the Rust `cdylib` | `apps/phocommander-android/` | open |
| The SAF mode without the special permission | `apps/` | open |
| The optional full access, declared in the Play Console and explained | `apps/`, `docs/` | open |
| The touch context with the stacked panels and the 44-point targets | `phocommander-ui` | partially — the events and the bindings stand, the arrangement of the controls is missing |
| The explanation in the first start, why the permission is needed | `apps/` | open |

---

## M5 — iPadOS

**Goal.** An iPadOS version for the App Store.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | open |
| The Xcode project and the Rust `cdylib` | `apps/phocommander-ios/` | open |
| The document picker with the granted roots | `apps/` | open |
| The security-scoped bookmarks that survive a restart | `apps/` | open |
| The split view instead of the pair of windows | `phocommander-ui` | open |
| The handling of a revoked permission, without a crash | `apps/`, test | open |
| The limitation is stated in the document, not hidden | `docs/` | open |

---

## M6 — Remote sources

**Goal.** Remote sources behave like local directories.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | open |
| SFTP, FTP, SMB and WebDAV behind the same interface | `phocommander-remote` | open |
| The WebDAV client written by the project, without a copyleft dependency | `phocommander-remote` | open |
| SMB 1 and NTLMv1 are not implemented | `phocommander-remote`, negative test | open |
| SMB signing and SMB 3 transport encryption are required where the server can do them | `phocommander-remote`, integration test | open |
| Every network operation has a time limit | `phocommander-remote`, test | open |
| The size of the answer is bounded independently of the announced size, by a reading budget that the user sets | `phocommander-remote`, test | open |
| The credentials lie in the key ring | `phocommander-remote` | open |
| The protocols name no user, no server and no IP address | `phocommander-remote`, test | open |
| The synchronisation and the search in the content | `phocommander-ops` | open |

---

## M7 — Cloud and controllers

**Goal.** Cloud access without a server of ours, and the operation with a controller.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | open |
| The OAuth 2.0 sign-in directly at the provider, without a detour | `phocommander-remote` | open |
| S3-compatible targets | `phocommander-remote` | open |
| No server of ours, no data processing of our own | Architecture review | open |
| The `controller` context on the Android version | `phocommander-ui` | open |
| The connection of the controller on the desktop | `phocommander-ui` | open |

---

## M8 — Television

**Goal.** A Fire TV and Google TV version that passes the review of the store.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | open |
| The `dpad` context with the action palette | `phocommander-ui` | open |
| One panel visible, the second one as an overlay | `phocommander-ui` | open |
| The ten-foot layout with a large type and a strong focus frame | `phocommander-ui` | open |
| The Leanback manifest with the banner and without the demand of a touch screen | `apps/` | open |
| **Every action is reachable in the `dpad` context** — checked by the reachability protocol | `docs/acceptance/` | open |
| The back button goes back and leaves the application without a loop | Test | open |
| The application does not depend on a menu key of the remote control | Test | open |

---

## M9 — The remaining capabilities

**Goal.** The extensions of the scope of functions.

| Point | Evidence | Status |
|---|---|---|
| The twelve common rules R-01 to R-22 | all gates | open |
| The viewer stage 3: audio, video, metadata, still frame | `phocommander-viewer` | open |
| The text editor with the highlighting of the syntax | `phocommander-viewer` | open |
| The tags on files and directories | `phocommander-viewer` | open |
| The macros | `phocommander-ops` | open |
| Everything is documented in both languages | `docs/user-guide/` | open |

### Which milestone triggers which release

| Milestone completed | Release | The platforms that ship with this release |
|---|---|---|
| M1 | **Alpha 1** (0.1.0-alpha.1) | macOS (Apple Silicon, Intel) |
| M2 | **Alpha 2** (0.1.0-alpha.2) | macOS — the interface ripens, the platform stays |
| M2b | **Beta 1** (0.2.0-beta.1) | macOS — **both languages complete** |
| M3 | **Beta 2** (0.2.0-beta.2) | in addition Linux (x86_64, aarch64), Windows (with the packages and the portable variant), FreeBSD and NetBSD |
| M4 and M5 | **Beta 3** (0.3.0-beta.3) | in addition Android and iPadOS from the stores |
| M6 and M7 | **Beta 4** (0.3.0-beta.4) | in addition the network sources, the cloud access, the controller context |
| M8 | **Beta 5** (0.3.0-beta.5) | in addition Fire TV and Google TV |
| M9 | **Release 1.0** (1.0.0) | all six platforms, all the ways above |

**A milestone triggers a release, but a release is not automatically due.** The checklist "What
triggers a release" with the conditions T-01 to T-15 applies in addition. If one of them is
missing, the release is **postponed** and not shipped incomplete.

---

## The releases

A **release** is not the same as a milestone. The milestone is the work, the release is the
**shipped binary** for certain platforms. Both belong together, but they are not identical: a
milestone can be completed without a release being shipped — for example because the signing
certificates do not exist yet.

The sequence of the versions follows the proven rule: `0.x` is the work on the product, `1.0` is the
release that somebody uses in everyday life.

| Release | Version | Trigger | Platforms | What is shipped |
|---|---|---|---|---|
| **Alpha 1** | 0.1.0-alpha.1 | The end of M1 | macOS (Apple Silicon, Intel) | The first runnable application: browse, copy, rename, view, open archives. Expressly **alpha**: errors are to be expected |
| **Alpha 2** | 0.1.0-alpha.2 | The end of M2 | macOS | Tabs, profiles, safe copy modes, the interface in both layouts |
| **Beta 1** | 0.2.0-beta.1 | The end of M2b | macOS | **Both languages complete**, the interface is usable on a real day, no data loss |
| **Beta 2** | 0.2.0-beta.2 | The end of M3 | macOS, Linux (x86_64, aarch64), Windows, FreeBSD, NetBSD | All desktop platforms, the Windows packages, the portable variant |
| **Beta 3** | 0.3.0-beta.3 | The end of M4 and M5 | in addition Android and iPadOS | The mobile versions from the stores |
| **Beta 4** | 0.3.0-beta.4 | The end of M6 and M7 | in addition all of the above | The network sources and the cloud access, the controller context |
| **Beta 5** | 0.3.0-beta.5 | The end of M8 | in addition Fire TV and Google TV | The television version |
| **Release 1.0** | 1.0.0 | The end of M9 | all six platforms | The first release that somebody uses in everyday life. No alpha, no beta |

### What triggers a release

A release is **not** triggered by the end of a milestone, but by a checklist. The checklist is the
condition, and it is worked through completely at every release.

| No. | Condition | Evidence |
|---|---|---|
| T-01 | The milestone that belongs to it is accepted | `docs/acceptance/` |
| T-02 | The twelve common rules R-01 to R-22 are fulfilled | all gates |
| T-03 | All tests run through, none is skipped | `cargo test --workspace --locked` |
| T-04 | **The version is written in exactly one place** and is identical everywhere | `Cargo.toml`, `CHANGELOG.md`, all shells |
| T-05 | The version follows the rule `MAJOR.MINOR.PATCH` with `0.x` before `1.0` | `Cargo.toml` |
| T-06 | The checksums of the artefacts are computed and filed with the release | `SHA256SUMS` |
| T-07 | Every artefact is **signed** | The signature of the manufacturer |
| T-08 | On macOS the artefact is **notarised** and the ticket is attached | `notarytool`, ticket |
| T-09 | On Windows the artefact is **signed with Authenticode** | The signature of the manufacturer |
| T-10 | At the stores the submission is accepted, the review has passed | The confirmation of the store |
| T-11 | The release notes name the changes, the known limitations and the triggers | `CHANGELOG.md` |
| T-12 | The documentation describes the state of this release | `docs/user-guide/` |
| T-13 | The security rules are fulfilled, including the three guarantees that cannot be switched off | `docs/security/` |
| T-14 | The binaries are **built** on the target platform, not only on one | The artefacts of the release |
| T-15 | The artefacts carry **no** telemetry, **no** advertising and **no** demand of an account | The check script plus the visual check |

### The platforms and their ways

Every platform has its own way into the hands of the user. That decides what has to be ready
before the release.

| Platform | Way | What has to be ready before | Signature |
|---|---|---|---|
| **macOS** | Over the own website, as `.dmg` | Notarisation with the ticket, hardened runtime, the signature of the Developer ID | Apple Developer ID, notarised |
| **Windows** | Over the own website, as `.msix` and as the portable variant | The Authenticode signature; the portable variant is **one** executable without an installer | An own code-signing certificate |
| **Linux** | Over the own website, as Flatpak, AppImage and as the package of the distributions | No signature needed, but reproducible builds | None, integrity through the checksums |
| **FreeBSD, NetBSD** | Over the own website, as a package | A port package, if a port exists | None |
| **Android** | Over the Play Store | The declaration of the special permission, the privacy statement, the age rating | Play App Signing |
| **iPadOS** | Over the App Store | Privacy statement, age rating, the screenshots in both languages | Apple Developer |
| **Meta Quest** | Over the Meta Horizon Store, or as an APK from the website | The app runs as a flat panel, no immersive VR port | Meta Store |
| **Fire TV, Google TV** | Over the Play Store | The Leanback manifest, the reachability with the D-pad, the banner | Play App Signing |

### Why Linux and Windows come with Beta 2

**Decision of 25.10.2026.** The release plan originally provided for Linux with Alpha 2 and
Windows with Beta 1. That contradicts the milestone plan: the **Linux port with the Wayland
path** and the **Windows port with the manifest** are parts of **M3**. An interface that runs on
Linux without the display server, or on Windows without the path support, is not a release but
an error message with a screen.

Therefore the following holds:

- **Alpha 1 and Alpha 2** ship **macOS**. They are the ripeness of the interface, not the reach.
- **Beta 1** ships **macOS**, with both languages complete. That is the promise to the user: the
  interface speaks their language, and it loses no data.
- **Beta 2** ships **all five desktop platforms**, because M3 ports the three others.

### What a release is **not**

- **No alpha without an acceptance.** An alpha without the acceptance of the milestone is an
  uncontrolled delivery. The trigger is the milestone, not the date.
- **No release without a signature.** An unsigned binary causes the Gatekeeper warning on macOS
  and the SmartScreen warning on Windows. Both are, for a tool that the user installs on their
  company laptop, the moment in which the trust is lost.
- **No release with open security findings.** An open finding from the threat model blocks the
  release.
- **No release that promises more than the software can.** The scope of a release stands in the
  changelog, and the changelog names the known limitations.

### The version in everyday life

The applications show their version in the about dialog: application, version, commit and build
platform. The configuration file carries its own `schema_version` — that is the version of the
**format** of the file, not the version of the application. The two are kept apart, because they
change at different times.


---

## The order and its reasons

The milestones are linear, because each one builds on the one before. Two decisions are made that
way on purpose:

**The core comes before the interface.** Every milestone delivers first a testable layer in the
Rust core. The interface binds itself to it. An interface function without the support of the
core is not begun.

**The milestones with a network come after the local ones.** A source that does not work is
repairable and visible. A half-finished network layer that hangs in between in M1 endangers the
core.

**The internationalisation is a milestone of its own.** It stands between M2 and M3, because the
completeness of the languages is a property of the product that has to be accepted separately.
Before M2b the interface may exist in one language only — for the development that is right, and
for a release it is wrong. The Beta 1 point of the roadmap therefore requires the completeness of
both languages, and a release that is only half translated is none.

## The decisions on the open points of the implementation

The owner has decided these points. They stand in the specification in section 14.2 "The
decisions on the open points of the implementation", and they are recorded here so that the plan
and the code cannot drift apart.

| Question | Decision | Affects |
|---|---|---|
| The reference for the series name | The directory name is the default, it is always visible and it can be overridden | M1, M2 |
| The scope of the action palette | Only the actions that are reachable in the current context are shown | M8 |
| The size limit for a network answer | A reading budget that the user sets in the settings; the default is 64 MiB, zero turns the limit off | M6 |
| The placeholder dialect in the rename dialog | Square brackets for the dialog, curly braces for the named templates | M1, M2 |
