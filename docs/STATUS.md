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
| **The milestone M2b** | The catalogues are in sync, the gate against hardcoded texts stands, the error messages are translated, the sizes and the dates follow the language, and the change of the language reaches the interface in the next frame. Open: the trigger of the change — the configuration file is read once per second, a menu entry would be the lasting solution |
| **The milestone M3** | Accepted with two open points — the manual protocol on real Windows and BSD machines, and the release signing |
| **The milestone M4** | The seam of the input is drawn. Keyboard, pointer, finger, remote control and gamepad bind to the same actions. Open: the layout of the finger, the sheet dialogs, the Android build |

## 3. The measured state

| What | Number |
|---|---|
| Tests | **1335 green, 0 red, 0 skipped** |
| Test binaries | 57 |
| Gates that prove themselves | **39 gates in 42 runs** — every one of them turns red on a broken state and green again after the repair. Gate 9 checks two agreements and therefore runs four times. Last measured on 27.10.2026 after the seam of the input of M4 and the language of the documentation: **PASS**, 42 of 42 |
| The linter | `cargo clippy --workspace --all-targets --locked -- -D warnings` — **no warning** |
| The formatting | `cargo fmt --all -- --check` — **without a deviation** |
| The licences | `cargo deny check` — advisories, bans, licences, sources: **ok** |
| The consistency gates | 14, all green |
| The language catalogues | 3 languages, 152 keys, the same set of keys |

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

## 6. The rules that follow from this milestone

These rules came out of the errors. They stand here, because the next time they are just as easy to
break:

| Rule | Why |
|---|---|
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
| 3 | The trigger of the change of the language | The desktop build reads the configuration file once per second. An entry in the native menu bar would be the lasting solution — `muda` comes with M3 |
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
