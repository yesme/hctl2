#!/usr/bin/env bash
# check_memo_review_baseline.sh — .memo/review/<date>-<baseline>/ hygiene:
# 1. directory names must be <8-digit date>-<token>;
# 2. every .md inside must carry a 基线 header line in its first 12 lines;
# 3. when the directory token looks like a version (vX.Y.Z), the header must
#    either contain that exact version or a version range covering it
#    (e.g. a retrospective baselined v0.7.0 → v0.12.0 covers dir v0.10.2).
# Non-version tokens (e.g. p0-plan) only require the header to exist.
# Allowlist: one repo-relative file path per line (for legitimate exceptions).
set -euo pipefail

tree="${1:?usage: check_memo_review_baseline.sh <repo_tree_root> [allowlist]}"
allowlist="${2:-}"
root="$tree/.memo/review"

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
if [ ! -d "$root" ]; then
    echo "check_memo_review_baseline: no .memo/review directory; nothing to check"
    exit 0
fi

is_allowlisted() {
    [ -n "$allowlist" ] && [ -f "$allowlist" ] || return 1
    while IFS= read -r a; do
        case "$a" in '' | \#*) continue ;; esac
        [ "$a" = "$1" ] && return 0
    done < "$allowlist"
    return 1
}

fail=0
for dir in "$root"/*/; do
    [ -d "$dir" ] || continue
    d=$(basename "$dir")
    if ! printf '%s' "$d" | grep -qE '^[0-9]{8}-.+$'; then
        echo "FAIL .memo/review/$d: directory name does not match <date>-<baseline>"
        fail=1
        continue
    fi
    token="${d#*-}"
    for f in "$dir"*.md; do
        [ -e "$f" ] || continue
        rel="${f#"$tree"/}"
        header=$(head -12 "$f" | grep -m1 -E '基线|对象' || true)
        if [ -z "$header" ]; then
            if is_allowlisted "$rel"; then
                echo "ALLOWED $rel: no 基线 header (allowlisted)"
            else
                echo "FAIL $rel: no 基线 header in first 12 lines"
                fail=1
            fi
            continue
        fi
        if printf '%s' "$token" | grep -qE '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
            versions=$(printf '%s' "$header" | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' | sort -uV || true)
            if printf '%s\n' "$versions" | grep -qx "$token"; then
                continue
            fi
            n=$(printf '%s\n' "$versions" | grep -c '^' || true)
            if [ "$n" -ge 2 ]; then
                lo=$(printf '%s\n' "$versions" | head -1)
                hi=$(printf '%s\n' "$versions" | tail -1)
                ge_lo=$(printf '%s\n%s\n' "$lo" "$token" | sort -V | head -1)
                le_hi=$(printf '%s\n%s\n' "$hi" "$token" | sort -V | tail -1)
                if [ "$ge_lo" = "$lo" ] && [ "$le_hi" = "$hi" ]; then
                    continue
                fi
            fi
            if is_allowlisted "$rel"; then
                echo "ALLOWED $rel: dir baseline $token not in header range (allowlisted)"
            else
                echo "FAIL $rel: dir baseline $token not covered by header ($header)"
                fail=1
            fi
        fi
    done
done

if [ "$fail" -ne 0 ]; then
    echo "check_memo_review_baseline: FAILED"
    exit 1
fi
echo "check_memo_review_baseline: OK"
exit 0
