#!/usr/bin/env python3
"""Check the interface profiles of the configuration directory.

WHAT: verifies that every profile below ``config/profiles/`` parses as TOML,
      carries exactly the keys of the profile schema of the specification
      (section 7.2), carries a comment above every key and every section, and
      that the two shipped profiles ``clean`` and ``full`` hold the values that
      their names promise.
WHY: a profile is the first contact point of a user who wants a different
      interface. A profile without a comment forces the user to guess, a
      profile with an unknown key is ignored silently, and a profile that
      carries more than interface decisions would store user data in a file
      that is meant to be shared (requirement FR-145).

EXAMPLE:
    python3 scripts/check-interface-profiles.py
    python3 scripts/check-interface-profiles.py --directory config/profiles

The script exits with code 0 when every check passes and with code 1 when at
least one check fails. It prints one line per check.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

# The keys of a profile. A profile carries interface decisions and nothing
# else; the specification, section 7.2, calls this out explicitly.
PROFILE_KEYS = ("name", "description", "panels", "commands", "display")

# The keys that a profile may carry in addition. A profile may hold a section
# per platform, which is the last step of the resolution order of the
# specification, section 7.3, and the decision 6 of ADR-0005.
OPTIONAL_KEYS = ("platform",)

# The areas of the interface that a profile can switch on and off.
PANEL_KEYS = (
    "show_toolbar",
    "show_directory_tree",
    "show_quick_view",
    "show_job_list",
    "show_status_bar",
    "show_menu_bar",
    "show_command_line",
)

# The kinds of visibility of the commands.
VISIBILITIES = ("visible", "palette", "hidden")

# The values of the two shipped profiles. The clean profile removes the
# furniture and keeps the capability (specification, section 7.4); the full
# profile shows everything (requirement FR-141).
EXPECTED = {
    "clean": {
        "name": "clean",
        "panels": {
            "show_toolbar": False,
            "show_directory_tree": False,
            "show_quick_view": False,
            "show_job_list": False,
            "show_status_bar": True,
            "show_menu_bar": False,
            "show_command_line": True,
        },
        "visibility": "palette",
    },
    "full": {
        "name": "full",
        "panels": {key: True for key in PANEL_KEYS},
        "visibility": "visible",
    },
}

# A table header and a key assignment of a TOML file.
TABLE_PATTERN = re.compile(r"^\[[^\]]+\]\s*$")
KEY_PATTERN = re.compile(r"^[A-Za-z0-9_\"'\-.]+\s*=")


def repository_root() -> Path:
    """Return the repository root, derived from the location of this script."""
    return Path(__file__).resolve().parent.parent


def check_parses(files: list[Path]) -> tuple[list[str], dict[Path, dict]]:
    """Return the failure messages and the parsed content of every file."""
    failures: list[str] = []
    parsed: dict[Path, dict] = {}
    for path in files:
        try:
            parsed[path] = tomllib.loads(path.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError as error:
            failures.append(f"{path}: does not parse as TOML ({error})")
    return failures, parsed


def check_keys(parsed: dict[Path, dict]) -> list[str]:
    """Return the failure messages for keys that the schema does not know.

    WHAT: a profile holds exactly the keys of the schema. An unknown key is
    ignored by the loader, so a profile that carries one would silently do
    something else than the user expects.
    """
    failures: list[str] = []
    for path, content in parsed.items():
        for key in content:
            if key not in PROFILE_KEYS and key not in OPTIONAL_KEYS:
                failures.append(f"{path}: the key '{key}' is not part of the profile schema")
        for platform, override in content.get("platform", {}).items():
            if not isinstance(override, dict):
                failures.append(
                    f"{path}: the override for the platform '{platform}' is not a table"
                )
                continue
            for key in override:
                if key not in ("panels", "commands", "display"):
                    failures.append(
                        f"{path}: the override for the platform '{platform}' holds the unknown key "
                        f"'{key}'"
                    )
        panels = content.get("panels", {})
        for key in panels:
            if key not in PANEL_KEYS:
                failures.append(f"{path}: the panel flag '{key}' is not part of the profile schema")
        commands = content.get("commands", {})
        for key in commands:
            if key != "visibility":
                failures.append(f"{path}: the command key '{key}' is not part of the profile schema")
        display = content.get("display", {})
        for key in display:
            if key != "hidden":
                failures.append(f"{path}: the display key '{key}' is not part of the profile schema")
    return failures


def check_comments(files: list[Path]) -> list[str]:
    """Return the failure messages for keys and sections without a comment.

    WHAT: walks the raw lines of the file. Every table header and every key
    assignment must be preceded by a comment line, because the file is the
    first contact point of the user.
    """
    failures: list[str] = []
    for path in files:
        previous = ""
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                previous = stripped
                continue
            if TABLE_PATTERN.match(line) or KEY_PATTERN.match(line):
                if not previous.startswith("#"):
                    kind = "section" if TABLE_PATTERN.match(line) else "key"
                    failures.append(f"{path}:{number}: the {kind} has no comment above it")
            previous = stripped
    return failures


def check_values(parsed: dict[Path, dict]) -> list[str]:
    """Return the failure messages for values that the shipped profiles must hold."""
    failures: list[str] = []
    for name, expected in EXPECTED.items():
        path = next((item for item in parsed if item.stem == name), None)
        if path is None:
            failures.append(f"the profile '{name}' does not exist")
            continue
        content = parsed[path]
        if content.get("name") != expected["name"]:
            failures.append(f"{path}: the name is '{content.get('name')}' instead of '{expected['name']}'")
        if not content.get("description"):
            failures.append(f"{path}: the description is empty")
        panels = content.get("panels", {})
        for key, value in expected["panels"].items():
            if panels.get(key) != value:
                failures.append(
                    f"{path}: '{key}' is '{panels.get(key)}' instead of '{value}'"
                )
        commands = content.get("commands", {})
        visibility = commands.get("visibility")
        if visibility != expected["visibility"]:
            failures.append(
                f"{path}: the visibility of the commands is '{visibility}' instead of "
                f"'{expected['visibility']}'"
            )
        if visibility not in VISIBILITIES:
            failures.append(
                f"{path}: the visibility '{visibility}' is not one of {VISIBILITIES}"
            )
        hidden = content.get("display", {}).get("hidden")
        if hidden is None:
            failures.append(f"{path}: the list of the hidden commands is missing")
        elif not isinstance(hidden, list):
            failures.append(f"{path}: the list of the hidden commands is not a list")
    return failures


def main() -> int:
    """Run every check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    root = repository_root()
    parser.add_argument("--directory", type=Path, default=root / "config" / "profiles")
    arguments = parser.parse_args()

    files = sorted(arguments.directory.glob("*.toml"))
    parse_failures, parsed = check_parses(files)

    checks: list[tuple[str, list[str]]] = [
        (f"all {len(files)} profiles parse as TOML", parse_failures),
        ("every profile carries exactly the keys of the schema", check_keys(parsed)),
        ("every key and every section carries a comment", check_comments(files)),
        ("the shipped profiles hold the values of their names", check_values(parsed)),
    ]

    print(f"profiles  : {arguments.directory}")
    for path in files:
        print(f"            {path.relative_to(root)}")
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
