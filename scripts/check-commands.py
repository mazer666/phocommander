#!/usr/bin/env python3
"""Prüft, dass kein Befehl der Oberfläche ins Leere läuft.

WHAT: reads the commands of the surface and checks that every command that
      is produced is also carried out, and that every action of the
      catalogue that is not carried out is on the list of the ones that
      belong to a later milestone.
WHY:  a command that is produced and never read is a dead way. The user
      presses a key, the surface does its work, and nothing happens — and
      the tests of the parts do not see it, because every part is correct
      on its own. This check found the gaps of the surface: the commands
      that were produced and never read, and the actions that were
      reported as "deferred" although their milestone had arrived.

EXAMPLE:
      python3 scripts/check-commands.py
      python3 scripts/check-commands.py --verbose

Exit code 0 means "every command of the surface leads somewhere", 1 means
"a command leads nowhere" — every one of them is named.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# The file that names the commands.
COMMANDS = REPO_ROOT / "crates" / "phocommander-ui" / "src" / "commands.rs"

# The places where the surface produces a command.
PRODUCERS = (
    REPO_ROOT / "crates" / "phocommander-ui" / "src" / "app.rs",
    REPO_ROOT / "crates" / "phocommander-ui" / "src" / "app",
)

# The places that carry the commands out.
#
# WHY: most of the commands become jobs of the waiting room, and the
#       adapter carries them out. The command that leaves the application
#       belongs to the frame and not to the file system, and the frame
#       carries it out. Both places count — a command that nobody carries
#       out is a command that leads nowhere.
CONSUMERS = (
    REPO_ROOT / "apps" / "phocommander-desktop" / "src" / "execute.rs",
    REPO_ROOT / "apps" / "phocommander-desktop" / "src" / "desktop.rs",
)
CONSUMER = CONSUMERS[0]

# The actions that belong to a later milestone.
#
# WHY: the catalogue of the specification holds more actions than this
#       milestone carries out. The surface reports them as deferred instead
#       of pretending that they happened — and this list is the place where
#       that promise is kept. A name that is not on the list has to be
#       carried out.
DEFERRED_ACTIONS = {
    # The search in the viewer belongs to milestone M2, with the text of the
    # PDF and the navigation between the pages (FR-56).
    "viewer.find_next",
    "viewer.find_previous",
    # The menu of the application is a native menu of the system, and it
    # belongs to the milestone of the platform integration (FR-28).
    "app.menu",
}

# The name of a variant of the command.
#
# WHY: a variant may carry a field or carry nothing. The pattern has to
#       know both — the variant that carries nothing is written with a
#       comma, and a pattern that only knows the brace saw twelve of the
#       fifteen commands.
VARIANT = re.compile(r"^    ([A-Z][A-Za-z]*)\s*(?:\{|,|$)", re.MULTILINE)

# The use of a variant.
USE = re.compile(r"Command::([A-Z][A-Za-z]*)")

# The handling of an action.
HANDLING = re.compile(r'"([a-z][a-z0-9_.]*)" =>')


def read_commands() -> list[str]:
    """Return the names of the variants of the command.

    WHY: the block of the enumeration ends at the closing brace in the
          first column, and not at the first brace after the opening one.
          A variant that carries a field ends with a brace in the column of
          the variant, and the search for that brace cut the enumeration
          after the twelfth variant — the commands behind it were never
          seen by this gate.
    """
    text = COMMANDS.read_text(encoding="utf-8")
    start = text.index("pub enum Command {")
    ende = text.index("\n}\n", start)
    # Das Ende der Aufzaehlung ist die schliessende Klammer in der ersten
    # Spalte. Die Klammern werden gezaehlt, damit ein Feld des Befehls den
    # Block nicht verkuerzt.
    tiefe = 0
    for index in range(start, len(text)):
        if text[index] == "{":
            tiefe += 1
        elif text[index] == "}":
            tiefe -= 1
            if tiefe == 0:
                ende = index
                break
    return VARIANT.findall(text[start:ende])


def read_uses(paths: tuple[Path, ...], nur_produktion: bool = False) -> set[str]:
    """Return the names of the variants that the given files use.

    WHY: the tests of the adapter build the commands as well, and they
          count as a use. A command that is only built in a test is a
          command that nobody carries out.
    """
    gefunden: set[str] = set()
    for path in paths:
        dateien = path.rglob("*.rs") if path.is_dir() else [path]
        for datei in dateien:
            text = datei.read_text(encoding="utf-8")
            if nur_produktion and "#[cfg(test)]" in text:
                text = text[: text.index("#[cfg(test)]")]
            gefunden |= set(USE.findall(text))
    return gefunden


def read_handled_actions() -> set[str]:
    """Return the names of the actions that the surface carries out.

    WHY: the actions are handled in two places — in the shell itself and
          in the dialog of the multiple rename, which has the focus while it
          is open. Both places count.
    """
    gefunden: set[str] = set()
    for datei in (REPO_ROOT / "crates" / "phocommander-ui" / "src" / "app.rs").parent.rglob("*.rs"):
        gefunden |= set(HANDLING.findall(datei.read_text(encoding="utf-8")))
    return gefunden


def read_grouped_actions() -> set[str]:
    """Return the actions that the shell carries out as a group.

    WHY: the bindings of the tabs carry a number in their name, and the
          shell handles all of them in one arm — otherwise the catalogue
          would have to grow with every tab. The arm counts as a handling
          of every action whose name follows the pattern.
    """
    text = (REPO_ROOT / "crates" / "phocommander-ui" / "src" / "app.rs").read_text(
        encoding="utf-8"
    )
    gefunden: set[str] = set()
    if 'strip_prefix("panel.tab.")' in text:
        for nummer in range(1, 10):
            gefunden.add(f"panel.tab.{nummer}")
    return gefunden


def main() -> int:
    """Run the check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="name every command of the surface",
    )
    arguments = parser.parse_args()

    varianten = read_commands()
    erzeugt = read_uses(PRODUCERS)
    bearbeitet = read_uses(CONSUMERS, nur_produktion=True)
    ausgefuehrt = read_handled_actions() | read_grouped_actions()

    failures: list[str] = []

    for name in varianten:
        if name in erzeugt and name not in bearbeitet:
            failures.append(
                f"the command {name} is produced and never carried out — "
                "the user presses a key and nothing happens"
            )

    # The consumer carries out every command in one arm of its match. Two
    # arms for the same command mean that the second one is unreachable —
    # and the unreachable arm is the one that was written last.
    for name in varianten:
        anzahl = 0
        for consumer in CONSUMERS:
            text = consumer.read_text(encoding="utf-8")
            # WHY: the tests of the adapter build the commands as well, and
            #       they count twice. Only the arms of the match are of
            #       interest.
            if "#[cfg(test)]" in text:
                text = text[: text.index("#[cfg(test)]")]
            anzahl += len(re.findall(rf"Command::{name}\b", text))
        if anzahl > 1:
            failures.append(
                f"the command {name} is carried out {anzahl} times — one of "
                "the arms is unreachable"
            )

    # The actions of the catalogue: every one of them has to be carried out
    # or to be named in the list of the later milestones.
    aktionen = (
        REPO_ROOT / "crates" / "phocommander-ui" / "src" / "actions.rs"
    ).read_text(encoding="utf-8")
    alle = re.findall(r'id: "([a-z][a-z0-9_.]*)"', aktionen)
    for name in alle:
        if name not in ausgefuehrt and name not in DEFERRED_ACTIONS:
            failures.append(
                f"the action {name} is neither carried out nor named as a "
                "later milestone"
            )

    if arguments.verbose:
        for name in varianten:
            zustand = "erzeugt und bearbeitet" if name in bearbeitet else "erzeugt"
            print(f"  {name:24} {zustand}")

    print(f"Commands of the surface : {len(varianten)}")
    print(f"Produced               : {len(erzeugt & set(varianten))}")
    print(f"Carried out            : {len(bearbeitet & set(varianten))}")
    print(f"Actions of the catalogue: {len(alle)}")
    print(f"Deferred by their milestone: {len(DEFERRED_ACTIONS)}")

    if failures:
        for zeile in failures:
            print(f"  FEHLER: {zeile}")
        print(
            "RESULT: FAIL — a command of the surface leads nowhere.",
            file=sys.stderr,
        )
        return 1

    print("RESULT: PASS — every command of the surface leads somewhere.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
