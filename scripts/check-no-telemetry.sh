#!/usr/bin/env bash
#
# Phocommander — telemetry and analytics dependency gate.
#
# WHAT: this script scans every Cargo.toml of the repository for a dependency
#       whose crate name belongs to the telemetry, analytics, tracking or
#       crash-reporting category, and fails with exit code 1 when it finds one.
# WHY:  the product principle "advertising-free" (no advertising, no tracking, no
#       analytics, no telemetry) is only credible if it is mechanically
#       enforced. The specification names this check as the proof of that
#       promise in section *2.3 Advertising-free* and carries the requirement
#       FR-03 with the acceptance criterion AC-M0-03.
#       The check is a heuristic over the manifests: it reads the dependency
#       tables, not the crates.io registry. That is honest about its reach —
#       it cannot see a telemetry SDK that hides behind a different name, and
#       it says so instead of pretending otherwise.
# EXAMPLE:
#       ./scripts/check-no-telemetry.sh
#
# The forbidden list below is the single place to extend the check. Add the
# crate name of a new telemetry/analytics dependency category there; the script
# needs no other change. The list may also be supplied from the outside with
# the environment variable FORBIDDEN_CRATES (space separated), which is how
# the negative test of this gate adds a violating crate without editing the
# script.
#
# Exit code 0 — no dependency of the forbidden category is referenced.
# Exit code 1 — at least one dependency of the forbidden category is referenced.

set -uo pipefail

# ---------------------------------------------------------------------------
# The forbidden list. Crate names are compared case-insensitively against the
# dependency key and against the `package = "..."` name of a renamed
# dependency.
#
# The list is deliberately explicit instead of pattern-based: a pattern such as
# "track*" would also hit legitimate crates ("tracing" is a logging facade and
# is allowed, and it is explicitly on the workspace dependency list).
# ---------------------------------------------------------------------------
DEFAULT_FORBIDDEN_CRATES="
posthog
segment
segmentio
analytics
amplitude
mixpanel
sentry
opentelemetry
opentelemetry-jaeger
opentelemetry-otlp
tracking
telemetry
plausible
matomo
gtag
google-analytics
countly
logrocket
bugsnag
rollbar
rudderstack
heap
flurry
statsd
backoff
honeycomb
datadog
newrelic
appsflyer
adjust
branch-sdk
clicky
kissmetrics
appcenter
bugly
crashlytics
firebase-analytics
firebase-crashlytics
instabug
smartlook
crazyegg
clicktale
fullstory
loggly
splunk
snowplow
tealium
segment-analytics
"

FORBIDDEN_CRATES="${FORBIDDEN_CRATES:-${DEFAULT_FORBIDDEN_CRATES}}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 2

# Collect every dependency name of every manifest.
#
# The awk program understands the three ways a dependency can be declared:
#   [dependencies]
#   serde = "1"
#   [dev-dependencies]
#   criterion = { version = "0.8", features = ["html_reports"] }
#   [dependencies.telemetry]
#   posthog = "1"
collect_dependencies() {
  find . -type d \( -name .git -o -name target -o -name node_modules -o -name vendor \) -prune \
    -o -type f -name "Cargo.toml" -print \
  | sort \
  | xargs -r awk '
      function emit(name) {
        gsub(/^[ \t]+|[ \t]+$/, "", name)
        if (name != "") {
          printf "%s\t%s\n", FILENAME, tolower(name)
        }
      }
      /^[ \t]*\[/ {
        section = $0
        gsub(/^[ \t]*\[|\][ \t]*$/, "", section)
        lowered = tolower(section)
        # "[dependencies]" and "[workspace.dependencies]" hold a table of
        # dependencies; "[dependencies.foo]" holds a single one.
        if (lowered ~ /(^|\.)dependencies$/) {
          in_table = 1
          single = ""
        } else if (lowered ~ /(^|\.)dependencies\./) {
          in_table = 0
          sub(/^.*dependencies\./, "", lowered)
          single = lowered
          emit(single)
        } else {
          in_table = 0
          single = ""
        }
        next
      }
      in_table && /^[ \t]*[A-Za-z0-9_.-]+[ \t]*=/ {
        key = $0
        sub(/[ \t]*=.*$/, "", key)
        emit(key)
        if (match($0, /package[ \t]*=[ \t]*"[^"]+"/)) {
          renamed = substr($0, RSTART, RLENGTH)
          sub(/^.*"/, "", renamed)
          sub(/".*$/, "", renamed)
          emit(renamed)
        }
      }
    '
}

violations=0
checked_names=0

while IFS=$'\t' read -r manifest name; do
  [ -n "$name" ] || continue
  checked_names=$((checked_names + 1))
  for forbidden in $FORBIDDEN_CRATES; do
    if [ "$name" = "$forbidden" ]; then
      violations=$((violations + 1))
      echo "VIOLATION  ${manifest}  references the forbidden crate '${name}'"
      echo "           category: telemetry / analytics / tracking / crash reporting"
    fi
  done
done < <(collect_dependencies)

echo
echo "Manifests scanned     : $(find . -type d \( -name .git -o -name target -o -name node_modules -o -name vendor \) -prune -o -type f -name Cargo.toml -print | wc -l)"
echo "Dependency names read : ${checked_names}"
echo "Forbidden crate names : $(echo ${FORBIDDEN_CRATES} | wc -w)"
echo "Violations            : ${violations}"

if [ "$violations" -gt 0 ]; then
  echo "RESULT: FAIL — a dependency of the telemetry category is referenced."
  echo "        Phocommander ships without telemetry (PRIVACY.md, product maximum 'Werbefrei')."
  exit 1
fi

echo "RESULT: PASS — no dependency of the telemetry or analytics category is referenced."
exit 0
