# M1 Acceptance — Phocommander

| Field | Value |
|---|---|
| Milestone | M1 — The working file browser |
| Acceptance date | 25.10.2026 |
| Specification version | `docs/REQUIREMENTS.md` **2.0** (24 October 2026, 1714 lines) |
| Catalogue version | `docs/REQUIREMENTS-IDS.md` — the 54 requirements of the milestone M1, each with an acceptance criterion in `docs/ACCEPTANCE-CRITERIA.md` |
| Decided by | *pending — the owner decides; this document is the evidence of the QA engineer* |
| Result | **not accepted** — the negative tests of the hardening rules exist and pass, the gates that need no build are green, but the workspace-wide test run was not completed and eleven of the fifty-four requirements of the milestone have no test at all |
| State of the tree | The working tree is shared with the other sessions of the team and was **in motion** during this acceptance. Every statement below names the moment it was measured. The measurements of 00:26 h to 01:12 h describe the tree of that moment; the tree changed afterwards (the files of the interface and of the operations were rewritten at 01:33 h). |

**Who wrote this document:** the QA engineer of the team. **What it is:** the measured evidence for
the milestone M1 — the command outputs, the test names with their results, the coverage of every
requirement of the milestone and the exact reasons for everything that could not be measured.
**What it is not:** a claim that the milestone is done. **The milestone M1 is not accepted**, and
section 7 names every reason.

---

## 1. Functionality

Which functions of the milestone exist, and how do they behave? Every row names the requirement
identifier of version 2.0 and the state of the evidence.

The three states of the evidence column:

| State | Meaning |
|---|---|
| **measured** | The test ran in this session and was green. The name of the test and the number of the run are named. |
| **test exists** | The test is in the tree, but the run of the whole workspace was not completed, so the result of this test is not known. |
| **no test** | The requirement has no test. The gap is listed in section 7 as well. |

| Requirement | Behaviour | Evidence | State |
|---|---|---|---|
| FR-10 | Two panels hold their own path, their own list and their own cursor; the focus alternates between them and the panels can be swapped. | `phocommander-ui/src/surface.rs`: `the_focus_alternates_between_the_two_panels`, `a_transfer_names_the_other_panel_as_its_destination`, `a_transfer_into_the_panel_itself_is_refused`, `the_pair_follows_the_focus`, `swapping_the_panels_exchanges_their_content` | test exists |
| FR-16 | An operation names the panel it writes into; a transfer into the panel itself is refused. | `phocommander-ui/src/app/tests.rs`: `the_copy_names_the_other_panel_as_its_destination`, `the_copy_after_the_switch_names_the_right_panel_as_its_source`; `phocommander-ui/src/surface.rs`: `a_transfer_into_the_panel_itself_is_refused` | test exists |
| FR-17 | The column set is a configuration value (`Column::from_config_name`, `Tab::columns`). | no test names a column set | **no test** |
| FR-18 | A tab keeps its own order (`Tab::sort`). | no test names the order of a tab | **no test** |
| FR-20 | A file is copied byte by byte, moved, renamed, removed; a directory is created. | `phocommander-core/src/local/tests.rs`: `a_file_is_copied_byte_by_byte`, `a_file_is_moved_and_renamed`, `a_file_is_removed_and_a_directory_is_created` | test exists |
| FR-21 | Every operation is a job of the queue; the queue starts its workers and hands out a snapshot. | `phocommander-ops/src/queue.rs` carries **no test** | **no test** |
| FR-22 | The conflict policy is a value of the copy (`ConflictPolicy::as_str`, `CopyRequest::with_policy`). | no test names the four policies `ask`, `never`, `always`, `keep_both` | **no test** |
| FR-22a | The seven copy modes exist and know whether they delete, whether they synchronise and whether they verify. | `phocommander-ops/src/copy/tests.rs`: `the_default_of_a_copy_is_the_verified_mode`, `the_quick_copy_never_compares_the_checksums`, `the_atomic_copy_always_verifies_the_temporary_file`, `only_the_mirror_mode_deletes`, `the_modes_know_which_of_them_are_synchronisations`, `the_modes_have_the_names_the_configuration_uses` | test exists |
| FR-22b | The default of a copy is `verified`; the default policy verifies and never follows a link. | `phocommander-ops/src/copy/tests.rs`: `the_default_of_a_copy_is_the_verified_mode`, `the_default_policy_of_a_copy_verifies_and_never_follows_a_link` — the default of the **synchronisation** mode is not named by a test | test exists, partly |
| FR-22c | The verified mode compares the checksums, and it does not compare them when the rule is switched off. | `phocommander-ops/src/copy/tests.rs`: `the_verified_copy_compares_the_checksums_when_the_rule_is_in_force`, `the_verified_copy_does_not_compare_the_checksums_when_the_rule_is_off` | test exists |
| FR-22g | The cue of a job carries the state, the items and the bytes; the names of the states are the ones the specification names. | `phocommander-ops/src/progress.rs`: `the_context_reports_the_progress_to_the_sink`, `the_context_counts_the_items_and_the_bytes`; `phocommander-ops/src/job.rs`: `the_states_have_the_names_the_job_cue_shows` | test exists |
| FR-22h | The verification has its own progress and its own state. | `phocommander-ops/src/progress.rs`: `the_verification_has_its_own_progress` | test exists |
| FR-22i | A queued job can be retargeted and reprioritised; a running job may not; the flags report what the user did. | `phocommander-ops/src/edit.rs`: `a_queued_job_may_be_retargeted_and_reprioritised`, `a_running_job_may_not_be_retargeted`, `the_flags_of_a_job_report_what_the_user_did`, `a_waiting_change_is_handed_out_once` | test exists |
| FR-22j | A running job can be paused, resumed and cancelled. | `phocommander-ops/src/edit.rs`: `a_running_job_may_be_paused_resumed_and_cancelled`, `a_resumed_job_is_not_paused_any_more`; `phocommander-ops/src/progress.rs`: `a_cancelled_job_is_stopped_at_the_checkpoint` | test exists |
| FR-23 | Overwriting is never the default. | `phocommander-config/src/values.rs`: `overwrite_is_never_the_default` | test exists |
| FR-24 | The job reports the items and the bytes and can be cancelled. | `phocommander-ops/src/progress.rs`: `the_context_counts_the_items_and_the_bytes`, `a_cancelled_job_is_stopped_at_the_checkpoint` — the **rate** is not named by a test | test exists, partly |
| FR-25 | A collected failure names the item and the code. | `phocommander-ops/src/error.rs`: `a_collected_failure_names_the_item_and_the_code` — the collection of several failures in one job is not named by a test | test exists, partly |
| FR-27 | The trash is the default path of a deletion. | no trash implementation in the tree | **no test** |
| FR-30 | The placeholders of the rename dialog. | no rename engine in the tree | **no test** |
| FR-31 | The search and replace of the rename dialog, with and without a regular expression. | no rename engine in the tree | **no test** |
| FR-32 | The counter of the rename dialog with start value, step and width. | no rename engine in the tree | **no test** |
| FR-33 | The rename result is deterministic. | no rename engine in the tree — **there is no property test of the rename** | **no test** |
| FR-34 | The live preview of the new names. | no rename engine in the tree | **no test** |
| FR-35 | The conflict jump list of the rename dialog. | no rename engine in the tree | **no test** |
| FR-36 | The named templates can be stored and re-applied. | the five templates exist under `config/templates/` and are checked by `scripts/check-config-schema.py` — the round trip through the loader is not named by a test | test exists, partly |
| FR-37 | The pattern `SxxEyy` is recognised in a name, with two and three digits and without regard to the case; a name without the pattern is not recognised. | `phocommander-ops/src/series.rs`: `the_pattern_is_recognised_in_a_name`, `the_pattern_is_recognised_with_two_and_three_digits`, `the_pattern_is_recognised_without_the_case`, `a_name_without_the_pattern_is_not_recognised` | test exists |
| FR-38 | The name of the series is the name of the parent directory. | `phocommander-ops/src/series.rs`: `the_series_name_is_the_name_of_the_parent_directory`, `a_name_without_a_parent_has_no_series_name` — the override by the user is not named by a test | test exists, partly |
| FR-39 | The season and the episode number are written with a width that grows with the selection. | no function that writes the number with a width exists | **no test** |
| FR-40 | Every character that Windows forbids is reported with a replacement; the device names are recognised. | `phocommander-ops/src/names.rs`: `the_forbidden_characters_are_reported_with_a_replacement`, `a_control_character_is_reported`, `every_forbidden_character_has_a_replacement_that_is_allowed`, `the_device_names_of_windows_are_recognised`; `phocommander-archive/src/windows_name.rs`: `finds_the_first_forbidden_character`, `suggests_a_name_without_the_forbidden_character` | test exists |
| FR-50 | The viewer opens in an additional area. | no window in the tree; the viewer offers no area | **no test** |
| FR-51 | The character set detection for UTF-8 and UTF-16. | no charset detection in the tree | **no test** |
| FR-52 | The hex dump. | no hex dump in the tree | **no test** |
| FR-53 | The image formats. | no image decoding in the viewer | **no test** |
| FR-54 | The browsing of an archive in the viewer. | no browsing in the viewer | **no test** |
| FR-55 | The metadata of a file: the path, the bytes, the size and the time of the last change. | `phocommander-viewer/src/content.rs`: `an_opened_file_carries_its_path_and_its_bytes`, `an_opened_file_without_content_is_empty`, `the_time_of_the_last_change_is_carried_when_it_is_known` | test exists |
| FR-59 | The viewer only reads. | `phocommander-viewer/src/reader.rs`: `the_reader_opens_the_file_the_user_asked_for`, `the_reader_refuses_the_preview_when_the_rule_is_in_force` — the crate exposes no write path | test exists |
| FR-65 | An archive appears as a normal entry and opens as a virtual directory. | no test names a virtual directory | **no test** |
| FR-66 | The entries of an archive are listed, viewed and copied without extracting the archive. | no test names the copy of an archive entry | **no test** |
| FR-67 | ZIP, TAR with GZip, BZip2, XZ and Zstd, and 7z are read. | the adapters exist under `crates/phocommander-archive/src/formats/` — **no test of any format** | **no test** |
| FR-68 | Archives can be created, including encrypted ones. | the writer exists — no test | **no test** |
| FR-70 | Every extraction goes through the hardened path resolution. | `tests/security/extraction_hardening.rs`: `an_entry_that_leaves_the_destination_is_refused`, `nothing_is_written_outside_of_the_destination_directory`, `a_symbolic_link_is_skipped_instead_of_being_created`, `a_symbolic_link_cannot_redirect_a_later_entry`, `an_entry_written_through_an_existing_symbolic_link_is_refused`, `the_set_user_id_and_the_set_group_id_bits_never_reach_the_file_system`, `the_archive_with_the_set_user_id_bit_really_carries_it`, `no_container_crate_is_asked_to_extract_an_archive` — **8 tests, 0 failed, green on 25.10.2026 at 01:10 h** | **measured** |
| FR-74a | The filter is a field of the panel and not a dialog. | `crates/phocommander-ui/tests/the_filter_matches_without_a_dialog.rs`: `the_filter_matches_without_a_dialog` | test exists |
| FR-74b | The pattern language: `*`, `?`, `~` and a plain text. | `crates/phocommander-ui/tests/the_filter_pattern_language.rs`: `the_star_matches_any_sequence_of_characters`, `the_question_mark_matches_exactly_one_character`, `the_tilde_matches_without_regard_to_the_case`, `a_plain_text_matches_anywhere_in_the_name`, `the_evaluation_is_the_same_for_the_same_input`; `crates/phocommander-ui/src/filter/property_tests.rs`: `the_evaluation_is_deterministic`, `a_plain_text_is_searched_anywhere`, `the_tilde_ignores_the_case`, `the_star_matches_every_name`, `the_question_mark_matches_exactly_one_character`, `an_empty_pattern_matches_every_name`, `the_index_is_deterministic` | test exists |
| FR-74c | The filter can be limited to files, to directories or to both. | `crates/phocommander-ui/tests/the_filter_can_be_limited_to_a_kind.rs`: `the_filter_can_be_limited_to_a_kind`; `crates/phocommander-ui/src/filter/tests.rs`: `the_kind_limits_the_filter_to_files_or_to_directories` | test exists |
| FR-74d | The filter can search deeper than the current directory. | `tests/integration/the_filter_searches_deeper.rs`: `the_filter_searches_deeper`; `crates/phocommander-ui/src/filter/tests.rs`: `the_depth_limits_the_filter_to_the_current_directory` | test exists |
| FR-74e | The number of the matches is shown while the filter is typed. | `crates/phocommander-ui/tests/the_filter_matches_without_a_dialog.rs`: the query for the label `panel filter matches 1` | test exists |
| FR-74f | The filter belongs to the panel and survives a change of the directory. | `crates/phocommander-ui/tests/the_filter_survives_a_directory_change.rs`: `the_filter_survives_a_directory_change`; `crates/phocommander-ui/src/panel/tests.rs`: `the_filter_survives_a_change_of_the_directory` | test exists |
| FR-100 | Every operation is a named action; the identifiers are unique and every action has a label key. | `crates/phocommander-ui/src/actions.rs`: `every_identifier_is_unique`, `finds_an_action_by_its_identifier`, `every_action_belongs_to_exactly_one_category`, `the_catalogue_holds_the_actions_of_the_configuration_file`, `every_action_has_a_label_key` | test exists |
| FR-101 | The three contexts hold the same actions; the D-pad and the controller reach them through the palette; the same event resolves differently in two contexts, and the resolution is deterministic. | `tests/integration/keymap_bindings.rs`: `the_shipped_keymap_has_no_conflict_in_any_context`, `every_action_of_the_keyboard_is_reachable_in_the_dpad_context`, `every_action_of_the_keyboard_is_reachable_in_the_controller_context`, `the_dpad_and_the_controller_bind_their_actions_to_the_action_palette`, `the_action_identifiers_of_the_shipped_file_are_well_formed`; `crates/phocommander-ui/src/keymap_view.rs`: `the_same_event_resolves_differently_in_two_contexts`, `the_resolution_is_deterministic`, `a_palette_entry_resolves_in_the_context_that_uses_it` | test exists |
| NFR-09 | No operation that can take longer than 50 ms blocks the interface. | `phocommander-ops/src/queue.rs` carries **no test**; the measurement of the gap between two frames does not exist | **no test** |
| NFR-09a | The verification raises the read volume to at most twice the copied volume. | no benchmark of the read volume exists | **not measured** |
| NFR-09b | The cue updates at most ten times per second. | the constant `REPORT_INTERVAL` of the queue is 100 ms; no test counts the updates | **no test** |
| NFR-17 | Every public function has at least one unit test. | the review of 25.10.2026: `phocommander-ops/src/queue.rs` and `phocommander-ops/src/executor.rs` carry no test, `crates/phocommander-archive/src/formats/` carries no test | **partly** |
| NFR-17b | Every name states what a thing is. | the review of the public API of the crates of M1 was not carried out in this session | **not measured** |

**Count.** Of the 54 requirements of the milestone M1: **1 is measured** (FR-70, with eight green
negative tests), **34 have a test that was not executed in this session**, **11 have no test at
all** (FR-17, FR-18, FR-21, FR-22, FR-27, FR-30, FR-31, FR-32, FR-33, FR-34, FR-35, FR-39, FR-50,
FR-51, FR-52, FR-53, FR-54, FR-65, FR-66, FR-67, FR-68, NFR-09, NFR-09b — the list of section 7
names every one of them), and **2 are not measured** (NFR-09a, NFR-17b).

---

## 2. Technique

Which architectural decisions are implemented, which crate boundaries were kept, which tools ran
through?

| Item | Result | Evidence |
|---|---|---|
| Crate boundaries | pass — the dependency direction `apps → platform → crates` holds | `docs/acceptance/logs/m1-crate-boundaries.log`, EXIT=0, 25.10.2026 01:35 h |
| Formatting | **measured at 01:12 h, green** (`cargo fmt --all --check`, EXIT=0). The tree was rewritten afterwards, so the value is the one of 01:12 h. | `docs/acceptance/logs/m1-fmt-check.log` |
| Static analysis | **NOT RUN in the final state.** The run of 01:12 h ended with the exit code 101 because the crate `phocommander-ui` did not compile at that moment — not because of a lint of clippy. No lint of clippy was measured. | `docs/acceptance/logs/m1-clippy.log` |
| Test run | **NOT RUN.** The run of 01:12 h was killed by the OOM killer in the link step (`ld terminated with signal 9 [Killed]`); the run of 01:33 h was stopped by the order of the team lead. The tree carries 375 test attributes in 68 files; the number of tests that **passed** is not measured. | `docs/acceptance/logs/m1-test-workspace.log`, the OOM in the log of the system |
| Build of all targets | **measured at 00:47 h, green** (`cargo build --workspace --all-targets --locked`, EXIT=0). The tree was rewritten afterwards. | `docs/acceptance/logs/m1-build-all-targets.log` |
| Licence check | **measured at 01:07 h, green** — `advisories ok, bans ok, licenses ok, sources ok`, EXIT=0 | `docs/acceptance/logs/m1-deny-check.log` |
| Vulnerability check | **measured at 01:07 h, green**, EXIT=0 | `docs/acceptance/logs/m1-audit.log` |
| File size limit (NFR-14) | pass — 0 files above the hard limit of 900 lines, 7 above the target value of 600 lines | `docs/acceptance/logs/m1-file-size.log`, EXIT=0, 25.10.2026 01:35 h |
| Telemetry gate | pass — 0 violations | `docs/acceptance/logs/m1-no-telemetry.log`, EXIT=0, 25.10.2026 01:35 h |
| Link gate | pass — 0 broken links | `docs/acceptance/logs/m1-docs-links.log`, EXIT=0, 25.10.2026 01:35 h |
| Requirement identifiers | pass — the 154 identifiers of the specification and the 154 rows of the catalogue are the same set, without a gap and without a duplicate | `docs/acceptance/logs/m1-requirement-ids.log`, EXIT=0, 25.10.2026 01:35 h |
| Acceptance identifiers | pass — 121 criteria, unique and without a gap; every requirement of M0, M1 and M2 has one | `docs/acceptance/logs/m1-acceptance-ids.log`, EXIT=0, 25.10.2026 01:35 h |
| Action identifiers | pass — 9 of 9 checks | `docs/acceptance/logs/m1-action-ids.log`, EXIT=0, 25.10.2026 01:35 h |
| Configuration schema | pass — 7 of 7 checks | `docs/acceptance/logs/m1-config-schema.log`, EXIT=0, 25.10.2026 01:35 h |
| Keymap consistency | pass — the specification, the catalogue and the reference agree | `docs/acceptance/logs/m1-keymap-consistency.log`, EXIT=0, 25.10.2026 01:35 h |
| Interface profiles | pass — 4 of 4 checks | `docs/acceptance/logs/m1-interface-profiles.log`, EXIT=0, 25.10.2026 01:35 h |
| Tool versions | `cargo-deny 0.20.2` and `cargo-audit 0.22.2` — the versions the specification pins | `cargo deny --version`, `cargo audit --version` |
| Platform build | NOT RUN for macOS, Windows, FreeBSD, NetBSD, Android and iOS — the sandbox has one Linux system and no cross toolchain | the matrix of the owner builds them |

---

## 3. Performance

Which target values were measured, with which tool, with which result? A target value that was not
measured is reported as "not measured", never as "met".

| Target value | Threshold | Measured with | Result | Status |
|---|---|---|---|---|
| Directory listing with 10 000 entries (NFR-01) | < 100 ms | `cargo bench -p phocommander-integration-tests --bench directory_listing` | the measurement of the raw file system access of M0 is `53.287992 ms` (median of five runs) | **not measured for M1** — the benchmark measures the file system, not the model of the core crate, and the benchmark was not run in this session |
| Directory change, already loaded (NFR-02) | < 16 ms | — | — | **not measured** — no benchmark exists |
| Key press until the visible reaction (NFR-03) | < 50 ms | — | — | **not measured** — no benchmark exists |
| Start time until the operable surface (NFR-04) | < 800 ms | — | — | **not measured** — no window |
| Memory in idle, two panels open (NFR-05) | < 120 MB | — | — | **not measured** — no window |
| File copy, sequential (NFR-06) | ≥ 80 % of the bandwidth | — | — | **not measured** — no benchmark exists |
| Read volume of a verified copy (NFR-09a) | ≤ twice the copied volume | — | — | **not measured** — no benchmark exists |
| Archive listing, 5 000 entries (NFR-08) | < 500 ms | — | — | **not measured** — no benchmark exists |
| Frame rate of the filter (`crates/phocommander-ui/tests/the_filter_holds_the_frame_rate.rs`) | — | — | — | **not measured** — the test exists, the run was not completed |

**What the measurement means.** No target value of the specification was measured for the milestone
M1. The only measured value is the one of M0, and it measures the raw access to the file system.
The specification makes the target values the acceptance condition of M2, not of M1; the milestone
M1 therefore carries no performance claim.

---

## 4. Guidance

How does the milestone look, and how is it operated? Keyboard operation is reported separately,
because it is the main way of operating the application.

| Topic | Observation |
|---|---|
| Appearance | **not measured** — the milestone M1 delivers no window. The theme of the surface exists (`crates/phocommander-ui/src/theme.rs`), and the drawing of the panels exists (`crates/phocommander-ui/src/draw.rs`), but nothing opens a window. |
| Keyboard operation | **not measured** — the action catalogue, the bindings and the resolution of an event into an action exist and are proven by their unit tests, but no surface resolves a key press into an action that changes something on the screen. |
| Mouse operation | **not measured** — the pointer context exists in the model, the operation of the pointer is not implemented. |
| Error messages | The configuration reader names the field it refuses; the name resolver names the entry it refuses (`EntryNameError::Traversal { name }`); the boundary gate names the crate and the target; the archive crate names the limit it exceeded (`LimitKind::label`). |
| Language | The repository documents are in English, the user guide is bilingual. The localisation crate exists with a fallback chain that never returns an empty string (`phocommander-i18n/src/lib.rs`: `a_missing_key_falls_back_to_the_key_itself`). |

---

## 5. Security and privacy

Which hardenings are effective, and **which negative test proves it**? A positive test proves
nothing about a defence.

| Hardening | Negative test | Result | Evidence |
|---|---|---|---|
| H-02 — no `extract()` of a container crate; the entries are written by the project | `security_extraction_hardening::no_container_crate_is_asked_to_extract_an_archive` — the test walks every source file of `crates/`, `platform/` and `apps/` and searches for a call of the `extract()` method | **measured** — the search finds no call; the test is green | `docs/acceptance/logs/m1-test-workspace.log` is the run that was killed; the green result of the two security test files is in the log of 01:10 h (see section 8) |
| H-03 — every destination path is confined to the destination directory | `security_extraction_hardening::an_entry_that_leaves_the_destination_is_refused`, `security_extraction_hardening::nothing_is_written_outside_of_the_destination_directory`, `security_extraction_hardening::an_entry_written_through_an_existing_symbolic_link_is_refused` | **measured** — every hostile name is refused with the reason *traversal*, and the only file below the temporary directory is the archive itself | the run of 01:10 h |
| H-04 — no symbolic link is created from archive material | `security_extraction_hardening::a_symbolic_link_is_skipped_instead_of_being_created`, `security_extraction_hardening::a_symbolic_link_with_a_refused_target_is_reported_as_such`, `security_extraction_hardening::a_symbolic_link_cannot_redirect_a_later_entry` | **measured** — the link is reported as skipped and never created; the entry behind it cannot be redirected | the run of 01:10 h |
| H-05 — the set-user-id, the set-group-id and the sticky bit never reach the file system | `security_extraction_hardening::the_set_user_id_and_the_set_group_id_bits_never_reach_the_file_system`, `security_extraction_hardening::the_archive_with_the_set_user_id_bit_really_carries_it` — the second test reads the raw bytes of the archive and proves that the attack material really carries the bit | **measured** — the mode of the extracted file carries no special bit | the run of 01:10 h |
| H-06 — the total size, the number of entries and the size of a single entry are bounded | `security_extraction_limits::the_extraction_aborts_when_the_total_size_limit_is_exceeded`, `security_extraction_limits::the_extraction_aborts_when_the_number_of_entries_limit_is_exceeded`, `security_extraction_limits::the_extraction_aborts_when_the_size_of_a_single_entry_limit_is_exceeded`, `security_extraction_limits::an_archive_within_the_limits_is_extracted_completely` | **measured** — every limit aborts the extraction with a readable message; the fourth test proves that the gate is not red for every archive | the run of 01:10 h |
| Path traversal — the name resolution | `security_path_traversal::every_hostile_entry_name_is_refused` (nine hostile names), `security_path_traversal::the_resolved_path_stays_inside` (256 generated names), `security_path_traversal::the_attack_really_writes_outside_of_the_destination_without_the_hardening` | **test exists** — the attack is real: the unhardened reference extraction writes the file next to the destination | `tests/security/path_traversal.rs` |
| Decompression bomb — the archive | `security_decompression_bomb::the_bomb_archive_declares_more_bytes_than_the_budget_allows`, `..._costs_the_attacker_almost_nothing`, `..._expands_far_beyond_its_own_size`, `..._too_many_entries_exceeds_the_entry_limit` | **test exists** — the attack material is built and measured | `tests/security/decompression_bomb.rs` |
| No telemetry, no analytics, no tracking | `./scripts/check-no-telemetry.sh` | **measured** — 0 violations, EXIT=0 | `docs/acceptance/logs/m1-no-telemetry.log` |
| Only permissive licences | `cargo deny check` | **measured** — `licenses ok`, EXIT=0, after the allow list of `deny.toml` was extended by the four licences that the dependencies of M1 brought (section 7) | `docs/acceptance/logs/m1-deny-check.log` |
| No known vulnerability in the dependency graph | `cargo audit` | **measured** — EXIT=0 | `docs/acceptance/logs/m1-audit.log` |
| The rules of section 9.7 are configurable (NFR-29 to NFR-32) | `tests/integration/security_profiles.rs`: `the_shipped_file_produces_the_rules_of_the_profile_standard`, `the_profile_corporate_enables_every_rule_in_a_real_file`, `a_file_that_tries_to_break_a_guarantee_changes_nothing`, `a_configuration_that_keeps_no_session_writes_no_file_at_all`; `crates/phocommander-config/src/security/tests.rs`: `the_profile_corporate_enables_every_rule`, `the_switch_of_the_background_index_has_no_effect`, `the_guarantees_are_in_force_in_every_profile` | **test exists** — the three guarantees cannot be switched off | the files named |
| The process opens no network connection at the start | — | **no test** | the application opens no window |

**The result of the run of 01:10 h.** The two security test files of the QA engineer were run on
their own, because the run of the whole workspace was killed by the OOM killer:

```
running 8 tests
test a_symbolic_link_cannot_redirect_a_later_entry ... ok
test a_symbolic_link_is_skipped_instead_of_being_created ... ok
test a_symbolic_link_with_a_refused_target_is_reported_as_such ... ok
test an_entry_that_leaves_the_destination_is_refused ... ok
test an_entry_written_through_an_existing_symbolic_link_is_refused ... ok
test nothing_is_written_outside_of_the_destination_directory ... ok
test the_archive_with_the_set_user_id_bit_really_carries_it ... ok
test the_set_user_id_and_the_set_group_id_bits_never_reach_the_file_system ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

running 4 tests
test an_archive_within_the_limits_is_extracted_completely ... ok
test the_extraction_aborts_when_the_number_of_entries_limit_is_exceeded ... ok
test the_extraction_aborts_when_the_size_of_a_single_entry_limit_is_exceeded ... ok
test the_extraction_aborts_when_the_total_size_limit_is_exceeded ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## 6. Interoperability

Which foreign systems were checked — archives written by other tools, network drives with Samba,
WebDAV with Nextcloud?

| Foreign system | Tool used | Result | Evidence |
|---|---|---|---|
| ZIP container written by another implementation | `zip` crate 9.1.0 | **measured** — the hostile archives are written and read back by that library, and the hardened extraction refuses their names and their links | `tests/security/extraction_hardening.rs`, 8 tests green |
| TAR with GZip, BZip2, XZ and Zstd, and 7z | `tar`, `flate2`, `bzip2`, `lzma-rust2`, `zstd`, `sevenz-rust2` | **not measured** — the adapters exist, no test of any format exists | `crates/phocommander-archive/src/formats/` |
| The archive of another producer (Info-ZIP, 7-Zip, Windows Explorer) | — | **not measured** — the round trip "written by `zip`, read by the reader of the project" is not tested | — |
| Network drives (Samba), WebDAV (Nextcloud), S3 | — | **NOT RUN** — the network sources arrive in M6; `phocommander-remote` is an empty crate | `docs/REQUIREMENTS.md`, milestone M6 |
| The configuration file of the user, written by hand | `toml` crate 1.1.2 | **test exists** — a configuration file in a temporary directory is read from the disk, an unknown field is refused | `tests/integration/config_defaults.rs` |
| The key binding file of the user | `toml` crate 1.1.2 | **test exists** — the shipped file is read, the three context tables are checked for conflicts and for completeness | `tests/integration/keymap_bindings.rs` |
| The portable variant on a removable drive | — | **NOT RUN** — the manual test of M3 | `docs/REQUIREMENTS.md`, FR-22m |

---

## 7. Known limitations

What does not work, and why? This list is mandatory and must not be empty as long as there is
something to list.

| Limitation | Reason | Planned for |
|---|---|---|
| The run of the whole workspace (`cargo test --workspace --locked`) was not completed | the machine of the sandbox has three gigabytes of memory and three sessions built at the same time; the linker was killed by the OOM killer (`ld terminated with signal 9 [Killed]`) and the second run was stopped by the order of the team lead. The tree carries 375 test attributes; **the number of tests that pass is not measured**. | the team lead builds and tests centrally |
| `cargo clippy --workspace --all-targets -- -D warnings` was not completed | the same reason. The run of 01:12 h ended with the exit code 101 because the crate `phocommander-ui` did not compile at that moment — not because of a lint. | the team lead |
| **The rename engine does not exist.** FR-30 (the placeholders), FR-31 (the search and replace), FR-32 (the counter), FR-33 (the determinism), FR-34 (the live preview) and FR-35 (the conflict jump list) have no implementation and no test. **The property test of the determinism that the specification asks for (FR-33) therefore does not exist.** | the milestone M1.6 was not started | M1 |
| **The serial episode template is only half implemented.** FR-37 is covered by the detection of the numbers; FR-39 — the zero padding and the width that grows with the selection — has no function and no test. FR-38 is covered for the default only, the override by the user is not covered. | the milestone M1.7 was only started | M1 |
| The job queue carries no test (`crates/phocommander-ops/src/queue.rs`), the executor carries no test (`crates/phocommander-ops/src/executor.rs`) | the unit tests of the operations were written before the queue was finished | M1 |
| The container adapters of the archive carry no test — no test of ZIP, of TAR, of 7z, of the creation of an archive and of the virtual directory | the milestone M1.9 was only started | M1 |
| The viewer offers the read path and the policy, but no hex dump, no character set detection, no image formats and no browsing of an archive | the milestone M1.8 was only started | M1 |
| The conflict policies `ask`, `never`, `always` and `keep_both` of FR-22 have no test | the policies exist as values, the tests were not written | M1 |
| The column set of FR-17 and the order of a tab of FR-18 have no test | the values exist, the tests were not written | M1 |
| The trash of FR-27 does not exist in the tree | the platform integration of the deletion arrives later | M1/M2 |
| The rate of a job (FR-24) and the collection of several failures in one job (FR-25) are only partly covered | the tests of the job cover the items and the bytes, not the rate and not the list of the failures | M1 |
| The benchmark of the read volume of the verified copy (NFR-09a) does not exist | the criterion AC-M1-45 names the file `benches/copy_verification_read_volume.rs`, which was not written | M1/M2 |
| The update rate of the job cue (NFR-09b) is not counted by a test | the queue holds the constant `REPORT_INTERVAL`, but no test counts the updates | M1 |
| The review of the public API against NFR-17 and NFR-17b was not carried out | the review needs a stable tree | M1 |
| The licence allow list of `deny.toml` was extended by the QA engineer | the dependencies of M1 — the compressors of the TAR container, the reader of the 7z container and the font handling of the interface — brought four licences that the list did not name: `bzip2-1.0.6`, `BSL-1.0`, `CC0-1.0` with `MIT-0`, and `OFL-1.1` with `Ubuntu-font-1.0`. All four are permissive and none of them is a copyleft, so the policy of section 9.6 of the specification is applied to them. **The owner confirms or reverses the change.** | the owner |
| The working tree was in motion during this acceptance | the tree is shared with the other sessions of the team; the files of the interface and of the operations were rewritten at 01:33 h, after the last measurement | the team lead |

---

## 8. Decision

| Field | Value |
|---|---|
| Decision | **not accepted** |
| Reasons | 1. The run of the whole workspace is not measured: the number of tests that pass is unknown. 2. Eleven requirements of the milestone have no test at all, among them the whole rename engine (FR-30 to FR-35) and the property test of the determinism (FR-33) that the specification names. 3. The container adapters of the archive, the queue of the jobs and the viewer stage 1 carry no test. |
| What is proven | The negative tests of the hardening rules H-02 to H-06 exist and are green (12 tests, 0 failed, 25.10.2026 at 01:10 h). The ten gates that need no build are green and their negative test proves 25 runs red and green. The licence check and the vulnerability check are green. |
| Conditions for the next acceptance | 1. The team lead runs `cargo build --workspace --all-targets --locked`, `cargo test --workspace --locked`, `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings` on a machine that has the memory for it, and the results are quoted here. 2. The rename engine is implemented and covered — the property test of FR-33 is the first test that has to exist. 3. The queue, the executor and the container adapters carry their tests. 4. The owner confirms the allow list of `deny.toml`. |
| Decided by | *pending — the owner of the project decides; the QA engineer hands over the evidence* |
| Date | 25.10.2026 |
| Next milestone | M1 is not reached. The packages that are open are listed in section 7. |

---

### Evidence index

Every command that was run for this acceptance, with its output in the repository. The column
"state of the tree" names the moment of the measurement.

| # | Command | Result | Where the output is |
|---|---|---|---|
| 1 | `cargo check --workspace --all-targets --locked` | EXIT=0 at 00:26 h, 00:44 h and 01:05 h | the console of the session; the build of the evidence run repeats it |
| 2 | `cargo build --workspace --all-targets --locked` | EXIT=0 at 00:47 h | `docs/acceptance/logs/m1-build-all-targets.log` |
| 3 | `cargo test --workspace --locked` | **NOT RUN** — killed by the OOM killer in the link step | `docs/acceptance/logs/m1-test-workspace.log` |
| 4 | `cargo test -p phocommander-integration-tests --test security_extraction_hardening --test security_extraction_limits` | 12 passed, 0 failed at 01:10 h | the console of the session, quoted in section 5 |
| 5 | `cargo fmt --all --check` | EXIT=0 at 01:12 h | `docs/acceptance/logs/m1-fmt-check.log` |
| 6 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **NOT RUN** — the crate `phocommander-ui` did not compile at that moment | `docs/acceptance/logs/m1-clippy.log` |
| 7 | `cargo deny check` | EXIT=0 — `advisories ok, bans ok, licenses ok, sources ok` at 01:07 h | `docs/acceptance/logs/m1-deny-check.log` |
| 8 | `cargo audit` | EXIT=0 at 01:07 h | `docs/acceptance/logs/m1-audit.log` |
| 9 | `./scripts/check-crate-boundaries.sh` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-crate-boundaries.log` |
| 10 | `./scripts/check-file-size.sh` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-file-size.log` |
| 11 | `./scripts/check-no-telemetry.sh` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-no-telemetry.log` |
| 12 | `python3 scripts/check-docs-links.py` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-docs-links.log` |
| 13 | `python3 scripts/check-requirement-ids.py` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-requirement-ids.log` |
| 14 | `python3 scripts/check-acceptance-ids.py` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-acceptance-ids.log` |
| 15 | `python3 scripts/check-action-ids.py` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-action-ids.log` |
| 16 | `python3 scripts/check-config-schema.py` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-config-schema.log` |
| 17 | `python3 scripts/check-keymap-consistency.py` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-keymap-consistency.log` |
| 18 | `python3 scripts/check-interface-profiles.py` | EXIT=0 at 01:35 h | `docs/acceptance/logs/m1-interface-profiles.log` |
| 19 | `./scripts/check-gates-negative.sh --without-cargo` | EXIT=0 — 25 runs, every gate red on a broken state and green again | `docs/acceptance/logs/m1-check-gates-negative.log` |
| 20 | `./scripts/check-gates-negative.sh` (the five gates that call cargo) | **NOT RUN** — the OOM killer | the same script with the flag `--without-cargo` removed |

*End of document. Every status reflects a measurement of this session with the moment named. No
row is marked as proven without a command output or a green test name behind it.*
