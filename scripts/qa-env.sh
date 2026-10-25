#!/usr/bin/env bash
# Shared environment for the QA runs of this session.
#
# WHY the redirections:
#   CARGO_HOME      The workspace lies on an NFS mount that does not support the
#                   lock file of the advisory database; `cargo deny` and
#                   `cargo audit` therefore need a cargo home on a local disk.
#   CARGO_TARGET_DIR A second cargo process must never write into the same
#                   target directory (specification, section 10.4), and the
#                   linker loses object files on the NFS mount.
#   CARGO_BUILD_JOBS
#                   The machine of the sandbox has three gigabytes of memory and
#                   shares them with the other sessions of the team; two
#                   linkers at the same time are enough to be killed by the
#                   OOM killer. One job at a time keeps the build alive.
#   CARGO_PROFILE_TEST_DEBUG
#                   The debug information of the test binaries is what makes
#                   the linker need gigabytes. Line tables are enough to read
#                   a backtrace and small enough to link.
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_HOME=/tmp/qa2-cargo
export CARGO_TARGET_DIR=/tmp/qa2-target
export CARGO_BUILD_JOBS=1
export CARGO_PROFILE_TEST_DEBUG=line-tables-only
export CARGO_PROFILE_DEV_DEBUG=line-tables-only
export RUST_BACKTRACE=1
mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR"
cd /workspace/phocommander || exit 1
