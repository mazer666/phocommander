# The state of the project

**Last updated:** 27.10.2026 · **State on GitHub:** see `git log` — the last commit carries the measurement

This document is the **status board**. It says where the project stands, what is running and what
comes next. The roadmap says *what* the milestones contain; this document says *how far* they are.

The source of truth for the code is the repository itself. This document describes the state of one
commit — with the commit hash in the headline.

---

## 1. The state of the milestones

| Milestone | Content | State | Evidence |
|---|---|---|---|
| **M0** — Foundation | Workspace, CI, configuration, threat model, documentation | **accepted** | `docs/acceptance/M0-ACCEPTANCE.md` |
| **M1** — The working file browser | File system model, job queue, rename, viewer stage 1, archives | **in review** — the engine and the dialog of the rename are done; open are the macOS protocol and the benchmarks | `docs/acceptance/M1-ACCEPTANCE.md` |
| **M2** — Maturity on macOS | Tabs, profiles, safe copy modes, layouts per context | **accepted** | `docs/acceptance/M2-ACCEPTANCE.md` |
| **M2b** — Internationalisation | English and German complete, localised formatting, change of the language at run time | **in work** | `docs/ACCEPTANCE-CRITERIA.md`, block M2b |
| **M3** — The other desktop platforms | Windows, Linux, BSD | **accepted with two open points** — the manual protocol and the signing | `docs/acceptance/M3-ACCEPTANCE.md` |
| **M4** — Mobile first | Pointer, finger, television, gamepad — one table of bindings, five contexts | **in work** — the input seam stands, the layout of the finger is missing | `docs/acceptance/M4-ACCEPTANCE.md` |
| **M5** … **M9** | iPadOS, remote sources, cloud, the rest | open | `docs/ROADMAP.md` |

## 2. What is running

| What | State |
|---|---|
| **The acceptance of M1** | The sixth measurement is written. Three reasons remain: the manual protocol on a real macOS machine, the benchmarks of the specification and the property test of the rename |
| **The milestone M2b** | The catalogues are in sync, the gate against hardcoded texts stands, the error messages are translated, the sizes and the dates follow the language, and the language changes while the program runs: the actions `app.language.de`, `app.language.en` and `app.language.fr` write the choice into the configuration file and the interface follows in the next frame. Open: the third language as a product promise and the text files of the stores |
| **The milestone M3** | Accepted with two open points — the manual protocol on real Windows and BSD machines, and the release signing |
| **The milestone M4** | The seam of the input is drawn. Keyboard, pointer, finger, remote control and gamepad bind to the same actions. Open: the layout of the finger, the sheet dialogs, the Android build |
| **The chain from the key to the file** | The commands of the surface became the jobs of the waiting room. The copy, the move, the deletion and the rename now reach the file system of the host. Open: the run on a machine with a screen, which the sandbox does not have |

## 3. The measured state

| What | Number |
|---|---|
| Tests | **1480 green, 0 red, 0 skipped** |
| Test binaries | 45 |
| Gates that prove themselves | **39 gates in 42 runs** — every one of them turns red on a broken state and green again after the repair. Gate 9 checks two agreements and therefore runs four times. Last measured on 27.10.2026 after the seam of the input of M4 and the language of the documentation: **PASS**, 42 of 42 |
| The linter | `cargo clippy --workspace --all-targets --locked -- -D warnings` — **no warning** |
| The formatting | `cargo fmt --all -- --check` — **without a deviation** |
| The licences | `cargo deny check` — advisories, bans, licences, sources: **ok** |
| The consistency gates | **14 in the collection** of `scripts/check-consistency.sh`, all green — and they run in **every** job of the matrix, not only in the one that checks the quality |
| The language catalogues | 3 languages, 158 keys, the same set of keys |

## 4. The decisions of the owner

The four open points of the implementation are decided. They stand in the specification in section
14.2, and the code follows them.

| Question | Decision | Affects |
|---|---|---|
| The reference for the series name | The directory name is the default, it is always visible, and it can be overridden | M1, M2 |
| The scope of the action palette | Only the actions that are reachable in the current context are shown | M8 |
| The size limit for a network answer | A reading budget that the user sets in the settings; the default is 64 MiB, zero turns the limit off | M6 |
| The placeholder dialect in the rename dialog | Square brackets in the dialog, curly braces in the named templates | M1, M2 |

## 5. What this milestone found

The milestone is not green because nothing happened. The check found **real errors** — in the code
and in the tests:

**Errors in the code**

1. The list of the text keys counted 106 entries and held 107.
2. The watcher of the temporary file carried a `Debug` that the trait bearer cannot fulfil.
3. The translator carried a `Debug` that the bundle of the Fluent library cannot fulfil.
4. The enumeration of the directory entries delivered file names instead of the entries that the
   comparisons need.
5. The check plan of the rename reported the same conflict twice.
6. The check of the archives compared the checksum even when the entry had no content — a symbolic
   link was reported as damaged.
7. The empty key binding reported "unknown key" instead of "empty binding".
8. The messages of the errors stood in English texts in the source — the status line showed them to
   the user.
9. The character functions carried eight or nine parameters that were always the same.

**Errors in the tests that carried false assumptions**

10. Two tests of the directory tree expected the root of the volume not, although the code provides
    it.
11. Two tests of the rename checked a situation that is no conflict.
12. Two tests of the comparison compared a directory that did not exist.
13. A test of the checksum passed no content and expected a result all the same.
14. The integration test of the directory entries was dead code — it was never in the manifest.

**Gaps that the check uncovered**

15. The job queue had **not a single test** — the retry of the failed entries was implemented, but
    it never ran.
16. The container adapters of the archives had **not a single test** — now they have 32.
17. The detection of the format knew **no compressed TAR** — the four compressions of the adapter
    were unreachable through the public API.
18. The file `keymap_view.rs` had grown to 999 lines and was therefore above the maximum of 900. It
    is split now: the input in `keymap_view/stroke.rs`, the binding in `keymap_view.rs`.
19. The negative test used the central target directory and polluted the binaries.
20. `AGENTS.md` named 400 and 550 lines, where the specification (NFR-14) and the gate say 600 and
    900. The file says itself that the specification wins.
21. Three gate scripts had lost the executable bit on GitHub. The way that writes the content does
    not carry the mode — therefore there is `scripts/restore-executable-bits.sh` now.
22. **The rename of M1 existed only half.** The *execution* exists and is tested — atomic, with the
    detection of conflicts and with the taking back of a failed rename. The *formation* of the new
    name from a template was missing: the placeholders, the searching and the replacing, the
    counter, the determinism, the preview, the list of the conflicts, the stored templates and the
    padding with zeros.
23. The **derived random value** was written with sixteen characters instead of eight: the width of
    a format is a *minimum*, not a maximum. A 64-bit hash does not fit into eight places without
    being cut short.
24. The module `format` of the i18n crate was **never compiled** — it was not declared. The first
    build of the crate would have been red.
25. Fluent does not register its built-in functions by itself. `NUMBER($count)` was therefore an
    unknown function, and the message fell back to the key.
26. Fluent 0.16 writes a number **without** the separators of the language — the field
    `use_grouping` exists, but it is not used. The numbers of the interface carry the separators
    now, because the i18n crate formats them itself with the rules of CLDR.
27. An argument of the interface had **no name**.
28. **`cargo clippy` was broken for hours** and looked like clean code. The compiler reads its
    source with "-" from the standard input, and that carried the output of the previous command.
    The wrapper `scripts/cargo.sh` empties it and points cargo at the real driver of the toolchain.
29. **The negative test found the error of the lead instance.** After the change to `Iterator::find`
    the source was no longer formatted after `cargo fmt` — and `cargo fmt --check` did not see it,
    because the call failed on the poisoned standard input and the error came as "failed to run
    rustc". The gate test found it because it calls `cargo fmt` **in a copy** whose standard input
    is empty.
30. **The standard input of the shell carries the output of the previous command.** `rustc -` reads
    it as Rust source and dies with "unterminated character literal". That was the cause of the
    "Operation aborted" messages — **no** memory problems, but a poisoned stdin. Every cargo call
    now gets `< /dev/null`.
31. **The input knew only the keyboard.** `InputEvent` and `Binding` carried only `Key` and
    `Palette` — the table of the bindings could not name a click or a tap, although the interface
    already knew `Context::Pointer` and `Context::Touch`.
32. **There were no default bindings in the code at all.** The bindings stood only in
    `config/keymap.toml`, and that file is never written — it never comes into being on the user's
    machine, and `read_keymap()` returned `None`. The interface had not a single binding: no `F5`,
    no `Tab`. The gap showed itself while the defaults were written, because the default of the
    controller context was `Grip` — and `Grip` is no key.
33. **The swipe went out twice.** The gesture went out when the finger passed the distance and
    once more when it lifted. The test found it.
34. **The formatter and the generator contradicted each other.** The generated file was broken up
    by `cargo fmt`, and the generator compared the bytes. The comparison now reads the content —
    the bindings — and leaves the spelling alone.
35. **The check of the bindings knew three contexts only.** It saw `dpad` and `controller` and
    measured on them whether every action is bound. The contexts `pointer` and `touch` were
    unchecked — and would have stayed empty without a gate noticing.
36. The formatter of `benches/interaction.rs` had never run.
37. **The commands of the surface were never read.** The surface produced the commands — copy, move, delete, rename — and `take_commands()` had no caller in the whole tree. The engine, the waiting room and the interface existed, and nothing joined them: the application could not copy a file. The adapter that translates the commands into the jobs closes the chain.
38. **The directory was never read.** The command that asks for a directory was produced and read by nobody. The panels stayed empty, and a file manager with empty panels shows nothing at all. This was the fourth gap of the same kind, and it is the one that made the application a file manager in name only.
39. **The quick view never showed anything.** The seam `PreviewSource` existed, and the desktop build never implemented it. The viewer could read a file and the quick view could show lines — and the two never met.
40. **The security rules were never applied.** The plan of the rules was reported on the screen and never reached a job: the copy compared no checksum, and the quick view read files that the rule forbade. The rules travel with the job now.
41. **The licence check never ran.** The advisory database needs a lock file, and the home of the sandbox lies on an NFS mount that cannot hold one. The check failed with the message of the lock, and nobody read it as a failure of the check. A withdrawn version of a crate was therefore in the tree. The script `scripts/audit.sh` puts the database on a local disk, and the withdrawn version was replaced.
42. **The number of the tests was too high in the reports.** The count was taken over the output of the test run with `strings`, which joins lines that the program wrote into one buffer without a line break — and the count then added up every number of a line, the zeros behind *failed*, *ignored* and *measured* with it. The figure was four times the number of the test binaries too high, and it looked plausible enough to be believed. The script `scripts/count-tests.sh` counts the tests now, and the figure in this document is 1448 and not 2004.
43. **A key that does nothing said nothing.** The surface reports the actions that belong to a later milestone — the drive list, the action palette, the search in the viewer — and the command that carries the report was never read. The user pressed a key, the program did nothing, and **nothing said so**. A key that does nothing without a word looks like a broken key. The status line carries the notice now, and it names the action. The gate `scripts/check-commands.py` proves that no command of the surface leads nowhere.
44. **Two getters carried a `get` where the project writes `lookup_with`.** In Rust a getter carries no `get`, and the rule of NFR-17b is that a name says what a thing is. The check of the names found them: `Texts::get_with` and `Texts::get_by_key`.
45. **The hex dump was calculated and never shown.** The desktop adapter builds the dump of a file that is no text, the lines are handed to the viewer — and the viewer replaced them with a notice: "this file is no text file". The bytes the user asked for were computed, carried across the interface, and then dropped at the last moment. The dump stands in the area now, the notice stands above it.
46. **The surface was never drawn in a test.** Every test of the interface proved the *state* of the shell; none of them drew it. A file that the shell carries and never draws is a file the user does not see. Three tests draw the surface now: the viewer with a text, the viewer without a file, and the viewer with the bytes of a file that is no text.
47. **The primary key of the platform was never set.** The actions are the same everywhere, only the key differs: `Cmd` on macOS and on iPadOS, `Ctrl` everywhere else. The shell always listened for the control key — on a mac every shortcut with a modifier was dead, and the key looked broken. M1 is the milestone of macOS. The composition root decides now, and the test drives both keys.
48. **The viewer could not be closed.** The viewer opens in an area below the panels, and the area takes the room of the panels. The function `close_viewer` existed and nobody called it, and no action existed at all. The action `viewer.close` is in the catalogue now, in all five contexts.
49. **The theme of the project was never shown.** The colours are built in one place — and that place was never called. The window showed the standard colours of the framework, and the theme of the project existed only in the source.
50. **The priority of a job did not change the list.** The queue runs the job with the highest priority first, and the list of the jobs showed the order in which they were submitted. The list shows now the order of the execution — what is going to happen next.
51. **The notice of an action that cannot do anything was built twice.** The shell built it in the language of the interface, the adapter built it again in English. The shell is the only place that knows the language, and it is the only place that builds the notice.
52. **Five getters and setters were never used.** `is_case_insensitive`, `other_mut`, `rename_dialog_mut`, `toggle_overlay`, `set_active_side`, `set_arrangement` — doors into rooms that nobody entered. Two of them now carry the path that sets the state, the others are gone. The gate `scripts/check-dead-code.py` asks for every public function of the workspace: who calls it?
53. **Six settings of the configuration were read and never applied.** The user writes in the file whether the hidden files and the files of the system shall be shown, how many workers the waiting room shall have, what the right mouse button does, how a directory is synchronised, and whether the session comes back. All six were read — and nothing happened. The user writes the value, sees nothing change, and concludes that the program is broken.
54. **The composition could not be tested.** The functions that build the application out of the configuration lived in the binary, and a composition that lives in the binary drifts apart from the configuration without anyone noticing. The composition is a module of the library now, and the settings of the user are proven to reach the shell.
55. **Eleven functions of the workspace are called by nobody.** A library has functions that its own workspace does not use — that is what a library is for. But eleven public functions were called nowhere, and three of them are the seams of the tests. The gate `scripts/check-dead-code.py` asks every function: who calls it? The eleven are named in the gate with the reason why they stay, so that a new one is noticed.
56. **The gate of the commands was blind in one eye.** The gate read the enumeration of the commands until the first brace after the opening one — and a command that carries a field ends with a brace. It saw twelve of the fifteen commands. The three it did not see were never carried out: the exchange of the two panels, the menu and the command that leaves the application. F10 did not close the program. The gate sees all fifteen now, and it knows both places that carry commands out — the adapter of the file system and the frame of the window.
57. **The exchange of the two panels did nothing.** The command was produced and nobody carried it out — the user pressed Ctrl+U and the two panels stayed where they were. The adapter exchanges them now, and a test proves it.
58. **The program could not be closed with the keyboard.** The command that leaves the application was produced and never carried out. The frame closes the window now, and the program ends with it. On macOS this is the difference between an application that can be left and one that cannot.
59. **The menu of the application says that it comes later.** The menu is a native menu of the system, and it belongs to the milestone of the platform integration (FR-28). The action says so in the status line instead of pretending that a menu opened.
60. **The synchronisation was unreachable.** The comparison of two directories, the plan of the synchronisation and the job that carries it out — all of it existed, and no key reached it. The setting `sync_mode` of the configuration file said which way the synchronisation goes, and the setting was read and never applied. The action `panel.sync` is in the catalogue now, in all five contexts, the adapter builds the plan and starts the job, and the way of the user reaches the job.
61. **The advisories are all gone.** The check of the licences reported an unsound defect in `rustix` — the functions of the extended attributes marked bytes as written that the system call never wrote. That is undefined behaviour that safe code can reach. The update to `rustix` 1.1.6 fixed it, and it took the four unmaintained crates with it. The check says: advisories ok, bans ok, licences ok, sources ok.
62. **Six tests shared one name in the temporary directory.** Two runs of the tests worked on the same file, and one of them removed what the other one needed. The test of the archive that is taken out of the container was flaky because of it, and it failed once in thirty runs. The names carry the number of the process now.
63. **The undo existed and no key reached it.** The ledger of the undo was a library, the notice of the undo area was drawn — and the state that says "there is something to take back" was never set. The user copied a file and could not take it back. The action `panel.undo` is in the catalogue now, the adapter records what it copies, and the undo removes the copies — the notice names the limitation, because the undo of the product is partial (FR-26).
64. **The right mouse button had no binding at all.** The setting `right_click_action` was read and never used, and the button itself was bound to nothing: the user pressed it and nothing happened. The action `panel.right_click` is in the catalogue, the setting reaches the shell, and the button either marks the file under the pointer or says that the menu of the system comes with the milestone of the platform integration (FR-17, FR-28).
65. **The plan of the synchronisation was shown nowhere.** The document of the plan was built and then thrown away: the user started the synchronisation and saw nothing that told him what it was going to do. The plan stands in the area of the viewer now, in the mode `sync-preview`, and the user reads it before he lets anything happen (FR-22b).
66. **The state of the session was never written and never read.** The setting `restore_session` was read and never used, and the program always opened in the working directory — the user walked to the same place again and again. The two directories of the last time are the ones that the panels show now. The rule of the security section decides whether the state is written at all: the profile `safe` leaves nothing behind (FR-16).
67. **The key of the menu said that the menu comes later.** The interface draws a menu bar in the top of the window, and it carries every action of the catalogue — the menu was there all the time. The action says now where it stands, so that the user who looks for it with the keyboard finds it.

## 6. The rules that follow from this milestone

These rules came out of the errors. They stand here, because the next time they are just as easy to
break:

| Rule | Why |
|---|---|
| **A check that fails for a reason of its environment is a check that does not run.** The licence check failed with the message of a lock file that NFS cannot hold, and the failure was read as a problem of the lock. A check that reports a reason of the environment has to be made to report the result, or it has to be called with another tool. |
| **One single cargo process in the workspace.** Every agent gets a `CARGO_HOME` and a `CARGO_TARGET_DIR` of its own on local disk. | Three gigabytes of RAM. Two builds at the same time kill both — and the agent then waits for an answer that never comes. |
| **The negative test builds into a target directory of its own.** | A test binary carries the path of the repository it was built in. A binary from the copy looks for the configuration of the copy — and that is gone when the script ends. |
| **The agents do not build.** They write, the lead instance builds and checks. | A build in a foreign directory eats 1 to 5 GB and blocks the NAS volume on which the repository lies. |
| **After every milestone the disk is cleaned.** | At the end of that day there were 20 GB of target directories in `/tmp`. |
| **After every `git reset` the executable bit is restored.** | `./scripts/restore-executable-bits.sh` — the way that writes the content does not carry the mode. |
| **Every cargo call gets `< /dev/null`.** | The standard input of the shell carries the output of the previous command, and `rustc -` reads it as Rust source. |
| **A gate that was never red is a wish.** | Every new gate belongs into `check-gates-negative.sh`, with a built-in defect. |
| **The documentation of the repository is English.** The German editions of the user guide are a product feature, not an exception to the rule. | The specification, the roadmap and the code speak one language. A second language in the documentation doubles the work and the drift. |

## 7. What comes next

| # | What | Why |
|---|---|---|
| 1 | The manual protocol of M1 on a macOS machine | The sandbox has a Linux system and no screen. The acceptance needs this protocol, otherwise M1 stays open |
| 2 | The benchmarks of NFR-02, NFR-03 and NFR-09a | They need the release profile and a quiet tree |
| 3 | The third language as a product promise | The French catalogue exists and is complete, and it proves that a new language needs no change of the code. What is missing is the decision of the owner, whether the French version is shipped and maintained. |
| 4 | The layout of the finger in M4 | The interface of the telephone stacks the panels, the rows are 44 points high, the dialogs come as a sheet from the bottom, and there is no hover |
| 5 | The defaults of the bindings in the CI | `scripts/generate-keymap-defaults.py --check` belongs beside the other gates, so that the file of the defaults does not become stale |
| 6 | The milestone M3 — Windows, Linux, BSD | The first milestone that runs on another platform. First the questions of the portability: the separator of the paths, the long paths under Windows, the key binding per platform and the matrix of the CI |

## 8. The questions that need a decision

| Question | State |
|---|---|
| How does the manual protocol run on a real macOS machine? | The sandbox has a Linux system and no screen. The acceptance of M1 needs this protocol. |
| Which certificates sign the binaries? | Windows needs a code-signing certificate, macOS a Developer ID. Both cost money and need a decision of the owner. |
| The target directory of the tests lies on a NAS volume | Every build pays the latency of the volume. A local storage would be faster — if the sandbox offers it. |
| The release plan was contradictory | Linux with Alpha 2 and Windows with Beta 1 — although both ports belong to M3. Corrected on 25.10.2026: Alpha 1, Alpha 2 and Beta 1 ship macOS, Beta 2 ships all five desktop platforms |
| The property test of the rename (FR-33) | The specification names it expressly. It is the last missing test of the acceptance of M1. |
| Should the extension be a feature? | **No**, decided on 25.10.2026: the extension stays a hint. The magic bytes decide. A file that is called `.zip` and is no ZIP shall be recognised as what it is. |
| Encrypted containers (FR-68) | Not touched. The milestone M1 covers the unencrypted formats; the encrypted ones come with the handling of the passwords. |

---

*The state of this document belongs to the commit in the headline. Whoever carries it forward
updates the headline with it.*
