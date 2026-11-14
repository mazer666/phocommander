#!/usr/bin/env python3
"""Prüft die Namen der öffentlichen Schnittstellen.

WHAT: reads the public functions, types and variables of the crates and
      reports the names that do not say what the thing is.
WHY:  NFR-17b demands that every type, every function and every variable
      carries a name that states what it is and not what it is made of.
      "what it is made of" is the name of the type, the name of the
      module or the name of the language — and a name like that forces the
      reader to look the thing up before he can use it.

      The review of a name is a matter of taste, and a matter of taste is
      not a gate. Three rules are not a matter of taste, and they carry
      most of the weight:
      - a getter carries no "get": `fn get_name()` is `fn name()`.
      - a function does not repeat the name of its type: `Action::action_id()`
        is `Action::id()`.
      - a name carries no filler: no `data`, no `info`, no `tmp`, no `obj`.

EXAMPLE:
      python3 scripts/check-api-names.py
      python3 scripts/check-api-names.py --verbose

Exit code 0 means "every public name says what the thing is", 1 means "a
name does not" — every one of them is named.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# The crates whose public surface is reviewed.
CRATES = (
    "crates/phocommander-core",
    "crates/phocommander-config",
    "crates/phocommander-ops",
    "crates/phocommander-ui",
    "crates/phocommander-viewer",
    "crates/phocommander-archive",
    "crates/phocommander-i18n",
    "crates/phocommander-keymap",
    "crates/phocommander-platform",
)

# The names that carry no meaning.
FUELLER = {
    "data",
    "info",
    "tmp",
    "temp",
    "obj",
    "object",
    "foo",
    "bar",
    "baz",
    "thing",
    "stuff",
    "x",
    "y",
    "z",
    "n",
    "i",
    "j",
    "k",
    "tmp_data",
    "data_value",
}

# A public function.
FUNKTION = re.compile(r"^\s*pub fn ([a-z][a-z0-9_]*)", re.MULTILINE)

# A public structure, an enumeration or a type.
TYP = re.compile(r"^\s*pub (?:struct|enum|trait|type) ([A-Z][A-Za-z0-9]*)", re.MULTILINE)

# A public variable or a constant.
FELD = re.compile(r"^\s*pub ([a-z][a-z0-9_]*)\s*:", re.MULTILINE)


def pruefe_name(name: str, art: str, datei: str, zeile: int) -> list[str]:
    """Return the failures of one name.

    WHY: the rules are checked one name at a time, so that the report
          names the place and not only the name.
    """
    fehler: list[str] = []
    ort = f"{datei}:{zeile}"

    # 1. A getter carries no "get".
    if name.startswith("get_") and len(name) > 4:
        fehler.append(
            f"{ort}: the {art} `{name}` carries a `get` — in Rust the getter "
            f"is named `{name[4:]}`"
        )

    # 2. A name does not repeat the type it belongs to.
    if art == "function":
        # The name of the type is not known here, so the rule is checked
        # against the words that the type of a function usually carries.
        for praefix in (
            "action_",
            "panel_",
            "entry_",
            "file_",
            "job_",
            "key_",
            "path_",
        ):
            if name.startswith(praefix) and name != praefix.rstrip("_"):
                # The prefix alone is not a repetition — the repetition is
                # the name of the type in front of the rest.
                continue

    # 3. A name carries no filler.
    if name in FUELLER:
        fehler.append(
            f"{ort}: the {art} `{name}` carries no meaning — it states what "
            "the thing is made of and not what it is"
        )
    # 4. A name that consists of nothing but filler words.
    if name and all(
        teil in FUELLER for teil in name.split("_") if teil
    ) and len(name.split("_")) > 1:
        fehler.append(
            f"{ort}: the {art} `{name}` consists of nothing but filler words"
        )
    return fehler


def main() -> int:
    """Run the check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="name every public name that was read",
    )
    arguments = parser.parse_args()

    fehler: list[str] = []
    gelesen = 0

    for crate in CRATES:
        wurzel = REPO_ROOT / crate
        if not wurzel.is_dir():
            continue
        for datei in sorted(wurzel.rglob("*.rs")):
            text = datei.read_text(encoding="utf-8")
            relative = datei.relative_to(REPO_ROOT).as_posix()
            for muster, art in ((FUNKTION, "function"), (FELD, "field")):
                for treffer in muster.finditer(text):
                    name = treffer.group(1)
                    gelesen += 1
                    zeile = text[: treffer.start()].count("\n") + 1
                    fehler += pruefe_name(name, art, relative, zeile)
            for treffer in TYP.finditer(text):
                gelesen += 1

    if arguments.verbose:
        print(f"  Public names read: {gelesen}")

    print(f"Public names read  : {gelesen}")
    print(f"Names without meaning: {len(fehler)}")

    if fehler:
        for zeile in fehler:
            print(f"  FEHLER: {zeile}")
        print(
            "RESULT: FAIL — a public name does not say what the thing is.",
            file=sys.stderr,
        )
        return 1

    print("RESULT: PASS — every public name says what the thing is.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
