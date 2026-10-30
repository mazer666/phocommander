#!/usr/bin/env bash
# Run the license check and the advisory check.
#
# WHAT: this script runs `cargo deny check` and `cargo audit` with a cargo
#       home and a home directory on a local disk.
# WHY: two things go wrong without it.
#
#       1. The advisory database of `cargo deny` needs a lock file, and the
#          home of the sandbox lies on an NFS mount. NFS has no locking, and
#          the check fails with "failed to acquire advisory database lock:
#          failed to obtain lock file ... NFS do not support locking". The
#          database is therefore placed on a local disk.
#
#       2. The tools themselves — `cargo-deny` and `cargo-audit` — are
#          installed in the cargo home of the owner. A cargo home that is
#          somewhere else does not find them. The script therefore copies
#          the two tools into the local cargo home, and it never overwrites
#          a copy that is already there.
#
# EXAMPLE:
#     ./scripts/audit.sh
#     ./scripts/audit.sh deny
#
# The script changes nothing in the repository. It only changes where the
# tools look for their data.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# WHY: the home of the owner lies on the NFS mount, and the advisory
#       database of cargo-deny is placed under the home. The database
#       therefore needs a home of its own on a local disk.
LOCAL_HOME="${PHOCOMMANDER_LOCAL_HOME:-/tmp/phocommander-audit-home}"
LOCAL_CARGO="${PHOCOMMANDER_LOCAL_CARGO:-/tmp/phocommander-audit-cargo}"

mkdir -p "$LOCAL_HOME" "$LOCAL_CARGO" "$LOCAL_CARGO/bin"

# WHY: the tools live in the cargo home of the owner, and the local cargo
#       home does not know them. The copy happens only when the tool is
#       missing, so that a newer copy in the local home survives.
for tool in cargo-deny cargo-audit; do
    if [ ! -x "$LOCAL_CARGO/bin/$tool" ] && [ -x "$HOME/.cargo/bin/$tool" ]; then
        cp "$HOME/.cargo/bin/$tool" "$LOCAL_CARGO/bin/$tool"
    fi
done

export HOME="$LOCAL_HOME"
export CARGO_HOME="$LOCAL_CARGO"
export PATH="$CARGO_HOME/bin:$PATH"

# WHY: the toolchain of the owner stays where it is. The home of the owner
# is not reachable any more under the new home.
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
if [ ! -d "$RUSTUP_HOME" ] && [ -d "/workspace/.home/.rustup" ]; then
    export RUSTUP_HOME="/workspace/.home/.rustup"
fi
export PATH="$RUSTUP_HOME/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"

# WHY: the default is the licence check. The advisory check reports the
#       four crates that nobody maintains any more, and the decision about
#       them belongs to the owner — the decision is open in the acceptance
#       of the milestone M1.
case "${1:-deny}" in
    deny)
        cargo deny check
        ;;
    audit)
        cargo audit
        ;;
    all)
        cargo deny check
        cargo audit
        ;;
    *)
        echo "usage: $0 [deny|audit|all]" >&2
        exit 2
        ;;
esac
