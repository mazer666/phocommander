#!/usr/bin/env python3
"""Erzeugt die Standardbindungen des Codes aus der Konfigurationsdatei.

WHAT: liest `config/keymap.toml` und schreibt
      `crates/phocommander-ui/src/keymap_view/defaults.rs`.
WHY: die Bindungen der Kontexte stehen in der Datei, die der Benutzer
      bearbeitet. Damit die Anwendung auch ohne diese Datei bedienbar bleibt,
      trägt der Code dieselben Bindungen als Vorgabe. Die Datei überschreibt
      sie danach.
      Zwei Quellen an derselben Sache driften auseinander — deshalb prüft
      `scripts/check-keymap-consistency.py`, dass die erzeugte Datei zur
      Konfigurationsdatei passt, und dieses Skript erzeugt sie neu.

BEISPIEL:
      python3 scripts/generate-keymap-defaults.py
      python3 scripts/generate-keymap-defaults.py --check

Rückgabewert 0 means "the file is written or it is up to date", 1 means
"the file is out of date and --check was given".
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
KEYMAP_TOML = REPO_ROOT / "config" / "keymap.toml"
TARGET = REPO_ROOT / "crates" / "phocommander-ui" / "src" / "keymap_view" / "defaults.rs"

# Die Reihenfolge der Kontexte. Sie ist die Reihenfolge, in der die
# Vorgaben im Code stehen.
CONTEXTS = ("keyboard", "pointer", "touch", "dpad", "controller")

# Der Kontext des Codes, in den die Konfiguration gehört.
RUST_TYPE = {
    "keyboard": "Context::Keyboard",
    "pointer": "Context::Pointer",
    "touch": "Context::Touch",
    "dpad": "Context::Dpad",
    "controller": "Context::Controller",
}

# Die Vorgabe, die der Kontext fuer eine Aktion nennt, wenn die Datei
# keine nennt. Der D-Pad und der Controller haben keine Funktionstasten,
# und die Aktionspalette nimmt ihren Platz ein.
KEYMAP = re.compile(r'^\s*"([^"]+)"\s*=\s*"([^"]+)"', re.MULTILINE)


def read_context(name: str) -> list[tuple[str, str]]:
    """Return the action and binding pairs of one context.

    WHAT: reads the section of the context and returns every action of it
          with the binding that the file gives it.
    """
    text = KEYMAP_TOML.read_text(encoding="utf-8")
    start = text.index(f"[input.{name}]")
    end = text.find("\n[", start + 1)
    section = text[start:] if end < 0 else text[start:end]
    return KEYMAP.findall(section)


def rust_string(value: str) -> str:
    """Return the value as a literal of Rust."""
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"'


def render() -> str:
    """Return the content of the generated file.

    WHY: the file is generated and not written by hand, because a table of
          five contexts with forty-three actions each is too long to keep
          correct in two places.
    """
    lines = [
        "//! The default bindings of the contexts.",
        "//!",
        "//! WHAT: the binding of every action of the catalogue in every",
        "//!       context, in the form of the configuration file.",
        "//! WHY: the configuration file is the one that the user edits, and a",
        "//!       file that is missing must not leave the application",
        "//!       without any binding. The code therefore carries the same",
        "//!       bindings as a default, and the file overrides them.",
        "//!       `scripts/generate-keymap-defaults.py` writes this file from",
        "//!       `config/keymap.toml`, and",
        "//!       `scripts/check-keymap-consistency.py` proves that the two",
        "//!       agree.",
        "//!",
        "//! THIS FILE IS GENERATED — do not change it by hand.",
        "",
        "use std::collections::BTreeMap;",
        "",
        "use crate::context::Context;",
        "",
        "/// The default bindings of every context.",
        "///",
        "/// EXAMPLE:",
        "/// ```no_run",
        "/// use phocommander_ui::keymap_view::defaults::default_bindings;",
        "///",
        "/// let bindings = default_bindings();",
        "/// assert!(!bindings.is_empty());",
        "/// ```",
        "pub fn default_bindings() -> BTreeMap<Context, BTreeMap<String, String>> {",
        "    let mut alle: BTreeMap<Context, BTreeMap<String, String>> = BTreeMap::new();",
    ]

    for name in CONTEXTS:
        lines.append("")
        lines.append(f"    // {name}")
        lines.append("    {")
        lines.append("        let mut tabelle: BTreeMap<String, String> = BTreeMap::new();")
        for aktion, bindung in read_context(name):
            lines.append(
                f"        tabelle.insert({rust_string(aktion)}.to_string(), "
                f"{rust_string(bindung)}.to_string());"
            )
        lines.append(f"        alle.insert({RUST_TYPE[name]}, tabelle);")
        lines.append("    }")

    lines.append("    alle")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


# Die Zeile, die eine Bindung in der erzeugten Datei traegt. Sie enthaelt
# die Aktion und die Bindung.
# WHY: the formatter of the repository breaks a long line, and the line
#      therefore comes in two shapes. The pattern tolerates both.
ERZEUGT = re.compile(
    r'tabelle\s*\.insert\(\s*"([^"]+)"\.to_string\(\),\s*'
    r'"([^"]+)"\.to_string\(\),?\s*\);',
    re.DOTALL,
)


def read_generated(path: Path) -> list[tuple[str, str]]:
    """Return the bindings that the generated file carries.

    WHAT: reads the lines that the generator writes and returns the action
          and the binding of every one of them.
    WHY: the check must compare the *content* and not the bytes. The
          formatter of the repository rewrites the generated file, and a
          check of the bytes would then report a difference that is not
          one.
    """
    if not path.exists():
        return []
    text = path.read_text(encoding="utf-8")
    return ERZEUGT.findall(text)


def main() -> int:
    """Write the file or prove that it is up to date."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="do not write, but fail when the file is out of date",
    )
    args = parser.parse_args()

    inhalt = render()

    if args.check:
        if not TARGET.exists():
            print(f"FEHLER: {TARGET} fehlt.", file=sys.stderr)
            return 1
        erwartet = []
        for name in CONTEXTS:
            erwartet.extend(read_context(name))
        vorhanden = read_generated(TARGET)
        if vorhanden != erwartet:
            print(
                f"FEHLER: {TARGET} ist nicht auf dem Stand der Konfigurationsdatei. "
                "Bitte scripts/generate-keymap-defaults.py laufen lassen.",
                file=sys.stderr,
            )
            return 1
        print("OK    die Vorgaben des Codes passen zur Konfigurationsdatei")
        return 0

    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_text(inhalt, encoding="utf-8")
    print(f"OK    {TARGET.relative_to(REPO_ROOT)} geschrieben")
    return 0


if __name__ == "__main__":
    sys.exit(main())
