#!/usr/bin/env python3
"""Compare the key binding reference with the catalogue and with the specification.

WHAT: this script checks that the three places that describe the key bindings agree:
      the action catalogue in ``config/keymap.toml`` (section ``[input.keyboard]``),
      the reference table in ``docs/KEYMAP.md`` and the statements of the
      specification in ``docs/REQUIREMENTS.md``.
WHY: a binding that is documented in one place and implemented in another is a
      defect the reader only discovers when a key does not do what the manual
      promised. Since version 2.0 of the specification the binding table is no
      longer part of the specification: the specification names the catalogue in
      section 6.1 and the classic key set in section 6.3 and in FR-35, and
      ``docs/KEYMAP.md`` is the reference that documents every action.
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

# The classic key set that the specification names in section 6.3: "F2 to F10
# carry the main operations", the Norton Commander tradition.
CLASSIC_BINDINGS: dict[str, str] = {
    "F2": "panel.rename",
    "F3": "panel.view",
    "F4": "panel.edit",
    "F5": "panel.copy",
    "F6": "panel.move",
    "F7": "panel.mkdir",
    "F8": "panel.delete",
    "F9": "app.menu",
    "F10": "app.quit",
}

# The bindings that the specification names for the conflict jump list in FR-35
# and for the search in the viewer.
PROSE_BINDINGS: dict[str, list[str]] = {
    "F3": ["viewer.find_next", "rename.conflict_next"],
    "Shift+F3": ["viewer.find_previous", "rename.conflict_previous"],
}

# The bindings of the keyboard context that the specification does not name.
# They are the completion of the classic set, and they are documented in the
# catalogue itself.
EXTRA_BINDINGS: dict[str, str] = {
    "panel.switch": "Tab",
    "panel.reload": "Ctrl+R",
    "panel.swap": "Ctrl+U",
    "panel.open_in_other": "Ctrl+Enter",
    "panel.next_tab": "Ctrl+Tab",
    "panel.filter": "Ctrl+F",
    "selection.toggle": "Ins",
    "selection.all": "Ctrl+A",
    "selection.none": "Ctrl+Num-",
    "selection.invert": "Ctrl+Num*",
    "selection.by_pattern": "Space",
    "panel.drives": "Alt+F1",
    "panel.parent": "Ctrl+PgUp",
    **{f"panel.tab.{number}": f"Ctrl+{number}" for number in range(1, 10)},
}

# The actions that belong to the surface and not to the catalogue of the
# operations. The action palette is opened by a key of the remote control and
# of the controller, but it is not an operation of the file manager.
SURFACE_ACTIONS = ("palette.open",)

# Deviations that are known, documented in the repository and therefore accepted.
ACCEPTED_DEVIATIONS: dict[str, str] = {
    "panel.prev_tab": (
        "config/keymap.toml and docs/KEYMAP.md mark Ctrl+Shift+Tab as the symmetric "
        "completion of Ctrl+Tab; the specification names only the forward direction"
    ),
}

# The notes that the catalogue carries about its own bindings. They are printed
# with the result, so that a reader sees them without opening the file.
DOCUMENTED_NOTES = (
    "panel.drives is bound to Alt+F1; config/keymap.toml documents Alt+F2 in the comment as the "
    "classic variant for the drive list of the other panel",
    "the action palette is opened with the Menu key, and with the Play/Pause key on a remote "
    "control that has no Menu key",
)


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


def read_catalogue() -> dict[str, str]:
    """Return the action id -> binding map of the keyboard context."""
    return read_context("input.keyboard")


def read_context(name: str) -> dict[str, str]:
    """Return the action id -> binding map of one context of the catalogue.

    WHAT: reads the section of the context and returns every action of it. The
    last section of the file ends at the end of the file, so the end of the
    section is optional.
    """
    text = KEYMAP_TOML.read_text(encoding="utf-8")
    start = text.index(f"[{name}]")
    end = text.find("\n[", start + 1)
    section = text[start:] if end < 0 else text[start:end]
    bindings: dict[str, str] = {}
    for line in section.splitlines():
        match = re.match(r'^\s*"([^"]+)"\s*=\s*"([^"]+)"', line)
        if match:
            bindings[match.group(1)] = match.group(2)
    return bindings


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


def check_specification_points_at_the_catalogue() -> list[str]:
    """Return the failure messages when the specification does not name the sources.

    WHAT: section 6.1 of the specification states that the action catalogue
    lives in ``config/keymap.toml`` and that it is documented in
    ``docs/KEYMAP.md``. The check makes sure that both statements are still
    true, so that the reference cannot point at a document that does not
    exist.
    """
    text = REQUIREMENTS.read_text(encoding="utf-8")
    failures: list[str] = []
    if "### 6.1 The action model" not in text:
        failures.append("the specification has no section '6.1 The action model'")
    for name in ("config/keymap.toml", "docs/KEYMAP.md"):
        if name not in text:
            failures.append(f"the specification does not mention {name}")
    return failures


def check_classic_bindings(catalogue: dict[str, str]) -> tuple[list[str], list[str]]:
    """Return the failures and the accepted deviations of the classic key set.

    WHAT: F2 to F10 carry the main operations, the tradition that the
    specification names in section 6.3, and FR-35 names F3 and Shift+F3 for
    the conflict jump list.
    """
    failures: list[str] = []
    accepted: list[str] = []
    for key, action_id in CLASSIC_BINDINGS.items():
        binding = catalogue.get(action_id)
        if binding != key:
            failures.append(
                f"the catalogue binds {action_id} to '{binding}', the specification names '{key}'"
            )
    for key, action_ids in PROSE_BINDINGS.items():
        for action_id in action_ids:
            binding = catalogue.get(action_id)
            if binding != key:
                failures.append(
                    f"the catalogue binds {action_id} to '{binding}', the specification names "
                    f"'{key}'"
                )
    for action_id, binding in EXTRA_BINDINGS.items():
        if catalogue.get(action_id) != binding:
            failures.append(
                f"the catalogue binds {action_id} to '{catalogue.get(action_id)}', the reference "
                f"names '{binding}'"
            )
    return failures, accepted


def check_reference_document(catalogue: dict[str, str]) -> list[str]:
    """Return the failure messages for actions that the reference does not hold."""
    documented = read_keymap_doc()
    failures: list[str] = []
    for action_id, binding in sorted(catalogue.items()):
        defaults = documented.get(action_id)
        if defaults is None:
            failures.append(f"{action_id} is not documented in docs/KEYMAP.md")
        elif binding not in defaults:
            failures.append(
                f"docs/KEYMAP.md documents {action_id} with {defaults}, the catalogue binds it to "
                f"'{binding}'"
            )
    return failures


def check_contexts(catalogue: dict[str, str]) -> list[str]:
    """Return the failure messages for a context that holds a different action set.

    WHAT: every context of the catalogue holds the same actions. The binding
    differs, the action does not.
    """
    failures: list[str] = []
    for name in ("input.dpad", "input.controller"):
        bindings = read_context(name)
        operations = set(catalogue)
        missing = sorted(operations - set(bindings))
        extra = sorted(set(bindings) - operations - set(SURFACE_ACTIONS))
        for action_id in missing:
            failures.append(f"{action_id} is missing in the context {name}")
        for action_id in extra:
            failures.append(f"{action_id} exists only in the context {name}")
    return failures


def check_undocumented_actions(catalogue: dict[str, str]) -> tuple[list[str], list[str]]:
    """Return the failures and the accepted deviations for actions without a rule.

    WHAT: every action of the catalogue is bound in the keyboard context, is
    documented in the reference and is named by one of the rules above. The
    two exceptions are the ones that the catalogue documents itself.
    """
    failures: list[str] = []
    accepted: list[str] = []
    named = set(CLASSIC_BINDINGS.values()) | set(EXTRA_BINDINGS)
    for action_id in PROSE_BINDINGS.values():
        named.update(action_id)
    for action_id in sorted(catalogue):
        if action_id in named:
            continue
        if action_id in ACCEPTED_DEVIATIONS:
            accepted.append(f"{action_id}: {ACCEPTED_DEVIATIONS[action_id]}")
            continue
        failures.append(
            f"{action_id} exists in the catalogue but is named neither in the specification nor in "
            "the reference"
        )
    return failures, accepted


def main() -> int:
    """Run every check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verbose", "-v", action="store_true", help="print every row, not only deviations")
    args = parser.parse_args()

    catalogue = read_catalogue()
    failures: list[str] = []
    accepted: list[str] = []

    print("=== A. The specification names the catalogue and the reference ===")
    specification_failures = check_specification_points_at_the_catalogue()
    if specification_failures:
        failures.extend(specification_failures)
        for failure in specification_failures:
            print(f"  DEVIATION {failure}")
    else:
        print("  section 6.1 names config/keymap.toml and docs/KEYMAP.md  ok")

    print()
    print("=== B. The classic key set of the specification ===")
    classic_failures, classic_accepted = check_classic_bindings(catalogue)
    failures.extend(classic_failures)
    accepted.extend(classic_accepted)
    for key, action_id in CLASSIC_BINDINGS.items():
        binding = catalogue.get(action_id)
        state = "ok" if binding == key else "DEVIATION"
        print(f"  {key:<14} {action_id:<26} {binding:<16} {state}")
    for key, action_ids in PROSE_BINDINGS.items():
        for action_id in action_ids:
            binding = catalogue.get(action_id)
            state = "ok" if binding == key else "DEVIATION"
            print(f"  {key:<14} {action_id:<26} {binding:<16} {state}")

    print()
    print("=== C. Action catalogue -> reference table (docs/KEYMAP.md) ===")
    reference_failures = check_reference_document(catalogue)
    failures.extend(reference_failures)
    for failure in reference_failures:
        print(f"  {failure}")

    print()
    print("=== D. The three contexts hold the same actions ===")
    context_failures = check_contexts(catalogue)
    failures.extend(context_failures)
    print(f"  keyboard / dpad / controller : {'ok' if not context_failures else 'DEVIATION'}")

    print()
    print("=== E. Actions that no rule names ===")
    action_failures, action_accepted = check_undocumented_actions(catalogue)
    failures.extend(action_failures)
    accepted.extend(action_accepted)
    for entry in action_accepted:
        print(f"  {entry}")
    for failure in action_failures:
        print(f"  {failure}")

    print()
    print(f"Actions in the catalogue        : {len(catalogue)}")
    print(f"Actions documented in KEYMAP.md : {len(read_keymap_doc())}")
    print()
    print(f"Accepted deviations (documented in the repository): {len(accepted) + len(DOCUMENTED_NOTES)}")
    for entry in accepted:
        print(f"  - {entry}")
    for entry in DOCUMENTED_NOTES:
        print(f"  - {entry}")
    print(f"Undocumented deviations: {len(failures)}")
    for failure in failures:
        print(f"  - {failure}")
    if failures:
        print("RESULT: FAIL — the three descriptions do not agree.")
        return 1
    print("RESULT: PASS — specification, catalogue and reference agree.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
