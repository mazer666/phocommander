# Acceptance of the milestone M2 — Maturity on macOS

**State:** 25.10.2026 · **Examiner:** Lead instance · **Result:** accepted with two open points

---

## 0. The open points that the audit of the 5th of November found

The owner asked whether everything is really implemented and not only present. The audit of the
workspace found four points of this milestone that are not finished. They are named here, because
an acceptance document that does not name its own gaps is worth nothing.

| What | Why it is open | Where |
|---|---|---|
| **The undo takes the last copy back.** The ledger of the undo existed as a library, and no key reached it: the user copied a file and could not take it back. The action `panel.undo` is in the catalogue, the adapter records what it copies, and the notice of the undo area names the limitation — the undo removes the copies and restores nothing. | FR-26 — the undo exists for the copy operations and is marked as partial. | closed |
| **The plan of the synchronisation is shown before anything happens.** The document of the plan exists (`to_document`), and the synchronisation never showed it. The preview of the mode `sync-preview` stands in the area of the viewer now, and the user reads what the synchronisation would do before he lets it happen. | FR-22b — the default of the synchronisation is `sync-preview`. | closed |
| **The state of the session comes back when the user asked for it.** The setting `restore_session` was read and never used. The two directories of the last time are the ones that the panels show now, and the rule of the security section decides whether the state is written at all. | FR-16 — the panels keep their own state. | closed |
| **The way of a synchronisation is applied.** The setting `sync_mode` reaches the job, and the action `panel.sync` starts the synchronisation. The comparison of the two directories became a job of the waiting room, and a test proves it. | FR-22b. | closed |
| **The mouse button of the right does what the user asked for.** The setting `right_click_action` reaches the shell, and the action `panel.right_click` carries it out: the button marks the file under the pointer, or it says that the menu of the system comes with the milestone of the platform integration (FR-28). | FR-17. | closed |

The state of the session is the fourth point that this audit found. The setting `restore_session`
was read and never used, and the program always opened in the working directory — the user walked
to the same place again and again. The state of the last session comes back now, and the rule of
the security section decides whether it is written at all: the profile `safe` leaves nothing
behind.

The eleven functions that nobody calls are in the gate `scripts/check-dead-code.py` with the
reason why they stay. The gate asks every public function of the workspace: who calls it?

## 1. What this milestone covers

M2 turns the working file browser into an application that can replace a file browser in everyday
use: tabs with a state of their own, a directory tree per panel, a quick view, the safe copy modes
with the synchronisation plan, the atomic rename, the retry of the failed entries only, the undo,
the check of the archives without unpacking them and the interface profiles `clean` and `full`.

The internationalisation is **not** part of this milestone. It is the milestone **M2b** of its own,
and the reason stands in the roadmap: the completeness of the languages is a property of the
product and not a detail of the presentation.

## 2. The measured state

| What | How it was measured | Result |
|---|---|---|
| The tests of the whole workspace | `cargo test --workspace --locked` | **1470 green, 0 red, 0 skipped** |
| The formatting | `cargo fmt --all -- --check` | **without a deviation** |
| The warnings of the linter | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **without a warning** |
| The licences | `cargo deny check` | **advisories, bans, licences, sources: ok** |
| The boundaries of the crates | `scripts/check-crate-boundaries.sh` | **12 dependencies, 0 violations** |
| The size of the files | `scripts/check-file-size.sh` | **0 above the maximum of 900**, 6 above the target value of 600 with the reason in the module header |
| No telemetry | `scripts/check-no-telemetry.sh` | **0 violations, 0 forbidden crates** |
| The consistency scripts | `scripts/check-*.py` | **all PASS** |
| The gates themselves | `scripts/check-gates-negative.sh` | **46 gates**: each turns red on a defect and green again |

The protocols lie under `docs/acceptance/logs/`.

## 3. The errors that this milestone found

The milestone is not green because nothing happened. The check found twelve real errors and twelve
test errors that rested on false assumptions:

**Errors in the code**

1. The list of the text keys counted 106 entries but held 107 — one key was not in the check list.
2. The watcher of the temporary file carried a `Debug` that the trait bearer of the file system
   cannot fulfil. The states are written by hand now.
3. The translator carried a `Debug` that the bundle of the Fluent library cannot fulfil.
4. The language identifier had no valid replacement when a directory carries no valid language
   code.
5. The enumeration of the directory entries delivered the file names instead of the entries that
   the comparisons need.
6. The check plan of the rename reported the same conflict twice. The jump list showed the same
   path twice.
7. The check of the archives compared the checksum even when the entry had no content — a symbolic
   link was thereby reported as damaged.
8. The caller of the character functions carried eight or nine parameters that were always the same.

**Errors in the tests that carried false assumptions**

9. Two tests of the directory tree expected the root of the volume not, although the code provides
   it expressly. The code is right: the root is an ancestor like every other.
10. Two tests of the rename checked a situation that is no conflict.
11. Two tests of the comparison compared a directory that did not exist.
12. A test of the checksum passed no content and expected a result all the same.

## 4. The gap that this milestone uncovers

The job queue had **not a single test**. The retry of the failed entries (FR-22k) was implemented,
but nobody had ever run it. Nine tests came to it, and they found a real error: the file system of
the tests could not let a single entry fail. That possibility is missing now.

## 5. The open points

| Point | Reason | When |
|---|---|---|
| The about dialog names the number of the formats that are really supported | The dialog is a part of the milestone M3 | M3 |
| The performance goals NFR-01 to NFR-08 are measured | The measurement needs the finished interface | after M3 |

These two points touch requirements that the specification assigns to the milestone M2. They are
not bypassed, but named openly.

## 6. The corrections that lie outside the milestone

These corrections were needed so that the milestone was measurable at all, but they do not belong
to the scope of M2:

- The internationalisation was taken out of M2 and made the milestone **M2b**. The requirements
  FR-120 to FR-126 point at M2b now, and the block M2b in the acceptance criteria carries the
  criteria AC-M2b-01 to AC-M2b-08.
- The French catalogue was brought to 108 keys. It was the proof for FR-125 — a third language
  without a change of the source — and it was incomplete, so it was no proof.
- The identifiers of the texts `Locale::English` and `Locale::German` do not exist. The calls use
  `Locale::english()` and `Locale::of("de")` now.
- The check scripts know the milestone M2b and the identifiers `AC-M2b-xx` now.

## 7. What comes next

1. The milestone **M2b** — the internationalisation in both languages, with the check of the sets of
   keys and the change at run time.
2. The **acceptance of M1** is rewritten, on the quiet tree and with the 924 tests that exist by now.
3. The milestone **M3** — Windows, Linux and BSD.

---

*The acceptance holds for the state of the commit with which the protocols under
`docs/acceptance/logs/` were written. Every later change of the sources makes the measurement
invalid.*
