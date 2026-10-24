# ADR-0005 — Interface profiles and the configurable surface

**Status:** accepted
**Date:** 2026-10-24
**Owner role:** Rust Architect, confirmed by the Product Manager
**Related requirement IDs:** FR-110 – FR-145, NFR-04, NFR-10, NFR-22

## Context

Two requirements pull in opposite directions. The first is that the product is clutter-free: the
start view should carry nothing that the user did not ask for. The second is that the product is
complete: every function that exists must be reachable, because a missing function is a missing
function regardless of how the interface looks.

A dual-pane file manager that follows the Norton Commander tradition has a large surface: two
panels, tabs, a directory tree, a quick view, a job list, a menu bar, a toolbar, a command line, a
status bar, and more than thirty commands. Showing all of it at once is the reference product's
weakness, and the user has said so: the interface of the reference "seems a bit dry". Hiding
capability instead of furniture would be worse, and it would contradict the principle that the
product is free and complete — a restricted version is a paid compromise.

The question is therefore not "which features are shown" but "how is the same feature set
presented differently depending on what the user wants".

## Decision

**1. Two shipped profiles, plus the user's own.** The product ships a `clean` profile and a
`full` profile. The `clean` profile shows the two panels, the path, the file list and the command
line. The `full` profile shows every command and every panel area. A user may create further
profiles; the two shipped ones are ordinary files in `config/profiles/` and are not special in the
code.

**2. A profile is a set of decisions, not a set of features.** A profile contains panel
visibility, command visibility and layout preferences. It contains no reference to a requirement,
no path, no history and no credential. This is what makes it exportable, shareable and safe.

**3. The clean profile removes furniture, never capability.** Every action remains reachable
through the action palette in every profile. A profile is therefore not a feature gate, and the
principle that the product is free and complete survives every profile the user can create.

**4. Command visibility has three states.** A command is `visible` (it appears in the menus and
the toolbar), `palette` (it is reachable through the action palette but not shown), or `hidden` (it
is not offered at all and the action is not executed). The third state exists because some
commands are dangerous or meaningless in a given context, and the user needs a way to remove them
entirely rather than merely hide them.

**5. The resolution order is fixed.** The built-in default for the active context, then the
selected profile, then the per-command overrides of that profile, then the per-platform override.
A fixed order is what makes the result predictable; a user who exports a profile to a colleague
knows what that colleague will see.

**6. The profile can differ per platform and per interaction context.** The same profile file may
carry a platform section. A profile that hides the toolbar on a television and shows it on a
desktop is one file, not two.

## Consequences

### Positive

- The clutter principle and the completeness principle stop competing. The user chooses the
  density, not the feature set.
- A profile is a small, readable file that can be attached to a bug report or shared with a
  colleague.
- The interface resolution is a pure function of four inputs, so it can be unit tested without a
  window.
- The action palette, which the D-pad context of M8 needs anyway, becomes the universal fallback
  that makes the clean profile honest.

### Negative / Trade-offs

- The resolution order has to be implemented once and documented, because a user who does not
  understand it cannot predict what they will see.
- The three visibility states are more concepts than a binary would be. The alternative — a
  binary — cannot express "reachable but not shown", which is the state the clean profile needs
  for most commands.
- The per-platform override means a profile is not a pure description of an interface. It is a
  description of an interface *on a platform*, which is what the user actually experiences.

## Alternatives Considered

- **A runtime toggle that hides the panels.** Rejected: a toggle has no state that can be
  exported, shared or restored, and it forces the user to repeat the decision at every start.
- **Removing features from the clean profile.** Rejected: that is a feature gate, and it
  contradicts the principle that the product is free and complete.
- **Two separate applications.** Rejected: the maintenance cost doubles, and the two would drift
  apart in behaviour, which makes the shared core the only possible place for the logic.
- **Letting the user hide individual panels only, without command visibility.** Rejected: the
  command surface is larger than the panel surface, and the commands are what make the interface
  feel cluttered.
