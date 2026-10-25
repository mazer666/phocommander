#!/usr/bin/env python3
"""Find the texts of the interface that are written down in the source.

WHAT: reads the source files of the interface crates and reports every string
      literal that carries a text a user could read — a literal with a letter
      and a space in it, that is neither a key of the catalogue nor a path nor
      a format string.
WHY: the specification (requirement FR-05) asks that every text of the
      interface comes from the catalogue. A text that is written down in the
      source stays in the source when the language changes, and the user sees
      an English sentence in a German interface. The gate reads the source
      because the source is the only place where such a text can hide.
EXAMPLE:
    python3 scripts/check-i18n-hardcoded.py
    python3 scripts/check-i18n-hardcoded.py --path crates/phocommander-ui/src

The script exits with code 0 when the interface layer holds no visible text
and with code 1 when it holds at least one. It prints one line per finding.

The gate is not a proof that every text of the interface comes from the
catalogue. It is a net for the texts that were written down by accident — the
sentence that a developer typed into a function instead of into the catalogue.
The proof itself is the review of the drawing, and this gate keeps the net
under it.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


# The directories that are searched by default.
#
# WHY: the gate reads the interface crate — the crate that draws the surface.
#       The composition root writes its report into the console at the start,
#       and that report is a diagnosis for the person at the keyboard, not a
#       text of the interface. The specification (section 8.4) names the
#       labels, the messages and the error messages of the interface — not
#       the report of the start.
DEFAULT_PATHS = ("crates/phocommander-ui/src",)

# A string literal that carries a text a user could read: a letter, a space
# and a letter. The pattern is deliberately simple — a sentence, a word with a
# space, a label.
VISIBLE_TEXT = re.compile(r'"[^"\\]*[A-Za-zÀ-ÿ][^"\\]*\s+[^"\\]*[A-Za-zÀ-ÿ][^"\\]*"')

# The literals that are not a text of the interface.
ALLOWED = (
    # A path, a key of the configuration or a name of a file.
    "application/x-www-form-urlencoded",
    "text/plain",
    "text/html",
    "text/csv",
    "Content-Type",
    "User-Agent",
    "application/json",
    "text/xml",
    "charset=utf-8",
    "phocommander",
    "Phocommander",
    "en-US",
    "de-DE",
    "de-CH",
    "fr-FR",
    "CamelCase",
    "kebab-case",
    "snake_case",
)

# The files that are searched, and the ones that are not.
SKIP_DIRECTORIES = ("tests", "benches", "target", ".git")
SKIP_FILES = ("tests.rs",)


def repository_root() -> Path:
    """Return the root of the repository.

    WHAT: the directory that holds the ``scripts`` folder.
    WHY: the script is called from anywhere, and the paths it reads are
          relative to the root of the repository.
    """
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "Cargo.toml").is_file() and (parent / "crates").is_dir():
            return parent
    return here.parents[1]


def is_skipped(path: Path, root: Path) -> bool:
    """Whether the file is not searched.

    WHAT: the files of the test modules and the benchmark are not searched.
    WHY: a test proves a rule with a literal of its own — the English text of
          the rule, the name of the key. Such a literal is not a text of the
          interface and must not make the gate red.
    """
    relative = path.relative_to(root)
    if any(part in SKIP_DIRECTORIES for part in relative.parts[:-1]):
        return True
    return path.name in SKIP_FILES


def is_allowed(literal: str) -> bool:
    """Whether the literal is one of the allowed ones.

    WHAT: the literals that are not a text of the interface.
    WHY: a media type, a header name and a language tag are not words that
          the user reads in the interface.
    """
    return literal in ALLOWED


def is_a_format(literal: str) -> bool:
    """Whether the literal is a format and not a text.

    WHAT: a literal that carries a format of a value — ``{year:04}`` — and no
          word of its own.
    WHY: the format of a date and of a size follows the locale of the system
          (FR-123). The literal here is the shape of the value and not a
          sentence, and the sentence is built from it by the formatter of the
          locale.
    """
    body = literal.strip('"')
    if "{" not in body:
        return False
    # A format of a date or of a size carries only the placeholders and the
    # separators between them. A word between two placeholders is the name
    # of a field and not a text.
    rest = re.sub(r"\{[^{}]*\}", "", body)
    return not re.search(r"[A-Za-zÀ-ÿ]{3,}", rest)


def looks_like_a_key(literal: str) -> bool:
    """Whether the literal is a key of the catalogue and not a text.

    WHAT: a literal without a space, or with the shape of an identifier.
    WHY: the keys of the catalogue look like ``panel-column-name``. They are
          written down in the source on purpose, and they are not words.
    """
    body = literal.strip('"')
    if " " not in body:
        return True
    return bool(re.fullmatch(r"[a-z0-9]+(-[a-z0-9]+)+", body))


def find_in_file(path: Path, root: Path) -> list[tuple[int, str]]:
    """Return the visible texts of one file.

    WHAT: a list of the line number and the literal of every text.
    WHY: the gate reports where the text stands, so that the developer does
          not have to search for it again.
    """
    findings: list[tuple[int, str]] = []
    in_test_module = False
    in_display = False
    in_error_attribute = False

    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        stripped = line.strip()

        # A test module is skipped from its marker to the end of the file.
        if stripped.startswith("#[cfg(test)]"):
            in_test_module = True
        if in_test_module:
            continue

        # A comment is not a text.
        if stripped.startswith("//") or stripped.startswith("*") or stripped.startswith("/*"):
            continue

        # WHY: the text of the `Display` of an error is the text of the
        #       protocol and of the panic message. The surface does not show
        #       it — the surface reads the key of the catalogue. A literal
        #       inside such an implementation is therefore not a text of the
        #       interface.
        if re.match(r"^impl(\s+std::fmt::)?Display\s+for\b", stripped) or re.match(
            r"^impl\s+.*fmt::Display\s+for\b", stripped
        ):
            in_display = True
        if in_display:
            if stripped == "}":
                in_display = False
            continue

        # The message of an error of the library `thiserror` is the text of
        # the protocol as well. The attribute may reach over several lines.
        if stripped.startswith("#[error("):
            in_error_attribute = True
            if stripped.endswith(")]"):
                in_error_attribute = False
            continue
        if in_error_attribute:
            if stripped.endswith(")]"):
                in_error_attribute = False
            continue

        for literal in VISIBLE_TEXT.findall(line):
            if is_allowed(literal):
                continue
            if looks_like_a_key(literal):
                continue
            if is_a_format(literal):
                continue
            findings.append((number, literal))
    return findings


def main() -> int:
    """Search the interface layer and report the findings.

    WHAT: prints one line per finding and a summary, and returns the exit
          code of the gate.
    WHY: the gate is a part of the chain of the acceptance, so it has to
          speak the same language as the other gates.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--path",
        action="append",
        default=None,
        help="the directory to search; may be given more than once",
    )
    arguments = parser.parse_args()

    root = repository_root()
    suchpfade = arguments.path if arguments.path else list(DEFAULT_PATHS)

    findings_total = 0
    files_total = 0

    for suchpfad in suchpfade:
        directory = Path(suchpfad)
        if not directory.is_absolute():
            directory = root / directory
        if not directory.is_dir():
            print(f"  {suchpfad}: the directory does not exist")
            continue

        for path in sorted(directory.rglob("*.rs")):
            if is_skipped(path, root):
                continue
            files_total += 1
            for number, literal in find_in_file(path, root):
                findings_total += 1
                relative = path.relative_to(root)
                print(f"  {relative}:{number}: {literal}")

    print()
    print(f"Files read               : {files_total}")
    print(f"Visible texts in the code : {findings_total}")

    if findings_total > 0:
        print()
        print("RESULT: FAIL — the interface carries texts that are written down in")
        print("        the source. Every visible word belongs into the catalogue.")
        return 1

    print("RESULT: PASS — the interface layer holds no text of its own.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
