# M0 Acceptance — Phocommander

> Template. Copy this file to `docs/acceptance/M0-ACCEPTANCE.md`, fill every
> placeholder in square brackets, and delete this quote block.
>
> The section order is prescribed by `docs/REQUIREMENTS.md`, section 13.1, and
> must not be changed.
>
> Evidence rules, from section 13.2:
>
> - An evidence is a command output, a test name with a green result, or an
>   extract from an acceptance document. "It works" is not evidence. "Should
>   fit" is not evidence. A coverage percentage without a measurement is not
>   evidence.
> - A test that could not be executed is reported as **NOT RUN** with the exact
>   reason. A NOT RUN with a reason is allowed and honest. A faked PASS is not.
> - No gate is weakened to make the build green. No lint is commented out with
>   an allow attribute, no check is removed from the CI, no test is disabled.
>
> The acceptance criteria that the functionality section refers to are listed
> in [`docs/ACCEPTANCE-CRITERIA.md`](../ACCEPTANCE-CRITERIA.md); the
> requirement identifiers are listed in
> [`docs/REQUIREMENTS-IDS.md`](../REQUIREMENTS-IDS.md).

| Field | Value |
|---|---|
| Milestone | M0 — Foundation |
| Acceptance date | [YYYY-MM-DD] |
| Specification version | [version of `docs/REQUIREMENTS.md`, e.g. 2.0] |
| Catalogue version | [version of `docs/REQUIREMENTS-IDS.md`] |
| Decided by | [name] |
| Result | [accepted / accepted with conditions / rejected] |

---

## 1. Functionality

Which functions of this milestone exist, and how do they behave? Every row
names the requirement identifier, the acceptance criterion and an observed
behaviour — never a claim.

| Requirement | Criterion | Behaviour observed | Evidence |
|---|---|---|---|
| [FR-xx] | [AC-M0-nn] | [what was observed, in one sentence] | [command output or test name] |
| [FR-xx] | [AC-M0-nn] | [what was observed, in one sentence] | [command output or test name] |
| [FR-xx] | [AC-M0-nn] | [what was observed, in one sentence] | [command output or test name] |

A criterion that could not be executed is listed here as well, with the marker
**NOT RUN** and the exact reason in the evidence column.

## 2. Technique

Which architecture decisions are implemented, which module boundaries were
kept, which tools ran through?

| Item | Result | Evidence |
|---|---|---|
| Crate boundaries (`scripts/check-crate-boundaries.sh`) | [pass / fail] | [command output] |
| Dependency direction | [pass / fail] | [command output] |
| Formatting (`cargo fmt --check`) | [pass / fail] | [command output] |
| Static analysis (`cargo clippy --all-targets -- -D warnings`) | [pass / fail] | [command output] |
| Licence check (`cargo deny check`) | [pass / fail / NOT RUN with the exact reason] | [command output] |
| Vulnerability check (`cargo audit`) | [pass / fail / NOT RUN with the exact reason] | [command output] |
| File size (`scripts/check-file-size.sh`) | [pass / fail] | [command output] |
| Telemetry check (`scripts/check-no-telemetry.sh`) | [pass / fail] | [command output] |
| Requirement identifiers (`scripts/check-requirement-ids.py`) | [pass / fail] | [command output] |
| Acceptance criteria (`scripts/check-acceptance-ids.py`) | [pass / fail] | [command output] |
| Interface profiles (`scripts/check-interface-profiles.py`) | [pass / fail] | [command output] |
| Test run (`cargo test --workspace`) | [pass / fail / NOT RUN with the exact reason] | [summary line] |
| Platform build | [pass / fail / NOT RUN with the exact reason] | [command output] |

## 3. Performance

Which target values were measured, with which tool, with which result? A target
value that was not measured is reported as "not measured", never as "met".

| Requirement | Target value | Threshold | Measured with | Result | Status |
|---|---|---|---|---|---|
| [NFR-xx] | [target value from the requirement] | [threshold] | [tool and command] | [number with unit] | [met / not met / not measured] |
| [NFR-xx] | [target value from the requirement] | [threshold] | [tool and command] | [number with unit] | [met / not met / not measured] |

## 4. Guidance

How does the milestone look, and how is it operated? The keyboard operation is
reported separately, because it is the primary way of operating the application.

| Topic | Observation |
|---|---|
| Appearance | [what the milestone looks like, with a screenshot reference] |
| Keyboard operation | [which functions are reachable without a mouse, which are not] |
| Pointer operation | [which functions need the pointer] |
| Error messages | [one example of a message that was triggered during the acceptance] |
| Configuration | [which file the user edits, and which section carries which setting] |

## 5. Security and privacy

Which hardenings are effective, and **which negative test proves it**? A
positive test proves nothing about a defence.

| Hardening | Negative test | Result | Evidence |
|---|---|---|---|
| [hardening from section 9.2 of the specification] | [test name that performs the attack] | [pass / fail / NOT RUN with the exact reason] | [command output] |
| [privacy promise from section 9.5] | [how it was proven] | [pass / fail / NOT RUN with the exact reason] | [command output] |

## 6. Interoperability

Which foreign systems were checked — archives written by other tools, network
shares from Samba, WebDAV from Nextcloud?

| Foreign system | Tool used | Result | Evidence |
|---|---|---|---|
| [foreign system] | [tool and version] | [pass / fail / NOT RUN with the exact reason] | [command output] |

## 7. Known limitations

What does not work, and why? This list is mandatory and must not be empty as
long as there is anything to list.

| Limitation | Reason | Planned for |
|---|---|---|
| [what does not work] | [why] | [milestone or issue] |

## 8. Decision

Accepted, accepted with conditions, or rejected. The decision names the person
and the date.

| Field | Value |
|---|---|
| Decision | [accepted / accepted with conditions / rejected] |
| Conditions | [condition per line, or "none"] |
| Decided by | [name] |
| Date | [YYYY-MM-DD] |
| Next milestone | [M<n>] |

---

### Evidence index

Every command that was run for this acceptance, with its output in the
repository or quoted here.

| # | Command | Result | Where the output is |
|---|---|---|---|
| 1 | `[command]` | [exit code and summary] | [file and line, or quoted below] |
| 2 | `[command]` | [exit code and summary] | [file and line, or quoted below] |
