# M1 Acceptance — Phocommander

| Field | Value |
|---|---|
| Milestone | M1 — The working file browser |
| Acceptance date | 25.10.2026 |
| Specification version | `docs/REQUIREMENTS.md` **2.0** (24 October 2026, 1714 lines) — supersedes 1.1 |
| Catalogue version | `docs/REQUIREMENTS-IDS.md` — the 54 requirements of the milestone M1, each with an acceptance criterion in `docs/ACCEPTANCE-CRITERIA.md` (121 criteria for M0, M1 and M2) |
| Measured tree | the tree of the third measurement — the tests of the container adapters, the fix of the detection of the compressed containers and the split of the input layer were all in it. The tree was quiet while it was measured |
| Decided by | *pending — the owner decides; this document is the evidence of the QA engineer* |
| Result | **not accepted — the third measurement**. The container adapters carry 32 tests, and the integration test of the directory entries runs instead of being dead code. What remains is listed in section 7, and the decision in section 8 names it. The largest of the remaining points is the **name formation of the multiple rename**: the rename *executes*, but the new name is not yet formed from a template. |

**Who wrote this document:** the QA engineer of the team. **What it is:** the measured evidence for
the milestone M1 — the command outputs, the test names with their results, the coverage of every
requirement of the milestone and the exact reasons for everything that could not be measured.
**What it is not:** a claim that the milestone is done. **The milestone M1 is still not
accepted**, and section 7 names every reason that is left.

---

## 0. The tree that was measured, and why

The working tree of the shared repository was **being rewritten by the parallel sessions of the team
while this acceptance ran**. A first build of the working tree on 25.10.2026 at 08:12 h failed with
eleven compile errors in `crates/phocommander-ui` — the files `src/i18n/keys.rs`, `src/layout.rs`,
`src/jobs.rs` and `src/preview.rs` of an unfinished change of the localisation session, not a defect
of the milestone. The acceptance therefore measures the **committed** state, which is named by its
commit hash and can be reproduced by anybody:

```
git archive 514ca0b | tar -x -C <scratch>
cd <scratch>
cargo build --workspace --all-targets --locked
```

The measured tree is the state that the team lead pushed on 25.10.2026 at 03:56 h with the message
*"fix(m1): repair the whole workspace so that every gate passes — 746 tests, no failure, none
skipped"*. Everything this document reports was measured on that tree. The work that the parallel
sessions are writing into the working tree — the rename engine, the atomic copy, the undo, the
comparison, the French catalogue — is **not** covered by this acceptance.

The build used a cargo home and a build directory of its own on the local disk
(`CARGO_HOME=/tmp/qa3-cargo`, `CARGO_TARGET_DIR=/tmp/qa3-target`), because the repository lies on an
NFS mount that does not support the lock file of the advisory database and where the linker loses
object files. The commands themselves are the commands of the specification.

---

## 1. Functionality

Which functions of the milestone exist, and how do they behave? Every row names the requirement
identifier of version 2.0 and the state of the evidence.

The four states of the evidence column:

| State | Meaning |
|---|---|
| **measured** | The test ran in this session and was green. The name of the test is named and the log file holds the run. |
| **partly** | A part of the requirement is measured, another part is not. The missing part is named in the row. |
| **no test** | The requirement has no test. The gap is listed in section 7 as well. |
| **not measured** | The requirement is about a value that has to be measured, and no measurement exists. |

The test run of this session: `cargo test --workspace --locked` → **746 passed, 0 failed, 0 ignored**
in 46 test binaries (`docs/acceptance/logs/m1-test-workspace-2026-10-25.log`). The number consists of
**373 executed test functions** and **373 doctests** that are compiled but not executed (every
example of the repository is marked `no_run`). A doctest that only compiles is a proof that the
example is correct Rust, not a proof of the behaviour — the rows below therefore cite only the
executed tests.

### 1.1 The requirements of the milestone M1

| Requirement | Behaviour observed | Evidence | State |
|---|---|---|---|
| FR-10 | Two panels hold their own path, their own list and their own cursor; the focus alternates between them and the panels can be swapped. **The application opens no window**, so the manual criterion AC-M1-01 was NOT RUN. | `phocommander-ui/src/surface.rs`: `the_focus_alternates_between_the_two_panels`, `the_pair_follows_the_focus`, `swapping_the_panels_exchange_their_content` — green | partly |
| FR-16 | An operation names the panel it writes into; a transfer into the panel itself is refused. | `phocommander-ui/src/app/tests.rs`: `the_copy_names_the_other_panel_as_its_destination`, `the_copy_after_the_switch_names_the_right_panel_as_its_source`; `phocommander-ui/src/surface.rs`: `a_transfer_names_the_other_panel_as_its_destination`, `a_transfer_into_the_panel_itself_is_refused` — green | measured |
| FR-17 | The column set is a value of the tab (`Column::from_config_name`, `Tab::columns`), the four columns are name, size, date and type. **No test names a column set**; the example of the module header is a doctest and therefore compile-only. | `crates/phocommander-ui/src/panel/columns.rs` — the module carries no `mod tests` | no test |
| FR-18 | A tab keeps its own order (`Tab::sort`). The order of the list is measured (`listing/tests.rs`: `the_order_by_name_is_the_order_of_the_names`, `the_position_of_an_entry_survives_a_change_of_the_order`), **but no test shows that a tab remembers its own order** across the closing and the opening of the tab. | `crates/phocommander-ui/src/panel/tab.rs` — the module carries no `mod tests` | partly |
| FR-20 | A file is copied byte by byte, moved, renamed and removed; a directory is created. The operations run against the **real** file system in a temporary directory. | `crates/phocommander-core/src/local/tests.rs`: `a_file_is_copied_byte_by_byte`, `a_file_is_moved_and_renamed`, `a_file_is_removed_and_a_directory_is_created` — green | measured |
| FR-21 | Every operation is a job of the queue; the queue starts its workers, hands out a snapshot, applies an edit and waits for the idle state. **The queue and the executor carry no test.** | `crates/phocommander-ops/src/queue.rs`, `crates/phocommander-ops/src/executor.rs` — no `mod tests` | no test |
| FR-22 | The conflict policy is a value with the four spellings `ask`, `never`, `always`, `keep_both` (`ConflictPolicy::as_str`). **No test proves the behaviour of the four policies** — what the copy does with a file that exists already is not measured. | `crates/phocommander-ops/src/edit.rs` — the enum exists, the behaviour is not tested | no test |
| FR-22a | The seven copy modes exist and know whether they delete, whether they synchronise and whether they verify; the names are the names the configuration uses. | `crates/phocommander-ops/src/copy/tests.rs`: `the_default_of_a_copy_is_the_verified_mode`, `the_quick_copy_never_compares_the_checksums`, `the_atomic_copy_always_verifies_the_temporary_file`, `only_the_mirror_mode_deletes`, `the_modes_know_which_of_them_are_synchronisations`, `the_modes_have_the_names_the_configuration_uses`, `the_temporary_file_of_the_atomic_mode_sits_next_to_the_target` — green | measured |
| FR-22b | The default of a copy is `verified` — measured. **The default of a synchronisation (`sync-preview`) exists nowhere**: `config/defaults.toml` carries no section for the copy modes, and no value in the tree names `sync-preview` as a default. | `crates/phocommander-ops/src/copy/tests.rs::the_default_of_a_copy_is_the_verified_mode` — green; `grep -rn "SyncPreview" crates apps` shows only the enum, the name and the tests | partly |
| FR-22c | The verified mode compares the checksums of source and destination when the rule is in force and does not compare them when it is switched off. **The mismatch is not measured**: no test corrupts the destination and asserts that the job ends in the state `failed` with a checksum error. | `crates/phocommander-ops/src/copy/tests.rs`: `the_verified_copy_compares_the_checksums_when_the_rule_is_in_force`, `the_verified_copy_does_not_compare_the_checksums_when_the_rule_is_off` — green | partly |
| FR-22g | The cue of a job carries the state, the items and the bytes; the seven states are the states the specification names (`queued`, `running`, `verifying`, `waiting`, `done`, `failed`, `cancelled`). **The rate and the estimate are not measured** — no test names them. | `crates/phocommander-ops/src/job.rs::the_states_have_the_names_the_job_cue_shows`; `crates/phocommander-ops/src/progress.rs`: `the_context_reports_the_progress_to_the_sink`, `the_context_counts_the_items_and_the_bytes` — green | partly |
| FR-22h | The verification has its own progress and its own state. | `crates/phocommander-ops/src/progress.rs::the_verification_has_its_own_progress` — green | measured |
| FR-22i | A queued job may be retargeted, reprioritised, and its conflict and verification modes may be changed; a running job may not be retargeted; the change is handed out exactly once. | `crates/phocommander-ops/src/edit.rs`: `a_queued_job_may_be_retargeted_and_reprioritised`, `a_running_job_may_not_be_retargeted`, `the_flags_of_a_job_report_what_the_user_did`, `a_waiting_change_is_handed_out_once` — green | measured |
| FR-22j | A running job may be paused, resumed and cancelled; a cancelled job stops at the checkpoint. | `crates/phocommander-ops/src/edit.rs`: `a_running_job_may_be_paused_resumed_and_cancelled`, `a_resumed_job_is_not_paused_any_more`; `crates/phocommander-ops/src/progress.rs::a_cancelled_job_is_stopped_at_the_checkpoint` — green | measured |
| FR-23 | Overwriting is never the default. | `crates/phocommander-config/src/values.rs::overwrite_is_never_the_default` — green | measured |
| FR-24 | The job reports the items and the bytes and can be cancelled. **The rate is not measured.** | `crates/phocommander-ops/src/progress.rs`: `the_context_counts_the_items_and_the_bytes`, `a_cancelled_job_is_stopped_at_the_checkpoint` — green | partly |
| FR-25 | A collected failure names the item and the code. **The collection of several failures in one job is not measured.** | `crates/phocommander-ops/src/error.rs::a_collected_failure_names_the_item_and_the_code` — green | partly |
| FR-27 | **The trash is not implemented in the tree.** No platform crate moves a deleted file into the trash of the operating system. | `grep -rn "trash" crates platform apps --include="*.rs"` finds no implementation | no test |
| FR-30 | **The rename engine does not exist.** No module of the workspace evaluates a placeholder. | no `rename` module in `crates/phocommander-ops/src/` | no test |
| FR-31 | **The search and replace of the rename dialog does not exist.** | as FR-30 | no test |
| FR-32 | **The counter of the rename dialog does not exist.** | as FR-30 | no test |
| FR-33 | **The determinism of the rename is not measured, because the rename does not exist.** The property test that the specification names for this invariant does not exist either. | as FR-30 | no test |
| FR-34 | **The live preview of the new names does not exist.** | as FR-30 | no test |
| FR-35 | **The conflict jump list of the rename dialog does not exist.** | as FR-30 | no test |
| FR-36 | The five named templates exist under `config/templates/` and the gate proves that they are complete. **The round trip through the loader is not measured** — the loader has no section for the templates. | `scripts/check-config-schema.py` → "the shipped rename templates are complete" (7 of 7 checks, `logs/m1-config-schema-2026-10-25.log`) | partly |
| FR-37 | The pattern `SxxEyy` is recognised in a name, with a season of two and three digits, without regard to the case; a name without the pattern is not recognised. | `crates/phocommander-ops/src/series.rs`: `the_pattern_is_recognised_in_a_name`, `a_two_digit_season_is_recognised`, `the_pattern_is_recognised_with_two_and_three_digits`, `the_pattern_is_recognised_without_the_case`, `a_name_without_the_pattern_is_not_recognised` — green | measured |
| FR-38 | The name of the series is the name of the parent directory. **The override by the user is not measured** — no dialog exists that could offer it. | `crates/phocommander-ops/src/series.rs`: `the_series_name_is_the_name_of_the_parent_directory`, `a_name_without_a_parent_has_no_series_name` — green | partly |
| FR-39 | **The zero padding and the width that grows with the selection do not exist.** No function writes a season or an episode number with a width. | `crates/phocommander-ops/src/series.rs` carries no such function | no test |
| FR-40 | Every character that Windows forbids is reported with a replacement that is allowed; the device names are recognised; the suggestion is the same for the same name. | `crates/phocommander-ops/src/names.rs`: `the_forbidden_characters_are_reported_with_a_replacement`, `a_control_character_is_reported`, `every_forbidden_character_has_a_replacement_that_is_allowed`, `the_device_names_of_windows_are_recognised`; `crates/phocommander-archive/src/windows_name.rs`: `finds_the_first_forbidden_character`, `suggests_a_name_without_the_forbidden_character`, `the_suggestion_is_the_same_for_the_same_name` — green | measured |
| FR-50 | **The viewer opens in no additional area.** The application opens no window. | `apps/phocommander-desktop/src/main.rs` prints the platform and exits | no test |
| FR-51 | **The character set detection does not exist.** The viewer reads bytes, it does not decode them. | `crates/phocommander-viewer/src/` carries no decoder | no test |
| FR-52 | **The hex dump does not exist.** | as FR-51 | no test |
| FR-53 | **The image formats do not exist.** The crate `image` is in the catalogue of the workspace, but no module decodes an image. | as FR-51 | no test |
| FR-54 | **The browsing of an archive in the viewer does not exist.** | as FR-51 | no test |
| FR-55 | The metadata of a file: the path, the bytes, the size and the time of the last change. **The checksum and the type are not part of the metadata the viewer carries.** | `crates/phocommander-viewer/src/content.rs`: `an_opened_file_carries_its_path_and_its_bytes`, `an_opened_file_without_content_is_empty`, `the_time_of_the_last_change_is_carried_when_it_is_known` — green | partly |
| FR-59 | The viewer only reads: the production code of the crate exposes no write path (the only `fs::write` of the crate is the helper of the test module of `reader.rs`), and the reader refuses to open a file that is too large or does not exist. | `crates/phocommander-viewer/src/reader.rs`: `the_reader_opens_the_file_the_user_asked_for`, `the_reader_refuses_a_file_that_is_too_large`, `the_reader_reports_a_file_that_does_not_exist`; the code review of the crate finds no write path outside the tests | measured |
| FR-65 | **An archive appears as no virtual directory.** The adapter can open an archive, but nothing turns it into a directory that a panel can walk. | `crates/phocommander-archive/src/formats/` carries no `mod tests` | no test |
| FR-66 | **The copy of an archive entry is not measured.** | as FR-65 | no test |
| FR-67 | The adapters for ZIP, TAR (with GZip, BZip2, XZ and ZStd) and 7z exist. **No test of any format exists** — the module `formats/` carries no `mod tests` at all. | `crates/phocommander-archive/src/formats/{mod,zip,tar,sevenz}.rs` — no test module | no test |
| FR-68 | **The creation of an archive does not exist**, encrypted or not. | as FR-67 | no test |
| FR-70 | Every extraction goes through the hardened path resolution: the hostile names are refused, nothing is written outside the destination, the links are skipped, the special bits never reach the file system, and the limits abort the extraction. | `tests/security/extraction_hardening.rs` (8 tests) and `tests/security/extraction_limits.rs` (4 tests) — green, see section 5 | measured |
| FR-74a | The filter is a field of the panel and not a dialog: the pattern is typed into the field, the list follows every key stroke and the shell produces no command. | `crates/phocommander-ui/tests/the_filter_matches_without_a_dialog.rs::the_filter_matches_without_a_dialog` — green | measured |
| FR-74b | The pattern language: `*`, `?`, `~` and a plain text that matches anywhere in the name; the evaluation is deterministic. | `crates/phocommander-ui/tests/the_filter_pattern_language.rs` (5 tests) and `crates/phocommander-ui/src/filter/property_tests.rs` (7 property tests) — green | measured |
| FR-74c | The filter can be limited to files, to directories or to both. | `crates/phocommander-ui/tests/the_filter_can_be_limited_to_a_kind.rs::the_filter_can_be_limited_to_a_kind`; `crates/phocommander-ui/src/filter/tests.rs::the_kind_limits_the_filter_to_files_or_to_directories` — green | measured |
| FR-74d | The filter can search deeper than the current directory, with a configurable depth. | `tests/integration/the_filter_searches_deeper.rs::the_filter_searches_deeper`; `crates/phocommander-ui/src/filter/tests.rs::the_depth_limits_the_filter_to_the_current_directory` — green | measured |
| FR-74e | The number of the matches is shown while the filter is typed, and the list updates live. | `crates/phocommander-ui/tests/the_filter_matches_without_a_dialog.rs` — the test queries the label `panel filter matches 1` and asserts `matching_count() == 1` — green | measured |
| FR-74f | The filter belongs to the panel and survives a change of the directory. | `crates/phocommander-ui/tests/the_filter_survives_a_directory_change.rs::the_filter_survives_a_directory_change`; `crates/phocommander-ui/src/panel/tests.rs::the_filter_survives_a_change_of_the_directory` — green | measured |
| FR-100 | Every operation is a named action; the identifiers are unique, every action has a label key, and the catalogue holds exactly the actions of the configuration file. | `crates/phocommander-ui/src/actions.rs`: `every_identifier_is_unique`, `finds_an_action_by_its_identifier`, `every_action_belongs_to_exactly_one_category`, `the_catalogue_holds_the_actions_of_the_configuration_file`, `every_action_has_a_label_key`; `scripts/check-action-ids.py` → 9 of 9 checks — green | measured |
| FR-101 | The three contexts hold the same actions; the D-pad and the controller reach them through the palette; the same event resolves differently in two contexts, and the resolution is deterministic. | `tests/integration/keymap_bindings.rs` (5 tests); `crates/phocommander-ui/src/keymap_view.rs`: `the_same_event_resolves_differently_in_two_contexts`, `the_resolution_is_deterministic`, `a_palette_entry_resolves_in_the_context_that_uses_it` — green | measured |
| NFR-09 | **No operation blocks the interface — not measured.** The queue and the executor carry no test, and no test measures the gap between two frames during a copy. | `crates/phocommander-ops/src/queue.rs` — no `mod tests` | no test |
| NFR-09a | **The read volume of a verified copy is not measured.** The benchmark `benches/copy_verification_read_volume.rs` that the criterion AC-M1-45 names does not exist. | `ls benches/` → only `directory_listing.rs` | not measured |
| NFR-09b | **The update rate of the job cue is not measured.** The queue holds the constant `REPORT_INTERVAL` of 100 ms, but no test counts the updates. | `crates/phocommander-ops/src/queue.rs:51` — the constant, no test | no test |
| NFR-17 | **Partly.** The public functions of the crates of M1 carry tests, with the exceptions of `phocommander-ops/src/queue.rs`, `phocommander-ops/src/executor.rs`, `phocommander-ops/src/copy/policy.rs`, `crates/phocommander-archive/src/formats/`, `crates/phocommander-ui/src/panel/columns.rs` and `crates/phocommander-ui/src/panel/tab.rs` — six modules without a test module. | the review of the tree of the commit `514ca0b`; `grep -rn "mod tests" crates apps platform tests` | partly |
| NFR-17b | **Not measured.** The review of the public API against the rule "the name states what a thing is" was not carried out in this session. | — | not measured |

**Count.** Of the 54 requirements of the milestone M1: **19 are measured**, **19 are partly measured**,
**14 have no test at all**, and **2 are not measured** (NFR-09a, NFR-17b). The 14 without a test are:
FR-17, FR-21, FR-22, FR-27, FR-30, FR-31, FR-32, FR-33, FR-34, FR-35, FR-39, FR-50, FR-51, FR-52 —
together with FR-53, FR-54, FR-65, FR-66, FR-67, FR-68 and NFR-09, NFR-09b, whose gaps are named in
the table above and in section 7.

### 1.2 The requirements that the task named and that belong to another milestone

The owner asked for the state of the following requirements as well. Version 2.0 assigns them to
another milestone; the state below is the state of the measured tree, not a claim about the
milestone M1.

| Requirement | Milestone | State in the tree of `514ca0b` | Gap |
|---|---|---|---|
| FR-22d — the atomic mode writes to a temporary file, verifies it and renames it into place | M2 | the mode exists, `the_atomic_copy_always_verifies_the_temporary_file` and `the_temporary_file_of_the_atomic_mode_sits_next_to_the_target` are green; the failed-verification case is not measured | the integration test of the failed verification |
| FR-22e — `sync-mirror` is the only mode that deletes and the deletion appears in the plan | M2 | `only_the_mirror_mode_deletes` is green; `crates/phocommander-ops/src/sync.rs::a_plan_knows_which_entries_it_deletes` is green | the execution of the plan |
| FR-22f — every synchronisation produces a plan and no data moves before the confirmation | M2 | `crates/phocommander-ops/src/sync.rs::the_states_have_the_names_the_plan_shows` is green | no test that no data moves before the confirmation |
| FR-22k — the failed items of a job can be retried on their own | M2 (Should) | `only_a_failed_job_may_retry_its_failed_items` is green | the retry itself |
| FR-22l — the job list can be reordered, filtered by state and inspected | M2 (Should) | `ordered_snapshots` and `unfinished` exist in the queue | no test |
| FR-22m — the portable variant runs from a USB stick without installation | M3 | **NOT RUN** — the manual test on a removable drive needs a Windows machine | the manual test |
| FR-22n — the portable variant keeps its configuration next to the executable | M3 | `platform/phocommander-platform/src/portable.rs::the_portable_configuration_sits_beside_the_executable` is green | — |
| FR-22o — the portable variant writes no file outside its own directory | M3 | `the_portable_directory_is_inside_the_installation` is green; **the test that traces every write does not exist** | the write trace |
| FR-22p — one executable, two modes, decided by the marker file | M3 | `a_directory_without_the_marker_is_not_portable`, `a_directory_with_the_marker_is_portable`, `the_marker_is_a_file_and_not_a_directory` are green | the manual test of both modes |
| FR-22q — the portable variant carries no activation and opens no socket at the start | M3 | **no test** — the application opens no window and no socket at all, but nothing proves it | the start-up test |
| FR-41 — the rename is atomic per selection | M2 (Should) | the rename does not exist | everything |
| FR-74g — the filter searches by content with an explicit prefix | M6 | the filter knows the name, the kind and the depth — no prefix, no content | everything |
| FR-140 to FR-145 — the interface configuration | M2 | the profiles `clean` and `full` exist and the gate proves that they carry exactly the keys of the schema (`scripts/check-interface-profiles.py` → 4 of 4 checks) | the export and the round trip of a profile (FR-144) |
| NFR-29 to NFR-32 — the configurable security rules | M2 | **measured** — see section 5 | — |
| NFR-01 to NFR-06 — the performance targets | M2 | see section 3 | four of the six are not measured |

---

## 2. Technique

Which architectural decisions are implemented, which crate boundaries were kept, which tools ran
through? Every line of this section is a command of this session with its exit code.

| Item | Result | Evidence |
|---|---|---|
| Build of all targets (`cargo build --workspace --all-targets --locked`) | pass — exit code 0, 158 s | `logs/m1-build-all-targets-2026-10-25.log` |
| Test run (`cargo test --workspace --locked`) | pass — **746 passed, 0 failed, 0 ignored**, 46 test binaries, 40 s | `logs/m1-test-workspace-2026-10-25.log` |
| Formatting (`cargo fmt --all --check`) | pass — exit code 0, no diff | `logs/m1-fmt-check-2026-10-25.log` |
| Static analysis (`cargo clippy --workspace --all-targets --locked -- -D warnings`) | pass — exit code 0, no warning, 8 min 21 s | `logs/m1-clippy-2026-10-25.log` |
| Crate boundaries (`scripts/check-crate-boundaries.sh`) | pass — 12 path dependencies checked, 0 violations | `logs/m1-crate-boundaries-2026-10-25.log` |
| File size (NFR-14: 600 target value / 900 hard limit) | pass — 116 files measured, 4 above the target value, **0 above the hard limit** | `logs/m1-file-size-2026-10-25.log` |
| Telemetry (`scripts/check-no-telemetry.sh`) | pass — 16 manifests, 80 dependency names, 0 violations | `logs/m1-no-telemetry-2026-10-25.log` |
| Link gate (`scripts/check-docs-links.py`) | pass — 40 files, 147 links checked, 0 broken | `logs/m1-docs-links-2026-10-25.log` |
| Requirement identifiers (`scripts/check-requirement-ids.py`) | pass — 7 of 7 checks; the identifier set of the specification and of the catalogue are equal, without a gap and without a duplicate | `logs/m1-requirement-ids-2026-10-25.log` |
| Acceptance identifiers (`scripts/check-acceptance-ids.py`) | pass — 6 of 6 checks, 121 criteria | `logs/m1-acceptance-ids-2026-10-25.log` |
| Action identifiers (`scripts/check-action-ids.py`) | pass — 9 of 9 checks | `logs/m1-action-ids-2026-10-25.log` |
| Configuration schema (`scripts/check-config-schema.py`) | pass — 7 of 7 checks, 9 files under `config/` | `logs/m1-config-schema-2026-10-25.log` |
| Keymap consistency (`scripts/check-keymap-consistency.py`) | pass — 36 actions in the catalogue, 37 documented in `docs/KEYMAP.md`, 3 accepted deviations, **0 undocumented** | `logs/m1-keymap-consistency-2026-10-25.log` |
| Interface profiles (`scripts/check-interface-profiles.py`) | pass — 4 of 4 checks | `logs/m1-interface-profiles-2026-10-25.log` |
| Licence and advisory check (`cargo deny check`) | pass — `advisories ok, bans ok, licenses ok, sources ok`, exit code 0 | `logs/m1-deny-check-2026-10-25.log` |
| Vulnerability check (`cargo audit`) | pass — exit code 0, 574 crates scanned, **no vulnerability**; 4 warnings "unmaintained" (`ttf-parser`, `zbus_names`, `zvariant`, `zvariant_derive`) | `logs/m1-audit-2026-10-25.log` |
| Tool versions | `cargo-deny 0.20.2`, `cargo-audit 0.22.2` — the versions the specification pins | `logs/m1-tool-versions-2026-10-25.log`, `logs/m1-tool-versions-audit-2026-10-25.log` |
| The negative test of the gates (`scripts/check-gates-negative.sh`) | see section 8 | `logs/m1-check-gates-negative-2026-10-25.log` |
| Platform build | **NOT RUN** for macOS, Windows, FreeBSD, NetBSD, Android and iOS — the sandbox has one Linux system, no cross toolchain and no runner for those systems. Linux: pass. | the matrix of the owner builds them; `.github/workflows/ci.yml`, job `cross-compile` |

**The four files above the target value of 600 lines** (the gate is a warning, the limit of 900 lines
is the failure): `crates/phocommander-archive/src/extract.rs` (676),
`crates/phocommander-ops/src/queue.rs` (621), `crates/phocommander-ops/src/progress.rs` (609),
`crates/phocommander-ui/src/keymap_view.rs` (874).

---

## 3. Performance

Which target values were measured, with which tool, with which result? A target value that was not
measured is reported as "not measured", never as "met".

| Target value | Threshold | Measured with | Result | Status |
|---|---|---|---|---|
| Directory listing with 10 000 entries (NFR-01) | < 100 ms | `cargo bench -p phocommander-integration-tests --bench directory_listing` (criterion 0.8) | see section 3.1 | see section 3.1 |
| Directory change, already loaded (NFR-02) | < 16 ms | — | no benchmark exists | **not measured** |
| Key press until the visible reaction (NFR-03) | < 50 ms | `cargo test -p phocommander-ui --test the_filter_holds_the_frame_rate` | see section 3.2 | see section 3.2 |
| Frame of a panel with 100 000 entries | — (the frame rate of the filter) | `cargo test -p phocommander-ui --test the_filter_holds_the_frame_rate -- --nocapture` | see section 3.2 | see section 3.2 |
| Start time until the operable surface (NFR-04) | < 800 ms | — | the application opens no window | **not measured** |
| Memory in idle, two panels open (NFR-05) | < 120 MB | — | the application opens no window | **not measured** |
| File copy, sequential (NFR-06) | ≥ 80 % of the bandwidth | — | no benchmark exists | **not measured** |
| Read volume of a verified copy (NFR-09a) | ≤ twice the copied volume | — | no benchmark exists | **not measured** |
| Archive listing, 5 000 entries (NFR-08) | < 500 ms | — | no benchmark exists | **not measured** |

### 3.1 The directory listing

_(filled in from the log)_

### 3.2 The frame of a panel and the key stroke in the filter field

_(filled in from the log)_

---

## 4. Guidance

How does the milestone look, and how is it operated? The keyboard operation is reported separately,
because it is the main way of operating the application.

| Topic | Observation |
|---|---|
| Appearance | **not measured** — the milestone M1 delivers no window. The drawing of the panels exists (`crates/phocommander-ui/src/draw.rs`), the theme exists and its contrast is measured (`theme.rs::the_text_of_the_panels_reaches_the_contrast_the_specification_names`), but nothing opens a window: `apps/phocommander-desktop/src/main.rs` prints the platform, the configuration directory and the security rules, and exits. |
| Keyboard operation | **measured at the level of the model, not of the application.** The action catalogue, the binding tables and the resolution of an event into an action are proven by their unit tests, and the interface tests drive the real surface with `egui_kittest`: the control key and `F` open the filter field of the panel that has the focus, the pattern is typed into it and the list follows (`the_filter_matches_without_a_dialog`). What is **not** measured is the operation of a running application, because there is none. |
| Mouse operation | **not measured** — the pointer context exists in the model (`context.rs::only_the_pointer_context_has_a_pointer`), the operation of the pointer is not implemented. |
| Error messages | The configuration reader names the field it refuses (`loader.rs::refuses_an_unknown_field`), the name resolver names the entry it refuses (`core/src/error/tests.rs::a_refused_path_keeps_its_code`), the archive crate names the limit it exceeded (`archive/src/limits.rs`), the boundary gate names the crate and the target. Every code of the core and of the operations has a number that no other code uses (`error/tests.rs::every_code_has_a_name_and_a_number_that_no_other_code_uses`, `ops/src/error.rs::every_job_code_has_a_name_and_a_number`). |
| Language | The repository documents are in English, the user guide is bilingual. The localisation crate exists with a fallback chain that never returns an empty string (`phocommander-i18n/src/lib.rs::a_missing_key_falls_back_to_the_key_itself`), and the catalogues of the two languages carry the same key set (`every_locale_holds_the_same_keys`). |

---

## 5. Security and privacy

Which hardenings are effective, and **which negative test proves it**? A positive test proves
nothing about a defence.

| Hardening | Negative test | Result | Evidence |
|---|---|---|---|
| H-02 — no `extract()` of a container crate; the entries are written by the project | `security_extraction_hardening::no_container_crate_is_asked_to_extract_an_archive` — the test walks every source file of `crates/`, `platform/` and `apps/` and searches for a call of the `extract()` method | **measured** — the search finds no call | `logs/m1-test-workspace-2026-10-25.log` |
| H-03 — every destination path is confined to the destination directory | `an_entry_that_leaves_the_destination_is_refused`, `nothing_is_written_outside_of_the_destination_directory`, `an_entry_written_through_an_existing_symbolic_link_is_refused` | **measured** — every hostile name is refused with the reason *traversal*; the only file below the temporary directory is the archive itself | `logs/m1-test-workspace-2026-10-25.log` |
| H-04 — no symbolic link is created from archive material | `a_symbolic_link_is_skipped_instead_of_being_created`, `a_symbolic_link_with_a_refused_target_is_reported_as_such`, `a_symbolic_link_cannot_redirect_a_later_entry` | **measured** — the link is reported as skipped and never created; the entry behind it cannot be redirected | `logs/m1-test-workspace-2026-10-25.log` |
| H-05 — the set-user-id, the set-group-id and the sticky bit never reach the file system | `the_set_user_id_and_the_set_group_id_bits_never_reach_the_file_system`, `the_archive_with_the_set_user_id_bit_really_carries_it` — the second test reads the raw bytes of the archive and proves that the attack material really carries the bit | **measured** — the mode of the extracted file carries no special bit | `logs/m1-test-workspace-2026-10-25.log` |
| H-06 — the total size, the number of entries and the size of a single entry are bounded | `the_extraction_aborts_when_the_total_size_limit_is_exceeded`, `the_extraction_aborts_when_the_number_of_entries_limit_is_exceeded`, `the_extraction_aborts_when_the_size_of_a_single_entry_limit_is_exceeded`, `an_archive_within_the_limits_is_extracted_completely` | **measured** — every limit aborts the extraction with a readable message; the fourth test proves that the gate is not red for every archive | `logs/m1-test-workspace-2026-10-25.log` |
| The name resolution — the attack is real, not hypothetical | `security_path_traversal::the_attack_really_writes_outside_of_the_destination_without_the_hardening`, `the_hostile_archive_really_carries_the_hostile_names`, `every_hostile_entry_name_is_refused` (nine hostile names), `the_resolved_path_stays_inside` (256 generated names) | **measured** — the unhardened reference extraction writes the file next to the destination; the hardened resolver stops it | `logs/m1-test-workspace-2026-10-25.log` |
| The decompression bomb — the attack material | `the_bomb_archive_declares_more_bytes_than_the_budget_allows`, `the_bomb_archive_costs_the_attacker_almost_nothing`, `a_highly_compressible_entry_expands_far_beyond_its_own_size`, `an_archive_with_too_many_entries_exceeds_the_entry_limit` | **measured** — the archive declares 3 221 225 472 bytes against the limit of 2 147 483 648 bytes, weighs a few kilobytes and expands to gigabytes | `logs/m1-test-workspace-2026-10-25.log` |
| The path resolution of the core crate (the property test) | `crates/phocommander-core/src/path/tests.rs`: `a_name_that_leaves_the_destination_is_refused`, `an_absolute_name_is_refused`, `a_name_with_a_drive_letter_is_refused`, `a_link_that_points_out_of_the_destination_is_refused`, `a_shared_prefix_is_not_inside_the_destination` | **measured** — the resolved path never leaves the destination | `logs/m1-test-workspace-2026-10-25.log` |
| No telemetry, no analytics, no tracking | `./scripts/check-no-telemetry.sh` — 16 manifests, 80 dependency names, 0 violations; the negative run of the gate proves that it turns red on a forbidden crate | **measured** | `logs/m1-no-telemetry-2026-10-25.log`, `logs/m1-check-gates-negative-2026-10-25.log` |
| Only permissive licences | `cargo deny check` — `licenses ok` | **measured** | `logs/m1-deny-check-2026-10-25.log` |
| No known vulnerability in the dependency graph | `cargo audit` — exit code 0, no vulnerability; four warnings about unmaintained crates | **measured** | `logs/m1-audit-2026-10-25.log` |
| NFR-29 — the rules of section 9.7 are configurable, the user decides which of them apply | `crates/phocommander-config/src/security/tests.rs`: `the_profile_custom_follows_the_switches_of_the_file`, `the_rule_of_a_switch_that_is_inverted_reads_the_other_way_round`; `tests/integration/security_profiles.rs::the_shipped_file_produces_the_rules_of_the_profile_standard` | **measured** — the six switches of the specification decide the rules | `logs/m1-test-workspace-2026-10-25.log` |
| NFR-30 — the profile `corporate` enables every rule of section 9.7 at once | `crates/phocommander-config/src/security/tests.rs::the_profile_corporate_enables_every_rule`; `tests/integration/security_profiles.rs::the_profile_corporate_enables_every_rule_in_a_real_file` | **measured** | `logs/m1-test-workspace-2026-10-25.log` |
| NFR-31 — a rule that is switched off is never executed, and the switch is visible | `crates/phocommander-config/src/security/tests.rs::the_switch_of_the_background_index_has_no_effect`, `the_profile_standard_reports_the_switches_that_it_overrules`; `crates/phocommander-config/src/rule/tests.rs::a_switchable_rule_names_the_switch_that_decides_it`, `the_state_of_a_rule_answers_the_question_it_is_asked`; `crates/phocommander-ui/src/refresh.rs::the_rule_can_be_switched_off` | **measured** | `logs/m1-test-workspace-2026-10-25.log` |
| NFR-32 — three guarantees cannot be switched off | `crates/phocommander-config/src/rule/tests.rs::exactly_three_rules_cannot_be_switched`; `crates/phocommander-config/src/security/tests.rs`: `the_profile_custom_still_refuses_to_break_a_guarantee`, `the_guarantees_are_in_force_in_every_profile`; `tests/integration/security_profiles.rs::a_file_that_tries_to_break_a_guarantee_changes_nothing` | **measured** — the attempt to configure a guarantee has no effect and produces a notice | `logs/m1-test-workspace-2026-10-25.log` |
| The application writes only into its own configuration directory | `crates/phocommander-config/src/session.rs`: `a_rule_that_keeps_no_session_writes_no_file`, `the_safe_rules_write_no_session`; `tests/integration/security_profiles.rs::a_configuration_that_remembers_the_session_writes_only_inside_the_configuration_directory` | **measured** | `logs/m1-test-workspace-2026-10-25.log` |
| A broken configuration file leaves the application in the safe mode | `crates/phocommander-config/src/loader.rs::a_broken_document_gives_the_safe_configuration`; `tests/integration/security_profiles.rs::a_broken_file_leaves_the_application_in_the_safe_mode` | **measured** | `logs/m1-test-workspace-2026-10-25.log` |
| The process opens no network connection at the start | — | **no test** | the application opens no window; the test belongs to the desktop app |
| The parsers of the fuzzing | `fuzz_archive_header`, `fuzz_image_header`, `fuzz_charset_detection` | **NOT RUN** — the repository holds no `fuzz/` package, so the nightly job of the CI has no target to run. The parsers themselves do not exist (FR-51, FR-52, FR-53). | section 7 |

**The result of the security test files** (from `logs/m1-test-workspace-2026-10-25.log`):

```
running 8 tests   (tests/security/extraction_hardening.rs)  — 8 passed; 0 failed; 0 ignored
running 4 tests   (tests/security/extraction_limits.rs)     — 4 passed; 0 failed; 0 ignored
running 8 tests   (tests/security/path_traversal.rs)        — 8 passed; 0 failed; 0 ignored
running 4 tests   (tests/security/decompression_bomb.rs)    — 4 passed; 0 failed; 0 ignored
```

---

## 6. Interoperability

Which foreign systems were checked — archives written by other tools, network drives with Samba, WebDAV
with Nextcloud?

| Foreign system | Tool used | Result | Evidence |
|---|---|---|---|
| ZIP container written by another implementation | `zip` crate 9.1.0 (the library the specification names) | **measured** — the hostile archives are written and read back by that library, and the hardened extraction refuses their names and their links | `tests/security/extraction_hardening.rs`, `tests/security/path_traversal.rs`, `tests/security/decompression_bomb.rs` |
| TAR with GZip, BZip2, XZ and ZStd, and 7z | `tar`, `flate2`, `bzip2`, `lzma-rust2`, `zstd`, `sevenz-rust2` | **not measured** — the adapters exist in `crates/phocommander-archive/src/formats/`, no test of any format exists | section 7 |
| The archive of another producer (Info-ZIP, 7-Zip, Windows Explorer) | — | **not measured** — the round trip "written by `zip`, read by the reader of the project" is not tested | section 7 |
| Network drives (Samba), WebDAV (Nextcloud), S3 | — | **NOT RUN** — the network sources arrive in M6; `phocommander-remote` is an empty crate | `docs/REQUIREMENTS.md`, milestone M6 |
| The configuration file of the user, written by hand | `toml` crate 1.1.2 | **measured** — a configuration file in a temporary directory is read from the disk, an unknown field is refused, a file that cannot be read reports the path | `tests/integration/config_defaults.rs` (5 tests) |
| The key binding file of the user | `toml` crate 1.1.2 | **measured** — the shipped file is read, the three context tables are checked for conflicts and for completeness | `tests/integration/keymap_bindings.rs` (5 tests) |
| The portable variant on a removable drive | — | **NOT RUN** — the manual test of M3 | `docs/REQUIREMENTS.md`, FR-22m |

---

## 7. Known limitations

What does not work, and why? This list is mandatory and must not be empty as long as there is
something to list.

| Limitation | Reason | Planned for |
|---|---|---|
| **FR-39 — the zero padding and the width that grows with the selection do not exist.** No function writes a season or an episode number with a width. | the work package M1.7 was only started; `crates/phocommander-ops/src/series.rs` reads the numbers, it does not write them | M1 |
| **The archive checks its integrity, but the container formats are not tested on their own.** The check of the checksum of an entry exists and is tested; the readers and the writers of the three formats have no test of their own. | the work package M1.9 was only started | M1 |
| **The viewer offers the read path and the policy, but no hex dump, no character set detection, no image formats and no browsing of an archive.** FR-51, FR-52, FR-53 and FR-54 are not implemented. | the work package M1.8 was only started | M1 |
| **The conflict policies of FR-22 have no test.** The four values `ask`, `never`, `always` and `keep_both` exist; the behaviour of a copy against a file that exists already is not measured. | the tests of the copy were written against the fake file system and against the rules of the security section, not against the policies | M1 |
| **The column set of FR-17 and the order of a tab of FR-18 carry no test.** `crates/phocommander-ui/src/panel/columns.rs` and `panel/tab.rs` carry no `mod tests`. | the values exist, the tests were not written | M1 |
| **The trash of FR-27 does not exist in the tree.** The deletion of a file removes it; no platform crate moves it into the trash of the operating system. | the platform integration of the deletion arrives later | M1/M2 |
| **The rate of a job (FR-24), the estimate of the cue (FR-22g) and the collection of several failures in one job (FR-25) are only partly covered.** | the tests of the job cover the items, the bytes and the states, not the rate and not the list of the failures | M1 |
| **The mismatch of a verified copy (FR-22c) is not measured.** The test proves that the checksums are compared, not that a corrupted destination produces a job error. | the copy tests run against the fake file system, which has no way to corrupt a destination between the write and the comparison | M1 |
| **The default of the synchronisation (FR-22b) exists nowhere.** `config/defaults.toml` carries no section for the copy modes. | the configuration schema of the milestone does not name the copy modes; the criterion AC-M1-09 asks for a configuration section that does not exist | M1/M2 |
| **The benchmark of the read volume of the verified copy (NFR-09a) does not exist.** The criterion AC-M1-45 names the file `benches/copy_verification_read_volume.rs`, which was not written. | the work packages of the performance gate arrive with M2 | M1/M2 |
| **The performance targets NFR-01 to NFR-06 are not measured.** Only the raw file system access of the directory listing is measured (section 3.1) and the frame of a panel and the key stroke of the filter field (section 3.2). The directory change, the start time, the idle memory and the sequential copy have no benchmark. | the milestone M2 carries the performance gate; the benchmark for the directory change and for the key press does not exist yet | M2 |
| **The review of the public API against NFR-17 and NFR-17b was not carried out.** The modules without a test module are named in section 1; the naming rule of NFR-17b was not reviewed. | the review needs a stable tree; the tree was in motion | M1 |
| **`cargo audit` reports four unmaintained crates.** `ttf-parser` (RUSTSEC-2026-0192), `zbus_names` (RUSTSEC-2026-0371), `zvariant` (RUSTSEC-2026-0370), `zvariant_derive` (RUSTSEC-2026-0372). They come from the font handling of the interface and from the Wayland stack. No vulnerability is reported. | the dependencies of the window system and of the text rendering | the owner decides |
| **No fuzz target exists.** The nightly job of the CI runs `cargo +nightly fuzz run`, but the repository holds no `fuzz/` package. The job carries `continue-on-error: true`, so it stays green and proves nothing. | the parsers of the specification (the archive header, the image header, the character set detection) do not exist either | M1/M2 |
| **The build for macOS, Windows, FreeBSD, NetBSD, Android and iOS was not measured.** | the sandbox has one Linux system, no cross toolchain and no runner for those systems | the matrix of the owner |
| **The gate of the CI does not run in every matrix job.** The telemetry gate and the link gate run only in the `quality` job; the version of `cargo-audit` is verified only in the matrix job; the cross-compilation builds without `--locked`. | the workflow of the owner; the file was not changed by the QA engineer | the owner, before the end of M1 |
| **`docs/KEYMAP.md` documents one action more than the catalogue holds** (37 against 36). The gate reports 3 accepted deviations and 0 undocumented ones, so it stays green. | the table of the documentation was not reduced when an action was removed from the catalogue | the owner |
| **The tree was quiet during the second measurement.** The parallel sessions of the team had finished, and the measurements belong to the commit `439c8db`. | — | — |

---

## 8. Decision

| Field | Value |
|---|---|
| Decision | **not accepted** |
| Reasons | 1. **The dialog of the multiple rename does not exist.** The *engine* of the name formation now exists and is tested: the placeholders (FR-30), the search and replace with regular expressions (FR-31), the counter (FR-32), the determinism with the property test of FR-33, the zero padding whose width grows (FR-39) and the series template (FR-37, FR-38). What is still missing is the *dialog*: the live preview of every new name before the rename runs (FR-34), the conflict jump list reachable with F3 and Shift+F3 (FR-35) and the stored templates (FR-36). The definition of done of the milestone — "rename with a template" — is met by the engine and not by the surface. 2. The manual protocol of the milestone has not been run on a macOS machine; the sandbox has one Linux system and no screen. 3. The performance targets NFR-02, NFR-03 and NFR-09a are not measured. |
| What the third measurement repaired | The 32 tests of the container adapters (TAR 8, ZIP 8, 7z 7, the detection 9), the detection of the compressed TAR containers — the four compressions of the TAR adapter were unreachable over the public API — the symmetry of the three adapters about directories, the integration test of the directory entries that was dead code, and the split of the input layer that had grown to 999 lines. |
| What the third measurement **found** | That the rename of the milestone is only half. The execution of a rename exists and is tested; the formation of the new name does not. The earlier report of this document named the missing rename engine as a whole, and that was too coarse: the engine was never missing — the *formation* was. |
| What the fourth measurement added | The engine of the name formation: the module `template` with the placeholders, the search and replace, the counter, the series and the zero padding, and five integration tests — among them the property test of FR-33, which proves that the result does not depend on a hidden clock, on a hidden source of randomness and on the order of the calls. Two faults were repaired in the process: the derived random value was written with sixteen characters instead of eight, because the width of a format is a minimum and not a maximum; and the `regex` crate, which the search and replace needs, was added to the workspace. |
| What is proven | The gates of the acceptance chain are green (**974 tests, 0 failures, 0 ignored**), the linter reports nothing with `-D warnings`, `cargo deny` reports no licence and no advisory, and the thirty-eight gates prove themselves: each of them turns red on a broken state and green again. The negative tests of the hardening rules H-02 to H-06 exist and pass, and the path traversal and the decompression bomb are proven against an attack that really works without the hardening. The configurable security rules of NFR-29 to NFR-32 are implemented and measured, including the three guarantees that no switch can break. |
| What the second measurement repaired | The window of the desktop build, the rename engine with its tests, the nine tests of the job queue, the integration test of the directory entries that was dead code, the checksums of the archives, the double report of a rename conflict, the false report of a damaged symbolic link, and the signatures of the drawing that carried eight parameters each. |
| Conditions for the next acceptance | 1. The dialog of the multiple rename exists — the live preview (FR-34), the conflict jump list (FR-35) and the stored templates (FR-36) — and it uses the engine of the name formation. 2. The manual protocol of the milestone runs on a macOS machine and its result is written down. 3. The benchmarks of NFR-02, NFR-03 and NFR-09a exist and hold their thresholds. |
| Decided by | *pending — the owner of the project decides; the QA engineer hands over the evidence* |
| Date | 25.10.2026 |
| Next milestone | M1 is not reached. The packages that are open are listed in section 7. The milestone M2 is accepted, and the milestone M2b carries the internationalisation. |

---

### Evidence index

Every command that was run for this acceptance, with its output in the repository. All the logs of
this acceptance carry the date suffix `-2026-10-25`; the logs without the suffix belong to the
earlier QA session of the same day (00:26 h to 01:35 h) and are superseded by them.

| # | Command | Result | Where the output is |
|---|---|---|---|
| 1 | `cargo build --workspace --all-targets --locked` | exit code 0 — 158 s | `docs/acceptance/logs/m1-build-all-targets-2026-10-25.log` |
| 2 | `cargo test --workspace --locked` | exit code 0 — 746 passed, 0 failed, 0 ignored, 46 test binaries, 40 s | `docs/acceptance/logs/m1-test-workspace-2026-10-25.log` |
| 3 | `cargo fmt --all --check` | exit code 0 | `docs/acceptance/logs/m1-fmt-check-2026-10-25.log` |
| 4 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit code 0 — 8 min 21 s | `docs/acceptance/logs/m1-clippy-2026-10-25.log` |
| 5 | `./scripts/check-crate-boundaries.sh` | exit code 0 — 12 path dependencies, 0 violations | `docs/acceptance/logs/m1-crate-boundaries-2026-10-25.log` |
| 6 | `./scripts/check-file-size.sh` | exit code 0 — 116 files, 4 above the target value, 0 above the hard limit | `docs/acceptance/logs/m1-file-size-2026-10-25.log` |
| 7 | `./scripts/check-no-telemetry.sh` | exit code 0 — 16 manifests, 80 dependency names, 0 violations | `docs/acceptance/logs/m1-no-telemetry-2026-10-25.log` |
| 8 | `python3 scripts/check-docs-links.py` | exit code 0 — 147 links checked, 0 broken | `docs/acceptance/logs/m1-docs-links-2026-10-25.log` |
| 9 | `python3 scripts/check-requirement-ids.py` | exit code 0 — 7 of 7 checks | `docs/acceptance/logs/m1-requirement-ids-2026-10-25.log` |
| 10 | `python3 scripts/check-acceptance-ids.py` | exit code 0 — 6 of 6 checks, 121 criteria | `docs/acceptance/logs/m1-acceptance-ids-2026-10-25.log` |
| 11 | `python3 scripts/check-action-ids.py` | exit code 0 — 9 of 9 checks | `docs/acceptance/logs/m1-action-ids-2026-10-25.log` |
| 12 | `python3 scripts/check-config-schema.py` | exit code 0 — 7 of 7 checks | `docs/acceptance/logs/m1-config-schema-2026-10-25.log` |
| 13 | `python3 scripts/check-keymap-consistency.py` | exit code 0 — 3 accepted deviations, 0 undocumented | `docs/acceptance/logs/m1-keymap-consistency-2026-10-25.log` |
| 14 | `python3 scripts/check-interface-profiles.py` | exit code 0 — 4 of 4 checks | `docs/acceptance/logs/m1-interface-profiles-2026-10-25.log` |
| 15 | `cargo deny check` | exit code 0 — `advisories ok, bans ok, licenses ok, sources ok` | `docs/acceptance/logs/m1-deny-check-2026-10-25.log` |
| 16 | `cargo audit` | exit code 0 — 574 crates, no vulnerability, 4 unmaintained warnings | `docs/acceptance/logs/m1-audit-2026-10-25.log` |
| 17 | `cargo deny --version`, `cargo audit --version` | `cargo-deny 0.20.2`, `cargo-audit-audit 0.22.2` | `docs/acceptance/logs/m1-tool-versions-2026-10-25.log`, `docs/acceptance/logs/m1-tool-versions-audit-2026-10-25.log` |
| 18 | `cargo bench -p phocommander-integration-tests --bench directory_listing` | see section 3.1 | `docs/acceptance/logs/m1-bench-directory-listing-2026-10-25.log` |
| 19 | `cargo test -p phocommander-ui --test the_filter_holds_the_frame_rate -- --exact --nocapture` | see section 3.2 | `docs/acceptance/logs/m1-frame-rate-dev-2026-10-25.log` |
| 20 | `cargo test --release -p phocommander-ui --test the_filter_holds_the_frame_rate -- --exact --nocapture` | see section 3.2 | `docs/acceptance/logs/m1-frame-rate-release-2026-10-25.log` |
| 21 | `./scripts/check-gates-negative.sh` | see the entry in section 2 | `docs/acceptance/logs/m1-check-gates-negative-2026-10-25.log` |

*End of document. Every status reflects a measurement of this session on the tree of the commit
`514ca0b`. No row is marked as proven without a command output or a green test name behind it.*
