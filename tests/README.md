# The test structure of Phocommander

**Binding source:** [`docs/REQUIREMENTS.md`](../docs/REQUIREMENTS.md) — section *Test and
quality strategy*. **Acceptance catalogue:**
[`docs/ACCEPTANCE-CRITERIA.md`](../docs/ACCEPTANCE-CRITERIA.md).

## The layout

```text
tests/
├── Cargo.toml          the package "phocommander-integration-tests" — a member of the workspace
├── common/mod.rs       the shared helpers: temporary directory, files, trees, inspection
├── integration/        the tests that cross a crate boundary
│   ├── config_defaults.rs       the shipped configuration file and the configuration reader
│   ├── core_limits.rs           the canonical limits and the shipped configuration file
│   ├── keymap_bindings.rs       the shipped key binding file and the keymap crate
│   └── directory_entries.rs     the directory model of the core crate (waiting for M1)
└── security/           the negative tests — the attack is performed, the defence is proved
    ├── README.md               the protocol table of the attack vectors
    ├── path_traversal.rs       zip slip, absolute paths, drive letters, too long names
    └── decompression_bomb.rs   the forged size, the expansion ratio, the entry count
```

The unit tests live next to the code they test, inside the crates — that is the rule of the
specification ("unit tests lie next to every public function"). This directory holds the tests
that cannot live there: the ones that hold two crates against each other, and the ones that read the
files of the repository.

## Why a package of its own

The workspace is flat: no crate below `crates/` may depend on a sibling. A test that compares the
constants of `phocommander-core` with the values of the configuration file therefore cannot live in
either crate — it needs both. The same is true for the tests that read `config/defaults.toml` and
`config/keymap.toml`: they belong to the repository, not to a crate.

## Running the tests

```bash
cargo test -p phocommander-integration-tests
cargo test -p phocommander-integration-tests --test security_path_traversal
cargo bench  -p phocommander-integration-tests --bench directory_listing
```

## The rules the helpers follow

- Every temporary artefact lives in the temporary directory of the system and is removed when the
  test ends — also when the test panics.
- No binary file is written into the repository. The hostile archives of the negative tests are
  built at run time.
- No test is skipped, no test is marked `#[ignore]`, no `unwrap` and no `expect`: the workspace
  denies them, and a test that swallows an error is a test that lies.
