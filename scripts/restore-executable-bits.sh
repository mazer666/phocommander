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

# WHY: the list is read from the tree of Git, because the tree is the truth
#       about which scripts are meant to be run directly.
while read -r mode path; do
    if [ "$mode" != "100755" ]; then
        continue
    fi
    if [ ! -f "$path" ]; then
        echo "  $path: the file is not in the working tree"
        continue
    fi
    if [ ! -x "$path" ]; then
        chmod +x "$path"
        echo "  $path: the executable bit is set again"
        restored=$((restored + 1))
    fi
done < <(git ls-tree -r HEAD scripts/ | awk '{print $1, $4}')

echo "Scripts made executable again: ${restored}"
