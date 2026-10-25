#!/usr/bin/env bash
# Run cargo without the poisoned standard input.
#
# WHAT: this script empties the standard input of the shell and puts a
#       wrapper around the compiler that keeps it empty.
# WHY: two things go wrong without it.
#
#       1. The standard input of the shell carries the output of the
#          *previous* command. Cargo asks the compiler for the target
#          information with "rustc - --print=file-names", and the "-" means
#          that the compiler reads its source from the standard input. The
#          compiler reads the output of the previous command as Rust source
#          and dies with "unknown start of token". The build then fails with
#          "failed to run rustc to learn about target-specific information",
#          which looks like a broken toolchain and like a full disk.
#
#       2. The proxy of rustup reads the standard input while it forwards
#          the call to the real driver, and the real driver then reads the
#          empty remainder. The result is the same failure.
#
#       The script therefore sets a wrapper around the compiler that empties
#       the standard input before the real driver starts, and it points
#       cargo at the real driver of the toolchain instead of the proxy.
#
# EXAMPLE:
#     ./scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings
#     ./scripts/cargo.sh test --workspace --locked
#     ./scripts/cargo.sh build --workspace --locked
#
# The script changes nothing in the repository. It only changes how cargo is
# started.

set -euo pipefail

# WHY: the standard input of the shell carries the output of the previous
#       command.
exec < /dev/null

# WHY: the real drivers live in the toolchain, and the proxies live in the
#       cargo directory. The proxies read the standard input.
TOOLCHAIN="${RUSTUP_HOME:-$HOME/.rustup}/toolchains"
if [ ! -d "$TOOLCHAIN" ]; then
    TOOLCHAIN="${HOME}/.rustup/toolchains"
fi

REAL=""
for directory in "$TOOLCHAIN"/*/bin; do
    if [ -x "$directory/rustc" ] && [ -x "$directory/clippy-driver" ]; then
        REAL="$directory"
        break
    fi
done

if [ -z "$REAL" ]; then
    echo "the toolchain of rust could not be found" >&2
    exit 1
fi

# The wrapper empties the standard input and then starts the real driver.
# WHY: the first parameter of the wrapper is the compiler that it replaces.
#       The real driver is the compiler itself, so the parameter is dropped.
WRAPPER="$(mktemp -d "${TMPDIR:-/tmp}/phocommander-driver-XXXXXX")"
trap 'rm -rf "$WRAPPER"' EXIT

cat > "$WRAPPER/clippy-driver" <<WRAPPER
#!/usr/bin/env bash
# WHY: the standard input is emptied before the real driver starts. The
#       driver reads its source from it, and it must not find the output of
#       the previous command there.
shift
cat > /dev/null
exec < /dev/null
exec "$REAL/clippy-driver" "\$@"
WRAPPER
chmod +x "$WRAPPER/clippy-driver"

cat > "$WRAPPER/rustc" <<WRAPPER
#!/usr/bin/env bash
# WHY: the standard input is emptied before the real compiler starts.
cat > /dev/null
exec < /dev/null
exec "$REAL/rustc" "\$@"
WRAPPER
chmod +x "$WRAPPER/rustc"

export PATH="$WRAPPER:$PATH"
export RUSTC="$WRAPPER/rustc"

# WHY: the linter replaces the compiler with the clippy driver, and the
#       driver of the toolchain replaces the proxy of rustup.
if [ "${1:-}" = "clippy" ]; then
    shift
    export RUSTC_WRAPPER="$WRAPPER/clippy-driver"
    exec cargo clippy "$@"
fi

exec cargo "$@"
