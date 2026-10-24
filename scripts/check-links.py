#!/usr/bin/env python3
"""Check that every relative Markdown link in the repository resolves.

WHAT: this script collects every Markdown link (inline links, reference
      definitions and images) from every ``*.md`` file in the repository and
      checks whether the target exists on disk.
WHY: documentation that links to files which are not there is worse than no
      documentation at all — a reader follows the link, lands on nothing, and
      loses trust in the whole project. The specification makes "no dead links"
      an acceptance criterion for the documentation lead
      (``docs/REQUIREMENTS.md``, section "Teamorganisation").
EXAMPLE:
      python3 scripts/check-links.py
      python3 scripts/check-links.py --verbose

Exit code 0 means "every link resolves". Exit code 1 means "at least one link
is broken" — with the single documented exception listed in ALLOWED_MISSING.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Directories that never contain checked-in documentation.
SKIP_DIRS = {".git", "target", "node_modules", ".mavis", ".scratch", ".home"}

# The one documented exception. The threat model is written by the security
# worker in parallel; until it lands the links pointing at it are reported as
# "in progress" instead of "broken".
ALLOWED_MISSING: dict[str, str] = {
    "docs/security/THREAT_MODEL.md": "in progress — written by the security worker in parallel",
}

# Inline link: [text](target "optional title"). The target may contain
# parentheses only when they are balanced, which is rare enough that the
# simple pattern below is sufficient for this repository.
INLINE_LINK = re.compile(r"(?<!\!)\[(?:[^\]\\]|\\.)*\]\(\s*<?([^)\s]+)>?(?:\s+\"[^\"]*\")?\s*\)")
# Image link: ![alt](target)
IMAGE_LINK = re.compile(r"!\[[^\]]*\]\(\s*<?([^)\s]+)>?(?:\s+\"[^\"]*\")?\s*\)")
# Reference definition: [label]: target
REFERENCE_DEF = re.compile(r"^\s{0,3}\[[^\]]+\]:\s*<?([^\s>]+)>?", re.MULTILINE)

EXTERNAL_PREFIXES = ("http://", "https://", "mailto:", "ftp://", "tel:", "#")


def is_external(target: str) -> bool:
    return target.startswith(EXTERNAL_PREFIXES)


def collect_links(path: Path) -> list[tuple[str, int]]:
    """Return (target, line number) for every link found in `path`."""
    text = path.read_text(encoding="utf-8")
    found: list[tuple[str, int]] = []
    for pattern in (INLINE_LINK, IMAGE_LINK):
        for match in pattern.finditer(text):
            found.append((match.group(1), text.count("\n", 0, match.start()) + 1))
    for match in REFERENCE_DEF.finditer(text):
        found.append((match.group(1), text.count("\n", 0, match.start()) + 1))
    return found


def resolve(source: Path, target: str) -> Path | None:
    """Return the filesystem path a relative link points at, or None if external."""
    if is_external(target):
        return None
    clean = target.split("#", 1)[0].split("?", 1)[0]
    if not clean:
        return None
    candidate = Path(clean)
    if candidate.is_absolute():
        return candidate
    return (source.parent / candidate).resolve()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verbose", "-v", action="store_true", help="print every link, not only failures")
    args = parser.parse_args()

    markdown_files = sorted(
        path
        for path in REPO_ROOT.rglob("*.md")
        if not any(part in SKIP_DIRS for part in path.relative_to(REPO_ROOT).parts)
    )

    rows: list[tuple[str, str, str, str]] = []
    broken = 0
    checked = 0

    for source in markdown_files:
        relative_source = source.relative_to(REPO_ROOT)
        for target, line in collect_links(source):
            resolved = resolve(source, target)
            if resolved is None:
                rows.append((f"{relative_source}:{line}", target, "external", "no check — not a repository file"))
                continue
            checked += 1
            if resolved.exists():
                rows.append((f"{relative_source}:{line}", target, "ok", "resolves"))
                continue
            try:
                relative_target = str(resolved.relative_to(REPO_ROOT))
            except ValueError:
                relative_target = str(resolved)
            if relative_target in ALLOWED_MISSING:
                rows.append(
                    (f"{relative_source}:{line}", target, "in progress", ALLOWED_MISSING[relative_target])
                )
                continue
            broken += 1
            rows.append((f"{relative_source}:{line}", target, "BROKEN", "create the file or fix the link"))

    width_source = max((len(row[0]) for row in rows), default=10)
    width_target = max((len(row[1]) for row in rows), default=10)
    print(f"{'SOURCE'.ljust(width_source)}  {'TARGET'.ljust(width_target)}  STATUS")
    print(f"{'-' * width_source}  {'-' * width_target}  {'-' * 16}")
    for source, target, status, action in rows:
        if status == "ok" and not args.verbose:
            continue
        print(f"{source.ljust(width_source)}  {target.ljust(width_target)}  {status}  ({action})")

    print()
    print(f"Markdown files scanned : {len(markdown_files)}")
    print(f"Repository links checked: {checked}")
    print(f"External links skipped : {len(rows) - checked}")
    print(f"Broken links           : {broken}")
    if broken:
        print("RESULT: FAIL — a dead link is a documentation defect.")
        return 1
    print("RESULT: PASS — every repository link resolves.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
