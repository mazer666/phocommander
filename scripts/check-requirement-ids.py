#!/usr/bin/env python3
"""Check the requirement catalogue for gapless IDs and consistent rows.

WHAT: verifies that docs/REQUIREMENTS-IDS.md contains no duplicate IDs, that
the FR and NFR sequences are gapless, that every row carries all six columns,
and that every source section named in a row exists as a heading in
docs/REQUIREMENTS.md.

WHY: the catalogue is the index into the binding specification. A duplicated or
skipped ID makes an acceptance criterion ambiguous, and a source section that
does not exist in the specification makes the row unverifiable. Both defects are
invisible to a reader and expensive to find later.

EXAMPLE:
    python3 scripts/check-requirement-ids.py
    python3 scripts/check-requirement-ids.py --catalog docs/REQUIREMENTS-IDS.md

The script exits with code 0 when every check passes and with code 1 when at
least one check fails. It prints one line per check.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# The controlled vocabulary of the milestone column.
MILESTONES = [f"M{index}" for index in range(10)]

# The controlled vocabulary of the priority column.
PRIORITIES = ["Must", "Should", "Could"]

# The controlled vocabulary of the verification column.
VERIFICATIONS = [
    "automated (CI gate)",
    "automated (benchmark)",
    "automated (config parse)",
    "automated (dependency scan)",
    "automated (fuzz test)",
    "automated (integration test)",
    "automated (property test)",
    "automated (UI test)",
    "automated (unit test)",
    "manual (document review)",
    "manual (operational test)",
    "manual (visual inspection)",
]

# A catalogue row: ID | Title | Source | Milestone | Priority | Verification
ROW_PATTERN = re.compile(
    r"^\|\s*(FR-\d{2,}|NFR-\d{2,})\s*\|(.+)\|\s*$",
)

# An identifier with its number.
ID_PATTERN = re.compile(r"^(FR|NFR)-(\d{2,})$")


def repository_root() -> Path:
    """Return the repository root, derived from the location of this script.

    WHY: the script has to find the catalogue and the specification no matter
    from which directory it is called. Deriving the root from the script
    location keeps it independent of the working directory.
    """
    return Path(__file__).resolve().parent.parent


def read_headings(specification: Path) -> set[str]:
    """Return the set of heading texts of the specification.

    WHAT: collects every line that starts with a hash sign and returns the
    heading text without the hashes.
    """
    headings: set[str] = set()
    for line in specification.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("#"):
            headings.add(stripped.lstrip("#").strip())
    return headings


def read_rows(catalog: Path) -> list[dict[str, str]]:
    """Return the catalogue rows as dictionaries.

    WHAT: reads every table row that starts with a requirement ID and splits
    it into its six columns.
    """
    rows: list[dict[str, str]] = []
    for number, line in enumerate(catalog.read_text(encoding="utf-8").splitlines(), start=1):
        match = ROW_PATTERN.match(line)
        if match is None:
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        rows.append(
            {
                "line": str(number),
                "id": cells[0],
                "title": cells[1],
                "source": cells[2],
                "milestone": cells[3],
                "priority": cells[4],
                "verification": cells[5],
                "cells": str(len(cells)),
            }
        )
    return rows


def check_no_duplicates(rows: list[dict[str, str]]) -> list[str]:
    """Return the failure messages for duplicated identifiers."""
    seen: dict[str, int] = {}
    failures: list[str] = []
    for row in rows:
        identifier = row["id"]
        if identifier in seen:
            failures.append(
                f"duplicate id {identifier} on lines {seen[identifier]} and {row['line']}"
            )
        else:
            seen[identifier] = int(row["line"])
    return failures


def check_gapless(rows: list[dict[str, str]], prefix: str) -> list[str]:
    """Return the failure messages for a gapless number sequence of one prefix.

    WHAT: collects all numbers of the given prefix, sorts them and reports every
    number that is missing between the lowest and the highest one.
    """
    numbers = sorted(
        int(match.group(2))
        for row in rows
        if (match := ID_PATTERN.match(row["id"])) and match.group(1) == prefix
    )
    if not numbers:
        return [f"no rows with prefix {prefix} found"]
    failures: list[str] = []
    for expected, actual in enumerate(range(numbers[0], numbers[-1] + 1), start=1):
        if expected not in numbers:
            failures.append(f"{prefix}-{expected:02d} is missing")
    if numbers[0] != 1:
        failures.append(f"{prefix} numbering starts at {numbers[0]}, expected 1")
    return failures


def check_columns(rows: list[dict[str, str]]) -> list[str]:
    """Return the failure messages for incomplete or invalid columns."""
    failures: list[str] = []
    for row in rows:
        if row["cells"] != "6":
            failures.append(f"line {row['line']}: {row['cells']} columns instead of 6")
        for column in ("title", "source", "milestone", "priority", "verification"):
            if not row[column]:
                failures.append(f"line {row['line']}: column {column} is empty")
        if row["milestone"] and row["milestone"] not in MILESTONES:
            failures.append(
                f"line {row['line']}: milestone '{row['milestone']}' is not one of {MILESTONES}"
            )
        if row["priority"] and row["priority"] not in PRIORITIES:
            failures.append(
                f"line {row['line']}: priority '{row['priority']}' is not one of {PRIORITIES}"
            )
        if row["verification"] and row["verification"] not in VERIFICATIONS:
            failures.append(
                f"line {row['line']}: verification '{row['verification']}' is not in the vocabulary"
            )
    return failures


def check_sources(rows: list[dict[str, str]], headings: set[str]) -> list[str]:
    """Return the failure messages for source sections that do not exist."""
    failures: list[str] = []
    for row in rows:
        for segment in (part.strip() for part in row["source"].split(">")):
            if segment and segment not in headings:
                failures.append(
                    f"line {row['line']}: source section '{segment}' is not a heading of the specification"
                )
    return failures


def check_acceptance_coverage(
    rows: list[dict[str, str]], acceptance: Path
) -> list[str]:
    """Return the failure messages for acceptance coverage of M0, M1 and M2.

    WHAT: every requirement of the milestones M0, M1 and M2 needs at least one
    acceptance criterion, and every requirement ID named in the acceptance
    criteria needs to exist in the catalogue.
    """
    if not acceptance.exists():
        return [f"{acceptance} does not exist"]

    text = acceptance.read_text(encoding="utf-8")
    referenced = set(re.findall(r"\b(?:FR|NFR)-\d{2,}\b", text))
    catalogue = {row["id"] for row in rows}

    failures: list[str] = []
    for identifier in sorted(referenced - catalogue):
        failures.append(f"{identifier} is used in the acceptance criteria but not in the catalogue")

    for row in rows:
        if row["milestone"] in {"M0", "M1", "M2"} and row["id"] not in referenced:
            failures.append(
                f"{row['id']} ({row['milestone']}) has no acceptance criterion"
            )
    return failures


def check_acceptance_ids(acceptance: Path) -> list[str]:
    """Return the failure messages for duplicated or gapped acceptance IDs.

    WHAT: the acceptance criteria carry the identifiers AC-M<n>-<nn>. Within a
    milestone the numbers must run without a gap and an identifier must not
    occur twice.
    """
    if not acceptance.exists():
        return [f"{acceptance} does not exist"]

    text = acceptance.read_text(encoding="utf-8")
    identifiers = re.findall(r"\bAC-M(\d)-(\d{2})\b", text)
    failures: list[str] = []
    seen: set[str] = set()
    for milestone, number in identifiers:
        identifier = f"AC-M{milestone}-{number}"
        if identifier in seen:
            failures.append(f"the acceptance criterion {identifier} occurs twice")
        seen.add(identifier)

    by_milestone: dict[str, list[int]] = {}
    for milestone, number in identifiers:
        by_milestone.setdefault(milestone, []).append(int(number))
    for milestone, numbers in sorted(by_milestone.items()):
        unique = sorted(set(numbers))
        for expected, actual in enumerate(unique, start=1):
            if expected != actual:
                failures.append(
                    f"AC-M{milestone}: the numbering jumps from {expected - 1:02d} to {actual:02d}"
                )
    return failures


def main() -> int:
    """Run every check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    root = repository_root()
    parser.add_argument("--catalog", type=Path, default=root / "docs" / "REQUIREMENTS-IDS.md")
    parser.add_argument("--specification", type=Path, default=root / "docs" / "REQUIREMENTS.md")
    parser.add_argument("--acceptance", type=Path, default=root / "docs" / "ACCEPTANCE-CRITERIA.md")
    arguments = parser.parse_args()

    rows = read_rows(arguments.catalog)
    headings = read_headings(arguments.specification)
    functional = [row for row in rows if row["id"].startswith("FR-")]
    non_functional = [row for row in rows if row["id"].startswith("NFR-")]

    checks: list[tuple[str, list[str]]] = [
        ("catalogue is readable and has rows", [] if rows else ["no rows found"]),
        ("FR sequence is gapless and starts at FR-01", check_gapless(rows, "FR")),
        ("NFR sequence is gapless and starts at NFR-01", check_gapless(rows, "NFR")),
        ("no duplicated identifiers", check_no_duplicates(rows)),
        ("all six columns are filled and valid", check_columns(rows)),
        ("every source section exists in the specification", check_sources(rows, headings)),
        (
            "every M0/M1/M2 requirement has an acceptance criterion",
            check_acceptance_coverage(rows, arguments.acceptance),
        ),
        (
            "the acceptance criteria have gapless identifiers",
            check_acceptance_ids(arguments.acceptance),
        ),
    ]

    print(f"catalogue : {arguments.catalog}")
    print(f"spec      : {arguments.specification}")
    print(f"rows      : {len(rows)} ({len(functional)} FR, {len(non_functional)} NFR)")
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
