# Contributing to Phocommander

Thank you for looking at Phocommander. This guide describes the path from an idea to a
merged pull request. It is short — the detailed rules for code, tests and documentation
live in [`AGENTS.md`](AGENTS.md), and the scope of the project lives in
[`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md).

**Before you write code:** read the relevant sections of
[`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md). It is the binding source. If your idea
is not covered by it, that is a good sign — it may become a proposal.

**Before you change a file:** read [`docs/WHERE_TO_CHANGE.md`](docs/WHERE_TO_CHANGE.md).
It tells you which crate owns which responsibility, and the crate boundaries are
checked mechanically, not by convention.

---

## 1. Open an issue first

Everything except a typo fix starts with an issue. The issue is where the scope is
agreed, so that the implementation does not silently become something else.

A good issue contains:

- **What** is missing or wrong, in one sentence.
- **Why** it matters — the user problem, not the implementation idea.
- **Where** it belongs: the crate, and the section of the specification it touches.
- **What "done" looks like**, in observable terms. "The viewer shows a hex dump of the
  selected file" beats "the viewer gets better".

Bugs follow the same shape, with a reproduction: the platform, the steps, and what
happened instead of what should have happened. If the bug is a security issue, do
**not** open a public issue — read [`SECURITY.md`](SECURITY.md) instead.

---

## 2. Branch

Work on a branch, never directly on `main`.

```
feature/<short-description>     new functionality
fix/<short-description>         bug fix
docs/<short-description>        documentation only
chore/<short-description>       build, CI, dependencies
```

One topic per branch. A branch that mixes a bug fix with a refactoring is hard to
review and hard to revert.

---

## 3. Commit

Commits are written in English, in the conventional format:

```
<type>(<scope>): <summary>

<optional body: what changed and why>
```

| Type | Use for |
|---|---|
| `feat` | New functionality |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `test` | Tests only |
| `refactor` | Behaviour-neutral restructuring |
| `perf` | Performance work, with the measurement in the body |
| `build`, `ci` | Build system, workspace manifest, CI |
| `chore` | Everything else that needs no review of the code itself |

Rules:

- The summary is imperative, in the present tense, and fits on one line: "add hex
  dump viewer for stage 1 formats".
- The body explains **why**, not **what** — the diff already shows what changed.
- One logical change per commit. A commit that needs "and" in its summary is two
  commits.
- A performance claim needs the measurement in the commit body: the command, the
  number, the machine. Without it, the claim is not made.
- No "WIP", "fixes", "misc" or "asdf" in a commit message that ends up in `main`.

---

## 4. Code rules

The full list is in [`AGENTS.md`](AGENTS.md). The ones that trip up newcomers most
often:

- **Read before you change.** A file you have not opened is a file you do not
  understand.
- **WHAT / WHY / EXAMPLE** doc comment on every public function. The WHY is where
  the project's decisions are recorded; a restatement of the signature teaches nobody
  anything.
- **Module header** on every module, naming its responsibility *and its boundary*.
- **400 lines soft, 550 lines hard** per source file. Above the soft limit, the
  module header states why. Above the hard limit, the file gets split.
- **No `unsafe`.** No exceptions to negotiate.
- **No `unwrap()` / `expect()`** on a path that a file name, an archive entry or a
  user input can reach.
- **No stubs, no `todo!()`, no dummy returns.** A missing implementation is a missing
  feature — say so in the issue, do not fake it.
- **English** for code, comments and identifiers. UI strings go through a translation
  key, never hardcoded.
- **The dependency direction is fixed:** `apps/*` → `platform/*` → `crates/*`, and no
  crate under `crates/*` may depend on a sibling under `crates/*`.

---

## 5. Tests

- **Every public function has at least one unit test.** Not "the important ones".
- **Every hardening has a negative test** that performs the attack and proves the
  defence holds. Path traversal, decompression bombs, header limits, symlink handling:
  each one gets a test that would fail without the fix.
- **Integration tests cover the crate boundaries**: a copy across a temporary
  directory, an extraction of an archive with traversal entries, a configuration file
  with an unknown field.
- **Property tests** cover the invariants you cannot enumerate — path resolution never
  leaves the destination, the rename rule is deterministic, key resolution decides
  deterministically on a conflict.
- **UI tests** use `egui_kittest` and assert behaviour, not pixels.

If a test could not be run, say so — with the exact reason. A NOT RUN with a reason
is acceptable; a faked PASS is not.

---

## 6. Pull request checklist

Before you open the pull request, run the gates locally:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check
cargo audit
python3 scripts/check-links.py
```

These are the same commands the CI runs, in the same form — the gates in
[`.github/workflows/ci.yml`](.github/workflows/ci.yml) add the crate-boundary check
and the file-size check, which the CI calls as repository scripts.

Then tick the boxes in the pull request description:

- [ ] The issue this pull request closes is linked.
- [ ] `cargo fmt --all --check` is clean.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` is clean.
- [ ] `cargo test --workspace --all-features` passes; the output is pasted into the
      description.
- [ ] `cargo deny check` and `cargo audit` pass — a new dependency is checked for
      licence and advisories **before** it is added, never afterwards.
- [ ] New or changed public functions have a unit test.
- [ ] Hardened code paths have a negative test.
- [ ] Documentation changed in the same pull request: the key binding table
      ([`docs/KEYMAP.md`](docs/KEYMAP.md)), the crate map
      ([`docs/WHERE_TO_CHANGE.md`](docs/WHERE_TO_CHANGE.md)), and the bilingual user
      guide where the behaviour is visible to a user.
- [ ] No dead link: `python3 scripts/check-links.py` passes.
- [ ] No file above 400 lines without a documented reason, none above 550.
- [ ] The description says what was **not** done and why.

---

## 7. Review

The maintainer reviews for the three product rules — free, clutter-free, ad-free —
and for the architecture rules. A review comment is about the code, never about the
person: the project follows the [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md), which is
the Contributor Covenant 2.1.

Reviews may ask for a measurement. "How fast is it?" is answered with a number and a
command, or with "not measured, here is how".

---

## 8. Licensing

Phocommander is dual-licensed under `MIT OR Apache-2.0` — see [`LICENSE-MIT`](LICENSE-MIT)
and [`LICENSE-APACHE`](LICENSE-APACHE). By opening a pull request you agree that your
contribution is distributed under those same terms. You keep the copyright of your
work.

The `cargo-deny` allow list in `deny.toml` contains no copyleft licence. A dependency
under GPL or AGPL cannot be added — not because of a technical problem, but because it
would pull the whole product into a licence that contradicts the promise made to
every user.

---

## 9. Where to ask

- Questions about the product → a GitHub issue.
- Vulnerabilities → [`SECURITY.md`](SECURITY.md), privately, not in an issue.
- Behaviour of the code → the issue tracker of the repository.
- Everything the specification does not answer → ask the owner; the answer is written
  down in the relevant document, so the next person does not have to ask again.
