#!/usr/bin/env bash
#
# check-crate-boundaries.sh — the dependency direction of the Phocommander
# workspace.
#
# WHAT: reads the Cargo.toml of every workspace member, evaluates every path
#       dependency and checks it against the dependency rule of the project.
# WHY: the rule "apps/* -> platform/* -> crates/*" and "no crate below
#       crates/ may depend on a sibling crate below crates/" is a machine rule
#       and not a convention. A convention is broken by the first change that
#       is convenient; a machine rule stops the change and names the reason.
#       The script distinguishes a path dependency (a crate of this
#       workspace) from a registry dependency (a crate from crates.io) and
#       checks only the first kind.
# EXAMPLE: ./scripts/check-crate-boundaries.sh
#
# The rule, in full:
#
#   crates/*    may not depend on any other crate of the workspace
#   platform/*  may depend on crates/*, may not depend on apps/* or xtask
#   apps/*      may depend on platform/* and crates/*
#   tests       may depend on crates/*, may not depend on apps/* or platform/*
#   xtask       may depend on crates/*, may not depend on apps/* or platform/*
#   the root    may point at any member, but only at a member
#
# `tests` is the package that carries the tests which cross a crate boundary.
# It exists because the workspace is flat: a test that holds the core crate
# and the configuration crate against each other needs both, and no crate
# below `crates/` may depend on a sibling.
#
# Exit code 0 means that every path dependency points in an allowed
# direction. Exit code 1 means that at least one dependency violates the
# rule; every violation is named with the crate, the target and the reason.

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

root_map="$work_dir/root-map"
members="$work_dir/members"
records="$work_dir/records"

# --- the catalogue of the workspace root -------------------------------------
#
# The root manifest may point at a member of the workspace; that is how a
# shared dependency is declared once for all crates. Every such entry is
# collected, because a crate that inherits it through `workspace = true` has
# exactly this dependency.

awk '
    /^[ \t]*\[/ {
        in_section = ($0 ~ /^[ \t]*\[workspace\.dependencies\][ \t]*$/)
        next
    }
    in_section {
        line = $0
        sub(/[ \t]*#.*$/, "", line)
        if (line !~ /^[A-Za-z0-9_-]+[ \t]*=/) next
        key = line
        sub(/[ \t]*=.*$/, "", key)
        gsub(/[ \t]/, "", key)
        if (match(line, /path[ \t]*=[ \t]*"[^"]*"/)) {
            path = substr(line, RSTART, RLENGTH)
            sub(/^path[ \t]*=[ \t]*"/, "", path)
            sub(/"[ \t]*$/, "", path)
            print key "\t" path
        }
    }
' Cargo.toml > "$root_map"

# --- the members of the workspace --------------------------------------------

: > "$members"
for manifest in crates/*/Cargo.toml apps/*/Cargo.toml platform/*/Cargo.toml tests/Cargo.toml xtask/Cargo.toml; do
    if [ -f "$manifest" ]; then
        directory="${manifest%/Cargo.toml}"
        printf '%s\n' "$directory" >> "$members"
    fi
done

if [ ! -s "$members" ]; then
    echo "ERROR: no workspace member was found" >&2
    exit 1
fi

# --- the path dependencies of every member -----------------------------------
#
# A dependency is either written directly as
#
#     name = { path = "../other" }
#
# or inherited from the catalogue of the root as
#
#     name = { workspace = true }
#
# In the second case the path is looked up in the catalogue. The record has
# four fields: the member, the name of the dependency, the path and the flag
# that says "inherited from the root".

: > "$records"
while IFS= read -r member; do
    awk -v member="$member" '
        function section_kind(header,   position, prefix, rest, kind, name) {
            position = index(header, "dependencies")
            if (position == 0) return ""
            prefix = substr(header, 1, position - 1)
            rest = substr(header, position + length("dependencies"))
            kind = "dependencies"
            if (prefix ~ /dev-$/) kind = "dev-dependencies"
            else if (prefix ~ /build-$/) kind = "build-dependencies"
            name = ""
            if (rest ~ /^\./) name = substr(rest, 2)
            return kind "\t" name
        }
        function path_of(line,   found) {
            if (!match(line, /path[ \t]*=[ \t]*"[^"]*"/)) return ""
            found = substr(line, RSTART, RLENGTH)
            sub(/^path[ \t]*=[ \t]*"/, "", found)
            sub(/"[ \t]*$/, "", found)
            return found
        }
        /^[ \t]*\[/ {
            header = $0
            sub(/^[ \t]*\[/, "", header)
            sub(/\][ \t]*$/, "", header)
            result = section_kind(header)
            in_section = (result != "")
            key = ""
            if (in_section) {
                split(result, parts, "\t")
                key = parts[2]
            }
            next
        }
        {
            line = $0
            sub(/[ \t]*#.*$/, "", line)
            if (!in_section) {
                # A dependency that is written as an inline table outside of a
                # dependency section cannot be parsed by cargo, but it must
                # not slip through the gate unnoticed either. A lone
                # `path = "..."` line is not a dependency: it names the file
                # of a target, as in [lib], [[test]] and [[bench]].
                if (line ~ /^[A-Za-z0-9_-]+[ \t]*=.*path[ \t]*=[ \t]*"/) {
                    print member "\t__outside__\t" path_of(line) "\t0"
                }
                next
            }
            if (line ~ /^[ \t]*path[ \t]*=/) {
                print member "\t" key "\t" path_of(line) "\t0"
                next
            }
            if (line ~ /^[ \t]*workspace[ \t]*=/) {
                print member "\t" key "\t" "\t1"
                next
            }
            if (line ~ /^[A-Za-z0-9_-]+[ \t]*=/) {
                name = line
                sub(/[ \t]*=.*$/, "", name)
                gsub(/[ \t]/, "", name)
                inherited = (line ~ /workspace[ \t]*=[ \t]*true/) ? 1 : 0
                print member "\t" name "\t" path_of(line) "\t" inherited
                next
            }
        }
    ' "$member/Cargo.toml" >> "$records"
done < "$members"

# --- the check ----------------------------------------------------------------

echo "Phocommander - crate boundary gate"
echo "rule: apps/* -> platform/* -> crates/*, no crate below crates/ may depend"
echo "      on another crate below crates/, registry dependencies are not checked"
echo ""

awk -F'\t' -v catalogue_file="$root_map" -v member_list="$members" '
    function normalize(base, relative,   count, parts, position, stack, piece, segments, number, segment) {
        count = split(base "/" relative, parts, "/")
        stack = ""
        for (position = 1; position <= count; position++) {
            piece = parts[position]
            if (piece == "" || piece == ".") continue
            if (piece == "..") {
                number = split(stack, segments, "/")
                stack = ""
                for (segment = 1; segment < number; segment++) {
                    stack = (stack == "") ? segments[segment] : stack "/" segments[segment]
                }
                continue
            }
            stack = (stack == "") ? piece : stack "/" piece
        }
        return stack
    }
    function level_of(path) {
        if (path == "xtask") return "xtask"
        if (path == "tests") return "tests"
        if (path ~ /^crates\//) return "crates"
        if (path ~ /^apps\//) return "apps"
        if (path ~ /^platform\//) return "platform"
        return "outside"
    }
    function reason(from, to) {
        if (to == "outside") return "the target is not a member of the workspace"
        if (from == "crates" && to == "crates") return "a crate below crates/ must not depend on another crate below crates/"
        if (from == "crates") return "a crate below crates/ must not depend on a crate above crates/"
        if (from == "platform" && to == "platform") return "a crate below platform/ must not depend on another crate below platform/"
        if (from == "platform" && to == "apps") return "a crate below platform/ must not depend on an application"
        if (from == "platform" && to == "xtask") return "a crate below platform/ must not depend on the build helpers"
        if (from == "tests" && to == "apps") return "the integration tests must not depend on an application"
        if (from == "tests" && to == "platform") return "the integration tests must not depend on a platform adapter"
        if (from == "tests" && to == "xtask") return "the integration tests must not depend on the build helpers"
        if (from == "tests" && to == "tests") return "the integration tests must not depend on themselves"
        if (from == "apps" && to == "apps") return "an application must not depend on another application"
        if (from == "apps" && to == "xtask") return "an application must not depend on the build helpers"
        if (from == "xtask" && to == "apps") return "the build helpers must not depend on an application"
        if (from == "xtask" && to == "platform") return "the build helpers must not depend on a platform adapter"
        if (from == "xtask" && to == "xtask") return "the build helpers must not depend on themselves"
        return "the direction of the dependency is not allowed"
    }
    BEGIN {
        checked = 0
        violations = 0
    }
    FILENAME == catalogue_file {
        catalogue[$1] = $2
        next
    }
    FILENAME == member_list {
        members[$0] = 1
        next
    }
    {
        member = $1
        name = $2
        path = $3
        inherited = $4

        if (name == "__outside__") {
            printf "ERROR %s: the path dependency %s is written outside a dependency section\n", member, path
            violations++
            next
        }

        if (path == "" && inherited == 1) {
            path = (name in catalogue) ? catalogue[name] : ""
        }
        if (path == "") next

        target = normalize(member, path)
        from = level_of(member)
        to = level_of(target)

        if (!(target in members)) {
            printf "ERROR %s -> %s: %s\n", member, target, reason(from, "outside")
            violations++
            next
        }

        allowed = 0
        if (from == "platform" && to == "crates") allowed = 1
        if (from == "apps" && (to == "crates" || to == "platform")) allowed = 1
        if (from == "xtask" && to == "crates") allowed = 1
        if (from == "tests" && to == "crates") allowed = 1

        if (allowed) {
            printf "OK    %s -> %s (%s -> %s)\n", member, target, from, to
            checked++
        } else {
            printf "ERROR %s -> %s: %s\n", member, target, reason(from, to)
            violations++
        }
    }
    END {
        printf "\n%d path dependencies checked, %d violations\n", checked, violations
        if (violations > 0) exit 1
    }
' "$root_map" "$members" "$records"

# --- the catalogue of the root must point at members only --------------------

while read -r name path; do
    if [ -z "$path" ]; then
        continue
    fi
    if [ ! -f "$path/Cargo.toml" ]; then
        echo "ERROR the catalogue entry '$name' points at '$path', which is not a member of the workspace" >&2
        exit 1
    fi
done < "$root_map"

exit 0
