#!/usr/bin/env python3
"""Check every relative Markdown link in the repository — alias for `check-links.py`.

WHAT: this script runs the link checker of ``scripts/check-links.py`` and
      forwards its exit code. It exists because the checker is referenced under
      two names in the project: ``scripts/check-links.py`` in `AGENTS.md` and in
      the documentation lead's checklist, and ``scripts/check-docs-links.py`` in
      the QA task description of the M0 milestone.
WHY:  two names for one behaviour would be two implementations that drift
      apart. This file keeps a single implementation — `check-links.py` — and
      offers the second name as a thin forwarder, so both documented commands
      produce the same output and the same exit code.
EXAMPLE:
      python3 scripts/check-docs-links.py
      python3 scripts/check-docs-links.py --summary-only

Exit code 0 means "every link resolves", exit code 1 means "at least one link
is broken". The behaviour is documented in `scripts/check-links.py`.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

IMPLEMENTATION = Path(__file__).resolve().parent / "check-links.py"


def load_implementation():
    """Import the link checker module without executing its `main`."""
    spec = importlib.util.spec_from_file_location("phocommander_check_links", IMPLEMENTATION)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load the link checker from {IMPLEMENTATION}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    module = load_implementation()
    return int(module.main())


if __name__ == "__main__":
    sys.exit(main())
