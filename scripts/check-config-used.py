#!/usr/bin/env python3
"""Finds the settings that nobody reads.

WHAT: reads every key of `config/defaults.toml` and looks for it in the
      source of the workspace.
WHY:  a setting that nobody reads is worse than a missing setting. The
      user opens the configuration file, writes the value, and nothing
      happens — and the value looks as if it worked. The file is the
      promise that the program takes the user seriously, and a promise
      that is not kept is worse than no promise.

      The search is a search over the text of the source, and the text of
      the source is not always the name of the field: the configuration
      crate reads the file into a structure whose fields carry the names
      of the file. A finding is therefore a *question*, and every question
      is read by hand before the file is called wrong.

EXAMPLE:
      python3 scripts/check-config-used.py
      python3 scripts/check-config-used.py --verbose

Exit code 0 means "every setting of the file is read somewhere", 1 means
"a setting of the file is read nowhere".
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULTS = REPO_ROOT / "config" / "defaults.toml"

# The directories whose Rust code belongs to the project.
QUELLEN = ("crates", "apps", "platform")

# The key of a setting with its value.
EINSTELLUNG = re.compile(r"^([a-z_][a-z0-9_]*)\s*=\s*(.+)$", re.MULTILINE)

# The section of a setting.
ABSCHNITT = re.compile(r"^\[([a-z_][a-z0-9_]*)\]$", re.MULTILINE)


def sammle_einstellungen(text: str) -> list[tuple[str, str]]:
    """Return the sections and the keys of the configuration file."""
    abschnitt = ""
    einstellungen: list[tuple[str, str]] = []
    for zeile in text.split("\n"):
        abschnitt_treffer = ABSCHNITT.match(zeile.strip())
        if abschnitt_treffer:
            abschnitt = abschnitt_treffer.group(1)
            continue
        treffer = EINSTELLUNG.match(zeile.strip())
        if treffer:
            einstellungen.append((abschnitt, treffer.group(1)))
    return einstellungen


def main() -> int:
    """Run the check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="name every setting that was read",
    )
    arguments = parser.parse_args()

    text = DEFAULTS.read_text(encoding="utf-8")
    einstellungen = sammle_einstellungen(text)

    quelle = "\n".join(
        datei.read_text(encoding="utf-8")
        for wurzel in QUELLEN
        for datei in sorted((REPO_ROOT / wurzel).rglob("*.rs"))
    )

    fragen: list[str] = []
    gelesen = 0
    for abschnitt, schluessel in einstellungen:
        # The name of the field in the structure of the configuration crate
        # is the name of the key, and the section carries the rest.
        # WHY: a name in a comment is a name that nobody reads. The
        #       search looks at the code and not at the prose around it.
        gefunden = False
        for zeile in quelle.split("\n"):
            zeile = re.sub(r'"[^"]*"', "", zeile.split("//")[0])
            if re.search(rf"\b{re.escape(schluessel)}\b", zeile):
                gefunden = True
                break
        if gefunden:
            gelesen += 1
        else:
            fragen.append(
                f"  FRAGE: the setting `{schluessel}` of the section "
                f"`{abschnitt}` is read nowhere in the workspace"
            )
        if arguments.verbose:
            print(f"  {abschnitt}.{schluessel} — {gefunden}")

    print(f"Settings read       : {len(einstellungen)}")
    print(f"Settings found      : {gelesen}")
    print(f"Settings read nowhere: {len(fragen)}")

    if fragen:
        for frage in fragen:
            print(frage)
        print(
            "RESULT: FAIL — a setting of the configuration file is read nowhere.",
            file=sys.stderr,
        )
        return 1

    print("RESULT: PASS — every setting of the configuration file is read.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
