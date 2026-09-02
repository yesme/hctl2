#!/usr/bin/env bash
# check_version_stamps.sh — every doc carrying a 「草案 vX.Y.Z」 header stamp (in
# its first 10 lines) must match the baseline declared in the root README.
# Scope: root README.md + docs/design/** + docs/usage.md. docs/research and
# .memo are excluded: research entries are pinned evidence snapshots and .memo
# files are historical process records — their stamps describe the past on
# purpose.
# Allowlist format (tab-separated): <repo-relative path>\t<vX.Y.Z>
# An entry permits exactly that lagging stamp; once the file is restamped the
# entry goes stale and is reported (stale entries are warnings, not failures).
set -euo pipefail

tree="${1:?usage: check_version_stamps.sh <repo_tree_root> [allowlist]}"
allowlist="${2:-}"

if [ ! -e "$tree/README.md" ]; then
    echo "repo tree not found at $tree — pass the repo//:docs_tree output" >&2
    exit 1
fi

baseline=$(grep -oE '草案 v[0-9]+\.[0-9]+\.[0-9]+' "$tree/README.md" | head -1 | awk '{print $2}')
if [ -z "$baseline" ]; then
    echo "check_version_stamps: no baseline 「草案 vX.Y.Z」 found in README.md" >&2
    exit 1
fi
echo "check_version_stamps: baseline is $baseline"

fails=$(
    {
        find "$tree/docs/design" -name '*.md' -type f
        printf '%s\n' "$tree/README.md" "$tree/docs/usage.md"
    } | sort | while IFS= read -r f; do
        rel="${f#"$tree"/}"
        stamp=$(head -10 "$f" | grep -oE '草案 v[0-9]+\.[0-9]+\.[0-9]+' | head -1 | awk '{print $2}' || true)
        [ -n "$stamp" ] || continue
        [ "$stamp" = "$baseline" ] && continue
        pinned=""
        if [ -n "$allowlist" ] && [ -f "$allowlist" ]; then
            pinned=$(awk -F'\t' -v p="$rel" '$1 == p {print $2}' "$allowlist" | head -1)
        fi
        if [ -n "$pinned" ] && [ "$pinned" = "$stamp" ]; then
            printf 'ALLOWED-LAG %s: %s (allowlisted; baseline %s)\n' "$rel" "$stamp" "$baseline"
            continue
        fi
        printf 'FAIL %s: stamp %s != baseline %s\n' "$rel" "$stamp" "$baseline"
    done
)

printf '%s\n' "$fails" | grep -v '^$' || true
if printf '%s\n' "$fails" | grep -q '^FAIL'; then
    echo "check_version_stamps: FAILED"
    exit 1
fi

# Stale allowlist entries: pinned stamp no longer differs from the file stamp.
if [ -n "$allowlist" ] && [ -f "$allowlist" ]; then
    while IFS=$'\t' read -r rel pinned; do
        case "$rel" in '' | \#*) continue ;; esac
        f="$tree/$rel"
        [ -f "$f" ] || { echo "STALE-ALLOWLIST $rel: file no longer exists"; continue; }
        stamp=$(head -10 "$f" | grep -oE '草案 v[0-9]+\.[0-9]+\.[0-9]+' | head -1 | awk '{print $2}' || true)
        if [ "$stamp" = "$baseline" ]; then
            echo "STALE-ALLOWLIST $rel: now matches baseline $baseline; remove the entry"
        fi
    done < "$allowlist"
fi

echo "check_version_stamps: OK"
exit 0
