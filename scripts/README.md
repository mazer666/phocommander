# The check scripts of Phocommander

Two gates that the CI runs and that a contributor can run locally. Both are
plain shell scripts, they need no toolchain of their own and they change
nothing in the repository.

## `check-crate-boundaries.sh`

Reads the `Cargo.toml` of every workspace member, evaluates every path
dependency and checks it against the dependency rule of the project.

The rule, in full:

| Level | May depend on | May not depend on |
|---|---|---|
| `crates/*` | — (no sibling crate) | `crates/*`, `platform/*`, `apps/*`, `xtask` |
| `platform/*` | `crates/*` | `platform/*`, `apps/*`, `xtask` |
| `apps/*` | `platform/*`, `crates/*` | `apps/*`, `xtask` |
| `xtask` | `crates/*` | `apps/*`, `platform/*` |
| the root manifest | any member | a directory that is not a member |

The script distinguishes a **path dependency** (a crate of this workspace)
from a **registry dependency** (a crate from crates.io). Only the path
dependencies are checked — a crate from crates.io is outside the boundary of
the project. A dependency that is written as `{ workspace = true }` is
resolved through the catalogue in the root manifest, so that the catalogue
cannot be used to smuggle a path dependency past the gate.

Exit code `0` means that every path dependency points in an allowed
direction. Exit code `1` means that at least one dependency violates the
rule; the script names the crate, the target and the reason.

```console
$ ./scripts/check-crate-boundaries.sh
Phocommander - crate boundary gate
rule: apps/* -> platform/* -> crates/*, no crate below crates/ may depend
      on another crate below crates/, registry dependencies are not checked

OK    apps/phocommander-desktop -> crates/phocommander-config (apps -> crates)
OK    apps/phocommander-desktop -> platform/phocommander-platform (apps -> platform)
OK    platform/phocommander-platform -> crates/phocommander-core (platform -> crates)

3 path dependencies checked, 0 violations
```

## `check-file-size.sh`

Counts the lines of every Rust source file of the repository.

* more than **400 lines** — a warning in the output, exit code `0`
* more than **550 lines** — a hard failure, exit code `1`

The soft limit is the convention of the project (a file above it needs a
reason in its module header), the hard limit is the limit that review
rejects. Only the hard limit fails the build, so the gate never blocks a
change that is still below it.

```console
$ ./scripts/check-file-size.sh
Phocommander - file size gate
soft limit: 400 lines (warning), hard limit: 550 lines (failure)

0 files above the soft limit, 0 files above the hard limit
```

## Running all of them

```console
$ cargo fmt --check
$ cargo clippy --workspace --all-targets -- -D warnings
$ cargo test --workspace
$ cargo deny check
$ ./scripts/check-crate-boundaries.sh
$ ./scripts/check-file-size.sh
```
