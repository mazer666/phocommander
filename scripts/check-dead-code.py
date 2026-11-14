#!/usr/bin/env python3
"""Finds the public functions that nobody calls.

WHAT: reads every public function of the workspace and counts how often
      its name occurs in the whole repository.
WHY:  a function that nobody calls is a promise that the project does
      not keep. Two defects of this kind were found by hand in the
      project — the command that the surface produced and nobody read,
      and the hex dump that was built and then dropped at the last moment
      before the user sees it. A gate that finds the third one is worth
      more than a good intention.

      The count is not a proof that a function is dead: a name can be
      used in a macro, and the name of a function can be used by a name
      of its own. Every finding is therefore reported as a *question* and
      not as a verdict, and the script never fails the build on its own.

EXAMPLE:
      python3 scripts/check-dead-code.py
      python3 scripts/check-dead-code.py --verbose

Exit code 0 means "no question is open", 1 means "at least one function
of the workspace is never called".
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# The directories whose Rust code belongs to the project.
QUELLEN = ("crates", "apps", "platform")

# The directories whose code is not counted as a caller.
AUSGENOMMEN = ("target", "fuzz/target", ".git")

# A public function with a name.
FUNKTION = re.compile(r"^\s*pub (?:async )?fn ([a-z][a-z0-9_]*)", re.MULTILINE)

# A public function in a trait, in an implementation and in a test.
TEST_MODUL = re.compile(r"#\[cfg\(test\)\]")


def sammle_funktionen(text: str) -> list[tuple[str, int]]:
    """Return the public functions of a file with their line."""
    funktionen: list[tuple[str, int]] = []
    for treffer in FUNKTION.finditer(text):
        zeile = text[: treffer.start()].count("\n") + 1
        funktionen.append((treffer.group(1), zeile))
    return funktionen


def ist_testzeile(text: str, zeile: int) -> bool:
    """Say whether the line lies in a test module."""
    zeilen = text.split("\n")
    for index in range(min(zeile, len(zeilen)) - 1, -1, -1):
        if TEST_MODUL.search(zeilen[index]):
            # The module of the tests begins here; the search ends, because
            # a module of the tests is always the last one of a file.
            return True
        if zeilen[index].startswith("mod ") and "tests" in zeilen[index]:
            return True
    return False


def main() -> int:
    """Run the check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="name every function that was read",
    )
    arguments = parser.parse_args()

    dateien: list[Path] = []
    for quelle in QUELLEN:
        wurzel = REPO_ROOT / quelle
        if wurzel.is_dir():
            dateien += sorted(wurzel.rglob("*.rs"))

    texte: dict[Path, str] = {}
    for datei in dateien:
        texte[datei] = datei.read_text(encoding="utf-8")

    # The whole text of the repository, for the count.
    alles = "\n".join(texte.values())

    funktionen: dict[str, list[str]] = {}
    for datei, text in texte.items():
        for name, zeile in sammle_funktionen(text):
            if ist_testzeile(text, zeile):
                continue
            funktionen.setdefault(name, []).append(
                datei.relative_to(REPO_ROOT).as_posix()
            )

    fragen: list[str] = []
    for name, orte in sorted(funktionen.items()):
        # The name must occur once more than in its own definition.
        vorkommen = len(re.findall(rf"\b{re.escape(name)}\b", alles))
        if vorkommen <= len(orte):
            fragen.append(
                f"  FRAGE: the function `{name}` is defined in "
                f"{', '.join(orte)} and is called nowhere in the workspace"
            )

    if arguments.verbose:
        for name, orte in sorted(funktionen.items()):
            print(f"  {name} — {', '.join(orte)}")

    print(f"Public functions read : {len(funktionen)}")
    print(f"Functions never called: {len(fragen)}")

    if fragen:
        for frage in fragen:
            print(frage)
        print(
            "RESULT: FAIL — a function of the workspace is called nowhere.",
            file=sys.stderr,
        )
        return 1

    print("RESULT: PASS — every function of the workspace is called somewhere.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
