# The handovers of 2026

**What this folder is.** The handover documents of the agents that worked on Phocommander in
2026. Every file names one task — the requirements of a milestone, the work of one role, the
result of one cycle — and carries the report of the agent that carried it out.

**Why they are archived and not translated.** These are historical records of work that is done.
The report of a finished task is not living documentation, and translating four thousand lines of
finished reports would cost a great deal of time and buy nothing: no agent reads them to learn
how the project works — [`docs/WHERE_TO_CHANGE.md`](../../WHERE_TO_CHANGE.md) and
[`AGENTS.md`](../../../AGENTS.md) do that, and both are in English.

They stay in German because that is the language they were written in. A record is not rewritten
after the fact.

**What replaced them.** From October 2026 on, the handover of a change is a document under
[`docs/handover/`](../../handover/) in English, named after the topic of the change. The rule
stands in [`AGENTS.md`](../../../AGENTS.md), section 8.

## The files

| File | The task |
|---|---|
| `deliverable.md` | The state of the build matrix and the separation of the BSD builds from the cross-compilation job |
| `deliverable-product-manager.md` | The requirements catalogue and the decisions of the product manager |
| `deliverable-qa-test-engineer.md` | The test harness, the coverage matrix and the gate of the quality |
| `deliverable-rust-architect.md` | The structure of the workspace, the boundaries of the crates and the build |
| `deliverable-security-manager.md` | The threat model, the hardening rules and the privacy statement |
| `deliverable.core-engineer.md` | The core of the file system, the error hierarchy and the archive adapters |
| `deliverable.core-engineer-m3.md` | The Windows and Linux port of the milestone M3 |
| `deliverable.core-engineer-rename.md` | The formation of the names of the multiple rename |
| `deliverable.packaging-ci-m3.md` | The packaging and the CI matrix of the milestone M3 |
| `deliverable.ui-engineer.md` | The interface of the milestone M1 |
| `deliverable.ui-engineer-rename.md` | The dialog of the multiple rename |
| `deliverable.security-manager.md` | The security rules and the negative tests of the milestone M1 |
