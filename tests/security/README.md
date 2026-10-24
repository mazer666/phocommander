# Negative tests — the security test protocol of Phocommander

**Binding source:** [`docs/REQUIREMENTS.md`](../../docs/REQUIREMENTS.md) — sections *Sicherheit*,
*Datenschutz* and *Test- und Qualitätsstrategie*. **Threat model:**
[`docs/security/THREAT_MODEL.md`](../../docs/security/THREAT_MODEL.md) — the rules H1 to H17 and the
residual risks.

## What this directory is

This directory holds the **negative tests** of the project: tests that perform an attack and prove
that the defence holds. A positive test — "a normal archive is extracted", "a normal file is
listed" — proves nothing about a hardening. The specification demands the negative form:

> "Für jede Härtung — Pfad-Traversal, Dekompressionsbombe, Header-Limit — existiert ein negativer
> Test, der den Angriff tatsächlich ausführt und die Abwehr belegt."

Every hostile archive in this directory is **built at run time** in a temporary directory. No binary
file is stored in the repository: a stored attack sample goes stale with every change of the
archive library, and it hides the question "what exactly does this file contain?".

## The status vocabulary

| Status | Meaning |
|---|---|
| `implemented` | The attack is performed by a test in this directory and the test is green. |
| `attack implemented` | The hostile input is built and measured by a test, the enforcement point does not exist yet. |
| `planned` | The rule and the test are fixed, the test is written when the enforcement point arrives. |

## The protocol table

| Attack vector | Test (name of the test) | What the test performs | What the defence proves | Status |
|---|---|---|---|---|
| Zip-Slip — path traversal during extraction | `security_path_traversal::every_hostile_entry_name_is_refused` | Builds a ZIP archive with the names `../escape.txt`, `../../escape.txt`, `docs/../../escape.txt`, `..\escape.txt`, `docs\..\..\escape.txt`, `/etc/passwd`, `\windows\system32\evil.dll`, `C:\Windows\evil.dll` and `d:evil.txt` at run time and hands every entry name to the hardened resolver. | The resolution refuses every name that would leave the destination — with the reason *traversal*, *absolute path* or *reserved name*. | `implemented` |
| Zip-Slip — the invariant for arbitrary names | `security_path_traversal::the_resolved_path_stays_inside` (property test, 256 cases) | Generates arbitrary names — including `..`, separators and drive letters — and resolves each of them. | Whatever the name is: the resolved path is relative and free of `..`, so it cannot leave the destination. This is the invariant of the specification's property tests. | `implemented` |
| Zip-Slip — the file that is written | `security_path_traversal::no_file_is_written_outside_of_the_destination_directory` | Resolves the names of the hostile archive and writes the accepted ones into a destination directory. | Nothing is written outside of the destination; the only file below the temporary directory is the archive itself. | `implemented` |
| Zip-Slip — during the extraction itself | `aborts_when_an_entry_leaves_the_destination` | Extracts the hostile archive with the reader of the archive crate. | The reader writes nothing outside of the destination. | `planned` — the container readers arrive in M1; `phocommander-archive` exposes `resolve_entry_name` today, nothing else. |
| Symlink escape (RUSTSEC-2025-0168, RUSTSEC-2026-0368, RUSTSEC-2026-0369) | `skips_symlink_entry_instead_of_creating_it`, `symlink_entry_cannot_redirect_a_later_entry` | Builds an archive with a symbolic link whose target points outside of the destination, followed by an entry that writes through that link. | No symbolic link is created from archive material; the entry after it cannot be redirected. | `planned` — the reader arrives in M1. The rule is H4 of the threat model. |
| SUID/SGID preservation (RUSTSEC-2026-0367) | `drops_suid_and_sgid_bits_from_extracted_file` | Builds an archive whose entry carries the set-user-id bit in `external_attributes` and extracts it. | The mode of the extracted file is masked; the set-user-id, set-group-id and the sticky bit never reach the file system. | `planned` — the reader arrives in M1. The rule is H5 of the threat model. |
| Decompression bomb — the total size | `security_decompression_bomb::the_bomb_archive_declares_more_bytes_than_the_budget_allows` | Builds a ZIP archive at run time, rewrites the size fields of its header so that it declares four gigabytes, and measures the result. | The archive exceeds the budget of the specification (`MAX_EXTRACTED_BYTES`, two gigabytes) — the operation has to abort. | `attack implemented` |
| Decompression bomb — the cost for the attacker | `security_decompression_bomb::the_bomb_archive_costs_the_attacker_almost_nothing` | Measures the size of the same archive on the disk. | The bomb weighs a few kilobytes and expands to gigabytes — the ratio that makes it dangerous. | `attack implemented` |
| Decompression bomb — the expansion ratio | `security_decompression_bomb::a_highly_compressible_entry_expands_far_beyond_its_own_size` | Builds an archive with one entry of eight megabytes of zeroes, compressed. | The entry expands by more than a factor of one thousand — the ratio the limit has to catch. | `attack implemented` |
| Decompression bomb — the number of entries | `security_decompression_bomb::an_archive_with_too_many_entries_exceeds_the_entry_limit` | Builds an archive with one entry more than `MAX_ARCHIVE_ENTRIES`. | The archive holds more entries than the limit of the specification allows. | `attack implemented` |
| Decompression bomb — the abort during the extraction | `aborts_when_total_size_limit_is_exceeded`, `aborts_when_entry_count_limit_is_exceeded`, `aborts_when_single_entry_size_limit_is_exceeded` | Extracts the three hostile archives with the reader of the archive crate. | The extraction aborts with a readable message and writes nothing. | `planned` — the reader arrives in M1. The rule is H6 of the threat model. |
| TOCTOU between the check and the access | `checks_the_path_immediately_before_the_access` | Swaps a path component for a symbolic link between the check and the access. | The access is descriptor-based, or the check is repeated immediately before it. The remaining window is a named residual risk (§4 of the threat model). | `planned` — the file operations arrive in M1. |
| Header overflow — the forged size | `security_decompression_bomb::the_bomb_archive_declares_more_bytes_than_the_budget_allows` | Rewrites the size fields of a ZIP header so that the archive declares a size it does not have. | A parser must not trust the declared size; it has to count the bytes it really processes. | `attack implemented` — the enforcement arrives with the reader in M1. The rule is H12 of the threat model. |
| Header overflow — the parsers | `fuzz_archive_header`, `fuzz_image_header`, `fuzz_charset_detection` | Feeds malformed headers to the parsers in a fuzz run. | The parsers survive malformed input; no panic, no oversized allocation. | `planned` — the fuzz targets arrive in M1; the nightly CI already has the job that would run them. |
| Windows path length | `security_path_traversal::a_name_that_is_too_long_is_refused` | Resolves a name that is longer than the limit of the resolver. | The name is refused before it reaches the file system, with the length and the limit named. | `implemented` — the limit of the resolver is `MAX_ENTRY_NAME_BYTES` (4096 bytes). The manifest entry `longPathAware` arrives with the Windows port in M3. |

## What is measured and what is not

- **Measured:** every statement in the *What the test performs* column really happens at run time.
  The archives are built, read back and measured; the numbers quoted in the test are the numbers the
  test asserts.
- **Not measured:** the rows with the status `planned`. They name the test that will exist and the
  rule it will prove. A row is only moved to `implemented` when the test is green in the CI — the
  status is not a promise.

## How to run them

```bash
cargo test -p phocommander-integration-tests --test security_path_traversal
cargo test -p phocommander-integration-tests --test security_decompression_bomb
```

Both tests build their hostile archives in the temporary directory of the system and remove it
afterwards. Nothing is written into the repository.
