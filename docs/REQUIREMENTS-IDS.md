# Requirement catalogue — Phocommander

This document is the index into [`docs/REQUIREMENTS.md`](REQUIREMENTS.md), the binding specification
and into [`docs/ROADMAP.md`](ROADMAP.md), the roadmap that tracks the state of every milestone. Version 2.0 of the specification replaced version 1.1; this catalogue was rebuilt from it and carries exactly the requirement identifiers of section 4 of that document.

Every row names the section **and the line** of the specification it was taken from, so a reviewer can check a row against the specification without searching for it.

## How to read the catalogue

| Column | Meaning |
|---|---|
| ID | `FR-xx` for a functional requirement, `NFR-xx` for a non-functional requirement. The identifier is the one of the specification and is never reused. |
| Title | One line that states the requirement, not the solution. |
| Source | The section of `docs/REQUIREMENTS.md` and the line in it, as `§<section> <title>, line <number>`. |
| Milestone | The milestone that delivers the requirement, `M0` to `M9`. It is the value of the specification. |
| Priority | `Must`, `Should` or `Could`, as the scale of section 4 of the specification defines it. |
| Verification | The verification method of the specification, prefixed with the kind of the check: `automated`, `manual` or `mixed`. |
| Acceptance criterion | The identifier of the acceptance criterion in [`ACCEPTANCE-CRITERIA.md`](ACCEPTANCE-CRITERIA.md), or `not yet assigned` for a milestone that has no criteria yet. |

## Rules for this catalogue

1. An identifier is assigned exactly once, is never reused for a different meaning and is never moved to another milestone without a note in the change log.
2. The set of identifiers is identical to the set of identifiers in section 4 of the specification. `scripts/check-requirement-ids.py` proves it and fails when it is not.
3. A requirement with a letter suffix (`FR-22a` to `FR-22q`, `FR-74a` to `FR-74g`, `NFR-09a`, `NFR-09b`, `NFR-17a` to `NFR-17c`) belongs to the block of its base identifier and carries the same base number.
4. Every requirement of `M0`, `M1` and `M2` has at least one acceptance criterion in [`ACCEPTANCE-CRITERIA.md`](ACCEPTANCE-CRITERIA.md).
5. A changed requirement keeps its identifier and gets a note in the change log at the end of this document, with the old reference named.

---

## 4.1 Product and principles

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-01 | The application is free of charge and has no paid tier | §4.1 Product and principles, line 188 | M0 | Must | mixed — Dependency check plus licence review | AC-M0-01 |
| FR-02 | The application starts without an account, a login or a network connection | §4.1 Product and principles, line 189 | M0 | Must | automated — Start-up test that asserts no socket is opened | AC-M0-02 |
| FR-03 | No telemetry, analytics, advertising or crash reporting is collected | §4.1 Product and principles, line 190 | M0 | Must | mixed — Dependency check plus code review | AC-M0-03 |
| FR-04 | The start view carries no banner, no promotional area and no onboarding overlay | §4.1 Product and principles, line 191 | M0 | Must | manual — Screenshot review in the acceptance document | AC-M0-04 |
| FR-05 | Every user-visible string is externalised and translatable | §4.1 Product and principles, line 192 | M2b | Must | automated — Gate that fails on a literal string in the UI layer | AC-M2b-01 |
| FR-06 | The product ships under MIT OR Apache-2.0 without a proprietary component | §4.1 Product and principles, line 193 | M0 | Must | mixed — `cargo deny` plus licence review | AC-M0-05 |

## 4.2 Dual-pane work surface

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-10 | Two independent directory panels are shown side by side or stacked | §4.2 Dual-pane work surface, line 199 | M1 | Must | manual — Manual test plus keyboard test | AC-M1-01 |
| FR-11 | The arrangement can be switched at run time without a restart | §4.2 Dual-pane work surface, line 200 | M2 | Should | manual — Manual test | AC-M2-01 |
| FR-12 | Each panel owns its tabs, and each tab keeps its own path, sorting and view state | §4.2 Dual-pane work surface, line 201 | M2 | Must | automated — State model unit test | AC-M2-02 |
| FR-13 | A tab can be moved from one panel to the other | §4.2 Dual-pane work surface, line 202 | M2 | Should | manual — Manual test | AC-M2-03 |
| FR-14 | A directory tree can be toggled per panel | §4.2 Dual-pane work surface, line 203 | M2 | Should | manual — Manual test | AC-M2-04 |
| FR-15 | A third area previews the marked file without opening it | §4.2 Dual-pane work surface, line 204 | M2 | Should | manual — Manual test | AC-M2-05 |
| FR-16 | Operations act on the active panel; a transfer to the other panel is always explicit | §4.2 Dual-pane work surface, line 205 | M1 | Must | automated — State model test that asserts the target panel is never implied | AC-M1-02 |
| FR-17 | The column set is configurable: name, size, date, type | §4.2 Dual-pane work surface, line 206 | M1 | Must | mixed — Configuration test plus manual test | AC-M1-03 |
| FR-18 | The sort order is configurable and is remembered per tab | §4.2 Dual-pane work surface, line 207 | M1 | Must | automated — Configuration test | AC-M1-04 |

## 4.3 File operations

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-20 | Copy, move, rename, delete and create-directory are available | §4.3 File operations, line 213 | M1 | Must | mixed — Unit test plus manual test | AC-M1-05 |
| FR-21 | Every operation runs as a background job and the interface stays responsive | §4.3 File operations, line 214 | M1 | Must | automated — Test that asserts the UI thread is not blocked | AC-M1-06 |
| FR-22 | The conflict policy is configurable: ask, never, always, keep both | §4.3 File operations, line 215 | M1 | Must | automated — Unit test per policy | AC-M1-07 |
| FR-22a | The copy mode is selectable: quick, verified, atomic, sync-mirror, sync-update, sync-two-way, sync-preview | §4.3 File operations, line 216 | M1 | Must | automated — Unit test per mode | AC-M1-08 |
| FR-22b | The default copy mode is `verified`, the default synchronisation mode is `sync-preview` | §4.3 File operations, line 217 | M1 | Must | automated — Configuration test | AC-M1-09 |
| FR-22c | The `verified` mode compares the checksums of source and destination and reports a mismatch as a job error | §4.3 File operations, line 218 | M1 | Must | automated — Unit test with a corrupted destination | AC-M1-10 |
| FR-22d | The `atomic` mode writes to a temporary file, verifies it and renames it into place | §4.3 File operations, line 219 | M2 | Must | automated — Integration test plus a failed-verification case | AC-M2-06 |
| FR-22e | `sync-mirror` is the only mode that deletes, and the deletion appears in the plan before it runs | §4.3 File operations, line 220 | M2 | Must | automated — Unit test on the plan | AC-M2-07 |
| FR-22f | Every synchronisation produces a plan with the state per entry, and no data moves before the plan is confirmed | §4.3 File operations, line 221 | M2 | Must | automated — Unit test on the plan | AC-M2-08 |
| FR-22g | The job cue shows state, progress, rate, estimate and the current item | §4.3 File operations, line 222 | M1 | Must | mixed — Manual test plus state model test | AC-M1-11 |
| FR-22h | The verification phase has its own progress and is visible as a separate state | §4.3 File operations, line 223 | M1 | Must | manual — Manual test | AC-M1-12 |
| FR-22i | A queued job can be reprioritised, retargeted, and its conflict and verification modes changed | §4.3 File operations, line 224 | M1 | Must | automated — Unit test per editable field | AC-M1-13 |
| FR-22j | A running job can be paused, resumed and cancelled, and a change takes effect at the next item boundary | §4.3 File operations, line 225 | M1 | Must | automated — Unit test with a controlled source | AC-M1-14 |
| FR-22k | The failed items of a job can be retried on their own | §4.3 File operations, line 226 | M2 | Should | automated — Unit test | AC-M2-09 |
| FR-22l | The job list can be reordered, filtered by state and inspected | §4.3 File operations, line 227 | M2 | Should | manual — Manual test | AC-M2-10 |
| FR-22m | The Windows build ships in a portable variant | §4.3 File operations, line 228 | M3 | Must | manual — Manual test on a removable drive | not yet assigned |
| FR-22n | The portable variant keeps its configuration next to the executable | §4.3 File operations, line 229 | M3 | Must | automated — Unit test of the path resolution | not yet assigned |
| FR-22o | The portable variant writes no file outside its own directory | §4.3 File operations, line 230 | M3 | Must | automated — Test that the portable directory is the only place written | not yet assigned |
| FR-22p | The installed and the portable variant share one executable | §4.3 File operations, line 231 | M3 | Must | manual — Manual test of both modes | not yet assigned |
| FR-22q | The portable variant carries no activation and no network call | §4.3 File operations, line 232 | M3 | Must | automated — Test that the start-up opens no socket | not yet assigned |
| FR-23 | Overwriting is not the default | §4.3 File operations, line 233 | M1 | Must | automated — Configuration test | AC-M1-15 |
| FR-24 | Every job reports progress with a rate and can be cancelled | §4.3 File operations, line 234 | M1 | Must | automated — Unit test on the job queue | AC-M1-16 |
| FR-25 | A job collects its errors instead of stopping at the first one | §4.3 File operations, line 235 | M1 | Should | automated — Unit test with a partially failing copy | AC-M1-17 |
| FR-26 | Undo exists for copy operations and is labelled as partial | §4.3 File operations, line 236 | M2 | Should | manual — Manual test plus documentation review | AC-M2-11 |
| FR-27 | The trash is the default path for deletion | §4.3 File operations, line 237 | M1 | Must | manual — Manual test | AC-M1-18 |
| FR-28 | The path length limit of the platform is detected and reported in plain language | §4.3 File operations, line 238 | M3 | Must | manual — Manual test on Windows [35] | not yet assigned |

## 4.4 Multiple rename

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-30 | The rename dialog offers placeholders for name, extension, parent, counter, date, time, size, short checksum, random value and case conversion | §4.4 Multiple rename, line 244 | M1 | Must | automated — Unit test per placeholder | AC-M1-19 |
| FR-31 | The dialog offers search-and-replace, optionally with regular expressions | §4.4 Multiple rename, line 245 | M1 | Must | automated — Unit test | AC-M1-20 |
| FR-32 | The dialog offers a counter with start value, step and zero-padded width | §4.4 Multiple rename, line 246 | M1 | Must | automated — Unit test | AC-M1-21 |
| FR-33 | The rename result is deterministic: the same input always yields the same output | §4.4 Multiple rename, line 247 | M1 | Must | automated — Property test | AC-M1-22 |
| FR-34 | The dialog shows a live preview of every new name before the rename runs | §4.4 Multiple rename, line 248 | M1 | Must | manual — Manual test | AC-M1-23 |
| FR-35 | Conflicts are marked and listed in a conflict jump list reachable with F3 and Shift+F3 | §4.4 Multiple rename, line 249 | M1 | Must | mixed — Unit test plus manual test | AC-M1-24 |
| FR-36 | Named templates can be stored and re-applied | §4.4 Multiple rename, line 250 | M1 | Must | automated — Configuration test | AC-M1-25 |
| FR-37 | The series episode template detects season and episode in an existing name and offers them as pre-filled values | §4.4 Multiple rename, line 251 | M1 | Must | automated — Unit test per input pattern | AC-M1-26 |
| FR-38 | The series name defaults to the parent directory, is always visible and can be overridden | §4.4 Multiple rename, line 252 | M1 | Must | mixed — Unit test plus manual test | AC-M1-27 |
| FR-39 | Season and episode numbers are zero-padded, and the width grows automatically with a three-digit number | §4.4 Multiple rename, line 253 | M1 | Must | automated — Unit test | AC-M1-28 |
| FR-40 | Characters the target platform forbids are reported before the rename and a replacement is suggested | §4.4 Multiple rename, line 254 | M1 | Must | automated — Unit test | AC-M1-29 |
| FR-41 | The rename is atomic per selection: either every file is renamed or none is | §4.4 Multiple rename, line 255 | M2 | Should | automated — Unit test with a failing case in the middle | AC-M2-12 |

## 4.5 Viewer

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-50 | The viewer opens in an additional area without leaving the application | §4.5 Viewer, line 261 | M1 | Must | manual — Manual test | AC-M1-30 |
| FR-51 | Text is shown with character set detection for UTF-8 and UTF-16 | §4.5 Viewer, line 262 | M1 | Must | automated — Unit test per encoding | AC-M1-31 |
| FR-52 | A hex dump shows offset, hex and printable ASCII | §4.5 Viewer, line 263 | M1 | Must | automated — Unit test | AC-M1-32 |
| FR-53 | Common image formats are displayed | §4.5 Viewer, line 264 | M1 | Must | automated — Unit test per format | AC-M1-33 |
| FR-54 | The contents of an archive can be browsed and a single entry extracted | §4.5 Viewer, line 265 | M1 | Must | automated — Integration test | AC-M1-34 |
| FR-55 | File metadata is shown: size, type, timestamps, checksum | §4.5 Viewer, line 266 | M1 | Must | automated — Unit test | AC-M1-35 |
| FR-56 | PDF text and page navigation are supported | §4.5 Viewer, line 267 | M2 | Should | manual — Manual test | AC-M2-13 |
| FR-57 | Syntax highlighting is supported | §4.5 Viewer, line 268 | M2 | Should | manual — Manual test | AC-M2-14 |
| FR-58 | Audio and video metadata are shown with a still frame | §4.5 Viewer, line 269 | M9 | Could | manual — Manual test | not yet assigned |
| FR-59 | The viewer is read-only; editing is a separate later feature | §4.5 Viewer, line 270 | M1 | Must | manual — Code review | AC-M1-36 |
| FR-60 | The number of actually supported formats is stated truthfully in the about dialog | §4.5 Viewer, line 271 | M2 | Must | manual — Documentation review | AC-M2-15 |

## 4.6 Archives

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-65 | An archive appears as a normal entry and opens as a virtual directory | §4.6 Archives, line 277 | M1 | Must | automated — Integration test | AC-M1-37 |
| FR-66 | Archive entries can be listed, viewed and copied to the other panel without extracting first | §4.6 Archives, line 278 | M1 | Must | automated — Integration test | AC-M1-38 |
| FR-67 | ZIP, TAR with GZip, BZip2, XZ and Zstd, and 7z are supported | §4.6 Archives, line 279 | M1 | Must | automated — Integration test per format | AC-M1-39 |
| FR-68 | Archives can be created, including encrypted ones | §4.6 Archives, line 280 | M1 | Must | automated — Integration test | AC-M1-40 |
| FR-69 | RAR and the exotic formats are not supported | §4.6 Archives, line 281 | M0 | Must | automated — Requirement, verified by the absence of a dependency | AC-M0-06 |
| FR-70 | Every extraction goes through the hardened path resolution | §4.6 Archives, line 282 | M1 | Must | automated — Negative security test | AC-M1-41 |
| FR-71 | An archive can be tested for integrity | §4.6 Archives, line 283 | M2 | Should | automated — Integration test | AC-M2-16 |
| FR-74a | The current directory can be filtered while typing, without a dialog | §4.7 Search, compare, synchronise, line 289 | M1 | Must | mixed — Manual test plus unit test of the matcher | AC-M1-48 |
| FR-74b | The filter matches the name with a pattern language | §4.7 Search, compare, synchronise, line 290 | M1 | Must | automated — Unit test per pattern form | AC-M1-49 |
| FR-74c | The filter can be limited to files, to directories or to both | §4.7 Search, compare, synchronise, line 291 | M1 | Must | automated — Unit test per kind | AC-M1-50 |
| FR-74d | The filter can search deeper than the current directory | §4.7 Search, compare, synchronise, line 292 | M1 | Should | automated — Integration test | AC-M1-51 |
| FR-74e | The number of matching entries is shown while the filter is typed | §4.7 Search, compare, synchronise, line 293 | M1 | Must | manual — Manual test | AC-M1-52 |
| FR-74f | The filter state belongs to the panel and survives a directory change | §4.7 Search, compare, synchronise, line 294 | M1 | Should | automated — State model test | AC-M1-53 |
| FR-74g | The filter can search by content with an explicit prefix | §4.7 Search, compare, synchronise, line 295 | M6 | Should | automated — Integration test | not yet assigned |

## 4.7 Search, compare, synchronise

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-75 | The search matches name, size, date and content | §4.7 Search, compare, synchronise, line 296 | M6 | Must | automated — Integration test | not yet assigned |
| FR-76 | Text, hex and binary search are supported | §4.7 Search, compare, synchronise, line 297 | M6 | Should | automated — Integration test | not yet assigned |
| FR-77 | The boolean operators AND, OR and NOT are supported | §4.7 Search, compare, synchronise, line 298 | M6 | Could | automated — Integration test | not yet assigned |
| FR-78 | The content search runs in the background with a growing result list | §4.7 Search, compare, synchronise, line 299 | M6 | Must | manual — Manual test | not yet assigned |
| FR-79 | Two directories are compared for content, date and size | §4.7 Search, compare, synchronise, line 300 | M2 | Must | automated — Integration test | AC-M2-17 |
| FR-80 | The synchronisation transfers differences in both directions from a shown plan | §4.7 Search, compare, synchronise, line 301 | M6 | Must | automated — Integration test | not yet assigned |

## 4.8 Remote sources

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-85 | A remote source behaves like a local directory in every operation | §4.8 Remote sources, line 307 | M6 | Must | automated — Integration test with an in-process server | not yet assigned |
| FR-86 | SFTP, FTP, SMB and WebDAV are supported | §4.8 Remote sources, line 308 | M6 | Must | automated — Integration test per protocol | not yet assigned |
| FR-87 | S3-compatible targets are supported | §4.8 Remote sources, line 309 | M7 | Must | automated — Integration test | not yet assigned |
| FR-88 | Cloud sign-in uses OAuth 2.0 directly against the provider | §4.8 Remote sources, line 310 | M7 | Must | manual — Manual test | not yet assigned |
| FR-89 | The application operates no server of its own | §4.8 Remote sources, line 311 | M7 | Must | mixed — Architecture review plus dependency review | not yet assigned |
| FR-90 | SMB 1 and NTLMv1 are not implemented | §4.8 Remote sources, line 312 | M6 | Must | automated — Negative test against a server that offers only SMB 1 | not yet assigned |
| FR-91 | SMB signing and SMB 3 transport encryption are required where the server supports them | §4.8 Remote sources, line 313 | M6 | Must | automated — Integration test against a server with and without signing | not yet assigned |
| FR-92 | The WebDAV client is part of the project, not a copyleft dependency | §4.8 Remote sources, line 314 | M6 | Must | mixed — `cargo deny` plus code review | not yet assigned |
| FR-93 | Every network operation has a timeout | §4.8 Remote sources, line 315 | M6 | Must | automated — Integration test against a server that stalls | not yet assigned |
| FR-94 | The response size is bounded independently of the announced content length | §4.8 Remote sources, line 316 | M6 | Must | automated — Integration test | not yet assigned |
| FR-95 | A server that offers an older protocol version is refused, not silently accepted | §4.8 Remote sources, line 317 | M6 | Must | automated — Integration test | not yet assigned |

## 4.9 Input model

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-100 | Every operation is bound to a named action, not to a physical key | §4.9 Input model, line 323 | M1 | Must | automated — Unit test on the action catalogue | AC-M1-42 |
| FR-101 | Keyboard, pointer, touch, D-pad and controller each bind to the same actions | §4.9 Input model, line 324 | M1 | Must | automated — Unit test per context — umgesetzt in M4, siehe `docs/acceptance/M4-ACCEPTANCE.md` | AC-M1-43 |
| FR-102 | The action palette lists every action that is reachable in the current context | §4.9 Input model, line 325 | M8 | Must | automated — Reachability protocol | not yet assigned |
| FR-103 | The action palette is fully operable with a D-pad and the confirm key | §4.9 Input model, line 326 | M8 | Must | manual — Manual test | not yet assigned |
| FR-104 | Focus and selection are separate states in the D-pad and controller contexts | §4.9 Input model, line 327 | M4 | Must | automated — State model test | not yet assigned |
| FR-105 | The multi-selection on a remote control uses an explicit selection mode | §4.9 Input model, line 328 | M8 | Must | manual — Manual test | not yet assigned |
| FR-106 | The interface adapts structurally per context, not only in colour and font size | §4.9 Input model, line 329 | M2 | Must | manual — Screenshot review per context | AC-M2-18 |
| FR-107 | Every action reachable with the keyboard is reachable with the D-pad | §4.9 Input model, line 330 | M8 | Must | automated — Reachability protocol | not yet assigned |

## 4.10 Interface configuration

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-110 | The user chooses between a clean interface and a full interface | §4.10 Interface configuration, line 336 | M2 | Must | manual — Manual test | AC-M2-19 |
| FR-140 | The clean interface shows only the essential commands | §4.10 Interface configuration, line 337 | M2 | Must | manual — Screenshot review | AC-M2-20 |
| FR-141 | The full interface shows every command and every panel | §4.10 Interface configuration, line 338 | M2 | Must | manual — Screenshot review | AC-M2-21 |
| FR-142 | The choice can be made per platform and per device context | §4.10 Interface configuration, line 339 | M2 | Should | automated — Configuration test | AC-M2-22 |
| FR-143 | Every action can be shown or hidden individually | §4.10 Interface configuration, line 340 | M2 | Should | automated — Configuration test | AC-M2-23 |
| FR-144 | The interface profile can be exported and imported | §4.10 Interface configuration, line 341 | M2 | Should | automated — Round-trip test | AC-M2-24 |
| FR-145 | The interface profile contains no data beyond the settings it configures | §4.10 Interface configuration, line 342 | M2 | Must | manual — Documentation review | AC-M2-25 |

## 4.11 Internationalisation

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| FR-120 | English and German are complete at the release of M2 | §4.11 Internationalisation, line 348 | M2b | Must | automated — Gate that compares the key sets of both catalogues | AC-M2b-02 |
| FR-121 | The catalogue key set is identical in every language | §4.11 Internationalisation, line 349 | M2b | Must | automated — Gate comparing the catalogues | AC-M2b-03 |
| FR-122 | A missing translation falls back to English and then to the key, never to an empty string | §4.11 Internationalisation, line 350 | M2b | Must | automated — Unit test | AC-M2b-04 |
| FR-123 | Numbers, sizes, dates and times follow the system locale | §4.11 Internationalisation, line 351 | M2b | Must | automated — Unit test | AC-M2b-05 |
| FR-124 | The language can be changed at run time without a restart | §4.11 Internationalisation, line 352 | M2b | Must | manual — Manual test | AC-M2b-06 |
| FR-125 | Adding a language requires no change to the source code | §4.11 Internationalisation, line 353 | M2b | Must | manual — Documentation review | AC-M2b-07 |
| FR-126 | The interface language and the content language are independent | §4.11 Internationalisation, line 354 | M2b | Should | manual — Manual test | AC-M2b-08 |

## 4.12 Non-functional requirements

| ID | Title | Source | Milestone | Priority | Verification | Acceptance criterion |
|---|---|---|---|---|---|---|
| NFR-01 | A directory of 10,000 entries is on screen within 100 ms | §4.12 Non-functional requirements, line 360 | M2 | Must | automated — Benchmark | AC-M2-26 |
| NFR-02 | Switching between loaded directories takes less than 16 ms | §4.12 Non-functional requirements, line 361 | M2 | Must | automated — Benchmark | AC-M2-27 |
| NFR-03 | A key press produces a visible reaction within 50 ms | §4.12 Non-functional requirements, line 362 | M2 | Must | automated — Benchmark | AC-M2-28 |
| NFR-04 | The application is usable within 800 ms of start on macOS | §4.12 Non-functional requirements, line 363 | M2 | Must | manual — Measurement | AC-M2-29 |
| NFR-05 | Idle memory with two panels open stays below 120 MB | §4.12 Non-functional requirements, line 364 | M2 | Must | manual — Measurement | AC-M2-30 |
| NFR-06 | A sequential copy reaches at least 80 % of the sequential read and write bandwidth | §4.12 Non-functional requirements, line 365 | M2 | Must | automated — Benchmark | AC-M2-31 |
| NFR-07 | A name search across 100,000 files completes within 30 s | §4.12 Non-functional requirements, line 366 | M6 | Must | automated — Benchmark | not yet assigned |
| NFR-08 | Listing an archive with 5,000 entries takes less than 500 ms | §4.12 Non-functional requirements, line 367 | M2 | Must | automated — Benchmark | AC-M2-32 |
| NFR-09 | No operation that can take longer than 50 ms blocks the interface | §4.12 Non-functional requirements, line 368 | M1 | Must | automated — Test on the job queue | AC-M1-44 |
| NFR-09a | The verification of a copy raises the read volume to at most twice the copied volume | §4.12 Non-functional requirements, line 369 | M1 | Must | automated — Benchmark | AC-M1-45 |
| NFR-09b | The job cue updates at most ten times per second, and never blocks the interface | §4.12 Non-functional requirements, line 370 | M1 | Must | automated — Unit test on the update rate | AC-M1-46 |
| NFR-10 | The whole operation is operable with the keyboard alone | §4.12 Non-functional requirements, line 371 | M2 | Must | manual — Keyboard-only test protocol | AC-M2-33 |
| NFR-11 | Text contrast is at least 4.5:1, focus frames at least 3:1 | §4.12 Non-functional requirements, line 372 | M2 | Must | automated — Computed contrast | AC-M2-34 |
| NFR-12 | `prefers-reduced-motion` is respected | §4.12 Non-functional requirements, line 373 | M2 | Must | manual — Manual test | AC-M2-35 |
| NFR-13 | A screen reader can traverse the interface through AccessKit | §4.12 Non-functional requirements, line 374 | M2 | Must | manual — Manual test with a screen reader | AC-M2-36 |
| NFR-14 | No source file exceeds 400 lines soft or 550 lines hard | §4.12 Non-functional requirements, line 375 | M0 | Must | automated — `scripts/check-file-size.sh` | AC-M0-07 |
| NFR-15 | No `unsafe`, no `unwrap()` and no `expect()` on a reachable path | §4.12 Non-functional requirements, line 376 | M0 | Must | automated — Workspace lints plus clippy | AC-M0-08 |
| NFR-16 | Every public function carries a WHAT, WHY, EXAMPLE doc comment | §4.12 Non-functional requirements, line 377 | M0 | Must | automated — `missing_docs` lint | AC-M0-09 |
| NFR-17 | Every public function has at least one unit test | §4.12 Non-functional requirements, line 378 | M1 | Must | manual — Coverage review | AC-M1-47 |
| NFR-17a | Every module carries a header that names its responsibility and its boundary | §4.12 Non-functional requirements, line 379 | M0 | Must | mixed — `missing_docs` lint plus review | AC-M0-14 |
| NFR-17b | Every name states what a thing is, not what it is made of | §4.12 Non-functional requirements, line 380 | M1 | Must | manual — Review | AC-M1-54 |
| NFR-17c | Every crate and module relationship is documented in a diagram | §4.12 Non-functional requirements, line 381 | M2 | Must | manual — Review of `docs/ARCHITECTURE.md` | AC-M2-38 |
| NFR-18 | No dependency carries a copyleft licence | §4.12 Non-functional requirements, line 382 | M0 | Must | automated — `cargo deny` | AC-M0-10 |
| NFR-19 | No dependency carries a known vulnerability | §4.12 Non-functional requirements, line 383 | M0 | Must | automated — `cargo audit` | AC-M0-11 |
| NFR-20 | The crate dependency direction is `apps` → `platform` → `crates` | §4.12 Non-functional requirements, line 384 | M0 | Must | automated — `scripts/check-crate-boundaries.sh` with a negative test | AC-M0-12 |
| NFR-21 | The build is reproducible from the committed lock file | §4.12 Non-functional requirements, line 385 | M0 | Must | automated — `cargo build --locked` | AC-M0-13 |
| NFR-22 | The application is fully operable with the keyboard, the pointer and the touch | §4.12 Non-functional requirements, line 386 | M2 | Must | manual — Per-context test protocol | AC-M2-37 |
| NFR-23 | The application never runs with elevated privileges | §4.12 Non-functional requirements, line 387 | M2 | Must | mixed — Code review plus a test that the process does not request elevation | AC-M2-39 |
| NFR-24 | The application writes only inside its own configuration directory | §4.12 Non-functional requirements, line 388 | M2 | Must | automated — Test that traces every write | AC-M2-40 |
| NFR-25 | The application opens no network connection unless the user asks for it | §4.12 Non-functional requirements, line 389 | M2 | Must | automated — Test that the start-up opens no socket | AC-M2-41 |
| NFR-26 | The application is signed on Windows and macOS | §4.12 Non-functional requirements, line 390 | M3 | Must | manual — Verification of the release artefact | not yet assigned |
| NFR-27 | The application requests no permission that it does not use | §4.12 Non-functional requirements, line 391 | M3 | Must | manual — Manifest review against the documented use | not yet assigned |
| NFR-28 | The application makes no change to the security configuration of the machine | §4.12 Non-functional requirements, line 392 | M3 | Must | manual — Code review | not yet assigned |
| NFR-29 | The rules of section 9.7 are configurable | §4.12 Non-functional requirements, line 393 | M2 | Must | automated — Configuration test per rule | AC-M2-42 |
| NFR-30 | The configuration carries a corporate profile that enables every rule | §4.12 Non-functional requirements, line 394 | M2 | Must | automated — Configuration test | AC-M2-43 |
| NFR-31 | A rule that is switched off is never executed | §4.12 Non-functional requirements, line 395 | M2 | Must | mixed — Unit test plus manual test | AC-M2-44 |
| NFR-32 | Three guarantees cannot be switched off | §4.12 Non-functional requirements, line 396 | M2 | Must | automated — Test that the attempt to configure them has no effect | AC-M2-45 |


---

## Change log

| Date | Change |
|---|---|
| 24.10.2026 | Catalogue rebuilt from `docs/REQUIREMENTS.md` version 2.0. The identifiers FR-01 to FR-145 and NFR-01 to NFR-22 of the specification replaced the identifiers of version 1.1. No identifier of the new catalogue is reused for a different meaning: the numbering of the specification is thematic and has gaps between the blocks, and those gaps are part of the numbering. |
| 24.10.2026 | The column `Acceptance criterion` was added and points to `docs/ACCEPTANCE-CRITERIA.md`. The column `Source` now names the section **and the line** of the specification. |
| 24.10.2026 | The catalogue was completed for version 2.0 of the specification. The 25 identifiers that version 2.0 added — `FR-22m` to `FR-22q`, `FR-74a` to `FR-74g`, `NFR-17a` to `NFR-17c` and `NFR-23` to `NFR-32` — were missing and are now rows of the catalogue; the source line of every row was brought back in line with the specification. The identifiers of version 1.1 that no longer exist — `FR-117`, `FR-121`, `FR-122`, `FR-139`, `FR-140`, `FR-146`, `FR-160`, `FR-162`, `FR-163`, `FR-164` and `NFR-40` — were removed. `scripts/check-requirement-ids.py` proves that the set of the identifiers is now the same on both sides. |
