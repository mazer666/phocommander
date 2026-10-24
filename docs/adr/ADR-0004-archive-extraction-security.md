# ADR-0004 — The security strategy for the extraction of archives
**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Rust Architect
**Related requirement IDs:** FR-59, FR-60, FR-61, FR-63, NFR-07, NFR-09, NFR-33, NFR-37

## Context

An archive is untrusted input. It arrives from a download, from a message, from a network
share. Everything in it — the names of the entries, the sizes they claim, the permissions
they carry, the links they point to — is chosen by whoever built the archive.

The specification names the attacks and the decisions:

- **Path traversal (zip slip).** The `zip` crate offers `enclosed_name()`, which rejects
  absolute paths and `..` components. The specification states that this alone is not
  enough: RUSTSEC-2025-0168 broke the path canonicalisation of the crate, and three further
  advisories — RUSTSEC-2026-0367 (SUID/SGID bits survive the extraction),
  RUSTSEC-2026-0368 (symlinks pointing outside the target directory) and RUSTSEC-2026-0369
  (Unix permissions set through symlinks) — use the names of the entries and the links
  inside the archive to write outside the directory the user chose.
- **Decompression bombs.** A small archive can claim or expand into gigabytes.
- **The window between the check and the use.** The standard library offers no atomic file
  operation with path semantics; a path can be replaced between the check and the write.
- **Symlinks during copy and delete.** Copying a symlink as a symlink and unlinking it
  instead of deleting its target.

Decision E3 of the owner fixes the strictness: no `extract()` of the archive crates, our own
canonicalised path resolution, no symlinks created from archive material, size limits against
decompression bombs, `cargo-deny` and `cargo-audit` as hard gates in the CI.

The strategy therefore has to be decided before the first reader is written, because it
decides the shape of the adapter.

## Decision

**1. The extraction never calls `extract()`.** The adapter iterates over the entries of the
container and writes every entry itself. Only the parsing of the container format comes from
the library; the decision about *where* something is written comes from this project.

**2. Every target path is resolved by one function, and only by that function.**
`phocommander_archive::resolve_entry_name` turns the untrusted name of an entry into a
relative path. It refuses:

| Refused | Because |
|---|---|
| an empty name | there is nothing to write |
| a name longer than 4096 bytes | no file system accepts it; the name is an attack or a mistake |
| a name with a control character | the name would not be shown, but would be used |
| a name that starts with `/` or `\` | the path would be absolute |
| a name with a drive letter (`C:`, `d:`) | the path would leave the destination on Windows |
| a `..` segment, with `/` or `\` as separator | the path would leave the destination |
| a segment with a colon | alternate data stream on NTFS |
| a reserved device name (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`) | Windows silently redirects such a write |

Both separators are honoured, because an archive written on Windows may use `\` and a name
that contains `\` must not become a single file name on Unix. The result is always relative
and never contains a `..` segment, so that joining it with the destination cannot leave the
destination.

**3. The destination is canonicalised, and the resolved path is checked against it.**
The destination is canonicalised once per extraction. Every resolved path is joined with the
canonical destination and canonicalised again — with the symbolic links of the destination
resolved — and the result has to be inside the destination. A path that is not inside is
refused, not clamped and not repaired.

**4. Symlinks are never created from archive material.** An entry that is a link is either
skipped with a message or written as a regular file that contains the target of the link as
text. The second choice is the safer one, because it keeps the content visible without
creating a link that points somewhere the user did not choose. The decision is applied by
the adapter of M1 and is recorded here so that every adapter makes the same choice.

**5. Three limits, and they are read from one place.** The canonical values live in
`phocommander-core::config` and reach the adapter as values:

| Limit | Value | What it stops |
|---|---|---|
| `MAX_EXTRACTED_BYTES` | 2 147 483 648 | the total size of an extraction |
| `MAX_ARCHIVE_ENTRIES` | 100 000 | an archive that declares millions of entries |
| `MAX_VIEWER_FILE_SIZE` | 10 485 760 | the viewer loading a file completely |

A limit is checked *before* the write, and the total is accumulated during the extraction,
so that an archive cannot exceed the limit by spreading it over many entries.

**6. The window between the check and the write is closed with descriptor-based access.**
Where the check of a target and the access to it can come apart, the access is made through a
descriptor relative to the already opened directory (`openat` semantics, through `rustix` or
`cap-std`), not through a path that is resolved a second time. Where that is not possible, the
check is repeated immediately before the access. The remaining race is named in the threat
model rather than hidden.

**7. The version of the `zip` crate is pinned from below at 9.0.5.** The four advisories
above are fixed in that release. The catalogue in the root manifest carries the pin, and
`cargo deny check` is a gate that fails the build when a version below the pin is resolved.

**8. Every hardening has a negative test that performs the attack.** A positive test proves
nothing about a defence. The tests in `tests/security/` build the hostile archives at run
time — no attack sample is stored in the repository — and assert that the defence holds.

## Consequences

### Positive
-
The trust boundary of the extraction is one function with one table of rules. A reader that
forgets a rule cannot be written, because there is only one place that resolves a name.
-
The negative tests are real: they build a ZIP with `../escape.txt`, `/etc/passwd`,
`C:\Windows\evil.dll` and the hidden variant `docs/../../escape.txt`, hand every name to the
resolution, and assert that each one is refused and that nothing was written outside of the
destination.
-
The limits are read from the canonical constants, and the integration test of the package
`tests` holds the constants and the shipped configuration file against each other, so the
two cannot drift apart unnoticed.
-
The strategy is independent of the container format. The same rules apply to ZIP, TAR and
7z, because the rules live in the project and not in the library.
-
`cargo deny check` is green with the pin, and the advisories check reports no known
advisory for the resolved versions.

### Negative / Trade-offs
-
The adapter writes more code than `extract()` would need: it has to create the directories,
set the permissions itself, decide about the links and count the bytes. That is the price of
the decision E3, and it is the intended price.
-
Descriptor-based access is more verbose and needs `rustix` or `cap-std`, which are not yet
dependencies of the workspace. They are in the catalogue; the adapter of M1 adds them.
-
Not every archive can be extracted: an archive with links, with reserved device names or
with absolute paths is refused as a whole or entry by entry. The specification demands this
strictness, and a user who misses such an archive is told why.
-
Windows has to be taken into account even when the application runs on Linux, because an
archive written on Windows may carry names that are dangerous on Windows. The rules are
therefore stricter than the file system of the machine that extracts.

## Alternatives Considered
-
**Using `extract()` of the archive crates with `enclosed_name()`.** Rejected: the
specification forbids it explicitly, and the advisories show that the protection of the
library was not sufficient on its own.
-
**Canonicalising the destination only and trusting the name.** Rejected: the name decides
the target, and a name that leaves the destination is exactly the attack.
-
**Repairing a hostile name instead of refusing it** — replacing `..` with `_` and writing
the file anyway. Rejected: the user would get a file that the archive did not contain, and
the extraction would look successful. A refusal with a clear message is honest; a silent
repair is not.
-
**Extracting into a temporary directory and moving the result afterwards.** Rejected as the
primary defence: it doubles the space that is needed and it does not remove the need to
resolve the names. It remains a reasonable addition for the decompression bomb, because it
makes the cleanup of a partial extraction easy.
-
**A different `zip` version with a newer fix.** The pin is `9.0.5`; the resolved version is
`9.1.0`. A newer version is acceptable — the pin is a minimum, not an exact version.
