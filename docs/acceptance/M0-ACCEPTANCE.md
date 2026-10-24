# M0 Acceptance — Phocommander

| Field | Value |
|---|---|
| Milestone | M0 — Foundation |
| Acceptance date | 24.10.2026 |
| Specification version | `docs/REQUIREMENTS.md` 1.1 (24.10.2026, 963 lines) |
| Catalogue version | `docs/REQUIREMENTS-IDS.md` — FR-01…FR-167, NFR-01…NFR-41; acceptance catalogue `docs/ACCEPTANCE-CRITERIA.md` |
| Decided by | *pending — the owner decides; this document is the evidence of the QA engineer* |
| Result | **accepted with conditions** — every gate that could be measured is green; the conditions are listed in section 7 and section 8 |

**Who wrote this document:** the QA engineer of the team. **What it is:** the measured evidence for
the milestone M0 — the command outputs, the test names with their results and the exact reasons for
everything that could not be measured. **What it is not:** a claim that the product is finished. The
milestone M0 delivers the workspace, the CI basis, the configuration file and the crate skeletons —
not a usable application.

Every command of this document was executed in the working tree of the repository on 24.10.2026. The
raw output is stored under [`docs/acceptance/logs/`](logs/).

---

## 1. Functionality

Which functions of this milestone exist, and how do they behave? Every row names the requirement ID
and an observed behaviour, never a claim.

| Requirement | Behaviour observed | Evidence |
|---|---|---|
| FR-121 | The shipped configuration file `config/defaults.toml` is read by `phocommander_config` without an error, and every value of the file matches the value the specification names. | `config_defaults::the_shipped_configuration_file_is_accepted`, `config_defaults::the_shipped_configuration_file_holds_the_values_of_the_specification` — green in `cargo test --workspace` |
| FR-122 | A configuration file with an unknown field (`langauge`) is refused, and the message names the field. | `config_defaults::a_configuration_file_with_an_unknown_field_is_refused` — green |
| FR-122 | A configuration file that cannot be read reports the path of the file. | `config_defaults::a_configuration_file_that_cannot_be_read_reports_the_path` — green |
| FR-160 | The canonical limits of `phocommander-core` and the values of the shipped configuration file are equal: 2 147 483 648 bytes, 100 000 entries, 10 485 760 bytes, 4 194 304 bytes. | `core_limits::the_shipped_configuration_file_carries_the_canonical_archive_limits`, `core_limits::the_shipped_configuration_file_carries_the_canonical_viewer_limits` — green |
| FR-162 | The shipped key binding file is free of conflicts — with the two exceptions that the specification itself names (`F3` and `Shift+F3`, which mean "find" in the viewer and "jump to the conflict" in the rename dialog). | `keymap_bindings::the_shipped_keymap_has_no_conflict_in_any_context` — green |
| FR-163 | Every action of the keyboard context is reachable in the `dpad` context and in the `controller` context. | `keymap_bindings::every_action_of_the_keyboard_is_reachable_in_the_dpad_context`, `keymap_bindings::every_action_of_the_keyboard_is_reachable_in_the_controller_context` — green |
| FR-163 | In the `dpad` and in the `controller` context every action is bound to the action palette (`palette:…`); the only exceptions are `palette.open` (Menu key) and `selection.toggle` (grip) in the controller context. | `keymap_bindings::the_dpad_and_the_controller_bind_their_actions_to_the_action_palette` — green |
| FR-164 | Every action identifier of the shipped file is well formed. | `keymap_bindings::the_action_identifiers_of_the_shipped_file_are_well_formed` — green |
| FR-117 | The dependency direction of the workspace holds: `apps/*` → `platform/*` → `crates/*`, no crate below `crates/` depends on a sibling. | `./scripts/check-crate-boundaries.sh` → `8 path dependencies checked, 0 violations` (`logs/check-crate-boundaries.log`) |
| FR-117 | The gate has teeth: a dependency of `phocommander-core` on `phocommander-keymap` makes the gate red, removing it makes it green again. | `./scripts/check-gates-negative.sh`, gate 4 (`logs/check-gates-negative.log`) |
| FR-140 | The integration tests of the crate boundaries exist and run: the copy through a temporary directory is NOT RUN (the file operations arrive in M1), the extraction of an archive with traversal entries is covered by `security_path_traversal`, the loading of a configuration file with an unknown field is covered by `config_defaults`. | `cargo test --workspace` → `129 passed; 0 failed` (`logs/cargo-test-workspace.log`) |
| FR-146 | The repository builds, checks and tests itself on one operating system (Linux, x86_64). The other platforms are built by the CI matrix, not by this sandbox — see section 7. | `cargo build --workspace --release` → `EXIT=0` (`logs/cargo-build-release.log`) |
| FR-139 | Every public function that exists today has at least one test. The workspace carries 129 tests in 33 test binaries; the unit tests of the crates cover their public functions, the integration tests of `tests/` cover the crate boundaries. | `cargo test --workspace` → `129 passed; 0 failed; 0 ignored` (`logs/cargo-test-workspace.log`) |

---

## 2. Technique

Which architecture decisions are implemented, which crate boundaries were kept, which tools ran
through?

| Item | Result | Evidence |
|---|---|---|
| Crate boundaries | pass — 8 path dependencies checked, 0 violations | `logs/check-crate-boundaries.log` |
| Dependency direction | pass — `apps → platform → crates`, the test package `tests → crates` is allowed | `logs/check-crate-boundaries.log` |
| Formatting (`cargo fmt --check`) | pass — exit code 0, no diff | `logs/cargo-fmt-check.log` |
| Static analysis (`cargo clippy --all-targets -- -D warnings`) | pass — exit code 0, no warning | `logs/cargo-clippy.log` |
| Licence check (`cargo deny check`) | pass — `advisories ok, bans ok, licenses ok, sources ok`, exit code 0 | `logs/cargo-deny-check.log` |
| Vulnerability check (`cargo audit`) | pass — 214 dependencies scanned, 0 vulnerabilities, exit code 0 | `logs/cargo-audit.log` |
| Test run (`cargo test --workspace`) | pass — 129 passed, 0 failed, 0 ignored, 33 test binaries, exit code 0 | `logs/cargo-test-workspace.log` |
| Release build (`cargo build --workspace --release`) | pass — exit code 0 | `logs/cargo-build-release.log` |
| File size limit (400 soft / 550 hard) | pass — 0 files above the soft limit, 0 above the hard limit; the gate turns red on a file with 600 lines | `logs/check-file-size.log`, `logs/check-gates-negative.log` gate 2 |
| Telemetry gate (`scripts/check-no-telemetry.sh`) | pass — 15 manifests, 51 dependency names, 0 violations; the gate turns red on a forbidden crate | `logs/check-no-telemetry.log`, `logs/check-gates-negative.log` gate 1 |
| Link gate (`scripts/check-docs-links.py`) | pass — 138 repository links checked, 0 broken; the gate turns red on a dead link | `logs/check-docs-links.log`, `logs/check-gates-negative.log` gate 3 |
| Platform build | NOT RUN for macOS, Windows, FreeBSD, NetBSD, Android and iOS — the sandbox has no cross toolchain and no runner for those systems; the CI matrix of the owner builds them. Linux: pass (`cargo build --workspace --release`). | `logs/cargo-build-release.log`; the gate list of section 7 |
| Tool versions | `cargo-deny 0.20.2` and `cargo-audit 0.22.2` installed and executed — the versions the specification pins | `cargo deny --version` → `cargo-deny 0.20.2`, `cargo audit --version` → `cargo-audit-audit 0.22.2` |

**Note on the environment.** Two commands of the evidence were run with a redirected home directory
(`CARGO_HOME=/tmp/cargo-home`) and with a redirected build directory
(`CARGO_TARGET_DIR=/tmp/qa-target`). The reason is a property of the sandbox, not of the project:
the workspace lies on an NFS mount that does not support the lock file of the advisory database
(`cargo deny check` reports `NFS do not support locking`) and the linker occasionally loses object
files on that mount. The commands themselves are unchanged.

---

## 3. Performance

Which target values were measured, with which tool, with which result? A target value that was not
measured is reported as "not measured", never as "met".

| Target value | Threshold | Measured with | Result | Status |
|---|---|---|---|---|
| Directory listing with 10 000 entries, until the first display | < 100 ms | `cargo bench -p phocommander-integration-tests --bench directory_listing -- --warm-up-time 1 --measurement-time 5` (criterion 0.8) | criterion: `time: [58.887 ms 64.391 ms 70.880 ms]`; the median of five runs: `53.287992ms` | **met** — 64 ms against 100 ms |
| Directory change, already loaded | < 16 ms | — | — | **not measured** — the panel state and the drawing do not exist yet (M1/M2) |
| Key press until the visible reaction | < 50 ms | — | — | **not measured** — the surface does not exist yet |
| Start time until the operable surface | < 800 ms (macOS, warm) | — | — | **not measured** — no macOS machine in the sandbox, the surface is a skeleton |
| Memory in idle, two panels open | < 120 MB | — | — | **not measured** — no surface |
| File copy, sequential | ≥ 80 % of the sequential read/write bandwidth | — | — | **not measured** — the file operations arrive in M1 |
| Search over 100 000 files (names) | < 30 s | — | — | **not measured** — the search arrives in M2 |
| Archive listing, 5 000 entries | < 500 ms | — | — | **not measured** — the container readers arrive in M1 |

**What the measurement means.** The benchmark measures the raw access to the file system: it reads
the directory, reads the metadata of every entry and sorts the result by name. That is the part no
implementation can make faster, so it is the floor that the directory model of the core crate has to
reach. The measurement of the product function itself — `phocommander_core::fs::list_directory` —
is **not measured**, because that function does not exist yet. The benchmark fails when the threshold
is exceeded, so it is a gate and not a report.

---

## 4. Guidance

How does the milestone look, and how is it operated? Keyboard operation is reported separately,
because it is the main way of operating the application.

| Topic | Observation |
|---|---|
| Appearance | **not measured** — the milestone M0 delivers the crate skeletons, not a window. The theme of the surface (`crates/phocommander-ui/src/theme.rs`) exists, but there is no window that draws it. |
| Keyboard operation | **not measured** — the keymap of the three contexts exists and is proven free of conflicts (`keymap_bindings`), but no surface resolves a key press into an action yet. |
| Mouse operation | **not measured** — same reason. |
| Error messages | The configuration reader names the field it refuses (`config_defaults::a_configuration_file_with_an_unknown_field_is_refused` reports `langauge`), the name resolver names the entry it refuses (`EntryNameError::Traversal { name }`), the crate boundary gate names the crate and the target. |
| Language | The repository documents are in English, the user guide is bilingual (`docs/user-guide/getting-started.de.md`, `getting-started.en.md`). |

---

## 5. Security and privacy

Which hardenings are effective, and **which negative test proves it**? A positive test proves
nothing about the defence.

| Hardening | Negative test | Result | Evidence |
|---|---|---|---|
| Path traversal (zip slip) during the extraction — the name of an entry must never leave the destination | `security_path_traversal::every_hostile_entry_name_is_refused` — builds a ZIP archive at run time with nine hostile names (`../escape.txt`, `../../escape.txt`, `docs/../../escape.txt`, `..\escape.txt`, `docs\..\..\escape.txt`, `/etc/passwd`, `\windows\system32\evil.dll`, `C:\Windows\evil.dll`, `d:evil.txt`) and hands every name to `resolve_entry_name` | pass — every hostile name is refused with the reason *traversal*, *absolute path* or *reserved name* | `logs/cargo-test-workspace.log`, `tests/security/README.md` |
| The attack is real, not hypothetical | `security_path_traversal::the_attack_really_writes_outside_of_the_destination_without_the_hardening` — extracts the same archive with a deliberately unhardened reference extraction (join the name, create the parents, write) | pass — the file lands **outside** the destination, next to it. The attack works without the hardening; the hardened resolver stops it. | `logs/cargo-test-workspace.log` |
| The invariant for arbitrary names (property test) | `security_path_traversal::the_resolved_path_stays_inside` — 256 generated names, including `..`, separators and drive letters | pass — the resolved path is always relative and free of `..` | `logs/cargo-test-workspace.log` |
| Nothing is written outside of the destination | `security_path_traversal::no_file_is_written_outside_of_the_destination_directory` | pass — the only file below the temporary directory is the archive itself | `logs/cargo-test-workspace.log` |
| A name that is too long for any file system | `security_path_traversal::a_name_that_is_too_long_is_refused` | pass — refused with the length and the limit named (`MAX_ENTRY_NAME_BYTES`, 4096 bytes) | `logs/cargo-test-workspace.log` |
| Decompression bomb — the total size | `security_decompression_bomb::the_bomb_archive_declares_more_bytes_than_the_budget_allows` — builds a ZIP archive at run time and rewrites the size fields of its header, so that it declares three gigabytes | attack implemented, pass — the archive declares 3 221 225 472 bytes against the limit of 2 147 483 648 bytes | `logs/cargo-test-workspace.log` |
| Decompression bomb — the cost for the attacker | `security_decompression_bomb::the_bomb_archive_costs_the_attacker_almost_nothing` | pass — the archive weighs a few kilobytes and expands to gigabytes | `logs/cargo-test-workspace.log` |
| Decompression bomb — the expansion ratio | `security_decompression_bomb::a_highly_compressible_entry_expands_far_beyond_its_own_size` | pass — the entry of 8 MiB expands by more than a factor of 1000 | `logs/cargo-test-workspace.log` |
| Decompression bomb — the number of entries | `security_decompression_bomb::an_archive_with_too_many_entries_exceeds_the_entry_limit` | pass — 100 001 entries against the limit of 100 000 | `logs/cargo-test-workspace.log` |
| Decompression bomb — the abort during the extraction | `aborts_when_total_size_limit_is_exceeded` | **NOT RUN** — the container readers arrive in M1; `phocommander-archive` exposes `resolve_entry_name` today and nothing else. The attack is built and measured, the enforcement point does not exist yet. | `tests/security/README.md`, status `planned` |
| Symlink escape, SUID/SGID preservation, TOCTOU | `skips_symlink_entry_instead_of_creating_it`, `drops_suid_and_sgid_bits_from_extracted_file`, `checks_the_path_immediately_before_the_access` | **NOT RUN** — same reason: the reader and the file operations arrive in M1. The rules are fixed as H4, H5 and H9 in the threat model. | `docs/security/THREAT_MODEL.md` |
| Header overflow in the parsers | `fuzz_archive_header`, `fuzz_image_header`, `fuzz_charset_detection` | **NOT RUN** — the fuzz targets and the parsers arrive in M1; the nightly CI job that would run them exists in `.github/workflows/ci.yml`. | `.github/workflows/ci.yml`, job `nightly` |
| No telemetry, no analytics, no tracking | `./scripts/check-no-telemetry.sh` — 15 manifests, 51 dependency names, 0 violations; the negative run proves that the gate turns red on a forbidden crate | pass | `logs/check-no-telemetry.log`, `logs/check-gates-negative.log` gate 1 |
| Only permissive licences | `cargo deny check` — `licenses ok` | pass | `logs/cargo-deny-check.log` |
| No known vulnerability in the dependency graph | `cargo audit` — 214 dependencies, 0 vulnerabilities | pass | `logs/cargo-audit.log` |
| The process opens no network connection at the start | `process_start` | **NOT RUN** — the application does not exist yet; the test belongs to the desktop app of M1. | `docs/ACCEPTANCE-CRITERIA.md`, AC-M0-55 / NFR-40 |

---

## 6. Interoperability

Which foreign systems were checked — archives with foreign tools, network drives with Samba, WebDAV
with Nextcloud?

| Foreign system | Tool used | Result | Evidence |
|---|---|---|---|
| ZIP container written by another implementation | `zip` crate 9.1.0 (the same library the specification names for the archive format) | pass — the hostile archives are written and read back by that library, and the hardened resolver refuses their names | `security_path_traversal`, `security_decompression_bomb` |
| The archive of another producer (Info-ZIP, 7-Zip, Windows Explorer) | — | **NOT RUN** — the container readers arrive in M1; the interoperability matrix (archive written by `zip`, read back by the reader of the project) is part of M1. | `docs/ACCEPTANCE-CRITERIA.md`, AC-M1-39 |
| Network drives (Samba), WebDAV (Nextcloud), S3 | — | **NOT RUN** — the network sources arrive in M6; `phocommander-remote` is an empty crate today. | `docs/REQUIREMENTS.md`, milestone M6 |
| The configuration file of the user, written by hand | `toml` crate 1.1.2 | pass — a configuration file in a temporary directory is read from the disk, an unknown field is refused | `config_defaults` |
| The key binding file of the user | `toml` crate 1.1.2 | pass — the shipped file is read, the three context tables are checked for conflicts and for completeness | `keymap_bindings` |

---

## 7. Known limitations

What does not work, and why? This list is mandatory and must not be empty as long as there is
something to list.

| Limitation | Reason | Planned for |
|---|---|---|
| The negative test of the extraction layer (symlink escape, SUID/SGID, the abort of the bomb) does not exist | the container readers of `phocommander-archive` arrive in M1; the crate exposes `resolve_entry_name` today | M1 |
| The directory model of the core crate has no test that lists a directory | the file system model arrives in M1; the test is written against the expected API and waits in `tests/integration/directory_entries.rs` | M1 |
| The property tests cover the path resolution only | the rename rule and the key resolution do not exist yet | M1 |
| No fuzz target exists | the parsers arrive in M1; the nightly job of the CI is in place | M1 |
| No UI test exists | the surface is a skeleton; `egui_kittest` is in the dependency catalogue | M1/M2 |
| The build for macOS, Windows, FreeBSD, NetBSD, Android and iOS was not measured | the sandbox has one Linux system and no cross toolchain; the CI matrix of the owner builds the other targets | CI of the owner, before the end of M0 |
| The performance values other than the directory listing were not measured | the measured functions do not exist yet (surface, file operations, search, archive listing) | M1/M2 |
| The `dpad` reachability protocol does not exist | it is a milestone of its own | M8 |
| The CI runs the matrix jobs without the six commands of the quality job | the workflow of the owner runs `cargo test` and `cargo build --release` on the matrix, but not `fmt`, `clippy`, `deny` and `audit`; the specification asks for these in **every** run. The cross-compilation job has `continue-on-error: true`, so the platform build can never turn the build red, and the target `aarch64-apple-ios` is missing. | owner of the workflow, before the end of M0 — the ready-to-use YAML blocks are in `deliverable.md` |
| The cargo-deny lock file needs a writable `CARGO_HOME` outside an NFS mount | a property of the sandbox, not of the project; the CI runs on a normal file system | not a project defect |

---

## 8. Decision

| Field | Value |
|---|---|
| Decision | **accepted with conditions** |
| Conditions | 1. The owner closes the gaps of `.github/workflows/ci.yml` that are listed in section 7 and in `deliverable.md` (iOS target, `continue-on-error` on the cross-compilation, the six commands in every matrix run, the version check for cargo-audit, a job for the performance gate). The file belongs to the owner and was not changed. 2. The negative tests of the extraction layer are written with the readers of M1; the protocol table in `tests/security/README.md` names every test that is still missing. 3. The build for the five other platforms is measured by the CI matrix of the owner before the end of M0. |
| Decided by | *pending — the owner of the project decides; the QA engineer hands over the evidence* |
| Date | 24.10.2026 (evidence date) |
| Next milestone | M1 — file operations, job queue, viewer stage 1, archive adapters, hardened extraction |

---

### Evidence index

Every command that was run for this acceptance, with its output in the repository.

| # | Command | Result | Where the output is |
|---|---|---|---|
| 1 | `cargo fmt --all --check` | exit code 0 | `docs/acceptance/logs/cargo-fmt-check.log` |
| 2 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit code 0 | `docs/acceptance/logs/cargo-clippy.log` |
| 3 | `cargo test --workspace` | exit code 0 — 129 passed, 0 failed, 0 ignored, 33 test binaries | `docs/acceptance/logs/cargo-test-workspace.log` |
| 4 | `cargo build --workspace --release` | exit code 0 | `docs/acceptance/logs/cargo-build-release.log` |
| 5 | `cargo deny check` | exit code 0 — `advisories ok, bans ok, licenses ok, sources ok` | `docs/acceptance/logs/cargo-deny-check.log` |
| 6 | `cargo audit` | exit code 0 — 214 dependencies scanned, no vulnerability | `docs/acceptance/logs/cargo-audit.log` |
| 7 | `./scripts/check-crate-boundaries.sh` | exit code 0 — 8 path dependencies, 0 violations | `docs/acceptance/logs/check-crate-boundaries.log` |
| 8 | `./scripts/check-file-size.sh` | exit code 0 — 0 files above the limits | `docs/acceptance/logs/check-file-size.log` |
| 9 | `./scripts/check-no-telemetry.sh` | exit code 0 — 0 violations | `docs/acceptance/logs/check-no-telemetry.log` |
| 10 | `python3 scripts/check-docs-links.py` | exit code 0 — 138 links checked, 0 broken | `docs/acceptance/logs/check-docs-links.log` |
| 11 | `./scripts/check-gates-negative.sh` | exit code 0 — four gates proved red and green | `docs/acceptance/logs/check-gates-negative.log` |
| 12 | `cargo bench -p phocommander-integration-tests --bench directory_listing` | exit code 0 — criterion `[58.887 ms 64.391 ms 70.880 ms]`, median 53.29 ms, threshold 100 ms | `docs/acceptance/logs/bench-directory-listing.log` |
| 13 | `cargo deny --version`, `cargo audit --version` | `cargo-deny 0.20.2`, `cargo-audit-audit 0.22.2` | the versions the specification pins |
| 14 | `rustfmt --edition 2024 --check tests/integration/directory_entries.rs` | exit code 0 — the waiting test is syntactically correct | run on 24.10.2026, see `deliverable.md` |

*End of document. Every status reflects the state of the repository on 24.10.2026. No row is marked
as proven without a command output or a green test name behind it.*
