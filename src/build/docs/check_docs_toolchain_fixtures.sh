#!/usr/bin/env bash
# Adversarial fixtures for the documentation terminology checkers and the
# PR-contract "existing script grew by more than 100 lines" signal.
# Portable: bash 3.2, no mapfile, no GNU-only flags.
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
lint="${DOC_STYLE_LINT:-$script_dir/doc_style_lint.pl}"
vocab="${SPEC_VOCABULARY:-$script_dir/testdata/spec-vocabulary.md}"
names_file="${IMPLEMENTATION_NAMES:-$script_dir/implementation_names.txt}"
docs_tree="${DOCS_TREE:-}"
pr_contract="${PR_CONTRACT_YML:-}"
allowlist_dir="${ALLOWLIST_DIR:-$script_dir}"

if [ -z "$docs_tree" ]; then
    docs_tree="$(CDPATH= cd -- "$script_dir/../../.." && pwd)"
fi
if [ -z "$pr_contract" ]; then
    pr_contract="$docs_tree/.github/workflows/pr-contract.yml"
fi

[[ -f "$lint" ]] || { echo "missing checker: $lint" >&2; exit 1; }
[[ -f "$vocab" ]] || { echo "missing vocabulary fixture: $vocab" >&2; exit 1; }
[[ -f "$names_file" ]] || { echo "missing implementation names: $names_file" >&2; exit 1; }
[[ -f "$pr_contract" ]] || { echo "missing PR contract workflow: $pr_contract" >&2; exit 1; }
[[ -f "$docs_tree/docs/design/spec/README.md" ]] || {
    echo "docs tree is missing spec README: $docs_tree" >&2
    exit 1
}

failures=0
work="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-doc-style-fixtures.XXXXXX")"
trap 'rm -rf "$work"' EXIT

note() { printf '%s\n' "$*"; }
fail() {
    printf 'FAIL %s\n' "$*" >&2
    failures=$((failures + 1))
}

architecture_files='README.md architecture.md context.md participant.md project.md repo.md run.md task.md vision.md'

write_tree() {
    local tree="$1"
    local file
    mkdir -p "$tree/docs/design/spec"
    printf '# fixture repo\n' >"$tree/README.md"
    printf '# fixture\n' >"$tree/docs/design/vision.md"
    for file in $architecture_files; do
        printf '# fixture\n' >"$tree/docs/design/$file"
    done
    cp "$vocab" "$tree/docs/design/spec/README.md"
}

write_file() {
    local path="$1"
    cat >"$path"
}

run_lint() {
    local mode="$1"
    local tree="$2"
    local allow="$3"
    shift 3
    perl "$lint" "$mode" "$tree" "$allow" "$@"
}

expect_pass() {
    local desc="$1"
    shift
    local out rc
    out="$("$@" 2>&1)" && rc=0 || rc=$?
    if [ "$rc" -ne 0 ]; then
        fail "$desc (expected pass, exit $rc)"
        printf '%s\n' "$out" >&2
        return
    fi
    note "PASS $desc"
}

expect_fail_matching() {
    local desc="$1"
    local needle="$2"
    shift 2
    local out rc
    out="$("$@" 2>&1)" && rc=0 || rc=$?
    if [ "$rc" -eq 0 ]; then
        fail "$desc (expected checker failure)"
        printf '%s\n' "$out" >&2
        return
    fi
    if ! printf '%s\n' "$out" | grep -F "$needle" >/dev/null; then
        fail "$desc (missing '$needle')"
        printf '%s\n' "$out" >&2
        return
    fi
    note "PASS $desc"
}

empty_allow="$work/empty.allowlist"
: >"$empty_allow"

# --- layer-terms -----------------------------------------------------------
tree="$work/layer-fp-code"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
The word `Seat` lives only in code font.
EOF
expect_pass "layer-terms ignores Seat inside inline code" \
    run_lint layer-terms "$tree" "$empty_allow"

tree="$work/layer-fn-vision"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
A Seat is a governance-internal object and must not appear here.
EOF
expect_fail_matching "layer-terms reports Seat in vision prose" \
    "vision contains governance-internal term 'Seat'" \
    run_lint layer-terms "$tree" "$empty_allow"

tree="$work/layer-fn-architecture"
write_tree "$tree"
write_file "$tree/docs/design/architecture.md" <<'EOF'
# architecture
Attach Descriptor belongs to the constraint index.
EOF
expect_fail_matching "layer-terms reports constraint-only term in architecture" \
    "architecture contains constraint-only term 'Attach Descriptor'" \
    run_lint layer-terms "$tree" "$empty_allow"

tree="$work/layer-fp-core"
write_tree "$tree"
write_file "$tree/docs/design/architecture.md" <<'EOF'
# architecture
Participant is a core product word, so architecture may use it.
EOF
expect_pass "layer-terms allows core product words in architecture" \
    run_lint layer-terms "$tree" "$empty_allow"

tree="$work/layer-allow"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
A Seat is named here only as a quoted historical example.
EOF
write_file "$work/layer.allowlist" <<'EOF'
docs/design/vision.md	Seat	quoted historical example
EOF
expect_pass "layer-terms honor a line-bound allowlist entry" \
    run_lint layer-terms "$tree" "$work/layer.allowlist"

# --- implementation-names --------------------------------------------------
tree="$work/impl-fp-code"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
The runtime is selected later; `Herdr` is an identifier, not prose.
EOF
expect_pass "implementation-names ignores Herdr inside inline code" \
    run_lint implementation-names "$tree" "$empty_allow" "$names_file"

tree="$work/impl-fp-link"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
See the runtime notes [here](https://example.invalid/Herdr).
EOF
expect_pass "implementation-names ignores Herdr in a link destination" \
    run_lint implementation-names "$tree" "$empty_allow" "$names_file"

tree="$work/impl-fn"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
Herdr is named as if it were a product noun.
EOF
expect_fail_matching "implementation-names reports Herdr in vision prose" \
    "design layer contains implementation product name 'Herdr'" \
    run_lint implementation-names "$tree" "$empty_allow" "$names_file"

tree="$work/impl-allow"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
Herdr is named as if it were a product noun.
EOF
write_file "$work/impl.allowlist" <<'EOF'
docs/design/vision.md	Herdr	named as if it were a product noun
EOF
expect_pass "implementation-names honor a line-bound allowlist entry" \
    run_lint implementation-names "$tree" "$work/impl.allowlist" "$names_file"

# --- camelcase -------------------------------------------------------------
tree="$work/camel-fp-kept"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
ChangeSet and ReviewSubjectRef are retained product identifiers.
EOF
expect_pass "camelcase keeps ChangeSet and ReviewSubjectRef" \
    run_lint camelcase "$tree" "$empty_allow"

tree="$work/camel-fp-code"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
The identifier `FooBar` is code, not a coined heading.
EOF
expect_pass "camelcase ignores FooBar inside inline code" \
    run_lint camelcase "$tree" "$empty_allow"

tree="$work/camel-fn"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
FooBar is an unallowlisted coinage.
EOF
expect_fail_matching "camelcase reports unallowlisted FooBar" \
    "unallowlisted CamelCase name 'FooBar'" \
    run_lint camelcase "$tree" "$empty_allow"

tree="$work/camel-allow"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
GitHub is an external proper name.
EOF
write_file "$work/camel.allowlist" <<'EOF'
docs/design/vision.md	GitHub	GitHub
EOF
expect_pass "camelcase honor a proper-name allowlist entry" \
    run_lint camelcase "$tree" "$work/camel.allowlist"

# --- first-use -------------------------------------------------------------
tree="$work/first-fp-paren"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
Repo（仓库） is introduced with the vocabulary counterpart.
A later Repo may omit the counterpart.
EOF
expect_pass "first-use accepts Term（对照） on first prose occurrence" \
    run_lint first-use "$tree" "$empty_allow"

tree="$work/first-fn-bare"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
Repo is introduced without a counterpart.
EOF
expect_fail_matching "first-use reports a bare first Repo" \
    "first use lacks Chinese counterpart 'Repo'" \
    run_lint first-use "$tree" "$empty_allow"

tree="$work/first-fp-table"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision

| 词 | 说明 |
| --- | --- |
| Repo（仓库） | first use lives in a table cell |
EOF
expect_pass "first-use accepts counterpart inside a table cell" \
    run_lint first-use "$tree" "$empty_allow"

tree="$work/first-fn-table"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision

| 词 | 说明 |
| --- | --- |
| Repo | first use in a table without counterpart |
EOF
expect_fail_matching "first-use reports a table cell without counterpart" \
    "first use lacks Chinese counterpart 'Repo'" \
    run_lint first-use "$tree" "$empty_allow"

tree="$work/first-fp-boundary"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
Repository and Kanbanize are different words from the product terms.
EOF
expect_pass "first-use uses token boundaries, not ASCII prefixes" \
    run_lint first-use "$tree" "$empty_allow"

tree="$work/first-fp-overlap"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
Workflow Revision（施工图版本） is the longer term.
EOF
expect_pass "first-use does not demand Workflow inside Workflow Revision" \
    run_lint first-use "$tree" "$empty_allow"

tree="$work/first-fp-zh-only"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
只写仓库，不写英文名。
EOF
expect_pass "first-use ignores Chinese-only mentions of the counterpart" \
    run_lint first-use "$tree" "$empty_allow"

tree="$work/first-allow"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision
Repo is introduced without a counterpart.
EOF
write_file "$work/first.allowlist" <<'EOF'
docs/design/vision.md	Repo	introduced without a counterpart
EOF
expect_pass "first-use honor a line-bound allowlist entry" \
    run_lint first-use "$tree" "$work/first.allowlist"

# --- spec-need -------------------------------------------------------------
tree="$work/need-fp"
write_tree "$tree"
write_file "$tree/docs/design/spec/task.md" <<'EOF'
# task
Blocked and 需要关注 are independent health states.
The identifier `需要` is code, not obligation prose.
EOF
expect_pass "spec-need allows 需要关注 and inline-code 需要" \
    run_lint spec-need "$tree" "$empty_allow"

tree="$work/need-fn"
write_tree "$tree"
write_file "$tree/docs/design/spec/task.md" <<'EOF'
# task
验收项需要人工确认。
EOF
expect_fail_matching "spec-need reports ambiguous 需要" \
    "constraint prose uses ambiguous word '需要'" \
    run_lint spec-need "$tree" "$empty_allow"

tree="$work/need-fn-substring"
write_tree "$tree"
write_file "$tree/docs/design/spec/task.md" <<'EOF'
# task
系统不需要猜测 Project。
EOF
expect_fail_matching "spec-need treats 不需要 as containing 需要" \
    "constraint prose uses ambiguous word '需要'" \
    run_lint spec-need "$tree" "$empty_allow"

# --- table-language-mix (report only) --------------------------------------
tree="$work/mix-report"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision

| English | Chinese |
| --- | --- |
| Repo | 仓库 |
EOF
out="$(run_lint table-language-mix "$tree" "$empty_allow" 2>&1)" && rc=0 || rc=$?
if [ "$rc" -ne 0 ]; then
    fail "table-language-mix must stay report-only (exit $rc)"
    printf '%s\n' "$out" >&2
elif ! printf '%s\n' "$out" | grep -F "table mixes 'Repo' and '仓库'" >/dev/null; then
    fail "table-language-mix should report a mixed table"
    printf '%s\n' "$out" >&2
else
    note "PASS table-language-mix reports a mixed table and exits 0"
fi

tree="$work/mix-gloss"
write_tree "$tree"
write_file "$tree/docs/design/vision.md" <<'EOF'
# vision

| 词 | 说明 |
| --- | --- |
| Repo（仓库） | explicit gloss is not a mix |
EOF
out="$(run_lint table-language-mix "$tree" "$empty_allow" 2>&1)" && rc=0 || rc=$?
if [ "$rc" -ne 0 ]; then
    fail "table-language-mix gloss case must exit 0"
    printf '%s\n' "$out" >&2
elif printf '%s\n' "$out" | grep -F "table mixes 'Repo'" >/dev/null; then
    fail "table-language-mix should not report an explicit gloss"
    printf '%s\n' "$out" >&2
else
    note "PASS table-language-mix ignores Term（对照） in a table"
fi

# --- live first-use passes with no exemptions at all --------------------------
# The 46 baseline exemptions Grok audited in R3 were all rule exemptions; Fable
# wrote the counterparts back into the design layer, so the live tree must pass
# against an empty allowlist and the checked-in allowlist must stay comment-only.
note "auditing live first-use against an empty allowlist"
live_out="$(perl "$lint" first-use "$docs_tree" "$empty_allow" 2>&1)" && live_rc=0 || live_rc=$?
if [ "$live_rc" -ne 0 ]; then
    fail "live first-use needs exemptions again; write the counterpart into the text instead:"
    printf '%s\n' "$live_out" | grep '^FAIL ' >&2 || printf '%s\n' "$live_out" >&2
else
    note "PASS live first-use passes with an empty allowlist"
fi

# Empty allowlists stay empty (comment-only).
for name in layer_terms.allowlist implementation_names.allowlist spec_need.allowlist table_language_mix.allowlist first_use_terms.allowlist; do
    extra="$(awk '/^[ \t]*#/ || NF == 0 { next } { print }' "$allowlist_dir/$name")"
    if [ -n "$extra" ]; then
        fail "$name should remain empty of exemptions; found:"$'\n'"$extra"
    else
        note "PASS $name has no exemption rows"
    fi
done

camel_rows="$(awk '/^[ \t]*#/ || NF == 0 { next } { print }' "$allowlist_dir/camelcase_names.allowlist" | wc -l | tr -d ' ')"
if [ "$camel_rows" -ne 24 ]; then
    fail "camelcase allowlist has $camel_rows rows, expected 24 proper-name exemptions (v0.17.0 added GitHub/GitLab in spec/repo.md and GitHub in contract-tests.md)"
else
    note "PASS camelcase allowlist still has 24 proper-name exemptions"
fi

# --- PR contract rename / move / grow --------------------------------------
if ! grep -F -- '--find-renames' "$pr_contract" >/dev/null; then
    fail "pr-contract.yml lost --find-renames on grown-script detection"
else
    note "PASS pr-contract.yml still uses --find-renames"
fi
if ! grep -F '($1 - $2) > 100' "$pr_contract" >/dev/null; then
    fail "pr-contract.yml lost the >100 net-line threshold"
else
    note "PASS pr-contract.yml still uses a per-file >100 net-line threshold"
fi
if ! grep -F -- '--diff-filter=A' "$pr_contract" >/dev/null; then
    fail "pr-contract.yml lost added-script detection"
else
    note "PASS pr-contract.yml still flags newly added scripts"
fi

grown_awk() {
    git diff --numstat --find-renames --diff-filter=MR "$1" |
        awk -F '\t' '
            $1 ~ /^[0-9]+$/ && $2 ~ /^[0-9]+$/ {
                path = $3
                if (path ~ /(\.(sh|bash|py|pl|rb|js|mjs|cjs|ts)(}|$)|^src\/build\/tools\/)/ && ($1 - $2) > 100) {
                    printf "%s\t+%d net lines\n", path, $1 - $2
                }
            }
        '
}

repo="$work/git"
mkdir "$repo"
(
    cd "$repo"
    git init -q
    git config user.email "fixture@hctl2.test"
    git config user.name "fixture"
    printf '%s\n' '#!/bin/sh' 'echo ok' >tool.sh
    git add tool.sh
    git commit -q -m 'add script'
    git mv tool.sh renamed.sh
    git commit -q -m 'rename script'
    grown="$(grown_awk HEAD~1...HEAD)"
    if [ -n "$grown" ]; then
        fail "rename was treated as script growth:"$'\n'"$grown"
    else
        note "PASS rename of a script is 0 net growth under --find-renames"
    fi

    mkdir sub
    git mv renamed.sh sub/moved.sh
    git commit -q -m 'move script'
    grown="$(grown_awk HEAD~1...HEAD)"
    if [ -n "$grown" ]; then
        fail "move was treated as script growth:"$'\n'"$grown"
    else
        note "PASS move of a script is 0 net growth under --find-renames"
    fi

    i=1
    while [ "$i" -le 101 ]; do
        printf 'echo %s\n' "$i" >>sub/moved.sh
        i=$((i + 1))
    done
    git add sub/moved.sh
    git commit -q -m 'grow script by 101 lines'
    grown="$(grown_awk HEAD~1...HEAD)"
    if ! printf '%s\n' "$grown" | grep -F 'sub/moved.sh' >/dev/null; then
        fail "101-line growth was not flagged:"$'\n'"$grown"
    else
        note "PASS 101-line growth of an existing script is flagged"
    fi

    printf '%s\n' '#!/bin/sh' 'echo other' >other.sh
    git add other.sh
    git commit -q -m 'add another script'
    added="$(git diff --name-only --diff-filter=A HEAD~1...HEAD | grep -E '\.(sh|bash|py|pl)$' || true)"
    if [ "$added" != "other.sh" ]; then
        fail "added-script detection missed other.sh (got '$added')"
    else
        note "PASS newly added scripts are still visible to --diff-filter=A"
    fi
)

if [ "$failures" -ne 0 ]; then
    echo "check_docs_toolchain_fixtures: FAILED ($failures)" >&2
    exit 1
fi
echo "check_docs_toolchain_fixtures: OK"
