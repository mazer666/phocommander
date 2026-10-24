#!/usr/bin/env bash
#
# Phocommander — source file size gate.
#
# WHAT: this script counts the lines of every source file in the repository and
#       reports the files that exceed the limits of the specification.
# WHY:  the specification fixes two limits for every source file — 400 lines as
#       a soft limit (a file above it needs a documented reason in its module
#       header) and 550 lines as a hard limit (a file above it is rejected and
#       must be split along a responsibility). A limit that is never checked is
#       a wish, so the check runs on every change in the CI.
#       The script is also the evidence for the acceptance criterion AC-M0-49
#       ("no source file exceeds 550 lines").
# EXAMPLE:
#       ./scripts/check-file-size.sh              # report and exit
#       ./scripts/check-file-size.sh --strict    # also fail on the soft limit
#
# Exit code 0 — no file exceeds the hard limit (soft-limit violations are
#               reported as warnings).
# Exit code 1 — at least one file exceeds the hard limit of 550 lines.

set -uo pipefail

SOFT_LIMIT=400
HARD_LIMIT=550

# Directories that hold generated or vendored code. They are not first-party
# sources and are therefore not measured.
SKIP_DIRS=(
  ".git"
  "target"
  "node_modules"
  "vendor"
  ".mavis"
  ".scratch"
)

# File extensions that count as source code. Extend this list when the project
# adds a language whose files shall obey the same limit.
DEFAULT_EXTENSIONS="rs"

STRICT=0
for argument in "$@"; do
  case "$argument" in
    --strict) STRICT=1 ;;
    --help|-h)
      echo "usage: $0 [--strict]"
      echo "  --strict   also fail the run when the soft limit of ${SOFT_LIMIT} lines is exceeded"
      exit 0
      ;;
    *)
      echo "unknown argument: $argument" >&2
      exit 2
      ;;
  esac
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 2

EXTENSIONS="${FILE_SIZE_EXTENSIONS:-$DEFAULT_EXTENSIONS}"

soft_violations=0
hard_violations=0
measured=0

for extension in $EXTENSIONS; do
  find_expression=(
    . -type d \( -name .git -o -name target -o -name node_modules -o -name vendor -o -name .mavis -o -name .scratch \) -prune
    -o -type f -name "*.$extension" -print
  )

  while IFS= read -r file; do
    lines=$(wc -l < "$file")
    measured=$((measured + 1))
    if [ "$lines" -gt "$HARD_LIMIT" ]; then
      hard_violations=$((hard_violations + 1))
      echo "HARD  ${lines} lines  ${file}"
    elif [ "$lines" -gt "$SOFT_LIMIT" ]; then
      soft_violations=$((soft_violations + 1))
      echo "SOFT  ${lines} lines  ${file}"
    fi
  done < <(find "${find_expression[@]}" | sort)
done

echo
echo "Source files measured : ${measured} (extensions: ${EXTENSIONS})"
echo "Soft limit            : ${SOFT_LIMIT} lines (warning)"
echo "Hard limit            : ${HARD_LIMIT} lines (failure)"
echo "Soft-limit violations : ${soft_violations}"
echo "Hard-limit violations : ${hard_violations}"

if [ "$hard_violations" -gt 0 ]; then
  echo "RESULT: FAIL — a source file exceeds the hard limit of ${HARD_LIMIT} lines. Split it."
  exit 1
fi

if [ "$soft_violations" -gt 0 ]; then
  if [ "$STRICT" -eq 1 ]; then
    echo "RESULT: FAIL (strict) — a source file exceeds the soft limit of ${SOFT_LIMIT} lines."
    exit 1
  fi
  echo "RESULT: PASS with warnings — ${soft_violations} file(s) exceed the soft limit of ${SOFT_LIMIT} lines."
  echo "        Each of them must name the reason in its module header."
  exit 0
fi

echo "RESULT: PASS — every source file stays below ${SOFT_LIMIT} lines."
exit 0
