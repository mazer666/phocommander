#!/usr/bin/env python3
"""Check the action catalogue of config/keymap.toml for consistency.

WHAT: verifies that every action ID occurs exactly once per operating context,
that the keyboard bindings match the key table of the specification, that the
dpad and controller contexts refer to the action palette, and that every action
ID is referenced in the acceptance criteria or in the specification.

WHY: the action ID is the contract between the input layer, the surface and
the configuration file. A duplicated ID, an ID that appears in only one context
or a binding that silently drifts away from the specification breaks that
contract without any compiler noticing.

EXAMPLE:
    python3 scripts/check-action-ids.py
    python3 scripts/check-action-ids.py --keymap config/keymap.toml

The script exits with code 0 when every check passes and with code 1 when at
least one check fails. It prints one line per check.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

# The key table of the specification, section "Tastatursteuerung". Every entry
# is the binding the specification names for this action.
SPECIFICATION_BINDINGS = {
    "panel.rename": "F2",
    "panel.view": "F3",
    "panel.edit": "F4",
    "panel.copy": "F5",
    "panel.move": "F6",
    "panel.mkdir": "F7",
    "panel.delete": "F8",
    "app.menu": "F9",
    "app.quit": "F10",
    "panel.switch": "Tab",
    "panel.reload": "Ctrl+R",
    "panel.swap": "Ctrl+U",
    "panel.open_in_other": "Ctrl+Enter",
    "panel.tab.1": "Ctrl+1",
    "panel.tab.2": "Ctrl+2",
    "panel.tab.3": "Ctrl+3",
    "panel.tab.4": "Ctrl+4",
    "panel.tab.5": "Ctrl+5",
    "panel.tab.6": "Ctrl+6",
    "panel.tab.7": "Ctrl+7",
    "panel.tab.8": "Ctrl+8",
    "panel.tab.9": "Ctrl+9",
    "panel.next_tab": "Ctrl+Tab",
    "selection.toggle": "Ins",
    "selection.all": "Ctrl+A",
    "selection.none": "Ctrl+Num-",
    "selection.invert": "Ctrl+Num*",
    "selection.by_pattern": "Space",
    "panel.drives": "Alt+F1",
    "panel.parent": "Ctrl+PgUp",
    "viewer.find_next": "F3",
    "viewer.find_previous": "Shift+F3",
    "rename.conflict_next": "F3",
    "rename.conflict_previous": "Shift+F3",
}

# The binding of the one action that the key table of the specification does
# not name. It is the symmetric completion of "Ctrl+Tab" and is marked as such
# in config/keymap.toml and in the open points of the handover.
DERIVED_BINDINGS = {
    "panel.prev_tab": "Ctrl+Shift+Tab",
}

# The action IDs that are not part of the catalogue but of the surface.
SURFACE_ACTIONS = {"palette.open"}

# The bindings that are allowed to be a physical key outside the keyboard
# context, and the reason they are allowed.
CONTEXT_EXCEPTIONS = {
    "input.dpad": {"palette.open": "Menu"},
    "input.controller": {"palette.open": "Menu", "selection.toggle": "Grip"},
}

# The contexts of the catalogue.
CONTEXTS = ["input.keyboard", "input.dpad", "input.controller"]

# The pattern of a palette reference.
PALETTE_PATTERN = re.compile(r"^palette:[a-z]+(\.[a-z0-9_]+)+$")

# The pattern of an action ID.
ACTION_PATTERN = re.compile(r"^[a-z]+(\.[a-z0-9_]+)+$")


def repository_root() -> Path:
    """Return the repository root, derived from the location of this script."""
    return Path(__file__).resolve().parent.parent


def read_contexts(keymap: Path) -> dict[str, dict[str, str]]:
    """Return the bindings of every context of the keymap file.

    WHAT: a context is a table that is addressed by a dotted path, for example
    "input.keyboard". In TOML such a table is nested, so the path is walked
    step by step.
    """
    data = tomllib.loads(keymap.read_text(encoding="utf-8"))
    contexts: dict[str, dict[str, str]] = {}
    for context in CONTEXTS:
        table: object = data
        for step in context.split("."):
            if not isinstance(table, dict) or step not in table:
                table = None
                break
            table = table[step]
        if isinstance(table, dict):
            contexts[context] = {str(key): str(value) for key, value in table.items()}
    return contexts


def check_sections_present(contexts: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for missing context sections."""
    return [
        f"the section {context} is missing"
        for context in CONTEXTS
        if context not in contexts
    ]


def check_catalogue(contexts: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for a wrong action catalogue."""
    failures: list[str] = []
    expected = set(SPECIFICATION_BINDINGS) | set(DERIVED_BINDINGS)
    for context in CONTEXTS:
        found = set(contexts.get(context, {}))
        catalogue = found - SURFACE_ACTIONS
        if catalogue != expected:
            for missing in sorted(expected - catalogue):
                failures.append(f"{context}: the action {missing} is missing")
            for unknown in sorted(catalogue - expected):
                failures.append(f"{context}: the action {unknown} is not part of the catalogue")
    return failures


def check_single_occurrence(contexts: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for actions that occur more than once.

    WHAT: TOML rejects a duplicated key in one table, so a duplicate can only
    appear as the same action ID in two different contexts. Every action must
    be bound in every context, and the count of occurrences per action must be
    the number of contexts.
    """
    failures: list[str] = []
    occurrences: dict[str, int] = {}
    for context in CONTEXTS:
        for action in contexts.get(context, {}):
            if action in SURFACE_ACTIONS:
                continue
            occurrences[action] = occurrences.get(action, 0) + 1
    for action, count in sorted(occurrences.items()):
        if count != len(CONTEXTS):
            failures.append(
                f"the action {action} occurs {count} times, expected {len(CONTEXTS)}"
            )
    return failures


def check_keyboard_bindings(contexts: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for keyboard bindings that drift away."""
    failures: list[str] = []
    keyboard = contexts.get("input.keyboard", {})
    for action, expected in sorted(SPECIFICATION_BINDINGS.items()):
        found = keyboard.get(action)
        if found != expected:
            failures.append(
                f"input.keyboard: {action} is bound to '{found}', the specification names '{expected}'"
            )
    for action, expected in sorted(DERIVED_BINDINGS.items()):
        found = keyboard.get(action)
        if found != expected:
            failures.append(
                f"input.keyboard: {action} is bound to '{found}', expected '{expected}'"
            )
    return failures


def check_palette_references(contexts: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for non-palette bindings outside the keyboard."""
    failures: list[str] = []
    for context in ("input.dpad", "input.controller"):
        exceptions = CONTEXT_EXCEPTIONS[context]
        for action, binding in sorted(contexts.get(context, {}).items()):
            if action in exceptions:
                if binding != exceptions[action]:
                    failures.append(
                        f"{context}: {action} is bound to '{binding}', expected '{exceptions[action]}'"
                    )
                continue
            if not PALETTE_PATTERN.match(binding):
                failures.append(
                    f"{context}: {action} is bound to '{binding}', expected a palette reference"
                )
    return failures


def check_surface_conflicts(contexts: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for two actions of one surface on one key.

    WHAT: the specification assigns the same key to several functions of
    different surfaces and calls it consistent: "F3 im Betrachter sucht
    weiter, Shift+F3 sucht rueckwaerts - durchgaengig, auch in den
    Einzelfenstern des Mehrfach-Umbenennungsdialogs, wo F3 und Shift+F3 den
    Konfliktsprung steuern". A key is therefore only a conflict when two
    actions of the SAME surface share it. The surface is the first segment of
    the action identifier, for example "panel" or "viewer".
    """
    failures: list[str] = []
    for context in CONTEXTS:
        by_key: dict[str, list[str]] = {}
        for action, binding in sorted(contexts.get(context, {}).items()):
            if action in SURFACE_ACTIONS:
                continue
            by_key.setdefault(binding, []).append(action)
        for binding, actions in sorted(by_key.items()):
            surfaces: dict[str, list[str]] = {}
            for action in actions:
                surfaces.setdefault(action.split(".")[0], []).append(action)
            for surface, group in sorted(surfaces.items()):
                if len(group) > 1:
                    failures.append(
                        f"{context}: the key '{binding}' is bound to {len(group)} actions of the "
                        f"surface '{surface}': {', '.join(group)}"
                    )
    return failures


def check_identifier_shape(contexts: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for action IDs that are not dotted."""
    failures: list[str] = []
    for context in CONTEXTS:
        for action in contexts.get(context, {}):
            if action in SURFACE_ACTIONS:
                continue
            if not ACTION_PATTERN.match(action):
                failures.append(f"{context}: the action ID '{action}' is not in dotted notation")
    return failures


def check_references(
    contexts: dict[str, dict[str, str]], acceptance: Path, specification: Path
) -> list[str]:
    """Return the failure messages for actions that nothing refers to."""
    failures: list[str] = []
    haystack = ""
    for path in (acceptance, specification):
        if path.exists():
            haystack += path.read_text(encoding="utf-8")
    for action in sorted(set(contexts.get("input.keyboard", {}))):
        if action in SURFACE_ACTIONS:
            continue
        if action not in haystack:
            failures.append(
                f"the action {action} is not referenced in {acceptance.name} or {specification.name}"
            )
    return failures


def check_defaults_overrides(defaults: Path, contexts: dict[str, dict[str, str]]) -> list[str]:
    """Return the failure messages for overrides of unknown actions."""
    failures: list[str] = []
    if not defaults.exists():
        return [f"{defaults} does not exist"]
    data = tomllib.loads(defaults.read_text(encoding="utf-8"))
    known = set(contexts.get("input.keyboard", {}))
    for section in ("keymap", "input.keyboard"):
        table = data.get(section)
        if not isinstance(table, dict):
            continue
        for action in table:
            if action not in known:
                failures.append(f"{section}: the override of '{action}' names an unknown action")
    return failures


def main() -> int:
    """Run every check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    root = repository_root()
    parser.add_argument("--keymap", type=Path, default=root / "config" / "keymap.toml")
    parser.add_argument("--defaults", type=Path, default=root / "config" / "defaults.toml")
    parser.add_argument("--acceptance", type=Path, default=root / "docs" / "ACCEPTANCE-CRITERIA.md")
    parser.add_argument("--specification", type=Path, default=root / "docs" / "REQUIREMENTS.md")
    arguments = parser.parse_args()

    contexts = read_contexts(arguments.keymap)
    catalogue = set(contexts.get("input.keyboard", {})) - SURFACE_ACTIONS

    checks: list[tuple[str, list[str]]] = [
        ("all three context sections exist", check_sections_present(contexts)),
        ("the action catalogue is complete and free of unknown actions", check_catalogue(contexts)),
        ("every action occurs exactly once per context", check_single_occurrence(contexts)),
        ("the keyboard bindings match the key table of the specification", check_keyboard_bindings(contexts)),
        ("dpad and controller refer to the action palette", check_palette_references(contexts)),
        (
            "no key is bound to two actions of the same surface",
            check_surface_conflicts(contexts),
        ),
        ("every action ID is in dotted notation", check_identifier_shape(contexts)),
        (
            "every action is referenced in the acceptance criteria or the specification",
            check_references(contexts, arguments.acceptance, arguments.specification),
        ),
        ("the overrides in defaults.toml name known actions", check_defaults_overrides(arguments.defaults, contexts)),
    ]

    print(f"keymap    : {arguments.keymap}")
    print(f"actions   : {len(catalogue)} action IDs, {len(SURFACE_ACTIONS)} surface action")
    print(
        f"bindings  : {len(SPECIFICATION_BINDINGS)} from the key table of the specification, "
        f"{len(DERIVED_BINDINGS)} derived ({', '.join(sorted(DERIVED_BINDINGS))})"
    )
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
