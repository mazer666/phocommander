# AGENTS.md — working rules for AI-assisted contributions

These rules apply to every automated agent that writes into this repository, and to
anyone who reviews such a contribution. They are short on purpose: everything here
exists because something in this project went wrong without it.

The binding source is [`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md). If a rule here
and the specification disagree, the specification wins — and the disagreement is a bug
in this file that you should fix in the same change.

Do not edit `docs/REQUIREMENTS.md` to match your implementation. If the specification
is wrong, describe the conflict in `deliverable.md` and let the owner decide; every
deviation from a decision needs a new decision, dated and justified.

---

## 1. Read before you change

Never edit a file you have not opened. Never edit a file you have not understood.

Before your first write in a task:

1. Read `docs/REQUIREMENTS.md` — at least the sections your task touches. It is ~960
   lines; read the whole thing if the task is bigger than one function.
2. Read the target file completely, plus the module header of its neighbours.
3. Read [`docs/WHERE_TO_CHANGE.md`](docs/WHERE_TO_CHANGE.md) to find out which crate
   is actually responsible for what. Touching the wrong crate is the most common
   mistake in this codebase, because the UI layer can do almost anything.
4. Check the git status. Someone else's uncommitted work may be in the tree — in
   this project several workers write in parallel.

Every module in this repository starts with a header comment that names its
responsibility *and its boundary*. If you cannot say in one sentence what your module
is not allowed to do, you do not yet understand where your change belongs.

---

## 2. No claim without a measurement

This is the rule that separates useful work from plausible-sounding work.

- Do not write "the copy is now faster" — run the measurement and paste the number
  and the command that produced it.
- Do not write "all tests pass" — run them, and paste the summary line.
- Do not write "the gate works" — break something on purpose, show the gate going red,
  put it back, show it going green. That is how a gate is proven to have teeth.
- Do not describe code you have not read. Do not describe a file you have not opened.
- Do not estimate a deadline, a percentage, a file size or a line count and present it
  as a fact. Either measure it or leave it out.
- Do not promise a behaviour that the code does not have yet. "Scheduled for M3" and
  "available today" are two different sentences, and only one of them may be true.

If you could not measure something, say so explicitly in the handover. "Not measured,
here is how to measure it" is a good handover. A guess dressed as a fact is not.

---

## 3. No stubs, ever

A stub is any of these:

- a function that returns a fixed value,
- an empty `Ok(())` where real work is required,
- `todo!()`, `unimplemented!()`, `unreachable!()` used to silence the compiler,
- a commented-out block that "should come back later",
- a placeholder comment where the real implementation belongs,
- a documentation page that says "to be filled in later" — a page is either finished
  or it does not exist.

`todo!()` and `unimplemented!()` are only acceptable in a `#[cfg(test)]` helper that
proves a code path is unreachable. Everywhere else, a missing implementation is a
missing feature — say so in the handover instead of faking it.

A half-implemented feature that is honestly labelled ("this function validates the
header but does not yet decode the payload, see issue #42") is acceptable. A
half-implemented feature that pretends to be complete is not.

---

## 4. Rust rules the workspace enforces

These are not style preferences. The CI gate rejects the workspace if they are
violated. The lints live in `[workspace.lints]` in `Cargo.toml`.

### 4.1 No `unwrap()` and no `expect()` on externally reachable paths

```rust
// Wrong: the file name comes from the file system, the archive or the user.
let size = entry.metadata().unwrap().len();

// Right: the failure becomes part of the error type the caller must handle.
let size = entry.metadata().map_err(Error::Metadata)?;
```

`unwrap()` is acceptable only where the invariant is enforced by the type system and
is local to the function — for example unwrapping a `Lock` that the same module
always locks. If a reader has to look at the whole file to be sure, use `expect()` with
a message that names the invariant, or restructure.

`panic!` is not an error-handling strategy in this codebase. Every fallible operation
returns `Result`.

### 4.2 No `unsafe`

`unsafe_code` is forbidden in the workspace. There is no exception to negotiate:
the entire security argument of this project — hardened archive extraction, path
canonicalisation, TOCTOU handling — is written in safe Rust and is auditable because
of that. If you believe you need `unsafe`, you have not found the right abstraction
yet. Write the safe version and explain the trade-off in the handover.

### 4.3 File size limits

| Limit | Value | Meaning |
|---|---|---|
| Soft | 400 lines | A source file above this needs a reason, and the reason belongs in the module header. |
| Hard | 550 lines | A source file above this is rejected by review. Split it. |

Splitting means splitting along a responsibility, not mechanically. A 600-line file
usually contains two modules that never talk to each other.

### 4.4 Documentation comments in the WHAT / WHY / EXAMPLE style

Every public function carries a doc comment that answers three questions in this
order:

````rust
/// Returns the archive entries of `path` that are safe to extract.
///
/// WHY: archive contents are untrusted input. Entries whose resolved path would
/// leave the destination directory are filtered out here so that no caller has to
/// repeat the check — and so that the check cannot be forgotten in one call site.
///
/// EXAMPLE:
/// ```no_run
/// let entries = safe_entries(std::path::Path::new("archive.zip"))?;
/// for entry in entries {
///     println!("{}", entry.name());
/// }
/// ```
pub fn safe_entries(path: &Path) -> Result<Vec<Entry>, ArchiveError> {
    // ...
}
````

"WHAT" alone is not enough — a restatement of the signature teaches nobody anything.
The WHY is where the project's hard-won decisions are recorded. The EXAMPLE shows the
caller.

Every module starts with a header comment naming its responsibility and its
boundary. Every constant carries a name that explains its purpose. There are no
magic numbers.

### 4.5 Every public function has a unit test

Not "the important ones". Every public function, at least one. The rule exists
because the core crate is the only place where coverage can be measured honestly —
the core has no UI dependencies, so it is testable on every target platform.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_name_inside_destination() {
        // arrange, act, assert
    }
}
```

### 4.6 Every hardening has a negative test

Path traversal, decompression bombs, header limits, symlink handling: each one gets a
test that actually performs the attack and proves the defence holds. A hardening
without a negative test is a comment, not a feature.

```rust
#[test]
fn rejects_entry_that_escapes_the_destination() {
    let archive = build_zip_with_entry("../escape.txt", "payload");
    let result = extract_into(&archive, tempdir.path());

    assert!(result.is_err());
    assert!(!tempdir.path().join("escape.txt").exists());
}
```

---

## 5. Architecture rules you must not break

The dependency direction is `apps/*` → `platform/*` → `crates/*`. Inside `crates/*`,
no crate may depend on a sibling crate of the same level. This is checked
mechanically — a script reads the `Cargo.toml` files and exits with code 1 on a
violation — so "I only added a small dependency" is not an excuse.

`phocommander-core` stays free of UI dependencies. No windows, no egui, no
platform-specific APIs. That is what makes it testable everywhere.

Viewer and archive formats are registered through a trait. A new format is a new
implementation plus one registration line — not a change in the core.

The configuration file has a `schema_version`. Breaking changes require a migration
path and a `.bak` copy of the original file.

---

## 6. Documentation is part of the change

A behaviour that is not documented does not exist for the next contributor.

- Change the binding table in `docs/KEYMAP.md` in the same change as the binding
  itself. The action identifiers are the contract between the keymap crate and the
  interface; they are spelled `panel.copy`, `panel.view`, `selection.toggle`.
- Change `docs/WHERE_TO_CHANGE.md` in the same change as a new crate, a new module
  or a moved file. A missing row is a documentation bug.
- The user guide is bilingual. A new screen, dialog or error message needs a German
  and an English page, with the same structure — not a German page and a note that
  the English one comes later.
- Every relative Markdown link must resolve. Run the checker:

  ```bash
  python3 scripts/check-links.py
  ```

  It exits with code 1 on a dead link. The single documented exception is
  `docs/security/THREAT_MODEL.md`, which the security worker writes in parallel;
  the checker knows about it and reports it as *in progress*.

---

## 7. Language rules

| Content | Language |
|---|---|
| Code, comments, identifiers, commit messages | English |
| Repository documents (README, specs, ADRs, guides) | English, except the bilingual user guide and glossary |
| User guide (`docs/user-guide/`) | German **and** English |
| Glossary (`docs/glossary.md`) | German **and** English, term by term |
| Your handover file `deliverable.md` | German |

Do not translate a code identifier into German. Do not write German doc comments in
English source files. If a UI string is needed, it goes through a translation key —
never a hardcoded string in the code.

---

## 8. Handover: changed files plus `deliverable.md`

**Do not commit, push, tag or release.** Not even a "fixup" commit, not even on your
own branch. The owner decides what enters the history.

Hand over:

1. **The changed files, in place in the working tree.** That is the primary
   handover: the owner sees exactly what you produced, unfiltered.
2. **`deliverable.md`, written in German**, containing:
   - what you changed, file by file, with line counts and the reason for each change,
   - the commands you ran and their real output (paste it, do not summarise it),
   - a link check and the result,
   - what you deliberately did *not* do, and why,
   - what is still open, what you could not measure, and what you would do next.
3. **A progress entry** in the team board, so the next agent knows where the work
   stands.

If your change touches several files, list every one of them in `deliverable.md` —
a handover that mentions only the "interesting" files hides the rest.

---

## 9. Checks before you hand over

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo audit
python3 scripts/check-links.py
```

- [ ] Every file I touched was read before it was written.
- [ ] No stub, no `todo!()`, no fake return value, no placeholder page.
- [ ] No `unwrap()`/`expect()` on a path reachable from outside the module.
- [ ] No `unsafe`.
- [ ] No source file above 400 lines without a documented reason; none above 550.
- [ ] Every new or changed public function has a unit test.
- [ ] Every hardening has a negative test that performs the attack.
- [ ] Every public function has a WHAT / WHY / EXAMPLE doc comment.
- [ ] Documentation changed in the same change as the behaviour it describes.
- [ ] Code, comments and commits are English; `deliverable.md` is German.
- [ ] All claims in the handover are backed by a command output I pasted.
- [ ] Nothing was committed or pushed; the handover is the working tree plus
      `deliverable.md`.
