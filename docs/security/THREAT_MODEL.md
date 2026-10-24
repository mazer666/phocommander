# Phocommander — Threat Model

**Status:** M0 baseline, version 1.0 · **Date:** 24.10.2026 · **Owner:** Security & Privacy
**Binding source:** [`docs/REQUIREMENTS.md`](../REQUIREMENTS.md) — sections *Sicherheit*, *Datenschutz*, *Netzwerk- und Cloud-Zugriffe*, *SMB-Sicherheitsregeln*, *Produktmaximen*. Where this document and the specification disagree, the specification wins and this document is the defect.

---

## 0. How to read this document

This document is the security contract of the project. It states three things and nothing else:

1. **What is assumed to be hostile.** The trust boundary, the attacker, and the entry points.
2. **What the code must do about it.** The binding hardening rules, each with the negative test that will prove the defence.
3. **What is accepted as a residual risk.** Named, explained, and owned — never silently dropped.

### Status vocabulary

Every row in every table carries a status. The vocabulary is deliberately narrow, because a
threat model that marks everything as "done" proves nothing.

| Status | Meaning |
|---|---|
| `documented` | The rule is fixed in this document and in the specification. No code enforces it yet. |
| `planned` | The rule is fixed, the enforcement point and the negative test are named. Implementation is scheduled but not written. |
| `in progress` | Implementation exists in the tree; the negative test is not yet green. |
| `implemented` | Implementation exists **and** the named negative test runs green in CI. |
| `accepted` | The risk is knowingly kept. The reason is written down next to it. |

**Current repository state at M0 (24.10.2026):** the workspace contains the crate skeletons and
the first hardening component (`resolve_entry_name` in `phocommander-archive`), but no archive
reader, no extraction code and no file operation. Therefore no row below may claim
`implemented` on the strength of code that does not exist yet. The one exception is stated
explicitly where it occurs: rule H3 is `in progress`, because the lexical half of the path
resolution exists with its negative tests, while the canonicalisation against the destination
directory does not. This document does not claim a single working defence that a test does not
prove.

### Evidence convention

A status of `implemented` is only allowed with a command output pasted into the milestone
acceptance document. "Should work" is not a status. "Not measured" is written as
`NOT RUN` with the exact reason, as the specification requires in *Belegregeln*.

---

## 1. Scope and trust boundary

### 1.1 The boundary

The trust boundary of Phocommander runs **between the process and the filesystem**:

```
        ┌──────────────────────────────────────────────┐
        │  Phocommander process (trusted)              │
        │  core · ops · archive · viewer · remote · ui │
        └───────────────┬──────────────────────────────┘
                        │  ← every byte crossing this line is untrusted
   ═════════════════════╪═════════════════════════════════════════════
        ┌───────────────┴──────────────────────────────┐
        │  Local filesystem · archive contents ·        │
        │  remote servers (SMB / WebDAV / S3 / SFTP)    │
        └──────────────────────────────────────────────┘
```

Everything below the line is attacker-controlled. There is no exception: not an archive the
user downloaded from a trusted website, not a file on the user's own disk, not a share on the
user's own LAN, not a file the user created with this application five minutes ago.

### 1.2 The attacker

The attacker is **anyone who can put content in front of the user**. Concretely:

| Attacker | Capability | Reach |
|---|---|---|
| The sender of an archive | Chooses entry names, sizes, contents, Unix mode bits, symlink targets, compression ratio | Any archive the user opens, previews or extracts |
| The author of a file on disk | Chooses the file name, the file size, the content, the symlink structure of the directory tree | Every directory the user lists, copies, searches or previews |
| A hostile or compromised server | Chooses directory names, file names, reported sizes, timestamps, response size, response timing, protocol version, TLS certificate | Every remote source |
| A local unprivileged process running as the same user | Writes and replaces files while Phocommander operates | TOCTOU windows, symlink swaps, race conditions |

The attacker does **not** need to defeat cryptography, the sandbox or the user. The attacker
needs the user to double-click one file, or the user to type one server address.

### 1.3 Content is hostile regardless of origin

Two rules follow, and they are the reason the rest of this document exists:

1. **Archive content is code-adjacent data.** An archive entry name is a string that the
   application will later turn into a filesystem path. It is treated exactly like a request
   from the network.
2. **Network sources are archives.** A directory listing from an SMB or WebDAV server is a
   list of attacker-chosen names, sizes and timestamps, delivered in a protocol that the
   application must not trust. The same canonicalisation, size limits, name normalisation and
   timeout rules apply to a remote listing as to an archive entry.

### 1.4 What is explicitly out of scope

Stating this is part of the threat model; an unbounded scope is not a threat model.

- Compromise of the operating system kernel, of the Phocommander process itself, or of the
  user account by means unrelated to Phocommander.
- Physical attacks on the host (cold boot, DMA, bus sniffing).
- Malware that is already running as the user and does not need a Phocommander bug.
- Legal interception with a trusted root certificate installed in the system trust store —
  the user has already made that decision for every other application on the machine.
- Denial of service that requires the attacker to already fill the user's disk.

---

## 2. Attack surface

| Attack surface | Attacker | Entry point | Damage potential | Countermeasure | Status |
|---|---|---|---|---|---|
| Archive extraction: path traversal (Zip-Slip) | Archive sender | Entry name with `..` or an absolute path, e.g. `../../.ssh/authorized_keys` | Arbitrary file write outside the destination → remote code execution, overwritten configuration, destroyed data | H2 (no `extract()`), H3 (canonicalise every target path and confine it to the destination directory) | `planned` |
| Archive extraction: symlink whose target points outside the destination directory | Archive sender | Unix symlink entry with an attacker-chosen link target | Read or write of arbitrary files outside the destination; combined with a later archive entry it becomes arbitrary file write | H4 (never create symlinks from archive material) — the entry is skipped and reported, never materialised | `planned` |
| Archive extraction: Unix permissions applied through symlinks | Archive sender | Symlink entry followed by a permission change on the link path | `chmod` of arbitrary directories and files outside the destination → privilege escalation on multi-user systems | H4 (no symlinks) plus H5 (masked mode bits) plus H3 (confinement before any metadata write) | `planned` |
| Archive extraction: SUID/SGID bits preserved | Archive sender | `external_attributes` with `S_ISUID` / `S_ISGID` set | Local privilege escalation when a privileged process (installer, CI job, container build) extracts the archive | H5 (mode bits are masked to `0o644` / `0o755`; SUID, SGID and sticky are dropped unconditionally) | `planned` |
| Archive extraction: decompression bomb | Archive sender | One entry or the whole archive with a huge expansion ratio | Memory exhaustion, process crash, disk exhaustion of the destination | H6 (three limits: total extracted bytes, entry count, single-entry size) | `planned` |
| TOCTOU between path check and access | Local process running as the same user | Symlink or file replaced between the check and the access | Write outside the destination despite a correct check | H9 (descriptor-based access via `openat` semantics through `rustix` / `cap-std`, or the check is repeated immediately before the access). The remaining window is a **named residual risk** — see §4 | `planned` |
| Symlink escape during recursive copy | Local process running as the same user, or a directory tree containing symlinks | Symlink inside the source tree | Content from outside the source tree copied into the destination, or content written outside it | H10 (symlinks are copied as symlinks, never dereferenced; delete unlinks instead of recursing). The remaining window is a **named residual risk** — see §4 | `planned` |
| Header overflow in the format parsers (text, image, archive) | Archive sender or file author | Malformed header with an implausible length or count field | Panic, integer overflow, huge allocation, out-of-bounds read in a dependency | H11 (every parser validates its header fields against explicit limits before allocating; fuzz targets in the nightly CI) | `planned` |
| Memory exhaustion through very large directories | Local process or hostile server | Directory with hundreds of thousands of entries | Process killed by the OOM killer, UI freeze | H12 (streaming directory read with a bounded in-memory window; the list is drawn from a slice, not from a full copy) | `planned` |
| Path length on Windows (260 characters without opt-in) | Archive sender or file author | Long entry names, deeply nested trees | Extraction fails with an opaque error, or silently truncates | H13 (`longPathAware` in the application manifest; explicit length check with a comprehensible error message; the manifest is checked in CI) | `planned` |
| Permission takeover on macOS, Android, iPadOS | User (accidentally) or a malicious profile | App requests broad access, or silently escalates it | Read or write of data the user never intended to expose | H14 (permission model per platform, §6: no App Sandbox on macOS with an explicit user-granted *Full Disk Access*; SAF as the Android default; security-scoped bookmarks on iPadOS) | `planned` |
| Network source: certificate verification bypassed (TLS interception) | Network attacker or malicious server | Self-signed certificate accepted without a check | Complete loss of confidentiality and integrity of the session, credential theft | H15 (strict certificate verification, **no opt-out**; a self-signed certificate is only accepted as a deliberately pinned, PIN-protected exception for exactly one host) | `planned` |
| Network source: response larger than expected (memory exhaustion) | Hostile server | Directory listing of several gigabytes, or a file that grows during download | Memory exhaustion of the client | H16 (fixed read budget; the client aborts and reports instead of growing with the response) | `planned` |
| Network source: protocol downgrade (SMB1, NTLMv1, WebDAV without TLS) | Network attacker | Server offers only an obsolete protocol version | Known-vulnerable protocol, relay and downgrade attacks | H17 (SMB1 and NTLMv1 are not implemented; SMB signing and SMB3 transport encryption are requested; a server that only offers SMB1/NTLMv1 is rejected with a readable message; WebDAV without TLS is rejected when HTTPS is not reachable) | `planned` |
| Network source: injected directory names (separators, control characters, over-long paths) | Hostile server | Directory listing containing `../`, `\` , NUL, ANSI escapes, or 4 KiB names | Path confusion, log injection, terminal escape injection, UI spoofing | H18 (names are normalised, checked against a forbidden-character set, and truncated for display while the real name is kept) | `planned` |
| Network source: hanging response without a time limit | Hostile server | Response never ends | Job queue stuck forever, user cannot cancel | H19 (every network operation has a timeout; connect, read and total duration are all bounded) | `planned` |

---

## 3. The binding hardening rules

The rules below are binding. They are not guidance, not a wish list, and not negotiable per
pull request. A change that weakens one of them needs a new decision in the specification and
a dated entry in *Getroffene Entscheidungen*.

Each rule names the crate that owns the enforcement and the negative test that will prove it.
**A rule without a negative test is a comment, not a feature** — so the test column is part of
the rule, not an optional extra.

| # | Rule | Enforcement point | Negative test (planned) | Status |
|---|---|---|---|---|
| H1 | `zip` must be **9.0.5 or newer**. This is the newest published release and it fixes all four RustSec advisories listed below. | `Cargo.toml` (workspace dependency catalogue) | `scripts/check-dependency-pins.sh` asserts `zip >= 9.0.5`; `cargo deny check` fails on any advisory against the locked version | `in progress` — the pin is present (`zip = "9.0.5"`) and the advisory gate runs **green** against the real dependency graph (`advisories ok`, 24.10.2026, see §8.3). The dedicated pin script is not written yet, so a downgrade would only be caught by the advisory gate, not by a dedicated check. |
| H2 | **Never call `extract()`** from `zip`, `tar` or any other crate. Iterate over the entries and write them yourself. | `crates/phocommander-archive` | `deny_extract_call.rs` — a source-level check that rejects `\.extract\(` in the archive crate; plus `extracts_archive_without_calling_extract` (behavioural) | `planned` |
| H3 | **Canonicalise every target path and confine it to the destination directory** before the first byte is written. A path that resolves outside the destination is rejected, not repaired. | `crates/phocommander-archive` | `rejects_entry_that_escapes_the_destination`, `rejects_absolute_path_entry`, `rejects_entry_escaping_via_intermediate_symlink` | `in progress` — the **lexical** half exists: `resolve_entry_name` rejects absolute paths, `..` segments, control characters, Windows device names, alternate data streams and over-long names, with negative tests (`cargo test -p phocommander-archive`: 12 unit tests + 3 doc-tests green on 24.10.2026). The **canonicalisation and confinement against the destination directory** is not implemented yet: no `canonicalize` call exists anywhere in the tree, so a symlinked component *inside* the destination is still not detected. |
| H4 | **Never create a symlink from archive material.** A symlink entry is reported and skipped; the extraction continues with the remaining entries. | `crates/phocommander-archive` | `skips_symlink_entry_instead_of_creating_it`, `symlink_entry_cannot_redirect_a_later_entry` | `planned` |
| H5 | **Never carry over SUID, SGID or sticky bits.** The mode is masked to `0o644` for files and `0o755` for directories; the owner, group and other write bits are decided by the application, not by the archive. | `crates/phocommander-archive` | `drops_suid_and_sgid_bits_from_extracted_file`, `drops_suid_and_sgid_bits_from_extracted_directory` | `planned` |
| H6 | **Decompression-bomb limits**: a limit on total extracted bytes, on the number of entries and on the size of a single entry. Exceeding a limit aborts the operation with a readable message. | `crates/phocommander-archive` | `aborts_when_total_size_limit_is_exceeded`, `aborts_when_entry_count_limit_is_exceeded`, `aborts_when_single_entry_size_limit_is_exceeded` | `planned` |
| H7 | **SMB1 and NTLMv1 are not implemented.** SMB signing and SMB3 transport encryption are requested; a server that only offers SMB1 or NTLMv1 is rejected with a readable message — never "tried and maybe it works". | `crates/phocommander-remote` | `rejects_server_that_only_offers_smb1`, `rejects_server_that_only_offers_ntlmv1`, `requests_smb_signing_when_the_server_supports_it` | `planned` |
| H8 | **HTTPS certificates are verified strictly. There is no opt-out.** A self-signed certificate may only be stored as a deliberately pinned, PIN-protected exception for exactly one host. | `crates/phocommander-remote` | `rejects_expired_certificate`, `rejects_hostname_mismatch`, `rejects_untrusted_self_signed_certificate`, `accepts_pinned_self_signed_certificate_for_that_host_only` | `planned` |
| H9 | **Every network operation has a time limit** — connect, per-read and total. A server that never finishes produces a cancellation with a readable message, not a stuck job. | `crates/phocommander-remote` | `times_out_on_a_server_that_never_answers`, `times_out_on_a_server_that_stops_mid_response` | `planned` |
| H10 | **Credentials live in the OS keychain, never in the configuration file.** The configuration file stores a keychain reference, not a secret. On platforms without a keychain the application falls back to the app sandbox and says so. | `crates/phocommander-remote`, `platform/phocommander-platform` | `configuration_file_contains_no_password`, `credential_round_trip_through_keyring` | `planned` |
| H11 | **Symlinks are copied as symlinks, never dereferenced**, and are unlinked on delete instead of being deleted recursively. | `crates/phocommander-ops` | `copies_symlink_as_symlink_instead_of_following_it`, `deletes_symlink_without_following_it` | `planned` |
| H12 | **Every parser validates its header before allocating.** Length, count and offset fields are range-checked against explicit limits. | `crates/phocommander-viewer`, `crates/phocommander-archive` | `rejects_text_file_with_implausible_length_field`, `rejects_image_header_with_implausible_dimensions`, fuzz targets `fuzz_archive_header`, `fuzz_image_header` | `planned` |
| H13 | **Long paths are handled, not truncated.** `longPathAware` is set in the Windows manifest; a path that exceeds the limit produces a comprehensible error. | `apps/phocommander-desktop` | `windows_manifest_contains_long_path_aware` (manifest assertion), `reports_path_length_limit_instead_of_truncating` | `planned` |
| H14 | **Permissions are requested, never taken.** The platform permission model of §6 is implemented as declared; nothing is escalated silently. | `platform/phocommander-platform` | `android_saf_mode_works_without_all_files_access`, `ipad_bookmark_access_can_be_revoked`, `macos_full_disk_access_denial_is_handled` | `planned` |
| H15 | **The response size is bounded.** The client reads within a fixed budget and aborts when the budget is exhausted. | `crates/phocommander-remote` | `aborts_when_directory_listing_exceeds_the_budget`, `aborts_when_download_exceeds_the_budget` | `planned` |
| H16 | **Remote names are normalised and validated.** Separators, control characters and over-long names are neutralised for display and for path building; the real name is preserved for the operation itself. | `crates/phocommander-remote` | `rejects_remote_name_containing_path_separators`, `rejects_remote_name_containing_control_characters`, `truncates_overlong_remote_name_for_display_only` | `planned` |
| H17 | **Secure deletion follows NIST SP 800-88 Rev. 2**, with the *Clear* level as the default and the flash-media limitation stated in the user interface — not only in the manual. | `crates/phocommander-ops` | `clear_overwrite_replaces_the_file_content`, `secure_delete_ui_states_the_flash_media_limitation` | `planned` |

### 3.1 The `zip` advisories this is based on

All four entries below were verified against the RustSec advisory database on **24.10.2026**
(`https://rustsec.org/`, source `https://github.com/rustsec/advisory-db`). The raw advisory
files and the rendered pages were fetched and read; the versions quoted are the ones the
database names.

| Advisory | Aliases | CVSS | Patched in | Affected function | Consequence |
|---|---|---|---|---|---|
| `RUSTSEC-2025-0168` | CVE-2025-29787, GHSA-94vh-gphv-8pm8 | 7.3 HIGH | `>= 2.3.0` | `zip::read::ZipArchive::extract`, `zip::unstable::stream::ZipStreamReader::extract` | Broken path canonicalisation during extraction: a symlink earlier in the archive steers a later entry out of the destination directory → arbitrary file overwrite |
| `RUSTSEC-2026-0367` | GHSA-vv5m-jxxh-4627 | 7.3 HIGH | `>= 9.0.0` | `ZipArchive::extract`, stream extraction API | SUID/SGID/sticky bits from `external_attributes` survive extraction → local privilege escalation when a privileged process extracts an attacker-controlled archive |
| `RUSTSEC-2026-0368` | GHSA-4v9h-p7m9-rh26 | — | `>= 9.0.0` | `ZipArchive::extract` | Symlinks are created with attacker-controlled targets outside the destination directory → subsequent reads and writes hit the external target |
| `RUSTSEC-2026-0369` | GHSA-4684-h2q9-2vcp | 9.1 CRITICAL | `>= 9.0.0` | `ZipStreamReader::extract`, `ZipArchive::extract` | Unix permissions are applied *through* extracted symlinks without confining the target → `chmod` of arbitrary paths |

**Why the pin is 9.0.5 and not 9.0.0.** The advisory floor is `>= 9.0.0` for the three 2026
entries and `>= 2.3.0` for `RUSTSEC-2025-0168`. The workspace pins **9.0.5**, which is the
newest release published on crates.io at the time of writing, so the pin sits strictly above
every advisory floor and above the release that the advisories describe as affected
(`9.0.0-pre2` in `RUSTSEC-2026-0368`). The pin is a floor, not a ceiling: `zip = "9.0.5"`
resolves to `>= 9.0.5, < 10.0.0`.

**Why the floor is not the whole defence.** `RUSTSEC-2026-0368` states that the link target
bytes were passed "directly to the platform symlink API" — the advisory floor fixes the crate,
it does not make `extract()` safe for a project that wants to refuse symlinks outright. A
project requirement ("symlinks are never created from archive material") cannot be expressed
as a version requirement. That is why **H2 (no `extract()`) and H4 (no symlinks) are separate
rules** and each needs its own negative test.

**The API surface the rules rely on** (`zip` 9.0.5, verified on docs.rs):

- `ZipFile::enclosed_name() -> Option<PathBuf>` — "Ensure the file path is safe to use as a
  `Path`". The documentation of `ZipFile::name` explicitly warns: "Carelessly writing to these
  paths allows an attacker to craft a ZIP archive that will overwrite critical files. You can
  use the `ZipFile::enclosed_name` method to validate the name as a safe path."
- `ZipFile::is_symlink() -> bool` — the check that implements H4.
- `ZipFile::unix_mode() -> Option<u32>` — the source of the mode bits that H5 masks.
- `ZipArchive::len()`, `ZipArchive::decompressed_size()`, `ZipArchive::has_overlapping_entries()`
  — the inputs for the H6 limits.
- `ZipArchive::extract()` — **exists and must not be called** (H2).

### 3.2 The same rules for the other archive formats

The hardening is a property of the extraction code in `phocommander-archive`, not of one
crate. `tar` is subject to the same rules, and the RustSec database carries two advisories for
`tar` that were verified on the same date:

| Advisory | Aliases | Patched in | Consequence |
|---|---|---|---|
| `RUSTSEC-2026-0067` | CVE-2026-33056, GHSA-j4xf-2g29-59ph | `>= 0.4.45` | `unpack_in` can `chmod` arbitrary directories by following symlinks (`fs::metadata()` follows the link) |
| `RUSTSEC-2026-0068` | CVE-2026-33055, GHSA-gchp-q4r4-x4ff | `>= 0.4.45` | `tar-rs` ignores PAX size headers when the header size is non-zero |

Both affect `tar::Entry::unpack` and `tar::Entry::unpack_in` — the same "let the crate
extract" pattern that H2 forbids. `tar = "0.4"` resolves to `>= 0.4.45` today, but the
hardening still does not rely on it: `phocommander-archive` iterates the entries itself and
applies H3 to H6 to every format. `flate2` has **no** advisory in the RustSec database (checked
24.10.2026); it is a decompressor, not an extractor, and it stays behind the H6 limits.

---

## 4. Residual risks that are knowingly accepted

The Rust standard library offers **no atomic path operations**. There is no "open this path
only if it stays inside this directory" call, and there is no way to make a check and the
subsequent access a single syscall. Any hardening that works on paths alone therefore leaves a
window. This section names the window instead of hiding it.

### 4.1 TOCTOU between the path check and the access

**The window.** Between the moment the canonicalised path is verified to be inside the
destination and the moment the file is opened or created, another process running as the same
user can replace a path component with a symlink. The check passed; the access lands outside.

**What shrinks the window.**

1. **Descriptor-based access.** `cap-std` and `rustix` provide `openat`-style access: the
   destination directory is opened once as a directory handle, and every subsequent operation
   happens *relative to that handle*. A symlink swapped in above the handle cannot redirect
   the operation, because the resolution starts from the already-open directory, not from a
   path string. This is the primary mitigation and it is the reason `cap-std` and `rustix`
   are in the workspace dependency catalogue.
2. **Repeating the check immediately before the access.** Where descriptor-relative access is
   not available, the canonicalisation and confinement check is re-executed directly before the
   open/create call, in the same code path, with no intervening operation that could yield.

**What remains.** Between the last `canonicalize()` and the `open()` there is still a window
of a few microseconds in which a cooperating local attacker can swap a component. The
mitigation reduces the attack to "the attacker must win a race on the same user account",
which is a genuine reduction in severity but not an elimination.

**Platform notes.**

- On Linux, FreeBSD, NetBSD and macOS, `openat`/`O_NOFOLLOW` semantics are available through
  `rustix`; `cap-std` gives the same capability in safe Rust on top of them.
- On Windows there is no `openat` equivalent. The mitigation is therefore the repeat-the-check
  variant plus handle-based operations (`CreateFileW` with `FILE_FLAG_OPEN_REPARSE_POINT`)
  for the paths where a reparse point matters.
- On Android and iPadOS the process is sandboxed and cannot write outside its container or the
  directories the user granted, so the residual window is limited to the granted roots.

**Acceptance.** The residual race is **accepted** with the mitigations above. It is not
fixable with the standard library, and a project that forbids `unsafe` cannot hand-roll the
syscalls. The residual risk is bounded to same-user attackers and is listed in *Verbleibende
Risiken* of every milestone acceptance.

### 4.2 TOCTOU during recursive copy

**The window.** A recursive copy first *lists* a directory and then *copies* the entries it
found. A symlink created or swapped in between the listing and the copy can make the copy
read content from outside the source tree, or write it outside the destination tree.

**What shrinks the window.** The copy uses descriptor-relative access (H10 applies: symlinks
are copied as symlinks, never dereferenced), so a symlink in the source is *reproduced as a
symlink* rather than followed. The destination side is written through a descriptor relative
to the destination directory handle.

**What remains.** Without a full descriptor-based directory walk — open the directory handle,
read the entries from that handle, open each child relative to that handle, repeat — the
listing and the access cannot be bound together, and a race between them cannot be eliminated
completely. The project accepts this residual risk for the recursive copy and compensates
with:

- the source directory being opened as a handle before the walk starts,
- symlinks never being dereferenced, so the worst case of the race is a copied symlink rather
  than copied foreign content,
- and the documented rule that the user is responsible for the contents of a directory tree
  that is being modified by another process at the same time.

**Acceptance.** `accepted`. The full fix is a descriptor walk, which is a structural change
to `phocommander-ops` and is planned as a follow-up. It is named here so that nobody
mistakes the current state for a complete defence.

### 4.3 Other accepted risks

| Risk | Why it is accepted | Compensation |
|---|---|---|
| A local process running as the same user can modify files while Phocommander writes them | Phocommander has no privilege the attacker does not have; this is not a privilege boundary crossing | Operation-level checks, atomic writes via temporary file and rename, and the "user owns the directory" statement in the user guide |
| The user grants *Full Disk Access* / *all files access* on macOS / Android | A file manager without it is a file dialog, not a file manager (decision E4 in the specification) | The permission is never forced and never bypassed; it is requested explicitly, explained on first start, and revocable at any time by the user |
| A hostile server learns which directories the user asked for | The user asked for them; that is the function | The connection log contains protocol, time, result and duration only — never host, user or path (§7) |
| The flash-media limitation of the *Clear* sanitization level cannot be removed in software | Wear levelling and over-provisioning move the physical blocks out of the user's reach | The limitation is stated in the user interface next to the "secure delete" option, not only in the manual (§5) |

---

## 5. Secure deletion

### 5.1 The standard

The measure for "secure delete" in Phocommander is **NIST SP 800-88 Rev. 2**, *Guidelines for
Media Sanitization* (September 2025). The specification defines three sanitization methods
(Section 3.1):

| Level | NIST definition | What it means for a file manager |
|---|---|---|
| **Clear** | "applies logical techniques to sanitize data in all user-addressable storage locations of an ISM for protection against simple, non-invasive data recovery techniques using the same interface that is available to the user (e.g., host interface)" | Overwrite the file content through the normal file interface, then delete it. Protects against recovery with the same interface and against forensic disk imaging. |
| **Purge** | "apply physical or logical techniques that make the recovery of target data infeasible using state-of-the-art laboratory techniques but preserves the ISM in a potentially reusable state. When possible, the purge sanitization method should be used instead of the clear sanitization method" | Device-level sanitization commands, block erase, cryptographic erase. Not reachable for a single file on a general-purpose filesystem by an unprivileged application. |
| **Destroy** | "render target data recovery infeasible using state-of-the-art laboratory techniques and results in the subsequent inability to use the ISM for the storage of data" | Physical destruction of the medium. Outside the scope of an application. |

Phocommander implements **Clear** and names it that way. It does not claim Purge or Destroy,
because it cannot deliver them.

### 5.2 Why the DoD 5220.22-M three-pass overwrite is history

The DoD manual prescribed multiple overwrite passes with different binary patterns (one to
39 passes, with optional verification). NIST records the end of that rule in footnote 5 of
SP 800-88 Rev. 2:

> "The U.S. Department of Defense (DoD) Manual 5220.22, or National Industrial Security Program
> Operating Manual (NISPOM), is now Part 117 of Title 32, Code of Federal Regulations.
> **In 2006, DoD removed overwriting specifications from NISPOM.**"

The reason is in the same section: on modern media, additional passes buy nothing, because the
controller — not the application — decides where the data physically lands. A three-pass
overwrite of an SSD does not overwrite the same physical cell three times; it may not even
overwrite it once.

### 5.3 Why overwriting on flash media is not reliable

SP 800-88 Rev. 2, Section 3.1.1, states the two mechanisms explicitly:

> "For certain ISM (e.g., SSDs with overprovisioning), such practices should be avoided as very
> little confidentiality protection is achieved."

> "flash memory-based storage devices that contain spare cells and perform wear levelling make
> it infeasible for a user to sanitize all previous data using this approach because the device
> cannot support directly addressing all areas in which sensitive data has been stored using
> the native read and write interface."

The NIST text names the consequence for users explicitly:

> "Users who have become accustomed to relying on overwrite techniques on magnetic ISM and who
> have continued to apply these techniques as ISM types evolved (e.g., to flash memory-based
> devices) can be exposing their data to increased risk of unintentional disclosure."

**Consequence for the product.** The *Clear* level is the strongest guarantee a file manager
can give, and on SSD, eMMC and other flash media it is a **best-effort** guarantee. Therefore:

- The *Clear* overwrite is offered as the "secure delete" option and is described as such.
- **The limitation is stated in the user interface**, next to the option — not only in the
  manual. The wording names flash media, wear levelling and over-provisioning and says
  plainly that on such media the overwrite cannot guarantee that the old content is gone.
- The default delete action is the **trash**, not the secure delete. Moving to the trash is
  reversible and honest; a secure delete is irreversible and, on flash media, only best-effort.
- No wording anywhere in the product promises that the data is unrecoverable.

---

## 6. Permission audit per platform

The trust boundary differs per platform, so the permission model differs with it. This table is
the audit; the "Recommendation" column is the binding design for the platform adapter in
`platform/phocommander-platform`.

| Platform | Permission | Purpose | Necessary | Rationale | Recommendation |
|---|---|---|---|---|---|
| **macOS** | *Full Disk Access* (TCC) | Read and write outside the user's home directory, e.g. `/Volumes`, `/Applications`, system folders | **Yes, for full function** | The App Sandbox only grants access to files the user selected in a dialog. A sandboxed file manager is a file dialog, not a file manager (specification decision E4). | Ship **without** App Sandbox, with notarisation and Hardened Runtime. *Full Disk Access* is **never forced and never bypassed** — Apple does not allow it to be granted by an entitlement, the user grants it in System Settings, and the app must operate correctly when it is denied. The first-start page explains the requirement. |
| **macOS** | App Sandbox | – | **No** | Would restrict the product to a dialog. | Not used. Hardened Runtime and notarisation replace it. |
| **iPadOS** | Document picker (`UIDocumentPickerViewController`) | Grant access to a directory the user selects | **Yes** | The process is fully sandboxed; there is no path to the filesystem outside the container. The picker returns security-scoped URLs with return rights; the permission applies recursively to the content. | The iPadOS build is **not** a filesystem browser. It is a viewer and organiser for the roots the user explicitly grants. Access is stored as **security-scoped bookmarks**; the user can revoke it at any time in Settings and the app must handle the failure cleanly. |
| **iPadOS** | Keychain access group | Store connection credentials | **Yes** | There is no shared keychain entitlement by default; the app falls back to its own container. | Store credentials in the keychain; if the platform offers no keychain extension, fall back to the app sandbox and **say so** in the UI. |
| **Android** | Storage Access Framework (SAF) | Read and write the directories the user grants | **Yes, as the default** | SAF is the recommended and more privacy-preserving path on Android (specification, *Android*). | SAF is the **default mode** and the app is fully functional in it. No special permission is requested at startup. |
| **Android** | `MANAGE_EXTERNAL_STORAGE` (all files access) | Whole-filesystem access as a file manager | **Optional extension** | Google names file management explicitly as a permitted use case, but it requires a core-functionality explanation, a manifest declaration, Play Console review and a clear user prompt. | Offer as an **optional, clearly labelled extension**. Declare it in the Play Console with the core-functionality justification. Without it the app runs in document-browser mode. The permission is never requested silently and never bundled with the first start. |
| **Android / Horizon OS (Meta Quest)** | Same as Android | Same as Android | Same as Android | Horizon OS is Android; a Quest application is a normal Android program with a flat panel as its surface. | Identical Android permission model. No immersive-VR port, no additional permission. |
| **Windows** | `longPathAware` in the application manifest | Paths beyond 260 characters | **Yes** | Without it, Windows caps paths at 260 characters; with the manifest entry and `LongPathsEnabled`, up to 32.767 characters are available. | Set the manifest entry in `apps/phocommander-desktop`. Note that the Windows Explorer does not use long paths, so Phocommander can reach a file the Explorer cannot display — that case is caught and reported with a comprehensible message, not silently ignored. |
| **Linux** | None (no sandbox) | – | **No** | The platform has no per-application filesystem permission model. | Request nothing. Restrict by convention: configuration and state in `$XDG_CONFIG_HOME/phocommander/` and `$XDG_DATA_HOME/`. |
| **BSD** | None (no sandbox) | – | **No** | Same as Linux; FreeBSD and NetBSD are `winit` Tier 2. | Same as Linux. |

**Cross-platform rules.**

- No permission is ever escalated silently. Every elevated access is a visible, revocable
  action of the user.
- The app never asks for a permission it does not use. A permission that is not in the table
  above is a defect.
- On a platform without a permission model (Linux, BSD, Windows), the trust boundary is the
  Unix user account. That boundary is stated in the user guide, not hidden.

---

## 7. Data protection boundaries

### 7.1 What Phocommander stores

| Data | Where | Lifetime | Notes |
|---|---|---|---|
| User settings, key bindings, favourites, last opened directories | Configuration directory (`~/Library/Application Support/Phocommander/`, `%APPDATA%\Phocommander\`, `$XDG_CONFIG_HOME/phocommander/`, app sandbox on Android/iPadOS) | Until the user deletes it | The only persistent state. Schema-versioned, with a `.bak` copy on migration. |
| Connection definitions (protocol, host, port, username) | Same configuration directory | Until the user deletes it | Personal data. **Never** written to the log. |
| Credentials (passwords, tokens) | **OS keychain** (Keychain, Credential Manager, Secret Service) | Until the user deletes the entry | The configuration file holds a *reference*, not a secret. On platforms without a keychain: the app sandbox, with an explicit note that the data is then not protected by the OS. |
| Log lines | Local log file | Rotated, short retention | Protocol, timestamp, result, duration. **No** user names, **no** server names, **no** IP addresses, **no** file paths of user content. |
| Job history and panel state | Memory only | Session | Not persisted, unless the user explicitly adds a directory to the favourites. |

### 7.2 What Phocommander does **not** store

- No telemetry, no analytics, no usage counters, no crash reports sent anywhere.
- No usage profile, no interest profile, no file-content index, no thumbnail cache of user
  content beyond the current session.
- No advertising identifiers, no device identifiers, no advertising or sponsorship data.
- No hidden "recently visited by other users" list. The connections list contains **only**
  entries the user created themselves.
- No network request that the user did not trigger themselves. There is no update check, no
  licence server, no crash ping, no "anonymous statistics" endpoint.

### 7.3 Why no telemetry is needed

The product does not need it, and the specification forbids it in the *Produktmaximen*
section ("Werbefrei"). The technical justification is equally simple: Phocommander is a local
file manager. Its quality does not depend on knowing anything about its users. The features
that would need telemetry — "which feature do people use", "how often do people copy files" —
are not features of this product; the product is judged on its speed, its keyboard handling
and its correctness.

The promise is not a self-declaration but a checkable property:

1. **No dependency in the telemetry or analytics category.** The licence and dependency
   policy (`deny.toml`, `cargo deny check`) is the gate. A crate that would phone home
   cannot enter the dependency tree unnoticed, because its licence is not on the allow list
   and the advisory check runs on every change.
2. **No network connection at start-up.** A test asserts that the process start opens no
   socket.
3. **No own server.** There is no backend, no account, no login. The specification forbids an
   own mediation server: it would be an operating cost, a cost centre, a data-processing
   point and a privacy question — the exact opposite of the product promise.

### 7.4 What goes to a remote server

A network source is a **source of directory content**, and the traffic is the equivalent of
the user standing in front of the machine and reading the files:

- **Sent:** the credentials the user stored for that connection, and the requests for the
  directory entries the user actually asked for (path, listing, file content the user opened,
  the write the user performed).
- **Not sent:** no device identifier, no installation identifier, no telemetry, no usage
  profile, no crash report, no "anonymous" user ID, no request counter, no timestamp of
  unrelated activity.
- **The server may of course see the requests.** That is inherent in using a file server. It
  is the reason the connection log deliberately does not add a second copy of the host name:
  the log stays local, and it records the *outcome* of an operation, not the *content*.

---

## 8. Audit protocol

### 8.1 What runs in CI, on every change

| Gate | Tool | Command | What it proves | Status |
|---|---|---|---|---|
| Formatting | rustfmt | `cargo fmt --all --check` | The tree is formatted; a diff is never hidden in a formatting commit | `planned` — the step exists in `.github/workflows/ci.yml`; no green CI run yet |
| Static analysis | clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | No lint is tolerated, including the denied `unwrap`/`expect`/`todo`/`unimplemented` lints of the workspace | `planned` — the step exists in `.github/workflows/ci.yml`; no green CI run yet |
| **Licence check** | `cargo-deny` **0.20.2** (pinned) | `cargo deny check` | Every dependency has a permissive licence that fits `MIT OR Apache-2.0`, and no dependency carries a known advisory. A copyleft or vulnerable crate cannot enter the tree unnoticed. | `implemented` — the check runs against the real dependency graph and passes (`advisories ok, bans ok, licenses ok, sources ok`, 24.10.2026). The deliberate-break test that proves the gate has teeth is still owed, see §8.2 |
| **Advisory check** | `cargo-audit` **0.22.2** (pinned) | `cargo audit` | The locked dependency graph contains no crate with a published RustSec advisory | `planned` — the CI step exists; not yet run green |
| Crate boundaries | shell script | `./scripts/check-crate-boundaries.sh` | The dependency direction `apps/* → platform/* → crates/*` holds and no crate depends on a sibling of the same level. The script exits with code 1 on a violation. | `implemented` — the script exists and runs green on 24.10.2026: "3 path dependencies checked, 0 violations" |
| File size limits | shell script | `./scripts/check-file-size.sh` | No source file exceeds the 400/550-line limits | `planned` — the CI step exists, the script does not yet |
| Test run | cargo | `cargo test --workspace --all-features` | The unit, integration and property tests pass | `planned` — the CI step exists |
| **Negative tests** | cargo | the tests named in §3 | Every hardening actually resists the attack it claims to resist. A positive test proves nothing about the defence. | `planned` (M1 ff.) |
| Fuzzing | cargo-fuzz | `cargo +nightly fuzz run` | The parsers survive malformed input. Nightly CI, not per-commit. | `planned` (M1 ff.) |

The tool versions are **pinned**. The specification records why: the same `deny.toml` is read
differently by different `cargo-deny` versions — an allow-list entry has been read as
`AGPL-3.0` and as `AGPL-3.0-or-later` depending on the version — and in the sibling project
Vocument that turned a licence check into a false alarm that accused the configuration
instead of the dependency. A floating tool version is a security defect here, not a
convenience.

### 8.2 What is checked manually, and what each check proves

| Manual check | Where | What it proves |
|---|---|---|
| Notarisation and Hardened Runtime of the macOS build | macOS release | The binary is accepted by Gatekeeper and the hardened runtime flags are in place |
| First-start page explains *Full Disk Access* | macOS | The user is told what the app needs and why before granting it |
| Denial of *Full Disk Access* is handled | macOS | The app stays usable and reports the missing access instead of crashing |
| SAF mode works without `MANAGE_EXTERNAL_STORAGE` | Android | The privacy-friendly default path is complete, not a stub |
| Play Console declaration for all-files access | Android | The store submission carries the core-functionality justification Google requires |
| Revocation of a security-scoped bookmark | iPadOS | The app handles a revoked grant cleanly, as the specification requires |
| `longPathAware` present in the shipped manifest | Windows | Paths beyond 260 characters work; the Explorer discrepancy is reported |
| SMB1-only and NTLMv1-only server is rejected | M6 | The downgrade rule is enforced against a real server, not only in a unit test |
| TLS interception attempt is rejected | M6 | Certificate verification has teeth against a real proxy |
| Hanging server produces a timeout, not a stuck job | M6 | The timeout is a real deadline, not a documentation promise |
| Secure delete on a flash medium shows the limitation | M1 | The UI states the wear-levelling caveat where the user makes the choice |
| Licence and advisory gate is *effective* | M0 | Deliberately introduce a violating dependency, show the gate going red, remove it, show it going green. A gate that was never seen red has not been proven. |

### 8.3 Evidence of the licence and advisory gate

`cargo-deny` 0.20.2 was installed in the working environment and `cargo deny check` was
executed against this repository on 24.10.2026, against the real dependency graph of the
workspace. The run found **two defects in the gate itself**, both of them fatal in the same
way the specification describes for the sibling project Vocument: the gate reported a
configuration problem instead of the problem it was supposed to look for.

| Defect | Effect | Fix |
|---|---|---|
| `unmaintained = "warn"` in `[advisories]` | `cargo-deny` 0.20.2 has **no severity level** for advisories. `unmaintained` is a *scope* (`all`, `workspace`, `transitive`, `none`), so the value aborts the whole configuration and the check exits **before it has looked at a single crate**. | `unmaintained = "none"` |
| `wildcards = "deny"` without `allow-wildcard-paths` | Every crate of this workspace is referenced by a path dependency without a version key. The wildcard check reports each of them, so the gate is red on **every** push. | `allow-wildcard-paths = true` in `[bans]` plus `publish = false` on the workspace members |

After both fixes the gate runs and passes:

```
advisories ok, bans ok, licenses ok, sources ok
```

The full command output, the diff of the configuration change and the two failing runs before
the fix are in `deliverable.md`. The three remaining `license-not-encountered` warnings are
informational: the allow list names three licences that the current dependency graph does
not contain. They do not fail the gate.

**The gate still has to prove itself.** A gate that has never been seen red has not been
proven. §8.2 lists the deliberate-break test that is still owed.

### 8.4 Review cadence

- The threat model is updated in **every milestone** and the change is listed in the
  acceptance document.
- The licence allow-list and the advisory policy are reviewed whenever a dependency is added.
- The permission table in §6 is re-checked whenever a platform adapter changes.
- The residual risks in §4 are re-checked whenever the file operations change; a new residual
  risk is written down before it is accepted.

---

## 9. Related documents

| Document | Content |
|---|---|
| [`docs/REQUIREMENTS.md`](../REQUIREMENTS.md) | The binding specification. Security, data protection, network access, SMB rules, product principles. |
| [`SECURITY.md`](../../SECURITY.md) | Reporting path, supported versions, hardening guarantees. |
| [`PRIVACY.md`](../../PRIVACY.md) | Which data is stored where, for how long, how to remove it; the zero-telemetry promise. |
| `docs/adr/` | Architecture decision records, including the archive extraction hardening decision (directory created with the first ADR). |
| [`deny.toml`](../../deny.toml) | Licence allow-list and advisory policy enforced by `cargo deny check`. |

---

*End of document. Version 1.0, 24.10.2026. Every status in this document reflects the state of
the repository on that date; no rule is marked `implemented` without a green negative test.*
