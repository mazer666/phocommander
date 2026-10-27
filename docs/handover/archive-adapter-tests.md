# Handover Core Engineer — the tests for the container adapters of `phocommander-archive`

| Field | Value |
|---|---|
| Role | Core Engineer (Rust core, file system model, job queue, rename, archive adapters) |
| Repository | `/workspace/phocommander`, branch `main`, **no commit, no push, no tag** |
| Working directory | `/workspace/phocommander` |
| Task | The three container adapters (`tar`, `zip`, `sevenz`) and the format detection get tests — the last open gap of the acceptance of M1 |
| Tools | Rust 1.99.1 / rustfmt 1.10.0. **No `cargo build`, no `cargo test`, no `cargo fmt`** (the instruction of the owner, the RAM conflict with the running gate test) |

---

## 0. Summary for the lead

**32 new tests** in five files. The state of the adapter tests is therefore complete: TAR (8), ZIP
(8), 7z (7), format detection (9).

**The report describes the state before the repair.** The defect that the one test uncovered is
repaired — see section 0.1.

**One test failed on purpose** — it was the specification of a real defect in the production code
(FR-67, section 6.1):
`formats::tests::a_compressed_tar_container_is_recognised_as_a_tar_container`.
`detect_format` knows only the *uncompressed* TAR marker. A `.tar.gz` / `.tar.bz2` / `.tar.xz` /
`.tar.zst` is therefore refused as "no container" — `open_archive`, `verify_archive` and with it
the **whole extraction** never reach the four compression levels of the TAR adapter. The fix is
five lines in `detect_format` (section 6.1).

**Two further points need a decision of the owner** (sections 6.2 and 6.3): the extension of the
file as a feature of the detection, and the directory entries in the 7z adapter.

**What the owner has to do now:** nothing — the defect is repaired, the test is green.

### 0.1 The decisions of the lead instance (25.10.2026)

| Point | Decision | Why |
|---|---|---|
| **The defect of `detect_format`** | **Repaired.** The detection now checks the four compressions as well and answers with `ContainerFormat::Tar`. `cargo test -p phocommander-archive` — **53 tests green**. | The TAR adapter could never reach its compressions through the public API. That was a hole in FR-67. |
| **The extension of the file as a feature (6.2)** | **No.** The extension stays a hint, not a feature. The test `the_extension_of_the_file_does_not_decide_the_format` stays as it is. | A file that is called `.zip` and is no ZIP shall be recognised as what it is. The plain error is more honest than a cryptic error out of the ZIP reader. |
| **The directories in the 7z adapter (6.3)** | **Repaired.** The 7z adapter passes nothing on for a directory, exactly like ZIP and TAR. | The three adapters behave the same now. A visitor that reads the content of a directory otherwise reads bytes that do not belong to it. |

---

## 1. The list of the files

| Path | Lines | Kind | Purpose |
|---|---:|---|---|
| `crates/phocommander-archive/src/formats/test_support.rs` | 196 | **new** | The helpers that all three adapter tests share: `TempDir` (a temporary directory that cleans itself up on the drop), `CollectingVisitor` (collects the entries of a container), `CollectedEntry` (an entry with content), `collect()` (reads all entries of a reader). Compiled only under `#[cfg(test)]` — the production code carries no weight that it does not need. |
| `crates/phocommander-archive/src/formats/tar_tests.rs` | 265 | **new** | The 8 tests of the TAR adapter. A file of its own, because `tar.rs` plus the tests would lie above 600 lines (the soft limit of the project). Included with `mod tar_tests;` in `formats/mod.rs`. |
| `crates/phocommander-archive/src/formats/mod.rs` | 547 (was 293) | changed | + `#[cfg(test)] mod test_support;`, + `#[cfg(test)] mod tar_tests;`, + the test block `formats::tests` (9 tests) for `detect_format`, `detect_tar_compression` and `open_archive`. |
| `crates/phocommander-archive/src/formats/zip.rs` | 539 (was 275) | changed | + the test block `formats::zip::tests` (8 tests) at the end of the file. The production code is unchanged. |
| `crates/phocommander-archive/src/formats/sevenz.rs` | 416 (was 234) | changed | + the test block `formats::sevenz::tests` (7 tests) at the end of the file. The production code is unchanged. |
| `crates/phocommander-archive/src/formats/tar.rs` | 398 | **unchanged** | The TAR adapter itself was not touched — its tests lie in `tar_tests.rs`. |

**The proof that no production code was changed:**
`git diff --numstat crates/phocommander-archive/` → `254 0 mod.rs`, `182 0 sevenz.rs`,
`264 0 zip.rs` — in all three files **only lines added, none changed, none deleted**.

**The line limit:** `./scripts/check-file-size.sh` reports nothing for my files (the largest file:
`mod.rs` with 547 lines). The existing excess (`crates/phocommander-ui/src/keymap_view.rs`,
999 lines) does not come from me.

---

## 2. The public API

The tests add **no** public API. All the new helpers lie in a private module
`crate::formats::test_support` under `#[cfg(test)]` and are therefore not reachable from the
application nor from an integration test.

| Crate | Module | Type/function | Purpose |
|---|---|---|---|
| `phocommander-archive` | `crate::formats::test_support` (private, `cfg(test)`) | `TempDir::new(label)` / `join(name)` / `write_file(name, bytes)` | A temporary directory with a name of its own (process, time, counter), that cleans itself up on the drop. |
| `phocommander-archive` | `crate::formats::test_support` | `CollectingVisitor` (`impl EntryVisitor`) | Collects the entries of a container including the content, so that a test can compare a container with what it has written. |
| `phocommander-archive` | `crate::formats::test_support` | `CollectedEntry` | A read entry: name, `name_is_lossy`, kind, size, mode, link target, stored checksum, content. |
| `phocommander-archive` | `crate::formats::test_support` | `collect(reader) -> Result<Vec<CollectedEntry>, ArchiveError>` | Reads all the entries of a reader with *one* visitor — so that a difference between two adapters is a difference of the adapters and not of the tests. |

---

## 3. The state of the tests

### 3.1 What was measured

| Command | Result |
|---|---|
| `rustfmt --edition 2024 --check` (on all five files, **without** `cargo fmt`) | **RUSTFMT CLEAN** — the files parse and are format-conform. `rustfmt` is the single binary call, no cargo process, no build, no test. |
| `./scripts/check-file-size.sh` | No new excess (the largest new file 547 lines). |
| `git diff --numstat` | Only additions, no production code changed. |
| `cargo test --workspace` | **Not run** — forbidden by the owner (RAM conflict with the running gate test). I therefore cannot deliver the test output; it arises in the central run of the owner. |

**I do not claim that the tests are green.** I have written them and checked the API of the three
container crates (`tar` 0.4.46, `zip` 9.1.0, `sevenz-rust2` 0.24.1) against the sources of the
registry, but **not compiled and not executed** them.

### 3.2 The stock of the tests

| File | Test module | Tests |
|---|---|---:|
| `src/formats/tar_tests.rs` | `formats::tar_tests` | 8 |
| `src/formats/zip.rs` | `formats::zip::tests` | 8 |
| `src/formats/sevenz.rs` | `formats::sevenz::tests` | 7 |
| `src/formats/mod.rs` | `formats::tests` | 9 |
| | **Sum** | **32** |

### 3.3 The single tests

**TAR — `formats::tar_tests`**

| Test | Case |
|---|---|
| `the_reader_returns_the_entries_that_were_written` | Round trip: name, kind, size, mode bits |
| `the_reader_returns_the_content_that_were_written` | Round trip: content byte for byte identical |
| `the_container_keeps_the_structure_of_the_directories` | The directory `docs` + the file `docs/readme.md` |
| `the_reader_reports_a_link_as_a_link_and_not_as_its_target` | The link comes back as a link (`EntryKind::Symlink`, `link_target`), not as the target |
| `the_container_is_read_back_with_every_compression` | **All four compressions + uncompressed** (None, Gzip, BZip2, Xz, Zstd) |
| `a_file_that_is_no_container_is_an_error_and_not_a_panic` | An error instead of a panic (`catch_unwind` + the return value) |
| `the_writer_refuses_a_name_that_leaves_the_destination` | `../escape.txt` → `Refused`/`Traversal` |
| `a_name_that_is_not_valid_utf8_is_reported_as_lossy_and_is_not_cut_short` | **Unix only**: the name `broken\xffname.txt`, built with the container crate (a foreign producer) → `name_is_lossy == true`, the name **not cut short** (`broken\u{fffd}name.txt`) |

**ZIP — `formats::zip::tests`**

| Test | Case |
|---|---|
| `the_reader_returns_the_entries_that_were_written` | Round trip: name, kind, size |
| `the_reader_returns_the_content_that_were_written` | Round trip: content identical |
| `the_container_keeps_the_structure_of_the_directories` | The directory (in the ZIP format with a `/` at the end) + the file |
| `the_reader_reports_a_link_as_a_link_and_not_as_its_target` | The link comes back as a link |
| `the_reader_reports_the_checksum_that_the_container_stores` | The CRC-32 stored in the container == the `Crc32` of the project (FR-71) |
| `a_file_that_is_no_container_is_an_error_and_not_a_panic` | An error instead of a panic |
| `the_writer_refuses_a_name_that_leaves_the_destination` | `../escape.txt` → `Refused`/`Traversal` |
| `a_name_that_is_not_valid_utf8_is_reported_as_lossy_and_is_not_cut_short` | The bytes of the name are patched after the writing (the writer of the container crate takes only a `String`) → `name_is_lossy == true`, the name not cut short |

**7z — `formats::sevenz::tests`**

| Test | Case |
|---|---|
| `the_reader_returns_the_entries_that_were_written` | Round trip: name, kind, size |
| `the_reader_returns_the_content_that_were_written` | Round trip: content identical |
| `the_container_keeps_the_structure_of_the_directories` | The directory `docs` + the file `docs/readme.md` |
| `the_reader_reports_the_checksum_that_the_container_stores` | The CRC-32 stored in the container == the `Crc32` of the project |
| `the_name_of_an_entry_is_never_lossy_because_the_format_carries_it_as_utf16` | The 7z format stores the names as UTF-16 — a name that is no UTF-8 **cannot** occur; `name_is_lossy` stays `false` |
| `the_writer_refuses_to_store_a_symbolic_link` | The format knows no links → `ArchiveError::Format` |
| `a_file_that_is_no_container_is_an_error_and_not_a_panic` | An error instead of a panic |

**Detection — `formats::tests`**

| Test | Case |
|---|---|
| `a_zip_container_is_recognised_by_its_magic_bytes` | ZIP by magic bytes |
| `a_tar_container_is_recognised_by_its_magic_bytes` | TAR by magic bytes |
| `a_7z_container_is_recognised_by_its_magic_bytes` | 7z by magic bytes |
| `a_container_with_a_wrong_extension_is_recognised_by_its_content` | A ZIP container with the extension `.tar` → **ZIP** (the content decides) |
| `a_compressed_tar_container_is_recognised_as_a_tar_container` | **FAILS TODAY** — see 6.1 |
| `the_extension_of_the_file_does_not_decide_the_format` | A text file with the extension `.zip` → **an error**, not "ZIP" — documents the contract of today (see 6.2) |
| `a_file_that_is_no_container_is_an_error_and_not_a_panic` | An error instead of a panic (`catch_unwind`) |
| `the_compression_of_a_tar_container_is_recognised_by_its_magic_bytes` | All five compression levels recognised at their magic bytes |
| `open_archive_opens_a_container_of_every_format` | `open_archive` opens ZIP, TAR and 7z and delivers the content |

---

## 4. The matrix of the coverage

| Area | tested? | Test file | Test name | Gap |
|---|---|---|---|---|
| TAR: round trip (entries) | ✅ | `formats/tar_tests.rs` | `the_reader_returns_the_entries_that_were_written` | — |
| TAR: round trip (content) | ✅ | `formats/tar_tests.rs` | `the_reader_returns_the_content_that_were_written` | — |
| TAR: directories | ✅ | `formats/tar_tests.rs` | `the_container_keeps_the_structure_of_the_directories` | — |
| TAR: symbolic links | ✅ | `formats/tar_tests.rs` | `the_reader_reports_a_link_as_a_link_and_not_as_its_target` | — |
| TAR: compression (5 levels) | ✅ | `formats/tar_tests.rs` | `the_container_is_read_back_with_every_compression` | — |
| TAR: no container → error | ✅ | `formats/tar_tests.rs` | `a_file_that_is_no_container_is_an_error_and_not_a_panic` | — |
| TAR: name without UTF-8 | ✅ (unix) | `formats/tar_tests.rs` | `a_name_that_is_not_valid_utf8_...` | Not executable under Windows (the format cannot carry byte names there) |
| TAR: checksum | ➖ | — | — | The TAR format stores none (only the checksum of the header) — deliberately not tested |
| ZIP: round trip (entries + content) | ✅ | `formats/zip.rs` | `the_reader_returns_the_entries_that_were_written`, `..._content_...` | — |
| ZIP: directories | ✅ | `formats/zip.rs` | `the_container_keeps_the_structure_of_the_directories` | — |
| ZIP: symbolic links | ✅ | `formats/zip.rs` | `the_reader_reports_a_link_as_a_link_and_not_as_its_target` | — |
| ZIP: stored CRC-32 | ✅ | `formats/zip.rs` | `the_reader_reports_the_checksum_that_the_container_stores` | — |
| ZIP: no container → error | ✅ | `formats/zip.rs` | `a_file_that_is_no_container_is_an_error_and_not_a_panic` | — |
| ZIP: name without UTF-8 | ✅ | `formats/zip.rs` | `a_name_that_is_not_valid_utf8_...` | — |
| ZIP: name with a directory (`../`) | ✅ | `formats/zip.rs` | `the_writer_refuses_a_name_that_leaves_the_destination` | — |
| 7z: round trip (entries + content) | ✅ | `formats/sevenz.rs` | `the_reader_returns_the_entries_that_were_written`, `..._content_...` | — |
| 7z: directories | ✅ | `formats/sevenz.rs` | `the_container_keeps_the_structure_of_the_directories` | — |
| 7z: stored CRC-32 | ✅ | `formats/sevenz.rs` | `the_reader_reports_the_checksum_that_the_container_stores` | — |
| 7z: name without UTF-8 | ✅ (as a property) | `formats/sevenz.rs` | `the_name_of_an_entry_is_never_lossy_...` | The format cannot carry such names |
| 7z: links | ✅ | `formats/sevenz.rs` | `the_writer_refuses_to_store_a_symbolic_link` | — |
| 7z: no container → error | ✅ | `formats/sevenz.rs` | `a_file_that_is_no_container_is_an_error_and_not_a_panic` | — |
| Detection: ZIP/TAR/7z by magic bytes | ✅ | `formats/mod.rs` | `a_zip_...`, `a_tar_...`, `a_7z_...` | — |
| Detection: wrong extension, right content | ✅ | `formats/mod.rs` | `a_container_with_a_wrong_extension_is_recognised_by_its_content` | — |
| Detection: **right extension, wrong content** | ⚠️ | `formats/mod.rs` | `the_extension_of_the_file_does_not_decide_the_format` | **Open:** the extension is not read today (6.2) |
| Detection: compressed TAR | ❌ **missing in the code** | `formats/mod.rs` | `a_compressed_tar_container_is_recognised_as_a_tar_container` | **Defect**, the test fails (6.1) |
| Detection: no container → error | ✅ | `formats/mod.rs` | `a_file_that_is_no_container_is_an_error_and_not_a_panic` | — |
| Detection: compression by magic bytes | ✅ | `formats/mod.rs` | `the_compression_of_a_tar_container_is_recognised_by_its_magic_bytes` | — |
| `open_archive` for all three formats | ✅ | `formats/mod.rs` | `open_archive_opens_a_container_of_every_format` | — |
| Extraction security (paths, bombs, limits) | ➖ | `tests/security/*` | — | **Not touched** — the adapter tests do not bypass the security, they only read and write |
| Encrypted containers (FR-68) | ➖ | — | — | The writing of encrypted containers does not exist in the adapter (see 6.4) |

---

## 5. Determinism

The adapter tests check no rename rules — those lie in `phocommander-ops`, not in
`phocommander-archive`. For the rename rules the evidence of the previous handover still holds
(`docs/archive/handover-2026/deliverable.core-engineer.md`); this task did not touch them.

What the new tests keep deterministic:
- **The order of the entries** is not determined by the position but by the name
  (`entry(&entries, name)`), because the 7z format delivers the directory entries after the blocks.
- **The temporary directories** get a name of their own (process id, nanoseconds, counter) — two
  tests that run in parallel share no directory.
- **No time or random value** flows into an assertion: the TAR adapter writes `mtime = 0`, and the
  checksums are computed against the content of the file, not against a stored value.

---

## 6. The findings (this is the most important part of this handover)

### 6.1 Defect: `detect_format` knows no compressed TAR (FR-67) — 1 test fails

`crates/phocommander-archive/src/formats/mod.rs`, `detect_format`:

```rust
if signature.starts_with(&signature::ZIP) || signature.starts_with(&signature::ZIP_EMPTY) { … }
if signature.starts_with(&signature::SEVENZ) { … }
if is_tar(&signature) { … }          // ← only the uncompressed TAR marker
Err(ArchiveError::Format { … })
```

`detect_tar_compression` knows GZip, BZip2, XZ and Zstd — but `detect_format` never calls it,
because it breaks off before that. Consequence: `open_archive("x.tar.gz")` fails, and with it
`verify_archive` and `extract_archive`. **The four compression levels of the TAR adapter are not
reachable through the public API.** FR-67 ("TAR with GZip, BZip2, XZ and Zstd are supported") is
therefore not fulfilled in half.

The test `a_compressed_tar_container_is_recognised_as_a_tar_container` is the specification of
this contract and **fails until the repair**. It is deliberately not left out and not rewritten to
the behaviour of today: a test that writes the defect down would be worse than no test.

**The proposed change (5 lines, not carried out by me — the task was "test files only"):**

```rust
    // A compressed stream can carry a TAR container. The detection has to
    // know that, or the four compressions of the adapter can never be
    // reached.
    if signature.starts_with(&signature::GZIP)
        || signature.starts_with(&signature::BZIP2)
        || signature.starts_with(&signature::XZ)
        || signature.starts_with(&signature::ZSTD)
    {
        return Ok(ContainerFormat::Tar);
    }
```

`open_archive` then already chooses the right decoder over `detect_tar_compression` ✓.

### 6.2 Open: the extension of the file as a feature of the detection

The task named "the extension **and** the magic bytes". `detect_format` does not read the
extension — the doc comment of the function says expressly: "the extension of a file is a hint,
not a fact. The magic bytes of the container decide what the file really is."

I have written the contract of today down in `the_extension_of_the_file_does_not_decide_the_format`
(a text file with the extension `.zip` → an error). **That is a deliberate decision, not a
refusal:** the test names the conflict between the letter and the code. If the owner wants the
extension as a feature, the test is to be turned around and `detect_format` is to be completed by
a fallback:

```rust
    // The extension is a hint. It is only used when the magic bytes tell
    // nothing, so that a file that is called ".zip" is at least reported as
    // the ZIP container it pretends to be — and the user then sees why it
    // is broken.
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("zip") => return Ok(ContainerFormat::Zip),
        Some("7z") => return Ok(ContainerFormat::SevenZ),
        Some("tar" | "tgz" | "tbz2" | "txz" | "tzst") => return Ok(ContainerFormat::Tar),
        _ => {}
    }
```

**The decision of the owner:** the extension as a fallback — yes or no?

### 6.3 Observation: the 7z adapter reports directories with content

`formats/sevenz.rs` passes a reader on for **every** entry (`Some(content)`), also for
directories. ZIP and TAR pass `None` on for directories. For the extraction that is uncritical (it
checks `entry.is_file()`), but the three adapters are thereby not symmetric. I have therefore
written the tests so that they check the content **only for files**. If the symmetry is wanted,
that is one line in the 7z adapter.

### 6.4 Not in this task: encrypted containers (FR-68)

FR-68 demands "Archives can be created, including encrypted". The writers of the three adapters
offer **no** encryption (the `zip` crate can do it, the adapter does not call it; 7z and TAR not
at all). That is a gap of the acceptance of M1, but it belongs in the production implementation and
not in the tests — I cannot close it with a test that expects a function that does not exist.

---

## 7. What the owner has to do now

1. `cargo test -p phocommander-archive` — expected: **31 green, 1 red**
   (`a_compressed_tar_container_is_recognised_as_a_tar_container`).
2. Decide whether `detect_format` gets the five lines of 6.1. After that the test is green and
   stays as a guard.
3. Decide on 6.2 (the extension as a fallback) — the test in `formats/mod.rs` is to be turned
   around if yes.
4. `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` (the formatter
   is already pre-formatted: `rustfmt --check` is green).

**Should the central run show something else than 31/1**, the most likely causes are in this
order: (a) the directory entries of the ZIP carry the extension `/` — the test checks `"docs/"`;
(b) the number of the occurrences of the name in the ZIP container (the test replaces **all**
occurrences and demands only `> 0`); (c) the mode value of the TAR header (`0o644`).

---

## 8. Not executed (truthfulness)

| Command | State | Reason |
|---|---|---|
| `cargo build` / `cargo check` | **not run** | The instruction of the owner (RAM conflict with the running gate test) |
| `cargo test` | **not run** | ditto — I cannot deliver the test output of this handover |
| `cargo clippy --workspace --all-targets -- -D warnings` | **not run** | ditto |
| `cargo fmt` | **not run** | ditto — instead the single `rustfmt` call on the five files (`--check` green) |
| `./scripts/check-file-size.sh` | **run** | needs no build: no new excess |
| `rustfmt --edition 2024 --check` (5 files) | **run** | RUSTFMT CLEAN |
| `git diff --numstat` | **run** | only additions |

No commits, no pushes, no tags were made. The files `README.md` and `docs/STATUS.md` were already
changed before this session — they do **not** come from me and were not touched.
