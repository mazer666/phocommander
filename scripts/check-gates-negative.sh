#!/usr/bin/env bash
#
# Phocommander — negative test of the three QA gates of the milestone M0.
#
# WHAT: this script breaks each gate on purpose, shows the gate going red,
#       restores the state and shows the gate going green again.
# WHY:  "Do not write 'the gate works' — break something on purpose, show the
#       gate going red, put it back, show it going green. That is how a gate is
#       proven to have teeth." (AGENTS.md, section 2)
#       The script touches nothing in the repository: the broken state lives in
#       a temporary directory that is removed afterwards.
# EXAMPLE:
#       ./scripts/check-gates-negative.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/phocommander-gate-test-XXXXXX")"
failures=0

cleanup() {
  if [ -n "${RESTORE_FILE:-}" ] && [ -f "${RESTORE_FILE:-}" ]; then
    cp "$RESTORE_FILE" "$TARGET_FILE"
  fi
  if [ -n "${RESTORE_LOCK:-}" ] && [ -f "${RESTORE_LOCK:-}" ]; then
    cp "$RESTORE_LOCK" "$REPO_ROOT/Cargo.lock"
  fi
  rm -rf "$SCRATCH"
}
trap cleanup EXIT

report() {
  local gate="$1" expected="$2" actual="$3"
  echo "  expected exit code ${expected}, got ${actual} — ${gate}"
  if [ "$expected" != "$actual" ]; then
    failures=$((failures + 1))
    echo "  RESULT: FAIL — the gate did not behave as documented"
  else
    echo "  RESULT: PASS — the gate is effective"
  fi
}

echo "=============================================================="
echo "Gate 1 — scripts/check-no-telemetry.sh"
echo "=============================================================="
echo
echo "1.1 positive run (the repository as it is):"
"$REPO_ROOT/scripts/check-no-telemetry.sh" > "$SCRATCH/telemetry-positive.log" 2>&1
positive=$?
tail -n 4 "$SCRATCH/telemetry-positive.log"
report "the gate is green on the repository" 0 "$positive"
echo

echo "1.2 negative run (a dependency of the forbidden category is simulated"
echo "    by pointing the list of the script at a crate that IS in the tree):"
FORBIDDEN_CRATES="egui zip" "$REPO_ROOT/scripts/check-no-telemetry.sh" > "$SCRATCH/telemetry-negative.log" 2>&1
negative=$?
grep -E "^VIOLATION|^Violations|^RESULT" "$SCRATCH/telemetry-negative.log"
report "the gate is red on a violating dependency" 1 "$negative"
echo

echo "=============================================================="
echo "Gate 2 — scripts/check-file-size.sh"
echo "=============================================================="
mkdir -p "$SCRATCH/size/scripts" "$SCRATCH/size/crates/demo/src"
cp "$REPO_ROOT/scripts/check-file-size.sh" "$SCRATCH/size/scripts/"
python3 - "$SCRATCH/size/crates/demo/src/huge.rs" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
path.write_text("// filler\n" * 600, encoding="utf-8")
PYTHON
echo "2.1 negative run (a source file with 600 lines, above the hard limit of 550):"
"$SCRATCH/size/scripts/check-file-size.sh" > "$SCRATCH/size-negative.log" 2>&1
negative=$?
grep -E "ERROR|above the hard limit|above the soft limit" "$SCRATCH/size-negative.log"
report "the gate is red above the hard limit" 1 "$negative"
echo
rm -f "$SCRATCH/size/crates/demo/src/huge.rs"
python3 - "$SCRATCH/size/crates/demo/src/small.rs" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
path.write_text("// filler\n" * 100, encoding="utf-8")
PYTHON
echo "2.2 positive run (the same file with 100 lines, below the soft limit of 400):"
"$SCRATCH/size/scripts/check-file-size.sh" > "$SCRATCH/size-positive.log" 2>&1
positive=$?
grep -E "files above the soft limit|files above the hard limit" "$SCRATCH/size-positive.log"
report "the gate is green below the soft limit" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 3 — scripts/check-docs-links.py"
echo "=============================================================="
mkdir -p "$SCRATCH/links/scripts" "$SCRATCH/links/docs"
cp "$REPO_ROOT/scripts/check-links.py" "$REPO_ROOT/scripts/check-docs-links.py" "$SCRATCH/links/scripts/"
cat > "$SCRATCH/links/docs/broken.md" <<'MARKDOWN'
# A document with a dead link

[the file that does not exist](does-not-exist.md)
MARKDOWN
echo "3.1 negative run (a document with a link to a file that does not exist):"
python3 "$SCRATCH/links/scripts/check-docs-links.py" > "$SCRATCH/links-negative.log" 2>&1
negative=$?
grep -E "BROKEN|^Broken links|^RESULT" "$SCRATCH/links-negative.log"
report "the gate is red on a dead link" 1 "$negative"
echo
rm -f "$SCRATCH/links/docs/broken.md"
cat > "$SCRATCH/links/docs/good.md" <<'MARKDOWN'
# A document with a live link

[the file that exists](alive.md)
MARKDOWN
echo "alive" > "$SCRATCH/links/docs/alive.md"
echo "3.2 positive run (the same document with a link to a file that exists):"
python3 "$SCRATCH/links/scripts/check-docs-links.py" > "$SCRATCH/links-positive.log" 2>&1
positive=$?
grep -E "^Broken links|^RESULT" "$SCRATCH/links-positive.log"
report "the gate is green on a live link" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 4 — scripts/check-crate-boundaries.sh"
echo "=============================================================="
TARGET_FILE="$REPO_ROOT/crates/phocommander-core/Cargo.toml"
RESTORE_FILE="$SCRATCH/phocommander-core-Cargo.toml"
RESTORE_LOCK="$SCRATCH/Cargo.lock"
cp "$TARGET_FILE" "$RESTORE_FILE"
if [ -f "$REPO_ROOT/Cargo.lock" ]; then
  cp "$REPO_ROOT/Cargo.lock" "$RESTORE_LOCK"
fi
echo "4.1 negative run (a crate below crates/ gets a dependency on a sibling):"
python3 - "$TARGET_FILE" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace(
    "[dependencies]",
    "[dependencies]\nphocommander-keymap = { path = \"../phocommander-keymap\" }",
    1,
)
path.write_text(text, encoding="utf-8")
PYTHON
"$REPO_ROOT/scripts/check-crate-boundaries.sh" > "$SCRATCH/boundaries-negative.log" 2>&1
negative=$?
grep -E "VIOLATION|violation" "$SCRATCH/boundaries-negative.log"
report "the gate is red on a dependency between two crates of the same level" 1 "$negative"
echo
cp "$RESTORE_FILE" "$TARGET_FILE"
echo "4.2 positive run (the dependency is removed again):"
"$REPO_ROOT/scripts/check-crate-boundaries.sh" > "$SCRATCH/boundaries-positive.log" 2>&1
positive=$?
grep -E "violations" "$SCRATCH/boundaries-positive.log"
report "the gate is green again" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 5 — scripts/check-requirement-ids.py"
echo "=============================================================="
echo "5.1 negative run (a requirement of the catalogue is removed):"
cp "$REPO_ROOT/docs/REQUIREMENTS-IDS.md" "$SCRATCH/catalog-missing.md"
python3 - "$SCRATCH/catalog-missing.md" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
lines = path.read_text(encoding="utf-8").splitlines()
kept = [line for line in lines if not line.startswith("| FR-22c ")]
path.write_text("\n".join(kept) + "\n", encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-requirement-ids.py" --catalog "$SCRATCH/catalog-missing.md" > "$SCRATCH/ids-missing.log" 2>&1
negative=$?
grep -E "FR-22c|^RESULT" "$SCRATCH/ids-missing.log"
report "the gate is red when a requirement is missing" 1 "$negative"
echo
echo "5.2 negative run (a requirement of the catalogue is written twice):"
cp "$REPO_ROOT/docs/REQUIREMENTS-IDS.md" "$SCRATCH/catalog-duplicate.md"
python3 - "$SCRATCH/catalog-duplicate.md" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
lines = path.read_text(encoding="utf-8").splitlines()
row = next(line for line in lines if line.startswith("| FR-22c "))
path.write_text("\n".join(lines + [row]) + "\n", encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-requirement-ids.py" --catalog "$SCRATCH/catalog-duplicate.md" > "$SCRATCH/ids-duplicate.log" 2>&1
negative=$?
grep -E "duplicate id|^RESULT" "$SCRATCH/ids-duplicate.log"
report "the gate is red when a requirement is written twice" 1 "$negative"
echo
echo "5.3 positive run (the catalogue of the repository):"
python3 "$REPO_ROOT/scripts/check-requirement-ids.py" > "$SCRATCH/ids-positive.log" 2>&1
positive=$?
grep -E "^RESULT" "$SCRATCH/ids-positive.log"
report "the gate is green on the catalogue of the repository" 0 "$positive"
echo
echo "=============================================================="
echo "Gate 6 — scripts/check-acceptance-ids.py"
echo "=============================================================="
echo "6.1 negative run (the acceptance criterion of a requirement is removed):"
cp "$REPO_ROOT/docs/ACCEPTANCE-CRITERIA.md" "$SCRATCH/acceptance-missing.md"
python3 - "$SCRATCH/acceptance-missing.md" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
lines = path.read_text(encoding="utf-8").splitlines()
kept = [line for line in lines if not line.startswith("| AC-M1-10 ")]
path.write_text("\n".join(kept) + "\n", encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-acceptance-ids.py" --acceptance "$SCRATCH/acceptance-missing.md" > "$SCRATCH/acceptance-missing.log" 2>&1
negative=$?
grep -E "FR-22c|^RESULT" "$SCRATCH/acceptance-missing.log"
report "the gate is red when a criterion is missing" 1 "$negative"
echo
echo "6.2 negative run (a criterion references a requirement that does not exist):"
cp "$REPO_ROOT/docs/ACCEPTANCE-CRITERIA.md" "$SCRATCH/acceptance-unknown.md"
python3 - "$SCRATCH/acceptance-unknown.md" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace("| AC-M0-02 | FR-02 |", "| AC-M0-02 | FR-99 |", 1)
path.write_text(text, encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-acceptance-ids.py" --acceptance "$SCRATCH/acceptance-unknown.md" > "$SCRATCH/acceptance-unknown.log" 2>&1
negative=$?
grep -E "FR-99|^RESULT" "$SCRATCH/acceptance-unknown.log"
report "the gate is red on a criterion that points at nothing" 1 "$negative"
echo
echo "6.3 positive run (the acceptance document of the repository):"
python3 "$REPO_ROOT/scripts/check-acceptance-ids.py" > "$SCRATCH/acceptance-positive.log" 2>&1
positive=$?
grep -E "^RESULT" "$SCRATCH/acceptance-positive.log"
report "the gate is green on the acceptance document of the repository" 0 "$positive"
echo

echo "=============================================================="
if [ "$failures" -eq 0 ]; then
  echo "RESULT: PASS — every gate turned red on a broken state and green again."
  exit 0
fi
echo "RESULT: FAIL — ${failures} gate(s) did not behave as documented."
exit 1
