#!/usr/bin/env python3
"""Compare the key sets of the localisation catalogues of Phocommander.

WHAT: reads every catalogue directory below ``crates/phocommander-i18n/locales/``,
      reads the identifiers of the messages out of every ``.ftl`` file and
      compares the key sets of all locales with each other. It reports the keys
      that are missing in a locale and the keys that a locale holds in excess.
WHY: the specification (section 8.5) names three gates for the localisation.
      This script is the first of them — the key parity. A translation that is
      missing a key is a defect that the user sees as an English word or as a
      visible key, and a key that no locale knows is a key that the code uses
      and no catalogue answers. The comparison is done on the files and not on
      a list that is written twice, so that the two cannot drift apart
      (requirements FR-120 and FR-121).

EXAMPLE:
    python3 scripts/check-i18n-keys.py
    python3 scripts/check-i18n-keys.py --locales crates/phocommander-i18n/locales

The script exits with code 0 when every locale holds exactly the same key set
and with code 1 when at least one locale differs. It prints one line per
locale and one line per deviation.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# A message identifier of a Fluent catalogue. An identifier starts with a
# letter and continues with letters, digits and the dash.
MESSAGE_PATTERN = re.compile(r"^([A-Za-z][A-Za-z0-9_-]*)\s*=")

# A term of a Fluent catalogue. A term is not a message and has no key of its
# own.
TERM_PATTERN = re.compile(r"^-([A-Za-z][A-Za-z0-9_-]*)\s*=")

# An attribute of a message. An attribute belongs to the message above it and
# has no key of its own.
ATTRIBUTE_PATTERN = re.compile(r"^[A-Za-z][A-Za-z0-9_-]*\.")

# The domains of the catalogue. Every locale holds exactly these files.
DOMAINS = ("common", "actions", "panel", "viewer", "palette")


def repository_root() -> Path:
    """Return the repository root, derived from the location of this script."""
    return Path(__file__).resolve().parent.parent


def keys_of_file(path: Path) -> tuple[set[str], list[str]]:
    """Return the message identifiers of a catalogue file and its problems.

    WHAT: reads the file line by line. A line that holds an identifier is a
          message; a line that holds a term (``-term =``) and a line that
          holds an attribute (``.attr =``) are not messages and are skipped.
    """
    keys: set[str] = set()
    problems: list[str] = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        # WHY: a message may span several lines. Every line that is indented
        # belongs to the message above it — a plural form or a continuation of
        # the value — and carries no identifier of its own.
        if line[:1].isspace():
            continue
        match = MESSAGE_PATTERN.match(stripped)
        if match is not None:
            keys.add(match.group(1))
            continue
        if TERM_PATTERN.match(stripped) is not None:
            continue
        if ATTRIBUTE_PATTERN.match(stripped) is not None:
            continue
        problems.append(f"{path}:{number}: the line is neither a comment nor a message")
    return keys, problems


def keys_of_locale(directory: Path) -> tuple[set[str], list[str]]:
    """Return the message identifiers of a locale directory and its problems."""
    keys: set[str] = set()
    problems: list[str] = []
    for domain in DOMAINS:
        path = directory / f"{domain}.ftl"
        if not path.exists():
            problems.append(f"{path}: the domain file is missing")
            continue
        domain_keys, domain_problems = keys_of_file(path)
        keys |= domain_keys
        problems.extend(domain_problems)
    return keys, problems


def main() -> int:
    """Run the comparison and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    root = repository_root()
    parser.add_argument(
        "--locales",
        type=Path,
        default=root / "crates" / "phocommander-i18n" / "locales",
    )
    arguments = parser.parse_args()

    if not arguments.locales.is_dir():
        print(f"ERROR: the directory {arguments.locales} does not exist", file=sys.stderr)
        return 1

    directories = sorted(
        (item for item in arguments.locales.iterdir() if item.is_dir()),
        key=lambda item: item.name,
    )
    if not directories:
        print(f"ERROR: the directory {arguments.locales} holds no locale", file=sys.stderr)
        return 1

    key_sets: dict[str, set[str]] = {}
    problems: list[str] = []
    for directory in directories:
        keys, directory_problems = keys_of_locale(directory)
        key_sets[directory.name] = keys
        problems.extend(directory_problems)

    # The reference is the locale with the most keys: a locale that holds more
    # keys than the others is not wrong by itself, but the difference has to be
    # reported.
    reference_name = max(key_sets, key=lambda name: len(key_sets[name]))
    reference = key_sets[reference_name]

    print(f"locales   : {arguments.locales}")
    for directory in directories:
        print(f"            {directory.name}: {len(key_sets[directory.name])} keys")
    print(f"reference : {reference_name} ({len(reference)} keys)")
    print("")

    deviations = 0
    for name, keys in key_sets.items():
        missing = sorted(reference - keys)
        extra = sorted(keys - reference)
        if not missing and not extra:
            print(f"OK    {name}: the key set is identical to {reference_name}")
            continue
        deviations += 1
        for key in missing:
            print(f"DIFF  {name}: the key '{key}' is missing")
        for key in extra:
            print(f"DIFF  {name}: the key '{key}' is not in {reference_name}")

    if problems:
        print("")
        for problem in problems:
            print(f"DIFF  {problem}")

    print("")
    failed = deviations + (1 if problems else 0)
    if failed:
        print(f"RESULT: FAIL — {failed} of {len(directories)} locales differ")
        return 1
    print(f"RESULT: PASS — {len(directories)} locales hold the same {len(reference)} keys")
    return 0


if __name__ == "__main__":
    sys.exit(main())
