#!/usr/bin/env python3
"""Check the requirement catalogue against the binding specification.

WHAT: reads the requirement catalogue in ``docs/REQUIREMENTS-IDS.md`` and the
      requirement catalogue of the specification in ``docs/REQUIREMENTS.md``
      and compares the two. The check reports the identifiers that the
      catalogue misses, the identifiers that it adds, duplicated identifiers,
      identifiers whose milestone, priority or verification method drifted away
      from the specification, source references that name a section or a line
      that does not hold the requirement, and acceptance criteria that do not
      exist in ``docs/ACCEPTANCE-CRITERIA.md``.
WHY: the catalogue is the index into the specification. An identifier that is
      missing, doubled or changed makes an acceptance criterion ambiguous, and
      a source reference that points at the wrong line makes a row
      unverifiable. Both defects are invisible to a reader and expensive to
      find later.
      The numbering of the specification is thematic: it has blocks that are
      numbered apart from each other, for example FR-06 and then FR-10. Those
      gaps are part of the numbering and are not defects. Inside a block the
      numbering must run without a gap, and a letter suffix such as FR-22a
      belongs to the block of its base number.

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

# A requirement identifier: a prefix, a number and an optional letter suffix.
IDENTIFIER = re.compile(r"^(FR|NFR)-(\d{2,})([a-z]?)$")

# A row of the specification: the identifier, the requirement, the milestone,
# the priority and the verification method.
SPEC_ROW = re.compile(
    r"^\|\s*(FR-\d+[a-z]?|NFR-\d+[a-z]?)\s*\|"
    r"(?P<requirement>[^|]*)\|"
    r"\s*(?P<milestone>M\d[a-z]?)\s*\|"
    r"\s*(?P<priority>Must|Should|Could)\s*\|"
    r"\s*(?P<verification>[^|]*?)\s*\|\s*$"
)

# A heading of section 4 of the specification.
SECTION_HEADING = re.compile(r"^###\s+(4\.\d+)\s+(.*)$")

# A source reference of the catalogue: section number, section title, line.
SOURCE_CELL = re.compile(r"^§(?P<number>\d+\.\d+)\s+(?P<title>.+),\s+line\s+(?P<line>\d+)$")

# A cell boundary of a Markdown table row. A pipe that is escaped with a
# backslash belongs to the content of the cell and does not end it.
CELL_BOUNDARY = re.compile(r"(?<!\\)\|")

# The kinds of verification the catalogue may name in front of the method of
# the specification.
CLASSIFICATIONS = ("automated", "manual", "mixed")

# The milestones that carry acceptance criteria today.
COVERED_MILESTONES = ("M0", "M1", "M2")

# The milestones that the specification knows. M2b is the milestone of the
# internationalisation; it stands between M2 and M3.
MILESTONES = tuple(f"M{index}" for index in range(10)) + ("M2b",)

# The priorities of the specification.
PRIORITIES = ("Must", "Should", "Could")


def repository_root() -> Path:
    """Return the repository root, derived from the location of this script.

    WHY: the script has to find the catalogue and the specification no matter
    from which directory it is called. Deriving the root from the script
    location keeps it independent of the working directory.
    """
    return Path(__file__).resolve().parent.parent


def split_cells(line: str) -> list[str]:
    """Return the cells of a Markdown table row.

    WHAT: splits a row at its cell boundaries and removes the whitespace and
    the outer pipes.
    """
    stripped = line.strip()
    if stripped.startswith("|"):
        stripped = stripped[1:]
    if stripped.endswith("|"):
        stripped = stripped[:-1]
    return [cell.strip() for cell in CELL_BOUNDARY.split(stripped)]


def read_specification(path: Path) -> dict[str, dict[str, str]]:
    """Return the requirements of the specification, keyed by their identifier.

    WHAT: reads every table row of section 4 and records the milestone, the
    priority, the verification method, the section and the line number.
    """
    requirements: dict[str, dict[str, str]] = {}
    section = ""
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        heading = SECTION_HEADING.match(line)
        if heading is not None:
            section = f"{heading.group(1)} {heading.group(2).strip()}"
            continue
        row = SPEC_ROW.match(line)
        if row is None:
            continue
        identifier = row.group(1)
        requirements[identifier] = {
            "milestone": row.group("milestone"),
            "priority": row.group("priority"),
            "verification": row.group("verification").strip(),
            "section": section,
            "line": str(number),
        }
    return requirements


def read_catalogue(path: Path) -> list[dict[str, str]]:
    """Return the rows of the catalogue.

    WHAT: reads every table row that starts with a requirement identifier and
    splits it into its seven columns.
    """
    rows: list[dict[str, str]] = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip().startswith("|"):
            continue
        cells = split_cells(line)
        if not cells or not re.fullmatch(r"(FR|NFR)-\d{2,}[a-z]?", cells[0]):
            continue
        row = {"line": str(number), "cells": str(len(cells))}
        for index, name in enumerate(
            ("id", "title", "source", "milestone", "priority", "verification", "acceptance")
        ):
            row[name] = cells[index] if index < len(cells) else ""
        rows.append(row)
    return rows


def check_duplicates(rows: list[dict[str, str]], requirements: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for identifiers that occur twice."""
    failures: list[str] = []
    seen: dict[str, str] = {}
    for row in rows:
        identifier = row["id"]
        if identifier in seen:
            failures.append(
                f"duplicate id {identifier} on lines {seen[identifier]} and {row['line']}"
            )
        else:
            seen[identifier] = row["line"]
    if len(requirements) != len(set(requirements)):
        failures.append("the specification lists an identifier twice")
    return failures


def check_same_set(rows: list[dict[str, str]], requirements: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for identifiers that differ on either side.

    WHAT: compares the set of the catalogue with the set of the specification.
    An identifier that the catalogue misses and an identifier that the
    catalogue adds are both reported.
    """
    catalogue = {row["id"] for row in rows}
    expected = set(requirements)
    failures: list[str] = []
    for identifier in sorted(expected - catalogue):
        failures.append(f"{identifier} is in the specification but not in the catalogue")
    for identifier in sorted(catalogue - expected):
        failures.append(f"{identifier} is in the catalogue but not in the specification")
    return failures


def check_gapless(
    rows: list[dict[str, str]], requirements: dict[str, dict[str, str]]
) -> list[str]:
    """Return the failure messages for a gap inside a block of the numbering.

    WHAT: the specification numbers its requirements thematically. Its numbers
    form blocks — runs of consecutive numbers — and the blocks are numbered
    apart from each other: FR-06 is followed by FR-10, and that gap belongs
    to the numbering. Inside a block no number may be missing from the
    catalogue. A letter suffix belongs to the block of its base number: the
    base must exist and the letters must run from "a" without a gap.
    WHY: a number that is missing inside a block means that a requirement was
    dropped from the catalogue; a suffix that starts at "b" means that a
    requirement was dropped from the middle of a series.
    """
    failures: list[str] = []
    catalogue = {row["id"] for row in rows}
    for prefix in ("FR", "NFR"):
        spec_numbers: set[int] = set()
        spec_bases: dict[int, set[str]] = {}
        for identifier in requirements:
            match = IDENTIFIER.match(identifier)
            if match is None or match.group(1) != prefix:
                continue
            number = int(match.group(2))
            spec_numbers.add(number)
            if match.group(3):
                spec_bases.setdefault(number, set()).add(match.group(3))

        catalogue_numbers: set[int] = set()
        catalogue_bases: dict[int, set[str]] = {}
        for identifier in catalogue:
            match = IDENTIFIER.match(identifier)
            if match is None or match.group(1) != prefix:
                continue
            number = int(match.group(2))
            catalogue_numbers.add(number)
            if match.group(3):
                catalogue_bases.setdefault(number, set()).add(match.group(3))

        if not spec_numbers:
            failures.append(f"no identifier with the prefix {prefix} was found")
            continue

        for number in sorted(spec_numbers):
            if number not in catalogue_numbers:
                failures.append(
                    f"{prefix}-{number:02d} is in the specification but not in the catalogue"
                )
        for number in sorted(catalogue_numbers - spec_numbers):
            failures.append(
                f"{prefix}-{number:02d} is in the catalogue but not in the specification"
            )

        for number, suffixes in sorted(spec_bases.items()):
            catalogue_suffixes = catalogue_bases.get(number, set())
            for suffix in sorted(suffixes - catalogue_suffixes):
                failures.append(
                    f"{prefix}-{number:02d}{suffix} is in the specification but not in the catalogue"
                )
            for suffix in sorted(catalogue_suffixes - suffixes):
                failures.append(
                    f"{prefix}-{number:02d}{suffix} is in the catalogue but not in the specification"
                )
            letters = sorted(suffixes)
            if letters:
                expected = [chr(ord("a") + index) for index in range(len(letters))]
                if letters != expected:
                    failures.append(
                        f"{prefix}-{number:02d}: the suffix sequence is "
                        f"{''.join(letters)} instead of {''.join(expected)}"
                    )
    return failures

def check_columns(rows: list[dict[str, str]], requirements: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for rows that are incomplete or that drifted.

    WHAT: every row carries seven columns, and the milestone, the priority
    and the verification method are the values of the specification. The
    verification cell starts with the kind of the check.
    """
    failures: list[str] = []
    for row in rows:
        if row["cells"] != "7":
            failures.append(f"line {row['line']}: {row['cells']} columns instead of 7")
        for name in ("title", "source", "milestone", "priority", "verification", "acceptance"):
            if not row[name]:
                failures.append(f"line {row['line']}: column {name} is empty")
        if row["milestone"] and row["milestone"] not in MILESTONES:
            failures.append(
                f"line {row['line']}: milestone '{row['milestone']}' is not one of {MILESTONES}"
            )
        if row["priority"] and row["priority"] not in PRIORITIES:
            failures.append(
                f"line {row['line']}: priority '{row['priority']}' is not one of {PRIORITIES}"
            )
        classification, _, method = row["verification"].partition("—")
        if classification.strip() not in CLASSIFICATIONS:
            failures.append(
                f"line {row['line']}: the verification kind '{classification.strip()}' is not one of "
                f"{CLASSIFICATIONS}"
            )
        expected = requirements.get(row["id"])
        if expected is None:
            continue
        if row["milestone"] != expected["milestone"]:
            failures.append(
                f"line {row['line']}: the milestone '{row['milestone']}' differs from the "
                f"specification ('{expected['milestone']}')"
            )
        if row["priority"] != expected["priority"]:
            failures.append(
                f"line {row['line']}: the priority '{row['priority']}' differs from the "
                f"specification ('{expected['priority']}')"
            )
        if method.strip() != expected["verification"]:
            failures.append(
                f"line {row['line']}: the verification method differs from the specification "
                f"('{expected['verification']}')"
            )
    return failures


def check_sources(rows: list[dict[str, str]], specification: Path) -> list[str]:
    """Return the failure messages for source references that do not hold.

    WHAT: every source cell names a section and a line of the specification.
    The section must exist as a heading, and the named line must be the line
    on which the requirement stands.
    """
    lines = specification.read_text(encoding="utf-8").splitlines()
    failures: list[str] = []
    for row in rows:
        match = SOURCE_CELL.match(row["source"])
        if match is None:
            failures.append(
                f"line {row['line']}: the source '{row['source']}' does not have the form "
                "'§<section> <title>, line <number>'"
            )
            continue
        heading = f"{match.group('number')} {match.group('title').strip()}"
        if not any(line.startswith(f"### {heading}") for line in lines):
            failures.append(
                f"line {row['line']}: the section '### {heading}' is not a heading of the "
                "specification"
            )
        number = int(match.group("line"))
        if number < 1 or number > len(lines):
            failures.append(f"line {row['line']}: the line {number} is outside the specification")
            continue
        if row["id"] not in lines[number - 1]:
            failures.append(
                f"line {row['line']}: the line {number} of the specification does not hold "
                f"{row['id']}"
            )
    return failures


def check_acceptance_column(rows: list[dict[str, str]], acceptance: Path) -> list[str]:
    """Return the failure messages for the acceptance criterion column.

    WHAT: a requirement of M0, M1 or M2 names an acceptance criterion that
    exists in the acceptance document. A requirement of a later milestone says
    "not yet assigned", because no criterion for it exists yet.
    """
    if not acceptance.exists():
        return [f"{acceptance} does not exist"]
    text = acceptance.read_text(encoding="utf-8")
    known = set(re.findall(r"\bAC-M\d[a-z]?-\d{2}\b", text))
    failures: list[str] = []
    for row in rows:
        cell = row["acceptance"]
        if cell == "not yet assigned":
            continue
        if not re.fullmatch(r"AC-M\d[a-z]?-\d{2}", cell):
            failures.append(
                f"line {row['line']}: '{cell}' is neither an acceptance criterion nor "
                "'not yet assigned'"
            )
            continue
        if cell not in known:
            failures.append(
                f"line {row['line']}: the acceptance criterion {cell} does not exist in "
                f"{acceptance.name}"
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

    requirements = read_specification(arguments.specification)
    rows = read_catalogue(arguments.catalog)

    checks: list[tuple[str, list[str]]] = [
        ("the specification and the catalogue are readable", [] if requirements and rows else
         ["a document could not be read or holds no row"]),
        ("the identifier sets are equal", check_same_set(rows, requirements)),
        ("no identifier occurs twice", check_duplicates(rows, requirements)),
        ("the numbering is gapless inside every block", check_gapless(rows, requirements)),
        ("every row carries the seven columns of the catalogue", check_columns(rows, requirements)),
        ("every source names the line of the requirement", check_sources(rows, arguments.specification)),
        ("every acceptance criterion exists in the acceptance document",
         check_acceptance_column(rows, arguments.acceptance)),
    ]

    print(f"catalogue   : {arguments.catalog}")
    print(f"specification: {arguments.specification}")
    print(f"requirements in the specification : {len(requirements)}")
    print(f"rows in the catalogue             : {len(rows)}")
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
