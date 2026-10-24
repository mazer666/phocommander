#!/usr/bin/env python3
"""Check the configuration files for schema, default values and comments.

WHAT: verifies that every file under config/ parses as TOML, that
config/defaults.toml carries the sections and the default values of the
specification unchanged, that every key and every section carries a comment,
and that the shipped rename templates are complete.

WHY: the configuration file is the first contact point for users and
contributors. A key without a comment and a default value that differs from
the specification are both defects that no compiler reports and every user
pays for.

EXAMPLE:
    python3 scripts/check-config-schema.py
    python3 scripts/check-config-schema.py --config config

The script exits with code 0 when every check passes and with code 1 when at
least one check fails. It prints one line per check.
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

# The sections the configuration file of the specification carries.
EXPECTED_SECTIONS = [
    "general",
    "appearance",
    "panels",
    "keymap",
    "viewer",
    "archives",
    "privacy",
    "advanced",
    "input",
    "input.keyboard",
    "input.dpad",
    "input.controller",
    "remote",
]

# The default values of the specification. The key of an entry is the dotted
# path of the table, a "|" and the name of the key. The two parts are kept
# apart because an action ID contains dots itself, for example "panel.copy".
EXPECTED_VALUES = {
    "|schema_version": 1,
    "general|language": "system",
    "general|restore_session": True,
    "appearance|theme": "system",
    "appearance|row_height": 0,
    "appearance|show_hidden_files": False,
    "appearance|show_system_files": False,
    "appearance|file_size_format": "binary",
    "panels|layout": "horizontal",
    "panels|right_click_action": "context_menu",
    "panels|overwrite_policy": "ask",
    "viewer|max_file_size": 10485760,
    "viewer|max_text_size": 4194304,
    "archives|max_extracted_bytes": 2147483648,
    "archives|max_entries": 100000,
    "privacy|log_level": "warn",
    "advanced|worker_threads": 0,
    "input|context": "auto",
    "remote|connect_timeout_seconds": 15,
    "remote|smb_signing": "prefer",
    "remote|smb_encryption": "prefer",
    "remote|max_remote_download_bytes": 0,
    "keymap|panel.copy": "F5",
    "keymap|panel.move": "F6",
    "input.keyboard|panel.copy": "F5",
    "input.keyboard|panel.move": "F6",
    "input.dpad|panel.copy": "palette:file.copy",
    "input.dpad|panel.move": "palette:file.move",
    "input.dpad|palette.open": "Menu",
    "input.controller|panel.copy": "palette:file.copy",
    "input.controller|selection.toggle": "Grip",
}

# The shipped rename templates of the specification.
EXPECTED_TEMPLATES = ["camera.toml", "invoice.toml", "music.toml", "podcast.toml", "series.toml"]

# A table header line in a TOML file.
TABLE_PATTERN = re.compile(r"^\s*\[[^\]]+\]\s*$")

# A key assignment line in a TOML file.
KEY_PATTERN = re.compile(r"^\s*(?:\"[^\"]+\"|[A-Za-z0-9_.-]+)\s*=")

# A Rust struct declaration.
STRUCT_PATTERN = re.compile(r"^pub struct (\w+)\s*\{", re.MULTILINE)

# A public field of a Rust struct.
FIELD_PATTERN = re.compile(r"^\s*pub (\w+):\s*([\w:<>, ]+?),", re.MULTILINE)

# The struct of the Rust schema behind each section of the configuration file.
SCHEMA_SECTIONS = {
    "general": "General",
    "appearance": "Appearance",
    "panels": "Panels",
    "viewer": "Viewer",
    "archives": "Archives",
    "privacy": "Privacy",
    "advanced": "Advanced",
    "input": "Input",
    "remote": "Remote",
}

# The structs that hold a free-form table of action identifiers. Their keys
# are checked by the keymap crate, not by this script.
FREE_FORM_STRUCTS = {"KeymapOverrides", "InputBindings"}


def repository_root() -> Path:
    """Return the repository root, derived from the location of this script."""
    return Path(__file__).resolve().parent.parent


def fetch(data: dict, path: str) -> object:
    """Return the value of a key in a table, or a marker when it is missing.

    WHAT: the path is written as "table.path|key". The table path is walked
    segment by segment, the key is taken as a whole because an action ID
    contains dots itself.
    """
    table_path, _, key = path.partition("|")
    current: object = data
    for segment in table_path.split("."):
        if segment == "":
            continue
        if not isinstance(current, dict) or segment not in current:
            return "<missing>"
        current = current[segment]
    if not isinstance(current, dict) or key not in current:
        return "<missing>"
    return current[key]


def collect_sections(data: dict) -> set[str]:
    """Return the dotted names of every table of the parsed TOML."""
    sections: set[str] = set()

    def walk(table: dict, prefix: str) -> None:
        for key, value in table.items():
            if isinstance(value, dict):
                name = f"{prefix}{key}"
                sections.add(name)
                walk(value, f"{name}.")

    walk(data, "")
    return sections


def check_parses(files: list[Path]) -> tuple[list[str], dict[Path, dict]]:
    """Return the failure messages and the parsed content of every file."""
    failures: list[str] = []
    parsed: dict[Path, dict] = {}
    for path in files:
        try:
            parsed[path] = tomllib.loads(path.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError as error:
            failures.append(f"{path}: {error}")
    return failures, parsed


def check_sections(data: dict) -> list[str]:
    """Return the failure messages for missing sections."""
    found = collect_sections(data)
    return [
        f"the section {section} is missing"
        for section in EXPECTED_SECTIONS
        if section not in found
    ]


def check_values(data: dict) -> list[str]:
    """Return the failure messages for default values that differ."""
    failures: list[str] = []
    for path, expected in EXPECTED_VALUES.items():
        found = fetch(data, path)
        if found != expected:
            failures.append(f"{path} is '{found}', the specification names '{expected}'")
    return failures


def check_comments(path: Path) -> list[str]:
    """Return the failure messages for keys and sections without a comment.

    WHAT: walks the raw lines of the file. Every table header and every key
    assignment must be preceded by a comment line, because the file is the
    first contact point for the user.
    """
    failures: list[str] = []
    lines = path.read_text(encoding="utf-8").splitlines()
    previous = ""
    for number, line in enumerate(lines, start=1):
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


def check_templates(directory: Path, parsed: dict[Path, dict]) -> list[str]:
    """Return the failure messages for the shipped rename templates."""
    failures: list[str] = []
    found = sorted(path.name for path in directory.glob("*.toml")) if directory.exists() else []
    for name in EXPECTED_TEMPLATES:
        if name not in found:
            failures.append(f"the template {name} is missing")
    for name in found:
        if name not in EXPECTED_TEMPLATES:
            failures.append(f"the template {name} is not part of the specification")
    for path, data in parsed.items():
        if path.parent != directory:
            continue
        if not isinstance(data.get("name"), str) or not data["name"].strip():
            failures.append(f"{path}: the template has no name")
        if not isinstance(data.get("description"), str) or not data["description"].strip():
            failures.append(f"{path}: the template has no description")
        rule = data.get("rule")
        if not isinstance(rule, dict) or not isinstance(rule.get("format"), str):
            failures.append(f"{path}: the template has no rule definition with a format")
        text = path.read_text(encoding="utf-8")
        if "{" not in text or "}" not in text:
            failures.append(f"{path}: the template documents no placeholder")
    return failures


def check_keymap_defaults(defaults: dict, keymap: dict) -> list[str]:
    """Return the failure messages for a mismatch between the two files."""
    failures: list[str] = []
    overrides = defaults.get("keymap")
    if not isinstance(overrides, dict):
        return ["the section keymap is missing in the configuration file"]
    keyboard = keymap.get("input", {})
    keyboard = keyboard.get("keyboard", {}) if isinstance(keyboard, dict) else {}
    for action, value in overrides.items():
        if action not in keyboard:
            failures.append(f"the override of {action} has no binding in the catalogue")
        elif keyboard[action] != value:
            failures.append(
                f"the override of {action} is '{value}', the catalogue names '{keyboard[action]}'"
            )
    return failures


def read_rust_schema(schema_path: Path) -> dict[str, dict[str, str]]:
    """Return the public fields of every struct of a Rust schema file.

    WHAT: collects the name and the type of every `pub` field of every
    `pub struct` of the file.
    """
    if not schema_path.exists():
        return {}
    text = schema_path.read_text(encoding="utf-8")
    structs: dict[str, dict[str, str]] = {}
    for match in STRUCT_PATTERN.finditer(text):
        name = match.group(1)
        body = text[match.end() :]
        end = body.find("\n}")
        if end == -1:
            continue
        fields = {
            field.group(1): field.group(2).strip()
            for field in FIELD_PATTERN.finditer(body[:end])
        }
        structs[name] = fields
    return structs


def check_schema_drift(schema_path: Path, defaults: dict) -> list[str]:
    """Return the failure messages for a drift between the file and the schema.

    WHAT: a key of the configuration file that the Rust schema does not know
    would be rejected by the loader, because the schema denies unknown fields.
    A field of the schema that the template does not provide would leave the
    value to be taken from somewhere else, and the template is the place where
    the defaults of the specification are written down.
    """
    structs = read_rust_schema(schema_path)
    if not structs:
        return []
    failures: list[str] = []
    for section, struct_name in SCHEMA_SECTIONS.items():
        fields = structs.get(struct_name)
        if fields is None:
            failures.append(f"the Rust schema has no struct {struct_name} for the section {section}")
            continue
        table = defaults.get(section)
        if not isinstance(table, dict):
            failures.append(f"the section {section} is missing in the configuration file")
            continue
        for key in sorted(table):
            if key in fields:
                continue
            if any(field_type in FREE_FORM_STRUCTS for field_type in fields.values()):
                continue
            failures.append(
                f"{section}.{key} is in the file, but the Rust schema does not know this field"
            )
        for key, field_type in sorted(fields.items()):
            if field_type in FREE_FORM_STRUCTS:
                continue
            if key not in table:
                failures.append(
                    f"{section}.{key} is in the Rust schema, but the template does not provide it"
                )
    return failures


def main() -> int:
    """Run every check and report the result."""
    parser = argparse.ArgumentParser(description=__doc__)
    root = repository_root()
    parser.add_argument("--config", type=Path, default=root / "config")
    parser.add_argument(
        "--schema",
        type=Path,
        default=root / "crates" / "phocommander-config" / "src" / "schema.rs",
    )
    arguments = parser.parse_args()

    config_dir = arguments.config
    template_dir = config_dir / "templates"
    defaults_path = config_dir / "defaults.toml"
    keymap_path = config_dir / "keymap.toml"
    schema_path = arguments.schema

    files = sorted(config_dir.rglob("*.toml"))
    parse_failures, parsed = check_parses(files)

    defaults = parsed.get(defaults_path, {})
    keymap = parsed.get(keymap_path, {})

    comment_targets = [defaults_path, keymap_path] + sorted(template_dir.glob("*.toml"))
    comment_failures: list[str] = []
    for path in comment_targets:
        if path.exists():
            comment_failures.extend(check_comments(path))

    checks: list[tuple[str, list[str]]] = [
        (f"all {len(files)} files under {config_dir.name}/ parse as TOML", parse_failures),
        ("the configuration file carries every section of the specification", check_sections(defaults)),
        ("the default values match the specification", check_values(defaults)),
        ("every key and every section carries a comment", comment_failures),
        ("the shipped rename templates are complete", check_templates(template_dir, parsed)),
        ("the overrides match the action catalogue", check_keymap_defaults(defaults, keymap)),
        (
            "the configuration file and the Rust schema agree",
            check_schema_drift(schema_path, defaults),
        ),
    ]

    print(f"config    : {config_dir}")
    print(f"files     : {len(files)}")
    for path in files:
        print(f"           {path.relative_to(root)}")
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
