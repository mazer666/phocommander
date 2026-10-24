# ADR-0006 — Safe copy modes and the job cue

**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Rust Architect, confirmed by the Product Manager
**Related requirement IDs:** FR-22a – FR-22l, NFR-06, NFR-09, NFR-09a, NFR-09b

## Context

A file manager that offers one copy operation is a file manager that eventually lies to its user.
A byte copy that reports success can leave a destination that differs from the source: a
truncated file after a full disk, a file altered by a failing filter driver, a file written from a
source that changed while it was being read.

The reference product treats this with a copy dialog that offers options, and the risk register
of the specification names the archive extraction as the most severe error class in file managers.
The question for this ADR is narrower: how is the copy operation structured, and how is its state
made visible and correctable while it runs?

Three observations shaped the decision.

First, the cost of verification is not uniform. Reading the source twice and comparing checksums
doubles the read volume, which matters on a spinning disk and on a network share, and does not
matter on a local SSD. The user knows the medium better than the tool does.

Second, a failure that is discovered late is expensive. A copy that fails halfway leaves a
half-written file at the destination, and the user finds out much later, possibly after the
source has been deleted. A copy that writes to a temporary file and renames it into place cannot
leave a half-written file under the final name.

Third, a job that cannot be corrected is a job that has to be guessed at. A queue of twenty
operations almost always contains one with the wrong target or the wrong mode. Discovering that
only after the twentieth operation has run is the worst possible moment to discover it.

## Decision

**1. The copy mode is a property of the operation, not a global setting.** The modes are `quick`,
`verified`, `atomic`, `sync-mirror`, `sync-update`, `sync-two-way` and `sync-preview`.

**2. The defaults are the safe ones.** A copy defaults to `verified`. A synchronisation defaults to
`sync-preview`, which produces a plan and moves nothing. `sync-mirror` is the only mode that
deletes, and its deletions appear in the plan before anything is deleted.

**3. The comparison behind every synchronisation mode is one function.** It yields, per entry, the
state `identical`, `different`, `missing-in-destination` or `extra-in-destination`, together with
the size and the timestamp of both sides and the checksum where one was computed. The four
synchronisation modes differ only in what they do with that result. This keeps the comparison
testable once, and it makes the plan a first-class artefact that the user confirms.

**4. The verification is a phase, not a step inside the copy.** A job in `verified` or `atomic`
mode passes through `Verifying` as its own state, with its own progress. The cue therefore shows
that the copy is finished but the verification is not, which is exactly the state in which a user
would otherwise assume everything is fine.

**5. The job cue is one state, presented in three places.** The state of a job — its phase, its
progress, its rate, its estimate, its current item — is a single value. It is shown in the status
bar of the owning panel, in the list of all jobs, and as a marker on the owning panel. Three
presentations of one state cannot disagree with each other.

**6. A job stays editable until it is finished.** The priority, the destination, the conflict
policy and the verification mode can be changed while the job is queued. A running job can be
paused, resumed and cancelled. The failed items can be retried on their own. A change to a
running job takes effect at the next item boundary, never in the middle of a file being written.

**7. The changes are recorded.** The job keeps a history of what was changed and when. An
operation that was corrected after an interruption remains traceable, which matters for the
acceptance document of a milestone and for a bug report.

## Consequences

### Positive

- The user chooses the level of trust per operation, and the default is the level that cannot
  silently lie.
- The synchronisation plan is an artefact. It can be shown, confirmed, exported and referenced in
  an acceptance document.
- The verification phase is visible, so "copied" and "verified" are never confused.
- The editable queue turns a mistake in the twentieth operation into a correction in the second.

### Negative / Trade-offs

- The verified mode doubles the read volume, which is a real cost on a network share. The mode
  table in the specification names the cost so that the choice is informed.
- The atomic mode needs free space next to the destination for the temporary file. A destination
  that is nearly full fails the atomic copy even though the quick copy would have fitted.
- The job history is state that has to be kept, serialised and bounded. It is bounded by the
  number of changes, not by the number of items.
- The comparison has to run before a synchronisation can start, so a synchronisation of a large
  tree has a visible planning phase. This is intentional: the alternative is deleting files that
  the user did not see in a plan.

## Alternatives Considered

- **Always verify.** Rejected: on a slow medium the cost is disproportionate, and the user is
  better placed to judge the medium than the tool is.
- **Never verify, offer it as an option.** Rejected: the failure is discovered too late to be
  useful, and the default would train users to trust a copy that cannot be trusted.
- **Copy to a temporary file always.** Rejected: the atomic mode is the right answer for files
  that a running program may read, but it is wasteful for bulk transfers where the destination is
  a fresh directory.
- **A progress dialog that cannot be edited.** Rejected: a queue that cannot be corrected
  guarantees that the user either waits for everything to run or cancels everything.
- **Showing the cue only in a notification area.** Rejected: a status that hides in the corner
  while a long operation runs is exactly the state the user needs to see.
