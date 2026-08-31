#!/usr/bin/env bash
# materialize_repo_tree.sh — copy the linted doc set (repo root, outside the Buck2
# cell at src/) into src/build/docs/repo_tree/ so Buck2 tests can declare it as a
# resource. Buck2 cell paths cannot escape the project root (verified: "expected a
# normalized path" for `..`), so this copy is the explicit bridge. Run before
# `buck2 test root//build/docs/...`; CI does so in the docs-check job.
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../../.." && pwd)
dest="$script_dir/repo_tree"

rm -rf "$dest"
mkdir -p "$dest"
cd "$repo_root"

paths="README.md AGENTS.md CLAUDE.md CONSTRAINTS.md WRITING-GUIDE.md LICENSE docs .memo"
count=0
while IFS= read -r -d '' f; do
    mkdir -p "$dest/$(dirname -- "$f")"
    cp "$f" "$dest/$f"
    count=$((count + 1))
done < <(git ls-files -z -- $paths)

# The docs link into src/ as well; the cell content stays out of the copy, but a
# tracked-file manifest lets the link checker verify those targets exist.
git ls-files -- src > "$dest/src.manifest"

if [ "$count" -eq 0 ]; then
    echo "materialize_repo_tree: no files matched under $repo_root" >&2
    exit 1
fi
echo "materialize_repo_tree: copied $count files into build/docs/repo_tree/" >&2
