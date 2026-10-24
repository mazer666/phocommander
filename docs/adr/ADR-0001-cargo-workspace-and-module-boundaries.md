# ADR-0001 — Cargo workspace and crate boundaries
**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Rust Architect
**Related requirement IDs:** FR-115, FR-116, FR-117, FR-118, FR-121, NFR-32, NFR-33, NFR-41

## Context

The specification fixes the shape of the repository: one Cargo workspace, flat `crates/*`
boxes, `apps/*` for the binaries, `platform/*` for the operating system adapters and `xtask`
for the build helpers. It also fixes the dependency direction — `apps/*` → `platform/*` →
`crates/*` — and states that inside `crates/*` no box may derive from another box of the same
level. It demands that this rule is checked by a script that exits with code 1 on a violation,
and that a negative test proves that the gate has teeth.

Two facts make the rule less obvious than it looks.

First, the rule as written forbids every dependency between two crates of the same level,
including a dependency on `phocommander-core`. The sibling project Photocraft, whose pattern
the specification cites as the model, does the opposite: `crates/algo` depends on
`crates/geom` and `crates/color`, and 24 of its 27 crates depend on at least one sibling. The
requirement catalogue of this project reproduces the strict wording in FR-116.

Second, a strict rule of that kind has consequences that are easy to overlook. If no crate may
depend on a sibling, then no two crates may share a type, a helper function or a constant.
Everything shared has to travel as data through the composition root — the application — or it
has to exist twice. The architecture page of this repository states in rule 2 that "the shared
part belongs in `phocommander-core`", which is exactly the dependency the strict rule forbids.
The rule and the guidance contradict each other, and the contradiction has to be resolved
before M1 writes the first line of real code.

The risk register of the specification names the flat structure as a decision that was taken
with the project, so the structure is not in question. What is in question is the exact
reading of one sentence.

## Decision

**1. The workspace is one Cargo workspace with `resolver = "3"`, edition 2024, and one
catalogue of pinned dependency versions in the root manifest.** Every crate inherits its
metadata with `version.workspace = true` and its lint set with `[lints] workspace = true`.
A version is written exactly once.

**2. The dependency direction is enforced mechanically by `scripts/check-crate-boundaries.sh`.**
The script reads the `Cargo.toml` of every member, resolves every path dependency — including
the ones that are inherited from the root catalogue with `workspace = true` — and checks them
against this matrix:

| Level | May depend on | May not depend on |
|---|---|---|
| `crates/*` | — | every other crate of the workspace |
| `platform/*` | `crates/*` | `platform/*`, `apps/*`, `xtask` |
| `apps/*` | `platform/*`, `crates/*` | `apps/*`, `xtask` |
| `tests` | `crates/*` | `apps/*`, `platform/*`, `xtask` |
| `xtask` | `crates/*` | `apps/*`, `platform/*` |

The script exits with code 1 and names the crate, the target and the reason when a
dependency violates the matrix. A dependency that is written as an inline table outside a
dependency section is reported as well, so that the catalogue cannot be used to smuggle a path
dependency past the gate.

**3. The strict reading is implemented as written: no crate below `crates/` depends on a
sibling.** Everything that two crates would have to share travels as data through the
composition root, and the composition root is the application:

- a limit that more than one crate has to agree on is written down exactly once, in
  `phocommander-core::config`, and reaches the other crates as a value that the application
  passes to them;
- a type that crosses a crate boundary is a plain `std` type (`Path`, `PathBuf`, `String`,
  `u64`) or a value that the application maps between the crates that own it;
- the tests that hold two crates against each other live in the package `tests`, which is
  outside `crates/` for exactly this reason.

**4. The contradiction between the strict rule and the architecture page is recorded here
and not resolved by the architect.** Two ways out exist, and the choice belongs to the owner:

- **Option A — one named exception.** `phocommander-core` becomes the vocabulary crate: every
  other crate may depend on it, and no crate may depend on any other sibling. The gate gets
  one additional allowed edge, the specification keeps its wording, and the architecture page
  becomes correct.
- **Option B — the strict rule, shared code in the composition root.** The gate stays as it
  is, the architecture page is corrected, and every shared type or constant has to be mapped
  in `apps/*`. This is what M0 implements.

## Consequences

### Positive
-
The rule is a machine rule. `scripts/check-crate-boundaries.sh` proves it: the negative test
builds a dependency of `phocommander-core` on `phocommander-ui`, the gate reports the
violation and exits with code 1, the dependency is removed and the gate exits with code 0.
-
The core stays testable on every platform, because it depends on nothing but `thiserror`.
-
The strict rule makes the ownership of every type explicit. The question "where does this
type live?" has exactly one answer, because there is only one place where a type can be
shared.
-
The composition root is visible. The application is the only crate that knows all the others,
which is what makes the wiring of M1 testable.
-
The catalogue of pinned versions is written once and inherited, so a version cannot drift
between two crates.

### Negative / Trade-offs
-
The strict rule makes the flat structure expensive: a type that is needed in two crates must
either be a `std` type, or be mapped in the application. For a value such as the conflict
policy this means that `phocommander-config` and `phocommander-ops` will each define their
own enum for the same file format until the owner decides about Option A.
-
Until the decision is taken, the architecture page of this repository contradicts the gate.
A contributor who follows rule 2 of that page writes a dependency that the gate rejects.
-
A shared test kit cannot be a crate below `crates/`, because that would be a sibling
dependency. The test kit therefore lives in the package `tests`, outside the level.
-
The rule as written also forbids a cycle-free but harmless dependency such as
`phocommander-ops` → `phocommander-core`, which Photocraft uses in 24 of its 27 crates.

## Alternatives Considered
-
**Keep the structure of Photocraft (crates may depend on siblings, the graph stays acyclic).**
Rejected for M0 because the specification and the requirement catalogue state the strict rule
in FR-116, and because the specification is the binding source. The option is recorded as
Option A above and needs a decision from the owner.
-
**A single crate instead of a flat workspace.** Rejected: the specification demands the flat
structure, and the crates of the six target systems have very different build requirements.
-
**One crate per feature area with a shared `common` crate.** Rejected: a `common` crate is a
sibling dependency under the strict rule, and a crate without a responsibility is a crate
without a boundary.
