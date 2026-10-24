# ADR-0002 — The filesystem abstraction
**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Rust Architect
**Related requirement IDs:** FR-14, FR-60, FR-96, FR-118, FR-120, NFR-32, NFR-33

## Context

The platform matrix of the specification is the reason the architecture exists. On macOS the
application runs without a sandbox and needs a permission the user grants by hand. On iPadOS
the process is fully sandboxed and can only reach the directories that the user released with
the document picker, through security-scoped URLs. On Android the Storage Access Framework
grants access to the roots the user selects, and full access to the storage is a special
permission that has to be declared, explained and requested. On Windows, Linux and BSD there
is no sandbox at all, but the path length rules differ and the trash behaves differently.

Without an abstraction, every one of these platforms grows its own special case: a different
directory listing here, a different copy operation there, a different error message for a
file that the user may not read. The core would grow six code paths for one concept, and the
test coverage would be measured on one of them.

The risk register of the specification names this as the highest risk of the project, with
the impact "high" and the likelihood "medium", and names the mitigation: the interface is
decided in an ADR before M1, and the network sources of M6 are played through in M1 as a
test implementation. This ADR is that decision.

## Decision

**1. One abstraction, one method that matters.** The core asks a source for the content of a
directory, and a source answers with a list of entries. The interface lives in
`phocommander-core`, because the core is the only level that every other level may use.

```rust
/// One entry of a directory, as the core sees it.
pub struct Entry {
    pub name: String,
    pub kind: EntryKind,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

/// A source of directory content: a local directory, an archive, a remote
/// server, a root the user released on iPadOS.
pub trait Source {
    type Error;
    fn list(&self, path: &Path) -> Result<Vec<Entry>, Self::Error>;
}
```

The trait is deliberately small. Everything that distinguishes the sources — the protocol, the
permissions, the timeouts, the format of the answer — belongs behind it, not in the core.

**2. A source is identified by an opaque handle, not by a path.** The iPadOS root is not a
path but a security-scoped bookmark, and the archive entry is not a path but an index into
the archive. The core therefore never builds a path to a remote entry and never assumes that
a path can be reopened later. A path is what a *local* source understands.

**3. The errors of the sources are their own.** The trait is generic over its error type, so
that the SFTP source reports an authentication failure and the SAF source reports a revoked
permission, and the core does not have to know either of them. The core turns them into its
own error type at the boundary — the rule is described in ADR-0003.

**4. The write side is a separate, narrower interface.** Reading a directory is the operation
that every source supports. Writing is not: an archive can be read but not modified, and a
released iPadOS root is read-only. The operations of M1 therefore work on a
`MutableSource`, and a source that cannot be written to simply does not implement it. The
conflict policy and the job queue belong to the operations, not to the source.

**5. The core never learns a protocol, and the protocol never learns the core.** The remote
crate of M6 implements the same interface for SFTP, FTP, SMB, WebDAV and S3; the archive
crate of M1 implements it for the archive formats; the platform adapter of M4 implements it
for the Android SAF roots and the iPadOS document picker.

## Consequences

### Positive
-
One concept, one code path in the core. A copy operation works the same on a local directory,
in an archive and on an SFTP server, because the operation only knows the interface.
-
The core stays testable everywhere: it depends on `thiserror` and on the standard library,
and the test implementation of the interface needs neither a network nor a permission.
-
The network sources of M6 become an addition instead of a special case. The specification
says it: "Netzwerkzugriff ist kein eigener Funktionsbereich, sondern eine weitere Quelle in
derselben Abstraktion."
-
The trust boundary stays in one place. Everything a server or an archive delivers — names,
sizes, timestamps — is untrusted input, and the interface is the place where that is
declared.
-
The iPadOS limitation becomes visible in the type system: a released root is a handle, not a
path, and the code cannot silently treat it like a directory on disk.

### Negative / Trade-offs
-
A generic trait with an associated error type is more code to write than a trait with one
error enum. The price is paid once per source, and it buys a core that stays free of
protocol details.
-
The source has to answer questions the local file system answers for free — how large is the
entry, when was it changed. A remote source that cannot answer has to say so, and the
interface needs a way to express "unknown". The optional fields of `Entry` are that way.
-
The write side being a separate interface means that the operations of M1 have to be written
twice for the two shapes. The alternative — one interface with a capability flag — moves the
distinction into a runtime check instead of into the type.
-
ADR-0001 makes this expensive: the trait and the `Entry` type live in
`phocommander-core`, and no other crate may depend on `phocommander-core` under the strict
reading of the boundary rule. The interface is therefore, for the moment, a decision on
paper. It becomes code as soon as the owner has decided about Option A of ADR-0001 — or the
composition root has to pass the trait object around, which is possible but clumsy.

## Alternatives Considered
-
**One type per source with a match statement in the core.** Rejected: the core would grow
one arm per source, and every new source would touch the core. The specification demands
that a new format is "a new implementation plus one registration line — without a change in
the core".
-
**A path-based abstraction with a prefix for the remote sources** (`sftp://host/path`).
Rejected: it invites the code to treat a remote entry like a local file, to open it with
`std::fs`, and to build a path from a string the server delivered. The trust boundary of
the specification forbids exactly that.
-
**An enum of the sources in the core.** Rejected for the same reason as the match statement:
the core would have to know every source.
-
**Using an existing library for the abstraction.** Rejected: no library covers a local
directory, an archive, an SFTP server and an iPadOS bookmark with the same interface, and a
library would decide the error handling of the project.
