#!/usr/bin/env bash
# check_dead_names.sh — retired names must not appear in current docs.
# Scope: all *.md in the doc tree (repo//:docs_tree) EXCEPT
#   docs/design/references/decision-history.md, .memo/**, docs/research/**
# (historical records and evidence may legally mention retired names).
# ASCII names match on token boundaries ([^A-Za-z0-9_]); CJK names match as
# substrings. Allowlist format (tab-separated):
#   <repo-relative path>\t<substring>   — exempt lines containing the substring
#   <repo-relative path>                — exempt the whole file (use sparingly)
# Portable: bash 3.2-compatible (no mapfile), no GNU-only grep flags.
set -euo pipefail

tree="${1:?usage: check_dead_names.sh <repo_tree_root> <dead_names.txt> [allowlist]}"
names_file="${2:?usage: check_dead_names.sh <repo_tree_root> <dead_names.txt> [allowlist]}"
allowlist="${3:-}"

if [ ! -e "$tree/README.md" ]; then
    if [ -x "build/docs/materialize_repo_tree.sh" ]; then
        echo "note: $tree missing; auto-running build/docs/materialize_repo_tree.sh" >&2
        build/docs/materialize_repo_tree.sh >&2
    fi
fi
if [ ! -e "$tree/README.md" ]; then
    echo "repo tree not found at $tree — run src/build/docs/materialize_repo_tree.sh first" >&2
    exit 1
fi

scope_list=$(mktemp "${TMPDIR:-/tmp}/dead-names-scope.XXXXXX")
trap 'rm -f "$scope_list"' EXIT

find "$tree" -name '*.md' -type f \
    ! -path "$tree/.memo/*" \
    ! -path "$tree/docs/research/*" \
    ! -path "$tree/docs/design/references/decision-history.md" \
    | sort > "$scope_list"

if [ ! -s "$scope_list" ]; then
    echo "check_dead_names: empty scan scope; refusing to pass vacuously" >&2
    exit 1
fi

is_allowed() {
    # $1 = repo-relative path, $2 = hit line number, $3 = absolute hit path
    [ -n "$allowlist" ] && [ -f "$allowlist" ] || return 1
    local a_path a_sub
    while IFS=$'\t' read -r a_path a_sub; do
        case "$a_path" in '' | \#*) continue ;; esac
        [ "$a_path" = "$1" ] || continue
        if [ -z "${a_sub:-}" ]; then
            return 0
        fi
        if sed -n "$2p" "$3" | grep -qF "$a_sub"; then
            return 0
        fi
    done < "$allowlist"
    return 1
}

fail=0
while IFS=$'\t' read -r name _note; do
    case "$name" in '' | \#*) continue ;; esac
    if LC_ALL=C printf '%s' "$name" | LC_ALL=C grep -q '[^ -~]'; then
        pattern="$name"
    else
        pattern="(^|[^A-Za-z0-9_])${name}([^A-Za-z0-9_]|\$)"
    fi
    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        hit_path="${hit%%:*}"
        rest="${hit#*:}"
        hit_line="${rest%%:*}"
        rel="${hit_path#"$tree"/}"
        if is_allowed "$rel" "$hit_line" "$hit_path"; then
            echo "ALLOWED $rel:$hit_line: $name"
        else
            echo "FAIL $rel:$hit_line: retired name '$name'"
            fail=1
        fi
    done < <(xargs grep -nHE "$pattern" < "$scope_list" 2>/dev/null || true)
done < "$names_file"

if [ "$fail" -ne 0 ]; then
    echo "check_dead_names: FAILED"
    exit 1
fi
echo "check_dead_names: OK"
exit 0
