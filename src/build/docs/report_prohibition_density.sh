#!/usr/bin/env bash
# report_prohibition_density.sh — count prohibition wording per file
# (不得|不能|禁止|不允许|永不|必须拒绝). Report-only: always exits 0.
# This is a heuristic occurrence count, not a semantic audit; it feeds the
# doc-overhaul density report and the 「大修后禁令总数必须净减」 acceptance.
# Scope matches the overhaul inventory: docs/design/**, README.md, docs/usage.md.
set -euo pipefail

tree="${1:?usage: report_prohibition_density.sh <repo_tree_root>}"

if [ ! -e "$tree/README.md" ]; then
    echo "repo tree not found at $tree — pass the repo//:docs_tree output" >&2
    exit 1
fi

echo "prohibition density (occurrences of 不得|不能|禁止|不允许|永不|必须拒绝):"
{
    find "$tree/docs/design" -name '*.md' -type f
    printf '%s\n' "$tree/README.md" "$tree/docs/usage.md"
} | sort | while IFS= read -r f; do
    n=$( { grep -oE '不得|不能|禁止|不允许|永不|必须拒绝' "$f" || true; } | wc -l | tr -d ' ')
    printf '%s\t%s\n' "$n" "${f#"$tree"/}"
done | sort -rn
