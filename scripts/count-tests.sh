#!/usr/bin/env bash
# Count the tests of a test run.
#
# WHAT: reads the output of a test run and prints the number of the tests.
# WHY: the number of the tests is the one figure that the status board and
#       the acceptance documents carry, and it has to be right. A count
#       that adds up every number of a line — the zeros of "failed",
#       "ignored" and "measured" with it — is too high by four times the
#       number of the test binaries, and such a number looks plausible
#       enough to be believed.
#
#       The output of the test run is read with `grep -a` and not with
#       `strings`: `strings` joins lines that the program wrote into one
#       buffer without a line break, and a line that holds four results
#       is counted once instead of four times.
#
# EXAMPLE:
#     ./scripts/cargo.sh test --workspace --locked > /tmp/test.log 2>&1
#     ./scripts/count-tests.sh /tmp/test.log
#
# The script changes nothing in the repository.

set -euo pipefail

LOG="${1:-/tmp/phocommander-test.log}"

if [ ! -f "$LOG" ]; then
    echo "the log $LOG does not exist" >&2
    exit 2
fi

# WHY: the pattern names the word "passed", so that the zeros behind it
#       are not part of the number.
BESTANDEN=$(grep -aoE "test result: ok\. [0-9]+ passed" "$LOG" | grep -oE "[0-9]+" \
    | awk '{ sum += $1 } END { print sum + 0 }')

FEHLER=$(grep -acE "test result: FAILED" "$LOG" || true)

echo "Tests green : $BESTANDEN"
echo "Binaries with a failure : $FEHLER"

if [ "$FEHLER" -gt 0 ]; then
    exit 1
fi
