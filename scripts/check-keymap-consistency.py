#!/usr/bin/env python3
"""Compare the key binding reference with the catalogue and with the specification.

WHAT: this script checks that the three places that describe the key bindings agree:
      the table in ``docs/REQUIREMENTS.md`` (section "Tastatursteuerung"), the
      action catalogue in ``config/keymap.toml`` (section ``[input.keyboard]``)
      and the reference table in ``docs/KEYMAP.md``.
WHY: a binding that is documented in one place and implemented in another is a
      defect the reader only discovers when a key does not do what the manual
      promised. The specification asks for a documented binding for every action
      ("Die Datei docs/KEYMAP.md dokumentiert jede Aktion mit Vorgabe, Kontext
      und Plattformvariante").
EXAMPLE:
      python3 scripts/check-keymap-consistency.py
      python3 scripts/check-keymap-consistency.py --verbose

Exit code 0 means "the three descriptions agree, or every deviation is the one
that is documented in the repository". Exit code 1 means "there is an
undocumented deviation" — each one is printed with the values that disagree.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REQUIREMENTS = REPO_ROOT / "docs" / "REQUIREMENTS.md"
KEYMAP_TOML = REPO_ROOT / "config" / "keymap.toml"
KEYMAP_DOC = REPO_ROOT / "docs" / "KEYMAP.md"

CODE_SPAN = re.compile(r"`([^`]+)`")
RANGE_SEPARATOR = ("…", "–", "—", "-")

# The action identifier behind every key of the specification's keyboard table.
# The mapping follows the wording of that table; the source section is
# "Tastatursteuerung" in docs/REQUIREMENTS.md.
SPEC_KEY_TO_ACTIONS: dict[str, list[str]] = {
    "F2": ["panel.rename"],
    "F3": ["panel.view"],
    "F4": ["panel.edit"],
    "F5": ["panel.copy"],
    "F6": ["panel.move"],
    "F7": ["panel.mkdir"],
    "F8": ["panel.delete"],
    "F9": ["app.menu"],
    "F10": ["app.quit"],
    "Tab": ["panel.switch"],
    "Ctrl+R": ["panel.reload"],
    "Ctrl+U": ["panel.swap"],
    "Ctrl+Enter": ["panel.open_in_other"],
    "Ctrl+Tab": ["panel.next_tab"],
    "Ins": ["selection.toggle"],
    "Ctrl+A": ["selection.all"],
    "Ctrl+Num-": ["selection.none"],
    "Ctrl+Num*": ["selection.invert"],
    "Space": ["selection.by_pattern"],
    "Alt+F1": ["panel.drives"],
    "Alt+F2": ["panel.drives"],
    "Ctrl+PgUp": ["panel.parent"],
    **{f"Ctrl+{number}": [f"panel.tab.{number}"] for number in range(1, 10)},
}

# The specification writes some keys in German. The catalogue uses the physical key
# name. This table maps one to the other; it is a notation difference, not a
# deviation.
SPEC_KEY_ALIASES: dict[str, str] = {
    "Leertaste": "Space",
}

# The two bindings that the specification names in the prose below its table.
SPEC_PROSE_ACTIONS: dict[str, list[str]] = {
    "F3": ["viewer.find_next", "rename.conflict_next"],
    "Shift+F3": ["viewer.find_previous", "rename.conflict_previous"],
}

# Deviations that are known, documented in the repository and therefore accepted.
# The comment names the place where the deviation is written down.
ACCEPTED_DEVIATIONS: dict[str, str] = {
    "Alt+F2": (
        "config/keymap.toml binds panel.drives to Alt+F1 and documents Alt+F2 in the "
        "comment as the classic variant for the other pane; the specification names "
        "both keys for one action"
    ),
    "panel.prev_tab": (
        "config/keymap.toml and docs/KEYMAP.md mark Ctrl+Shift+Tab as the symmetric "
        "completion of Ctrl+Tab; the specification names only the forward direction"
    ),
}


def expand(cell: str) -> list[str]:
    """Return the code spans of a cell, expanding "a … b" ranges."""
    spans = CODE_SPAN.findall(cell)
    if len(spans) != 2:
        return spans
    if not any(separator in cell for separator in RANGE_SEPARATOR):
        return spans
    first, last = spans
    first_match = re.match(r"^(.*?)(\d+)$", first)
    last_match = re.match(r"^(.*?)(\d+)$", last)
    if not first_match or not last_match:
        return spans
    if first_match.group(1) != last_match.group(1):
        return spans
    width = len(first_match.group(2))
    start, end = int(first_match.group(2)), int(last_match.group(2))
    if end < start:
        return spans
    return [f"{first_match.group(1)}{number:0{width}d}" for number in range(start, end + 1)]


def read_spec_rows() -> list[tuple[list[str], str]]:
    """Return the (keys, action) rows of the specification's keyboard table."""
    text = REQUIREMENTS.read_text(encoding="utf-8")
    start = text.index("### Tastatursteuerung")
    end = text.index("\n### ", start + 10)
    section = text[start:end]
    rows: list[tuple[list[str], str]] = []
    for line in section.splitlines():
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) != 4:
            continue
        if set(cells[0]) <= set("-: "):
            continue
        for key_cell, action_cell in ((cells[0], cells[1]), (cells[2], cells[3])):
            keys = expand(key_cell)
            if keys:
                rows.append((keys, action_cell))
    return rows


def read_catalogue() -> dict[str, str]:
    """Return the action id -> binding map of the keyboard context."""
    text = KEYMAP_TOML.read_text(encoding="utf-8")
    start = text.index("[input.keyboard]")
    end = text.index("\n[", start + 1)
    section = text[start:end]
    catalogue: dict[str, str] = {}
    for line in section.splitlines():
        match = re.match(r'^\s*"([^"]+)"\s*=\s*"([^"]+)"', line)
        if match:
            catalogue[match.group(1)] = match.group(2)
    return catalogue


def read_keymap_doc() -> dict[str, list[str]]:
    """Return the action id -> documented default keys map of the reference table."""
    text = KEYMAP_DOC.read_text(encoding="utf-8")
    documented: dict[str, list[str]] = {}
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) != 5:
            continue
        action_ids = expand(cells[0])
        defaults = expand(cells[1])
        if not action_ids or not defaults:
            continue
        if len(action_ids) == len(defaults):
            for action_id, default in zip(action_ids, defaults):
                documented[action_id] = [default]
        else:
            for action_id in action_ids:
                documented[action_id] = defaults
    return documented


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verbose", "-v", action="store_true", help="print every row, not only deviations")
    args = parser.parse_args()

    spec_rows = read_spec_rows()
    catalogue = read_catalogue()
    documented = read_keymap_doc()

    deviations: list[str] = []
    accepted: list[str] = []

    print("=== A. Specification table -> action catalogue (config/keymap.toml) ===")
    for keys, action in spec_rows:
        for key in keys:
            canonical = SPEC_KEY_ALIASES.get(key, key)
            action_ids = SPEC_KEY_TO_ACTIONS.get(canonical)
            if action_ids is None:
                deviations.append(f"the specification row '{key} {action}' has no action id")
                print(f"  {key:<14} {action:<34} -> NO ACTION ID")
                continue
            for action_id in action_ids:
                binding = catalogue.get(action_id)
                if binding == canonical:
                    print(f"  {key:<14} {action:<34} -> {action_id:<24} {binding:<14} ok")
                    continue
                if key in ACCEPTED_DEVIATIONS:
                    accepted.append(f"{key} ({action_id}): {ACCEPTED_DEVIATIONS[key]}")
                    print(f"  {key:<14} {action:<34} -> {action_id:<24} {binding:<14} ACCEPTED")
                    continue
                deviations.append(
                    f"the catalogue binds {action_id} to '{binding}', the specification binds it to '{key}'"
                )
                print(f"  {key:<14} {action:<34} -> {action_id:<24} {binding:<14} DEVIATION")

    print()
    print("=== B. Action catalogue -> reference table (docs/KEYMAP.md) ===")
    for action_id, binding in sorted(catalogue.items()):
        defaults = documented.get(action_id)
        if defaults is None:
            deviations.append(f"{action_id} is not documented in docs/KEYMAP.md")
            print(f"  {action_id:<26} catalogue={binding:<16} document=missing              DEVIATION")
        elif binding not in defaults:
            deviations.append(
                f"docs/KEYMAP.md documents {action_id} with {defaults}, the catalogue binds it to '{binding}'"
            )
            print(f"  {action_id:<26} catalogue={binding:<16} document={defaults}          DEVIATION")
        else:
            if args.verbose:
                print(f"  {action_id:<26} catalogue={binding:<16} document={defaults}          ok")

    print()
    print("=== C. Actions named in the prose of the specification ===")
    for key, action_ids in SPEC_PROSE_ACTIONS.items():
        for action_id in action_ids:
            binding = catalogue.get(action_id)
            state = "ok" if binding == key else "DEVIATION"
            if binding != key:
                deviations.append(
                    f"the catalogue binds {action_id} to '{binding}', the specification names '{key}'"
                )
            print(f"  {key:<14} {action_id:<26} {binding:<16} {state}")

    print()
    print("=== D. Actions in the catalogue that the specification names nowhere ===")
    named = {
        action_id
        for ids in list(SPEC_KEY_TO_ACTIONS.values()) + list(SPEC_PROSE_ACTIONS.values())
        for action_id in ids
    }
    for action_id in sorted(catalogue):
        if action_id in named:
            continue
        if action_id in ACCEPTED_DEVIATIONS:
            accepted.append(f"{action_id}: {ACCEPTED_DEVIATIONS[action_id]}")
            print(f"  {action_id:<26} catalogue={catalogue[action_id]:<16} ACCEPTED")
            continue
        deviations.append(
            f"{action_id} exists in the catalogue but is named neither in the table nor in the "
            "prose of the specification"
        )
        print(f"  {action_id:<26} catalogue={catalogue[action_id]:<16} DEVIATION")

    print()
    print(f"Rows in the specification table : {len(spec_rows)}")
    print(f"Actions in the catalogue          : {len(catalogue)}")
    print(f"Actions documented in KEYMAP.md   : {len(documented)}")
    print()
    print(f"Accepted deviations (documented in the repository): {len(accepted)}")
    for entry in accepted:
        print(f"  - {entry}")
    print(f"Undocumented deviations: {len(deviations)}")
    for deviation in deviations:
        print(f"  - {deviation}")
    if deviations:
        print("RESULT: FAIL — the three descriptions do not agree.")
        return 1
    print("RESULT: PASS — specification, catalogue and reference agree.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
