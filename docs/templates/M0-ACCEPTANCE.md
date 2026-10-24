# M0 Acceptance — Phocommander

> Template. Copy this file to `docs/acceptance/M0-ACCEPTANCE.md`, fill every
> placeholder in square brackets, and delete this quote block.
>
> The section order is prescribed by `docs/REQUIREMENTS.md`, section
> "Abnahmeregelungen > Aufbau einer Abnahme", and must not be changed.
>
> Evidence rules, from the same section:
>
> - An evidence is a command output, a test name with a green result, or an
>   extract from an acceptance documentation. "It works" is not evidence.
>   "Should fit" is not evidence. A coverage percentage without a measurement
>   is not evidence.
> - A test that could not be executed is reported as **NOT RUN** with the exact
>   reason. A NOT RUN with a reason is allowed and honest. A faked PASS is not.
> - No gate is weakened to become green. No lint is commented out with an
>   allow attribute, no check is removed from the CI, no test is disabled.

| Field | Value |
|---|---|
| Milestone | M0 — Foundation |
| Acceptance date | [YYYY-MM-DD] |
| Specification version | [version of `docs/REQUIREMENTS.md`, e.g. 1.1] |
| Catalogue version | [version of `docs/REQUIREMENTS-IDS.md`] |
| Decided by | [name] |
| Result | [accepted / accepted with conditions / rejected] |

---

## 1. Functionality

Which functions of this milestone exist, and how do they behave? Every row
names the requirement ID and an observed behaviour, never a claim.

| Requirement | Behaviour observed | Evidence |
|---|---|---|
| [FR-xx] | [what was observed, in one sentence] | [command output or test name] |
| [FR-xx] | [what was observed, in one sentence] | [command output or test name] |

## 2. Technique

Which architecture decisions are implemented, which crate boundaries were
kept, which tools ran through?

| Item | Result | Evidence |
|---|---|---|
| Crate boundaries | [pass / fail, with the script that proves it] | [command output] |
| Dependency direction | [pass / fail] | [command output] |
| Formatting (`cargo fmt --check`) | [pass / fail] | [command output] |
| Static analysis (`cargo clippy --all-targets -- -D warnings`) | [pass / fail] | [command output] |
| Licence check (`cargo deny check`) | [pass / fail] | [command output] |
| Vulnerability check (`cargo audit`) | [pass / fail] | [command output] |
| Test run (`cargo test --workspace`) | [pass / fail] | [summary line] |
| Platform build | [pass / fail / NOT RUN with the exact reason] | [command output] |

## 3. Performance

Which target values were measured, with which tool, with which result? A target
value that was not measured is reported as "not measured", never as "met".

| Target value | Threshold | Measured with | Result | Status |
|---|---|---|---|---|
| [target value from the performance table] | [threshold] | [tool and command] | [number with unit] | [met / not met / not measured] |
| [target value from the performance table] | [threshold] | [tool and command] | [number with unit] | [met / not met / not measured] |

## 4. Guidance

How does the milestone look, and how is it operated? Keyboard operation is
reported separately, because it is the main way of operating the application.

| Topic | Observation |
|---|---|
| Appearance | [what the milestone looks like, with a screenshot reference] |
| Keyboard operation | [which functions are reachable without a mouse, which are not] |
| Mouse operation | [which functions need the pointer] |
| Error messages | [one example of a message that was triggered during the acceptance] |

## 5. Security and privacy

Which hardenings are effective, and **which negative test proves it**? A
positive test proves nothing about the defence.

| Hardening | Negative test | Result | Evidence |
|---|---|---|---|
| [hardening from the security section] | [test name that performs the attack] | [pass / fail / NOT RUN with the exact reason] | [command output] |
| [privacy promise] | [how it was proven] | [pass / fail / NOT RUN with the exact reason] | [command output] |

## 6. Interoperability

Which foreign systems were checked — archives with foreign tools, network
drives with Samba, WebDAV with Nextcloud?

| Foreign system | Tool used | Result | Evidence |
|---|---|---|---|
| [foreign system] | [tool and version] | [pass / fail / NOT RUN with the exact reason] | [command output] |

## 7. Known limitations

What does not work, and why? This list is mandatory and must not be empty as
long as there is something to list.

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
