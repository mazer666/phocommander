#!/usr/bin/env python3
"""Check the acceptance criteria for a consistent link to the requirements.

WHAT: reads the acceptance criteria in ``docs/ACCEPTANCE-CRITERIA.md`` and the
      requirement catalogue in ``docs/REQUIREMENTS-IDS.md`` and compares the
      two. The check reports acceptance identifiers that occur twice or that
      leave a gap in their milestone, criteria that reference a requirement
      that does not exist, requirements of the milestones M0, M1 and M2 that
      have no criterion, criteria whose milestone differs from the milestone
      of the requirement, and criteria whose text is not a verifiable
      statement.
WHY: the chain from the specification to the test to the acceptance document
      is only as strong as its weakest link. A criterion that references a
      requirement that does not exist points at nothing, a requirement without
      a criterion has no acceptance, and a pass condition that says "works"
      cannot be failed and therefore proves nothing (specification, section
      13.2).

EXAMPLE:
    python3 scripts/check-acceptance-ids.py
    python3 scripts/check-acceptance-ids.py --acceptance docs/ACCEPTANCE-CRITERIA.md

The script exits with code 0 when every check passes and with code 1 when at
least one check fails. It prints one line per check.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# A cell boundary of a Markdown table row. A pipe that is escaped with a
# backslash belongs to the content of the cell and does not end it.
CELL_BOUNDARY = re.compile(r"(?<!\\)\|")

# The milestones that carry acceptance criteria.
COVERED_MILESTONES = ("M0", "M1", "M2")

# The words that must not stand in for a measurable pass condition. The
# patterns match whole words, so that "workspace" does not count as "works".
FORBIDDEN_PHRASES = (
    r"\bworks\b",
    r"\bis implemented\b",
    r"\bshould fit\b",
    r"\bshould work\b",
    r"\bprobably\b",
    r"\bseems to\b",
)

# The markers of the evidence column and the phrases that explain them.
EVIDENCE_MARKERS = ("PASS —", "PARTIAL —", "NOT RUN —")


def repository_root() -> Path:
    """Return the repository root, derived from the location of this script."""
    return Path(__file__).resolve().parent.parent


def split_cells(line: str) -> list[str]:
    """Return the cells of a Markdown table row."""
    stripped = line.strip()
    if stripped.startswith("|"):
        stripped = stripped[1:]
    if stripped.endswith("|"):
        stripped = stripped[:-1]
    return [cell.strip() for cell in CELL_BOUNDARY.split(stripped)]


def read_catalogue(path: Path) -> dict[str, dict[str, str]]:
    """Return the requirements of the catalogue, keyed by their identifier.

    WHAT: reads every row of the catalogue and records the milestone and the
    acceptance criterion the row names.
    """
    requirements: dict[str, dict[str, str]] = {}
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip().startswith("|"):
            continue
        cells = split_cells(line)
        if len(cells) < 7 or not re.fullmatch(r"(FR|NFR)-\d{2,}[a-z]?", cells[0]):
            continue
        requirements[cells[0]] = {
            "milestone": cells[3],
            "acceptance": cells[6],
            "line": str(number),
        }
    return requirements


def read_criteria(path: Path) -> list[dict[str, str]]:
    """Return the acceptance criteria of the acceptance document.

    WHAT: reads every table row that starts with an acceptance identifier and
    splits it into its six columns. A pipe that is escaped with a backslash
    belongs to the content of a cell and does not end it.
    """
    criteria: list[dict[str, str]] = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip().startswith("|"):
            continue
        cells = split_cells(line)
        if len(cells) != 6 or not re.fullmatch(r"AC-M\d-\d{2}", cells[0]):
            continue
        criteria.append(
            {
                "line": str(number),
                "id": cells[0],
                "requirement": cells[1],
                "verification": cells[2],
                "condition": cells[3],
                "milestone": cells[4],
                "evidence": cells[5],
            }
        )
    return criteria


def check_identifiers(criteria: list[dict[str, str]]) -> list[str]:
    """Return the failure messages for acceptance identifiers that are wrong."""
    failures: list[str] = []
    seen: dict[str, str] = {}
    for criterion in criteria:
        identifier = criterion["id"]
        if identifier in seen:
            failures.append(
                f"the acceptance criterion {identifier} occurs twice (lines {seen[identifier]} "
                f"and {criterion['line']})"
            )
        else:
            seen[identifier] = criterion["line"]

    by_milestone: dict[str, list[int]] = {}
    for criterion in criteria:
        milestone, number = criterion["id"].split("-")[1], int(criterion["id"].split("-")[2])
        by_milestone.setdefault(milestone, []).append(number)
    for milestone, numbers in sorted(by_milestone.items()):
        for expected, actual in enumerate(sorted(set(numbers)), start=1):
            if expected != actual:
                failures.append(
                    f"{milestone}: the numbering jumps from {expected - 1:02d} to {actual:02d}"
                )
    return failures


def check_references(
    criteria: list[dict[str, str]], requirements: dict[str, dict[str, str]]
) -> list[str]:
    """Return the failure messages for criteria that point at nothing."""
    failures: list[str] = []
    for criterion in criteria:
        requirement = criterion["requirement"]
        if requirement not in requirements:
            failures.append(
                f"line {criterion['line']}: {criterion['id']} references {requirement}, which is "
                "not in the catalogue"
            )
            continue
        expected = requirements[requirement]["milestone"]
        if criterion["milestone"] != expected:
            failures.append(
                f"line {criterion['line']}: {criterion['id']} belongs to {criterion['milestone']}, "
                f"but {requirement} belongs to {expected}"
            )
    return failures


def check_coverage(
    criteria: list[dict[str, str]], requirements: dict[str, dict[str, str]]
) -> list[str]:
    """Return the failure messages for requirements without a criterion.

    WHAT: every requirement of the milestones M0, M1 and M2 has at least one
    acceptance criterion, and no requirement of a later milestone has one.
    """
    failures: list[str] = []
    covered = {criterion["requirement"] for criterion in criteria}
    for identifier, entry in sorted(requirements.items()):
        if entry["milestone"] in COVERED_MILESTONES and identifier not in covered:
            failures.append(
                f"{identifier} ({entry['milestone']}) has no acceptance criterion"
            )
        if entry["milestone"] not in COVERED_MILESTONES and identifier in covered:
            failures.append(
                f"{identifier} ({entry['milestone']}) has an acceptance criterion although its "
                "milestone has none yet"
            )
    return failures


def check_bidirection(
    criteria: list[dict[str, str]], requirements: dict[str, dict[str, str]]
) -> list[str]:
    """Return the failure messages for a broken link between the two documents.

    WHAT: the acceptance criterion that the catalogue names for a requirement
    is the criterion that the acceptance document holds for it, and every
    criterion of the document is named by the catalogue.
    """
    failures: list[str] = []
    by_requirement: dict[str, list[str]] = {}
    for criterion in criteria:
        by_requirement.setdefault(criterion["requirement"], []).append(criterion["id"])
    for identifier, entry in sorted(requirements.items()):
        named = entry["acceptance"]
        actual = by_requirement.get(identifier, [])
        if named == "not yet assigned":
            if actual:
                failures.append(
                    f"{identifier} is marked as 'not yet assigned' in the catalogue but holds the "
                    f"criterion {actual[0]}"
                )
            continue
        if named not in actual:
            failures.append(
                f"{identifier} names the criterion {named} in the catalogue, but the acceptance "
                f"document holds {actual or 'no criterion'}"
            )
    return failures


def check_text(criteria: list[dict[str, str]]) -> list[str]:
    """Return the failure messages for criteria that cannot be verified.

    WHAT: every criterion names a command or an operation sequence, states a
    measurable pass condition and reports its evidence with one of the three
    markers of the specification.
    """
    failures: list[str] = []
    for criterion in criteria:
        if not criterion["verification"].startswith(("[automated]", "[manual]")):
            failures.append(
                f"line {criterion['line']}: the verification does not start with '[automated]' or "
                "'[manual]'"
            )
        if not criterion["condition"]:
            failures.append(f"line {criterion['line']}: the pass condition is empty")
        else:
            lowered = criterion["condition"].lower()
            for phrase in FORBIDDEN_PHRASES:
                if re.search(phrase, lowered):
                    shown = phrase.replace(r"\b", "")
                    failures.append(
                        f"line {criterion['line']}: the pass condition contains the unmeasurable "
                        f"phrase '{shown}'"
                    )
        if not criterion["evidence"].startswith(EVIDENCE_MARKERS):
            failures.append(
                f"line {criterion['line']}: the evidence does not start with 'PASS —', 'PARTIAL —' "
                "or 'NOT RUN —'"
            )
    return failures


def main() -> int:
    """Run every check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    root = repository_root()
    parser.add_argument("--acceptance", type=Path, default=root / "docs" / "ACCEPTANCE-CRITERIA.md")
    parser.add_argument("--catalog", type=Path, default=root / "docs" / "REQUIREMENTS-IDS.md")
    arguments = parser.parse_args()

    requirements = read_catalogue(arguments.catalog)
    criteria = read_criteria(arguments.acceptance)

    checks: list[tuple[str, list[str]]] = [
        ("the acceptance document and the catalogue are readable",
         [] if requirements and criteria else ["a document could not be read or holds no row"]),
        ("the acceptance identifiers are unique and gapless", check_identifiers(criteria)),
        ("every criterion references a requirement of the catalogue",
         check_references(criteria, requirements)),
        ("every requirement of M0, M1 and M2 has a criterion", check_coverage(criteria, requirements)),
        ("the catalogue and the acceptance document name each other",
         check_bidirection(criteria, requirements)),
        ("every criterion is verifiable", check_text(criteria)),
    ]

    print(f"acceptance   : {arguments.acceptance}")
    print(f"catalogue    : {arguments.catalog}")
    print(f"requirements in the catalogue : {len(requirements)}")
    print(f"criteria in the document      : {len(criteria)}")
    print("")

    failed = 0
    for name, failures in checks:
        if failures:
            failed += 1
            print(f"FAIL  {name}")
            for failure in failures:
                print(f"        {failure}")
        else:
            print(f"OK    {name}")

    print("")
    if failed:
        print(f"RESULT: FAIL — {failed} of {len(checks)} checks failed")
        return 1
    print(f"RESULT: PASS — {len(checks)} of {len(checks)} checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
