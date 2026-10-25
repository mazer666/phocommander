#!/usr/bin/env bash
# Restore the executable bits of the scripts of the repository.
#
# WHAT: reads the modes from the tree of Git and sets the bit on the scripts
#       that carry it there.
# WHY: the scripts of the gates are called directly — "./scripts/check.sh" and
#       not "bash scripts/check.sh". The mode of a file is part of the tree of
#       Git and not part of its content, and a way of writing that ignores the
#       mode — the way of the connector — loses the bit. After every update
#       of the tree the bit therefore has to be restored, or the gates stop
#       running.
# EXAMPLE:
#     ./scripts/restore-executable-bits.sh
#
# The script reads the tree of the local repository, not the network. It needs
# no token and no connection.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

restored=0

# WHY: the list is written down here and not read from the tree of Git. A
#       way of writing that does not carry the mode loses it — in the tree as
#       well as on the disk — and a script that reads its own list from the
#       tree would then restore nothing. The list of the scripts that are run
#       directly does not change often.
SCRIPTS=(
    "scripts/cargo.sh"
    "scripts/check-crate-boundaries.sh"
    "scripts/check-docs-links.py"
    "scripts/check-file-size.sh"
    "scripts/check-gates-negative.sh"
    "scripts/check-links.py"
    "scripts/check-no-telemetry.sh"
    "scripts/qa-env.sh"
    "scripts/restore-executable-bits.sh"
)

for path in "${SCRIPTS[@]}"; do
    if [ ! -f "$path" ]; then
        echo "  $path: the file is not in the working tree"
        continue
    fi
    if [ ! -x "$path" ]; then
        chmod +x "$path"
        echo "  $path: the executable bit is set again"
        restored=$((restored + 1))
    fi
done

echo "Scripts made executable again: ${restored}"
