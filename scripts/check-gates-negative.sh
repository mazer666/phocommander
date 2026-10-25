#!/usr/bin/env bash
#
# Phocommander — negative test of the QA gates.
#
# WHAT: this script breaks each gate on purpose, shows the gate going red,
#       restores the state and shows the gate going green again.
# WHY:  "Do not write 'the gate works' — break something on purpose, show the
#       gate going red, put it back, show it going green. That is how a gate is
#       proven to have teeth." (AGENTS.md, section 2)
#       A gate that was never seen red is a wish. The script therefore covers
#       every gate of section 13.3 of the specification and of the acceptance
#       chain that the specification names in section 13.5:
#
#         1  formatting            cargo fmt --all --check
#         2  static analysis       cargo clippy --all-targets -- -D warnings
#         3  tests                 cargo test --workspace
#         4  licence               cargo deny check
#         5  advisories            cargo audit
#         6  crate boundaries      scripts/check-crate-boundaries.sh
#         7  file size             scripts/check-file-size.sh
#         8  telemetry             scripts/check-no-telemetry.sh
#         9  link check            scripts/check-docs-links.py
#        10  requirement IDs       scripts/check-requirement-ids.py
#        11  action IDs            scripts/check-action-ids.py
#        12  configuration schema  scripts/check-config-schema.py
#        13  keymap consistency    scripts/check-keymap-consistency.py
#        14  interface profiles    scripts/check-interface-profiles.py
#        15  acceptance IDs        scripts/check-acceptance-ids.py
#
#       The script touches nothing in the repository: every broken state lives
#       in a temporary directory that is removed afterwards. The one exception
#       is the boundary gate, which has to change a manifest of the workspace
#       and puts it back immediately.
#
#       The gates that call cargo need a cargo home and a target directory of
#       their own, because two cargo processes must never write into the same
#       place (specification, section 10.4). The script uses the environment it
#       is started with and prints it.
#
# EXAMPLE:
#       ./scripts/check-gates-negative.sh
#       CARGO_HOME=/tmp/qa-cargo CARGO_TARGET_DIR=/tmp/qa-target ./scripts/check-gates-negative.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/phocommander-gate-test-XXXXXX")"
failures=0
gates=0
WITH_CARGO=1

for argument in "$@"; do
  case "$argument" in
    --without-cargo)
      # The gates that call cargo are skipped. The flag exists for the machine
      # that cannot build: the ten gates that need no build are still proven.
      WITH_CARGO=0
      ;;
    --help|-h)
      echo "usage: $0 [--without-cargo]"
      echo "  --without-cargo   skip the five gates that call cargo (11 to 15)"
      exit 0
      ;;
    *)
      echo "unknown argument: $argument" >&2
      exit 2
      ;;
  esac
done

export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

# WHY: the target directory is always the one of this run and never the one
#       of the environment. The script builds a copy of the repository, and a
#       test binary carries the path of the repository it was built in — the
#       constant of cargo. A binary that was built in the copy would look for
#       the configuration file of the copy, and the copy is gone when the
#       script ends. The test would then fail for a reason that has nothing
#       to do with the gate.
export CARGO_TARGET_DIR="$SCRATCH/target"

cleanup() {
  rm -rf "$SCRATCH"
}
trap cleanup EXIT

# Reports the result of one run against the expected exit code.
report() {
  local label="$1" expected="$2" actual="$3"
  gates=$((gates + 1))
  echo "  expected exit code ${expected}, got ${actual} — ${label}"

  # WHY: a gate does not have to end with the exit code 1. A compiler that
  # refuses the code ends with 101, and a licence checker that refuses a
  # dependency ends with its own code. What matters is that the gate says
  # "no", so a value of "nonzero" accepts every refusal.
  local failed=0
  if [ "$expected" = "nonzero" ]; then
    if [ "$actual" = "0" ]; then
      failed=1
    fi
  elif [ "$expected" != "$actual" ]; then
    failed=1
  fi

  if [ "$failed" -eq 1 ]; then
    failures=$((failures + 1))
    echo "  RESULT: FAIL — the gate did not behave as documented"
  else
    echo "  RESULT: PASS — the gate is effective"
  fi
}

# Prints the interesting lines of a log.
show() {
  local pattern="$1" file="$2"
  grep -E "$pattern" "$file" | head -n 12
}

# Copies the repository without the build directory into the scratch directory.
# WHY: the gates that call cargo must not change the working tree of the
#       owner. The copy is small — the repository without the target directory
#       is a few megabytes — and it holds the same sources, so the gate that
#       runs in the copy is the gate of the repository.
make_copy() {
  local destination="$SCRATCH/repo"
  mkdir -p "$destination"
  tar -C "$REPO_ROOT" --exclude=./target --exclude=./.git -cf "$SCRATCH/repo.tar" .
  tar -C "$destination" -xf "$SCRATCH/repo.tar"
  rm -f "$SCRATCH/repo.tar"
  echo "$destination"
}

echo "=============================================================="
echo "Phocommander — negative test of the QA gates"
echo "=============================================================="
echo "repository : $REPO_ROOT"
echo "scratch    : $SCRATCH"
echo "cargo home : $CARGO_HOME"
echo "target dir : $CARGO_TARGET_DIR"
echo

# ---------------------------------------------------------------------------
# Gate 1 — scripts/check-no-telemetry.sh
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 1 — telemetry: scripts/check-no-telemetry.sh"
echo "=============================================================="
echo "1.1 negative run (a dependency of the forbidden category is simulated"
echo "    by pointing the list of the script at a crate that IS in the tree):"
FORBIDDEN_CRATES="egui zip" "$REPO_ROOT/scripts/check-no-telemetry.sh" > "$SCRATCH/telemetry-negative.log" 2>&1
negative=$?
show "^VIOLATION|^Violations|^RESULT" "$SCRATCH/telemetry-negative.log"
report "the gate is red on a violating dependency" 1 "$negative"
echo
echo "1.2 positive run (the repository as it is):"
"$REPO_ROOT/scripts/check-no-telemetry.sh" > "$SCRATCH/telemetry-positive.log" 2>&1
positive=$?
show "^Violations|^RESULT" "$SCRATCH/telemetry-positive.log"
report "the gate is green on the repository" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 2 — scripts/check-file-size.sh
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 2 — file size: scripts/check-file-size.sh"
echo "=============================================================="
mkdir -p "$SCRATCH/size/scripts" "$SCRATCH/size/crates/demo/src"
cp "$REPO_ROOT/scripts/check-file-size.sh" "$SCRATCH/size/scripts/"
python3 - "$SCRATCH/size/crates/demo/src/huge.rs" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
path.write_text("// filler\n" * 901, encoding="utf-8")
PYTHON
echo "2.1 negative run (a source file with 901 lines, above the hard limit of 900):"
"$SCRATCH/size/scripts/check-file-size.sh" > "$SCRATCH/size-negative.log" 2>&1
negative=$?
show "^MAX|^Maximum violations|^RESULT" "$SCRATCH/size-negative.log"
report "the gate is red above the hard limit" 1 "$negative"
echo
rm -f "$SCRATCH/size/crates/demo/src/huge.rs"
python3 - "$SCRATCH/size/crates/demo/src/over.rs" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
path.write_text("// filler\n" * 601, encoding="utf-8")
PYTHON
echo "2.2 positive run with a warning (a source file with 601 lines, above the target value of 600):"
"$SCRATCH/size/scripts/check-file-size.sh" > "$SCRATCH/size-warning.log" 2>&1
positive=$?
show "^OVER|^Target-value warnings|^RESULT" "$SCRATCH/size-warning.log"
report "the gate reports the target value as a warning and stays green" 0 "$positive"
echo
rm -f "$SCRATCH/size/crates/demo/src/over.rs"
python3 - "$SCRATCH/size/crates/demo/src/small.rs" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
path.write_text("// filler\n" * 100, encoding="utf-8")
PYTHON
echo "2.3 positive run (the same file with 100 lines, below the target value of 600):"
"$SCRATCH/size/scripts/check-file-size.sh" > "$SCRATCH/size-positive.log" 2>&1
positive=$?
show "^Target-value warnings|^Maximum violations|^RESULT" "$SCRATCH/size-positive.log"
report "the gate is green below the target value" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 3 — scripts/check-docs-links.py
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 3 — link check: scripts/check-docs-links.py"
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
show "BROKEN|^Broken links|^RESULT" "$SCRATCH/links-negative.log"
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
show "^Broken links|^RESULT" "$SCRATCH/links-positive.log"
report "the gate is green on a live link" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 4 — scripts/check-crate-boundaries.sh
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 4 — crate boundaries: scripts/check-crate-boundaries.sh"
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
show "VIOLATION|violation" "$SCRATCH/boundaries-negative.log"
report "the gate is red on a dependency between two crates of the same level" 1 "$negative"
echo
cp "$RESTORE_FILE" "$TARGET_FILE"
if [ -f "$RESTORE_LOCK" ]; then
  cp "$RESTORE_LOCK" "$REPO_ROOT/Cargo.lock"
fi
echo "4.2 positive run (the dependency is removed again):"
"$REPO_ROOT/scripts/check-crate-boundaries.sh" > "$SCRATCH/boundaries-positive.log" 2>&1
positive=$?
show "violations" "$SCRATCH/boundaries-positive.log"
report "the gate is green again" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 5 — scripts/check-requirement-ids.py
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 5 — requirement identifiers: scripts/check-requirement-ids.py"
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
show "FR-22c|^RESULT" "$SCRATCH/ids-missing.log"
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
show "duplicate id|^RESULT" "$SCRATCH/ids-duplicate.log"
report "the gate is red when a requirement is written twice" 1 "$negative"
echo
echo "5.3 positive run (the catalogue of the repository):"
python3 "$REPO_ROOT/scripts/check-requirement-ids.py" > "$SCRATCH/ids-positive.log" 2>&1
positive=$?
show "^RESULT" "$SCRATCH/ids-positive.log"
report "the gate is green on the catalogue of the repository" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 6 — scripts/check-acceptance-ids.py
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 6 — acceptance identifiers: scripts/check-acceptance-ids.py"
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
show "FR-22c|^RESULT" "$SCRATCH/acceptance-missing.log"
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
show "FR-99|^RESULT" "$SCRATCH/acceptance-unknown.log"
report "the gate is red on a criterion that points at nothing" 1 "$negative"
echo
echo "6.3 positive run (the acceptance document of the repository):"
python3 "$REPO_ROOT/scripts/check-acceptance-ids.py" > "$SCRATCH/acceptance-positive.log" 2>&1
positive=$?
show "^RESULT" "$SCRATCH/acceptance-positive.log"
report "the gate is green on the acceptance document of the repository" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 7 — scripts/check-action-ids.py
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 7 — action identifiers: scripts/check-action-ids.py"
echo "=============================================================="
echo "7.1 negative run (the binding of an action is removed from a context):"
cp "$REPO_ROOT/config/keymap.toml" "$SCRATCH/keymap-missing.toml"
python3 - "$SCRATCH/keymap-missing.toml" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
lines = path.read_text(encoding="utf-8").splitlines()
kept = [line for line in lines if not line.startswith('"panel.copy" = "F5"')]
path.write_text("\n".join(kept) + "\n", encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-action-ids.py" --keymap "$SCRATCH/keymap-missing.toml" > "$SCRATCH/actions-missing.log" 2>&1
negative=$?
show "panel.copy|^RESULT" "$SCRATCH/actions-missing.log"
report "the gate is red when an action has no binding" 1 "$negative"
echo
echo "7.2 negative run (an action is bound to two keys in the same context):"
cp "$REPO_ROOT/config/keymap.toml" "$SCRATCH/keymap-conflict.toml"
python3 - "$SCRATCH/keymap-conflict.toml" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace('"panel.move" = "F6"', '"panel.move" = "F5"', 1)
path.write_text(text, encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-action-ids.py" --keymap "$SCRATCH/keymap-conflict.toml" > "$SCRATCH/actions-conflict.log" 2>&1
negative=$?
show "conflict|^RESULT" "$SCRATCH/actions-conflict.log"
report "the gate is red when a key is bound to two actions" 1 "$negative"
echo
echo "7.3 positive run (the keymap of the repository):"
python3 "$REPO_ROOT/scripts/check-action-ids.py" > "$SCRATCH/actions-positive.log" 2>&1
positive=$?
show "^RESULT" "$SCRATCH/actions-positive.log"
report "the gate is green on the keymap of the repository" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 8 — scripts/check-config-schema.py
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 8 — configuration schema: scripts/check-config-schema.py"
echo "=============================================================="
cp -a "$REPO_ROOT/config" "$SCRATCH/config-copy"
echo "8.1 negative run (the default of the conflict policy is changed):"
python3 - "$SCRATCH/config-copy/defaults.toml" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace('overwrite_policy = "ask"', 'overwrite_policy = "always"', 1)
path.write_text(text, encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-config-schema.py" --config "$SCRATCH/config-copy" > "$SCRATCH/schema-negative.log" 2>&1
negative=$?
show "overwrite_policy|^RESULT" "$SCRATCH/schema-negative.log"
report "the gate is red when a default value is changed" 1 "$negative"
echo
echo "8.2 negative run (a key of the configuration file loses its comment):"
cp "$REPO_ROOT/config/defaults.toml" "$SCRATCH/config-copy/defaults.toml"
python3 - "$SCRATCH/config-copy/defaults.toml" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
lines = path.read_text(encoding="utf-8").splitlines()
kept = [line for line in lines if not line.startswith("# Height of a file list row in points.")]
path.write_text("\n".join(kept) + "\n", encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-config-schema.py" --config "$SCRATCH/config-copy" > "$SCRATCH/schema-comment.log" 2>&1
negative=$?
show "no comment|^RESULT" "$SCRATCH/schema-comment.log"
report "the gate is red when a key has no comment" 1 "$negative"
echo
echo "8.3 positive run (the configuration of the repository):"
python3 "$REPO_ROOT/scripts/check-config-schema.py" > "$SCRATCH/schema-positive.log" 2>&1
positive=$?
show "^RESULT" "$SCRATCH/schema-positive.log"
report "the gate is green on the configuration of the repository" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 9 — scripts/check-keymap-consistency.py
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 9 — keymap consistency: scripts/check-keymap-consistency.py"
echo "=============================================================="
mkdir -p "$SCRATCH/keymap/scripts" "$SCRATCH/keymap/docs" "$SCRATCH/keymap/config"
cp "$REPO_ROOT/scripts/check-keymap-consistency.py" "$SCRATCH/keymap/scripts/"
cp "$REPO_ROOT/docs/REQUIREMENTS.md" "$REPO_ROOT/docs/KEYMAP.md" "$SCRATCH/keymap/docs/"
cp "$REPO_ROOT/config/keymap.toml" "$SCRATCH/keymap/config/"
echo "9.1 negative run (the binding of the copy action does not match the key table of the specification):"
python3 - "$SCRATCH/keymap/config/keymap.toml" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace('"panel.copy" = "F5"', '"panel.copy" = "F9"', 1)
path.write_text(text, encoding="utf-8")
PYTHON
python3 "$SCRATCH/keymap/scripts/check-keymap-consistency.py" > "$SCRATCH/keymap-negative.log" 2>&1
negative=$?
show "DEVIATION|^RESULT|specification, catalogue" "$SCRATCH/keymap-negative.log"
report "the gate is red when a binding differs from the specification" 1 "$negative"
echo
echo "9.2 positive run (the keymap of the repository):"
cp "$REPO_ROOT/config/keymap.toml" "$SCRATCH/keymap/config/keymap.toml"
python3 "$SCRATCH/keymap/scripts/check-keymap-consistency.py" > "$SCRATCH/keymap-positive.log" 2>&1
positive=$?
show "^RESULT" "$SCRATCH/keymap-positive.log"
report "the gate is green on the keymap of the repository" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# Gate 10 — scripts/check-interface-profiles.py
# ---------------------------------------------------------------------------
echo "=============================================================="
echo "Gate 10 — interface profiles: scripts/check-interface-profiles.py"
echo "=============================================================="
cp -a "$REPO_ROOT/config/profiles" "$SCRATCH/profiles-copy"
echo "10.1 negative run (the clean profile shows the toolbar):"
python3 - "$SCRATCH/profiles-copy/clean.toml" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace("show_toolbar = false", "show_toolbar = true", 1)
path.write_text(text, encoding="utf-8")
PYTHON
python3 "$REPO_ROOT/scripts/check-interface-profiles.py" --directory "$SCRATCH/profiles-copy" > "$SCRATCH/profiles-negative.log" 2>&1
negative=$?
show "clean|^RESULT" "$SCRATCH/profiles-negative.log"
report "the gate is red when a profile does not hold the values of its name" 1 "$negative"
echo
echo "10.2 positive run (the profiles of the repository):"
python3 "$REPO_ROOT/scripts/check-interface-profiles.py" > "$SCRATCH/profiles-positive.log" 2>&1
positive=$?
show "^RESULT" "$SCRATCH/profiles-positive.log"
report "the gate is green on the profiles of the repository" 0 "$positive"
echo

# ---------------------------------------------------------------------------
# The gates that call cargo. They run in a copy of the repository, so that the
# working tree of the owner stays untouched.
# ---------------------------------------------------------------------------
if [ "$WITH_CARGO" -eq 0 ]; then
  echo "=============================================================="
  echo "Gates 11 to 15 (formatting, static analysis, tests, licence,"
  echo "advisories) were SKIPPED — the run was started with --without-cargo."
  echo "=============================================================="
  echo
  echo "=============================================================="
  if [ "$failures" -eq 0 ]; then
    echo "RESULT: PASS — every gate that ran turned red on a broken state and green again."
    echo "Gates proved: ${gates}"
    exit 0
  fi
  echo "RESULT: FAIL — ${failures} of ${gates} runs did not behave as documented."
  exit 1
fi

COPY="$(make_copy)"
COPY_FILE="$COPY/crates/phocommander-core/src/checksum.rs"
cp "$COPY_FILE" "$SCRATCH/checksum.rs"

echo "=============================================================="
echo "Gate 11 — formatting: cargo fmt --all --check"
echo "=============================================================="
echo "11.1 negative run (one source file is not formatted):"
python3 - "$COPY_FILE" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text + "\npub fn qa_gate_probe( ) -> u32 { return 1 }\n"
path.write_text(text, encoding="utf-8")
PYTHON
(cd "$COPY" && cargo fmt --all --check < /dev/null) > "$SCRATCH/fmt-negative.log" 2>&1
negative=$?
show "^Diff in|^error" "$SCRATCH/fmt-negative.log"
report "the gate is red on a source file that is not formatted" 1 "$negative"
echo
cp "$SCRATCH/checksum.rs" "$COPY_FILE"
echo "11.2 positive run (the file is formatted again):"
(cd "$COPY" && cargo fmt --all --check < /dev/null) > "$SCRATCH/fmt-positive.log" 2>&1
positive=$?
show "^Diff in|^error" "$SCRATCH/fmt-positive.log"
report "the gate is green again" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 12 — static analysis: cargo clippy --all-targets -- -D warnings"
echo "=============================================================="
echo "12.1 negative run (one source file carries a lint of clippy):"
python3 - "$COPY_FILE" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text + """
/// A probe that trips the lint `len_zero` of clippy.
pub fn qa_gate_probe(values: &[u32]) -> u32 {
    if values.len() == 0 {
        0
    } else {
        values[0]
    }
}
"""
path.write_text(text, encoding="utf-8")
PYTHON
# WHY: the gate checks the crate that carries the probe and not the whole
#       workspace. The gate proves that a lint turns the run red — it does
#       not prove that every crate of the workspace is free of lints, and
#       the lint of the whole workspace does not fit into the memory of the
#       sandbox beside the build that the script itself makes.
(cd "$COPY" && cargo clippy -p phocommander-core --all-targets --locked -- -D warnings < /dev/null) > "$SCRATCH/clippy-negative.log" 2>&1
negative=$?
show "len_zero|^error" "$SCRATCH/clippy-negative.log"
report "the gate is red on a lint of clippy" "nonzero" "$negative"
echo
cp "$SCRATCH/checksum.rs" "$COPY_FILE"
echo "12.2 positive run (the lint is removed again):"
(cd "$COPY" && cargo clippy -p phocommander-core --all-targets --locked -- -D warnings < /dev/null) > "$SCRATCH/clippy-positive.log" 2>&1
positive=$?
show "len_zero|^error" "$SCRATCH/clippy-positive.log"
report "the gate is green again" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 13 — tests: cargo test --workspace"
echo "=============================================================="
TEST_FILE="$COPY/tests/integration/config_defaults.rs"
cp "$TEST_FILE" "$SCRATCH/config_defaults.rs"
echo "13.1 negative run (one test expects a value that is wrong):"
python3 - "$TEST_FILE" <<'PYTHON'
import sys
from pathlib import Path
path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace(
    "fn the_shipped_configuration_file_is_accepted()",
    "fn the_shipped_configuration_file_is_accepted() {\n    assert_eq!(1, 2, \"the QA gate probe fails on purpose\");",
    1,
)
path.write_text(text, encoding="utf-8")
PYTHON
# WHY: the gate runs the test package of the probe and not the whole
#       workspace. The gate proves that a failing test turns the run red —
#       it does not prove that every crate of the workspace compiles, and
#       the whole workspace does not fit into the memory of the sandbox
#       beside the build that the script itself makes.
(cd "$COPY" && cargo test -p phocommander-integration-tests --test config_defaults --locked < /dev/null) > "$SCRATCH/test-negative.log" 2>&1
negative=$?
show "the QA gate probe fails on purpose|test result: FAILED|^error" "$SCRATCH/test-negative.log"
report "the gate is red when a test fails" "nonzero" "$negative"
echo
cp "$SCRATCH/config_defaults.rs" "$TEST_FILE"
echo "13.2 positive run (the test is repaired):"
(cd "$COPY" && cargo test -p phocommander-integration-tests --test config_defaults --locked < /dev/null) > "$SCRATCH/test-positive.log" 2>&1
positive=$?
show "test result: ok|^error|failed to run" "$SCRATCH/test-positive.log"
report "the gate is green again" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 14 — licence: cargo deny check"
echo "=============================================================="
mkdir -p "$SCRATCH/copyleft/src"
cat > "$SCRATCH/copyleft/Cargo.toml" <<'TOML'
[package]
name = "qa-copyleft-probe"
version = "0.1.0"
edition = "2024"
license = "GPL-3.0-only"
TOML
echo "//! A probe that carries a copyleft licence." > "$SCRATCH/copyleft/src/lib.rs"
CORE_MANIFEST="$COPY/crates/phocommander-core/Cargo.toml"
cp "$CORE_MANIFEST" "$SCRATCH/core-Cargo.toml"
cp "$COPY/Cargo.lock" "$SCRATCH/copy-Cargo.lock"
echo "14.1 negative run (a copyleft dependency enters the dependency graph):"
python3 - "$CORE_MANIFEST" "$SCRATCH/copyleft" <<'PYTHON'
import sys
from pathlib import Path
manifest = Path(sys.argv[1])
probe = Path(sys.argv[2]).resolve()
text = manifest.read_text(encoding="utf-8")
text = text.replace(
    "[dependencies]",
    "[dependencies]\nqa-copyleft-probe = { path = \"%s\" }" % probe,
    1,
)
manifest.write_text(text, encoding="utf-8")
PYTHON
(cd "$COPY" && cargo deny check licenses < /dev/null) > "$SCRATCH/deny-negative.log" 2>&1
negative=$?
show "license|error|^error" "$SCRATCH/deny-negative.log"
report "the gate is red on a copyleft dependency" "nonzero" "$negative"
echo
cp "$SCRATCH/core-Cargo.toml" "$CORE_MANIFEST"
cp "$SCRATCH/copy-Cargo.lock" "$COPY/Cargo.lock"
echo "14.2 positive run (the copyleft dependency is removed again):"
(cd "$COPY" && cargo deny check licenses < /dev/null) > "$SCRATCH/deny-positive.log" 2>&1
positive=$?
show "licenses ok|^error" "$SCRATCH/deny-positive.log"
report "the gate is green again" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 15 — advisories: cargo audit"
echo "=============================================================="
echo "15.1 positive run (the dependency graph of the repository):"
(cd "$COPY" && cargo audit < /dev/null) > "$SCRATCH/audit-positive.log" 2>&1
positive=$?
show "vulnerabilities|^error" "$SCRATCH/audit-positive.log"
report "the gate is green on the dependency graph of the repository" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 16 — localisation: scripts/check-i18n-keys.py"
echo "=============================================================="
echo "16.1 negative run (a key is missing in the French catalogue):"
cp "$COPY/crates/phocommander-i18n/locales/fr/panel.ftl" "$SCRATCH/fr-panel.ftl"
printf '\n# The key that is missing for the test.\n' >> "$COPY/crates/phocommander-i18n/locales/fr/panel.ftl"
python3 - "$COPY/crates/phocommander-i18n/locales/fr/panel.ftl" <<'PYTHON'
import sys, pathlib
path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
# Ein Schluessel wird aus dem franzosischen Katalog entfernt.
text = "\n".join(
    line for line in text.split("\n")
    if not line.startswith("panel-column-name =")
)
path.write_text(text, encoding="utf-8")
PYTHON
(cd "$COPY" && python3 scripts/check-i18n-keys.py) > "$SCRATCH/i18n-keys-negative.log" 2>&1
negative=$?
show "fr|missing|FAIL" "$SCRATCH/i18n-keys-negative.log"
report "the gate is red on a missing translation" "nonzero" "$negative"
echo
cp "$SCRATCH/fr-panel.ftl" "$COPY/crates/phocommander-i18n/locales/fr/panel.ftl"
echo "16.2 positive run (the catalogues are complete again):"
(cd "$COPY" && python3 scripts/check-i18n-keys.py) > "$SCRATCH/i18n-keys-positive.log" 2>&1
positive=$?
show "RESULT: PASS|same" "$SCRATCH/i18n-keys-positive.log"
report "the gate is green again" 0 "$positive"
echo

echo "=============================================================="
echo "Gate 17 — hardcoded texts: scripts/check-i18n-hardcoded.py"
echo "=============================================================="
echo "17.1 negative run (a text is written down in the source):"
cp "$COPY/crates/phocommander-ui/src/preview.rs" "$SCRATCH/preview.rs"
python3 - "$COPY/crates/phocommander-ui/src/preview.rs" <<'PYTHON'
import sys, pathlib
path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
# Ein Text wird eingebaut, wie es ein Entwickler tun wuerde.
text = text.replace(
    "pub struct Preview {",
    "pub struct Preview {\n    /// A text that was written down by accident.\n    _untranslated: String,",
    1,
)
# Der Text selbst steht als Literal in der Datei — so, wie es ein
# Entwickler tun wuerde, der den Katalog nicht kennt.
zeilen = text.split("\n")
for num, zeile in enumerate(zeilen):
    if zeile.startswith("use "):
        zeilen[num:num] = [
            "",
            "/// The text that was written down by accident.",
            'const UNTRANSLATED: &str = "The file could not be read.";',
        ]
        break
text = "\n".join(zeilen)
path.write_text(text, encoding="utf-8")
PYTHON
(cd "$COPY" && python3 scripts/check-i18n-hardcoded.py) > "$SCRATCH/i18n-hardcoded-negative.log" 2>&1
negative=$?
show "Visible texts|FAIL" "$SCRATCH/i18n-hardcoded-negative.log"
report "the gate is red on a text that is written down in the source" "nonzero" "$negative"
echo
cp "$SCRATCH/preview.rs" "$COPY/crates/phocommander-ui/src/preview.rs"
echo "17.2 positive run (the text is removed again):"
(cd "$COPY" && python3 scripts/check-i18n-hardcoded.py) > "$SCRATCH/i18n-hardcoded-positive.log" 2>&1
positive=$?
show "RESULT: PASS|Visible texts" "$SCRATCH/i18n-hardcoded-positive.log"
report "the gate is green again" 0 "$positive"
echo

echo "=============================================================="
if [ "$failures" -eq 0 ]; then
  echo "RESULT: PASS — every gate turned red on a broken state and green again."
  echo "Gates proved: ${gates}"
  exit 0
fi
echo "RESULT: FAIL — ${failures} of ${gates} runs did not behave as documented."
exit 1
