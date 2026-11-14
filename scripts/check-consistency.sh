#!/usr/bin/env bash
# Run every consistency gate of the project.
#
# WHAT: runs the gates that compare the files of the repository with each
#       other — the catalogue of the actions with the key bindings, the
#       configuration with the schema, the documentation with the language
#       of the project, and the commands of the surface with the code that
#       carries them out.
# WHY: these gates are cheap — they read files and compare them, and they
#       build nothing. They are therefore worth running in **every** job of
#       the matrix, and not only in the one that checks the quality. A gate
#       that runs in one job out of six catches the defect in one sixth of
#       the cases.
#
#       The script names the gate that failed, because a report of eleven
#       lines of "PASS" and one line of "FAIL" without a name is of no use
#       to the person who has to fix it.
#
# EXAMPLE:
#     ./scripts/check-consistency.sh
#
# The script changes nothing in the repository.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

GATES=(
    "scripts/check-action-ids.py"
    "scripts/check-api-names.py"
    "scripts/check-dead-code.py"
    "scripts/check-commands.py"
    "scripts/check-config-schema.py"
    "scripts/check-docs-language.py"
    "scripts/check-docs-links.py"
    "scripts/check-i18n-hardcoded.py"
    "scripts/check-i18n-keys.py"
    "scripts/check-interface-profiles.py"
    "scripts/check-keymap-consistency.py"
    "scripts/check-requirement-ids.py"
    "scripts/check-acceptance-ids.py"
)

fehler=0

# WHY: der Windows-Runner hat Python unter dem Namen `python`, und nicht
#       immer unter dem Namen `python3`.
PYTHON=python3
if ! command -v python3 > /dev/null 2>&1; then
    PYTHON=python
fi

for gate in "${GATES[@]}"; do
    if [ ! -f "$gate" ]; then
        echo "FEHLER: $gate fehlt"
        fehler=$((fehler + 1))
        continue
    fi
    if output=$("$PYTHON" "$gate" 2>&1); then
        echo "PASS  $gate"
    else
        echo "FEHLER: $gate"
        echo "$output" | tail -n 12 | sed 's/^/        /'
        fehler=$((fehler + 1))
    fi
done

echo
if [ "$fehler" -eq 0 ]; then
    echo "RESULT: PASS — alle ${#GATES[@]} Konsistenzgates sind grün."
    exit 0
fi
echo "RESULT: FAIL — ${fehler} von ${#GATES[@]} Konsistenzgates sind rot."
exit 1
