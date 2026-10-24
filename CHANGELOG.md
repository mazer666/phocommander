# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The milestone column follows the roadmap of the binding specification,
[`docs/REQUIREMENTS.md`](docs/REQUIREMENTS.md).

---

## [Unreleased]

### Added

- **Project documentation for M0** — the documentation set that completes the
  foundation milestone:
  - [`README.md`](README.md) — the landing page: status, the three product rules, the
    feature overview, the quickstart, the first ten keys and the five input contexts.
  - [`docs/KEYMAP.md`](docs/KEYMAP.md) — the complete binding reference with default,
    context, platform variant and note for every action, plus the procedure for
    changing a binding.
  - [`docs/glossary.md`](docs/glossary.md) — twenty terms that carry a specific
    meaning in this project, German and English.
  - [`docs/user-guide/`](docs/user-guide/) — the task-oriented guide in German and
    English: *getting started* and *multiple rename*, including the serial-episode
    walkthrough.
  - [`docs/architecture/README.md`](docs/architecture/README.md) — the layers, the
    crate boundaries and the extension points.
  - [`docs/WHERE_TO_CHANGE.md`](docs/WHERE_TO_CHANGE.md) — "which file do I touch for
    which feature?", indexed by topic, file, crate and instruction.
  - [`docs/REQUIREMENTS-IDS.md`](docs/REQUIREMENTS-IDS.md) — the requirement
    catalogue with stable identifiers, milestone, priority and verification method.
  - [`docs/ACCEPTANCE-CRITERIA.md`](docs/ACCEPTANCE-CRITERIA.md) — the acceptance
    criteria per milestone and per requirement identifier.
  - [`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md) — the threat
    model with the hardening rules, their negative tests and the residual risks.
  - [`docs/templates/M0-ACCEPTANCE.md`](docs/templates/M0-ACCEPTANCE.md) — the
    template for an acceptance document.
  - [`SECURITY.md`](SECURITY.md), [`PRIVACY.md`](PRIVACY.md),
    [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md), [`CONTRIBUTING.md`](CONTRIBUTING.md)
    and [`AGENTS.md`](AGENTS.md) — the governance documents.
- **Repository check scripts** — link check, file-size check, telemetry check,
  requirement-identifier check and configuration-schema check under
  [`scripts/`](scripts/).

### Changed

- The link check now also validates anchors against the headings of the target file.

---

## [0.1.0] - 2026-10-24

The M0 foundation: the repository builds, checks and tests itself. It is **not yet a
usable file manager** — the two panes, the file operations, the multiple rename and
the viewer arrive with M1. No release artefact has been published yet; this entry
describes the state of the repository at this date.

### Added

- **Cargo workspace** with the pinned catalogue of every external dependency, the
  pinned Rust toolchain (`rust-toolchain.toml`) and the workspace lint set that
  forbids `unsafe`, `unwrap`, `expect`, `todo!()` and `unimplemented!()`.
- **Continuous integration** — the gates of the specification: format, clippy, tests,
  crate boundaries, file sizes, licence check, advisory check, a desktop build matrix
  over Linux, macOS and Windows, cross-compilation for Android, iOS, FreeBSD and
  NetBSD, and a nightly fuzzing run.
- **Configuration** — the central configuration template with every key documented,
  the action catalogue with the key bindings per input context, and five
  multiple-rename templates (series, camera, podcast, invoice, music).
- **Core crates** — the filesystem model and the error hierarchy, the configuration
  parser with schema versioning and migration.
- **Licence check** — the `cargo-deny` configuration with a licence allow list that
  contains no copyleft, and the pinned `cargo-deny` and `cargo-audit` versions.

### Documentation

- **Requirements specification 1.1** — scope, product vision, the three product
  maxims, the input models and the five operating contexts, network and cloud access,
  the functional scope, the non-functional requirements, the system architecture, the
  platform matrix, the test and quality strategy, the project plan with its
  acceptance gates, and the decisions E1 to E9.

---

[Unreleased]: https://github.com/mazer666/phocommander/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mazer666/phocommander/releases/tag/v0.1.0
