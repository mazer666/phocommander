# ADR-0003 — The error hierarchy and the error codes
**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Rust Architect
**Related requirement IDs:** FR-25, FR-28, FR-118, NFR-32, NFR-37, NFR-38

## Context

Every fallible operation in this workspace returns a `Result`. The rules of the project
forbid `unwrap()` and `expect()` on every path that a user can reach, and they forbid
`panic!` as an error handling strategy. That means every failure has to travel as a value,
and the value has to be good enough for three different readers:

- the user interface, which has to show a message that says what happened and what the user
  can do about it;
- the log, which has to be written without personal data — no user name, no server name, no
  IP address;
- the caller of the function, which has to decide whether to retry, to skip or to abort.

A flat error enum with a hundred variants does not serve any of the three. A string error
serves none. And a project with six crates has to decide where the errors of the crates are
composed: the crates of `crates/*` may not depend on each other (ADR-0001), so a shared
error enum for the whole project cannot live in one of them.

## Decision

**1. Every crate owns its own error type, and the types are composed at the edge.**
A crate returns the error type that describes its own failures. The composition root — the
application — turns them into what the user sees.

```rust
// in phocommander-core
#[derive(Debug, thiserror::Error)]
pub enum Error {
    NoConfigDirectory,
    TooLarge { path: PathBuf, size: u64, limit: u64 },
    Io { path: PathBuf, #[source] source: io::Error },
}
```

The crate that owns the failure knows most about it: the archive crate knows that an entry
name was refused and why, the configuration crate knows that a field is unknown, the
platform adapter knows that the operating system reported no configuration directory.

**2. The error of a crate is a closed set, not a string.** Every variant carries the data
that the reader needs: the path, the size, the limit, the name of the field. An error that
cannot name the thing that failed is a bug in the error, not in the caller.

**3. The cause is kept, never flattened.** `thiserror` with `#[source]` keeps the original
error reachable. The rule from the specification applies: the path is usually lost when an
`io::Error` travels upwards, and the path is what the user needs.

**4. The user message and the log message are two different things.** The error type carries
the facts; the user interface decides the wording, and the wording is a translation key. The
log never receives the facts that are personal — the specification is explicit about that.
An error therefore never formats itself into a sentence that is shown to the user.

**5. The error codes are stable, and they are the contract with the user interface.**
Every failure that the user can trigger gets a code. The code is what the interface maps to
a message, what the log records, and what the acceptance documentation cites.

| Code | Meaning | First crate |
|---|---|---|
| `io.read` | A file or directory could not be read | core |
| `io.write` | A file could not be written | core |
| `io.permission` | The operating system refused the access | core |
| `path.absolute` | A path was absolute where a relative one was required | core |
| `path.traversal` | A path tried to leave the destination directory | archive |
| `path.reserved` | A name is not valid on every target platform | archive |
| `limit.too_large` | A file or an archive exceeded a limit | core |
| `limit.entries` | An archive has too many entries | core |
| `config.unknown_field` | The configuration file has a field that does not exist | config |
| `config.schema_version` | The configuration file was written by a newer version | config |
| `config.invalid_value` | A value in the configuration file cannot be used | config |
| `keymap.invalid_action` | An action identifier is malformed | keymap |
| `keymap.conflict` | Two actions are bound to the same key combination | keymap |
| `platform.no_config_directory` | The operating system reported no configuration directory | platform |
| `remote.unreachable` | A remote source did not answer | remote |
| `remote.auth` | A remote source refused the credentials | remote |
| `remote.protocol` | A remote source offered a protocol that is not accepted | remote |

The codes are part of the public surface of the project. A new error gets a new code; an
existing code never changes its meaning.

**6. The prohibition on `unwrap()` and `expect()` is a lint, not a convention.**
`[workspace.lints.clippy]` denies `unwrap_used` and `expect_used`, and
`[workspace.lints.rust]` denies `unsafe_code`. The denial covers the test code as well,
because `cargo clippy --workspace --all-targets` is part of the gate. A test that needs to
fail loudly panics with a message that names the cause.

## Consequences

### Positive
-
The user interface never has to parse a string. It matches on a code and shows the message
of its translation catalogue.
-
The log stays free of personal data by construction, because the error carries facts about
paths and sizes, and the log writer decides what it writes.
-
The cause of a failure survives the trip upwards, so that a bug report can name the actual
failing call.
-
Every crate can be tested on its own: the error type of a crate is a closed set that the
crate tests can match on exhaustively.
-
The codes make the acceptance documentation precise: a test can cite `path.traversal`
instead of "the error message looked right".

### Negative / Trade-offs
-
A closed set per crate means that the number of error types grows with the number of crates.
The application has to know how to present each of them, and a new crate means a new
presentation rule.
-
The composition at the edge has to be written: the application needs a function that turns
the error of a crate into the code and the message of the interface. That function is
written once per crate and tested once per crate.
-
The code table has to be maintained. A code that is added but never used is harmless; a
code that is used but never documented makes the acceptance documentation incomplete.
-
The errors of the remote sources of M6 will need a code of their own for every failure mode
the protocols produce. The table above names three; the real number will be larger.

## Alternatives Considered
-
**One error enum for the whole project in `phocommander-core`.** Rejected: under the
strict boundary rule of ADR-0001 the core would have to name the failures of crates that may
not depend on it, and every crate would have to depend on the core to use the enum — the
forbidden sibling dependency.
-
**A single error enum in the application.** Rejected: the crates would have to return
`Box<dyn Error>` or a string, and the caller would lose the structured information that the
interface needs to decide between "retry" and "abort".
-
**Error strings only.** Rejected: the user interface would have to match on text, the log
would carry text that cannot be counted, and a translated message would break the match.
-
**The `anyhow` crate for the application.** Rejected for the same reason: `anyhow` keeps
the cause but throws away the structure that the interface needs to choose a message and the
log needs to record a code.
