#!/usr/bin/env bash
# Regression checks for Code/Release incremental revalidation (#257 follow-up).
# Portable: bash 3.2. The resolver PATH has no working DotSlash/jq-bin; the
# mock gh evaluates the production --jq program with the pinned jq-bin.
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
resolve="${RESOLVE_VALIDATION_RANGE:-$script_dir/../ci/resolve-validation-range}"
code_yml="${CODE_WORKFLOW:-$script_dir/../../../.github/workflows/code.yml}"
release_yml="${RELEASE_WORKFLOW:-$script_dir/../../../.github/workflows/release.yml}"
jq_bin="${JQ_BIN:-$script_dir/../tools/jq-bin}"
orig_path=$PATH

failures=0
fail() {
    printf 'FAIL %s\n' "$*" >&2
    failures=$((failures + 1))
}
note() { printf '%s\n' "$*"; }

[[ -x "$resolve" ]] || { echo "missing resolver: $resolve" >&2; exit 1; }
[[ -f "$code_yml" ]] || { echo "missing Code workflow: $code_yml" >&2; exit 1; }
[[ -f "$release_yml" ]] || { echo "missing Release workflow: $release_yml" >&2; exit 1; }
[[ -x "$jq_bin" ]] || { echo "missing jq-bin: $jq_bin" >&2; exit 1; }

work="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-validation-range.XXXXXX")"
trap 'rm -rf "$work"' EXIT

# --- YAML contract: production steps call the helper, not jq-bin ----------
for workflow in "$code_yml" "$release_yml"; do
    name=$(basename "$workflow")
    if ! grep -F 'src/build/ci/resolve-validation-range' "$workflow" >/dev/null; then
        fail "$name Resolve step does not call resolve-validation-range"
    else
        note "PASS $name invokes resolve-validation-range"
    fi
    if awk '
        $0 ~ /name: Resolve validation range/ { in_step = 1; next }
        in_step && $0 ~ /^      - name:/ { in_step = 0 }
        in_step && $0 ~ /jq-bin/ { found = 1 }
        END { if (found) exit 0; exit 1 }
    ' "$workflow"; then
        fail "$name Resolve step still calls jq-bin"
    else
        note "PASS $name Resolve step does not call jq-bin"
    fi
done

if grep -F 'src/build/tools/jq-bin' "$resolve" >/dev/null; then
    fail "resolver still invokes jq-bin"
else
    note "PASS resolver does not invoke jq-bin"
fi
if ! grep -F -- '--jq' "$resolve" >/dev/null; then
    fail "resolver does not use gh api --jq"
else
    note "PASS resolver uses gh api --jq"
fi

# Prove the unguarded expression still collapses a missing list to false,
# so the production type-guard is not an unused string of PASS lines.
old_missing=$(printf '%s' '{"message":"unexpected response"}' | "$jq_bin" -r \
    'any(.workflow_runs[]?; .head_sha == "deadbeef" and .conclusion == "success")')
if [ "$old_missing" = false ]; then
    note "PASS unguarded expression still misclassifies missing workflow_runs as false"
else
    fail "unguarded expression no longer returns false for missing workflow_runs (got ${old_missing:-<empty>})"
fi

# --- Isolated PATH: real git, mock gh, no DotSlash / jq-bin ---------------
bin="$work/bin"
mkdir "$bin"
ln -s "$(command -v git)" "$bin/git"
ln -s "$(command -v mktemp)" "$bin/mktemp"
ln -s "$(command -v tr)" "$bin/tr"
ln -s "$(command -v rm)" "$bin/rm"
# Keep a fake dotslash that always fails so accidental calls are visible.
printf '%s\n' '#!/bin/sh' 'echo "dotslash must not be called" >&2' 'exit 127' >"$bin/dotslash"
printf '%s\n' '#!/bin/sh' 'echo "jq-bin must not be called" >&2' 'exit 127' >"$bin/jq-bin"
chmod +x "$bin/dotslash" "$bin/jq-bin"

mock_gh="$bin/gh"
write_mock_gh() {
    cat >"$mock_gh" <<'EOF'
#!/bin/sh
printf '%s\n' "$*" >>"${MOCK_GH_LOG:-/dev/null}"
if [ "${MOCK_GH_EXIT:-0}" -ne 0 ]; then
    printf '%s\n' "${MOCK_GH_ERR:-gh api failed}" >&2
    exit "$MOCK_GH_EXIT"
fi
jq_expr=
while [ "$#" -gt 0 ]; do
    case "$1" in
        --jq)
            jq_expr=$2
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done
if [ -z "$jq_expr" ]; then
    printf 'mock gh: missing --jq\n' >&2
    exit 2
fi
if [ -z "${JQ_BIN:-}" ] || [ ! -x "$JQ_BIN" ]; then
    printf 'mock gh: JQ_BIN is required to evaluate the production --jq expression\n' >&2
    exit 2
fi
# Match gh api --jq: raw output, original PATH so DotSlash can run jq-bin.
PATH="${ORIG_PATH:-$PATH}"
export PATH
printf '%s' "${MOCK_GH_BODY-}" | "$JQ_BIN" -r "$jq_expr"
EOF
    chmod +x "$mock_gh"
}
write_mock_gh

# Restrict PATH only for the resolver so helpers like grep stay available.
resolve_path="$bin"

repo="$work/repo"
mkdir "$repo"
(
    cd "$repo"
    git init -q
    git config user.email "fixture@hctl2.test"
    git config user.name "fixture"
    mkdir -p src/agency/skills/hctl2-shaping docs/user-experience
    printf '# skill\n' >src/agency/skills/hctl2-shaping/SKILL.md
    printf '# docs\n' >docs/user-experience/README.md
    git add src docs
    git commit -q -m 'base'
    git rev-parse HEAD >"$work/base"
    printf '# skill v2\n' >src/agency/skills/hctl2-shaping/SKILL.md
    git add src/agency/skills/hctl2-shaping/SKILL.md
    git commit -q -m 'skill'
    git rev-parse HEAD >"$work/prev"
    printf '# docs v2\n' >docs/user-experience/README.md
    git add docs/user-experience/README.md
    git commit -q -m 'docs only'
    git rev-parse HEAD >"$work/head"
)
base_sha=$(cat "$work/base")
prev_sha=$(cat "$work/prev")
head_sha=$(cat "$work/head")

run_resolve() {
    local out=$1
    shift
    : >"$out"
    (
        cd "$repo"
        export GITHUB_OUTPUT="$out"
        export EVENT_NAME="${EVENT_NAME:-pull_request}"
        export EVENT_ACTION="${EVENT_ACTION:-synchronize}"
        export BASE_COMMIT="${BASE_COMMIT:-$base_sha}"
        export CURRENT_HEAD="${CURRENT_HEAD:-$head_sha}"
        export PREVIOUS_HEAD="${PREVIOUS_HEAD:-$prev_sha}"
        export GITHUB_REPOSITORY="${GITHUB_REPOSITORY:-yesme/hctl2}"
        export WORKFLOW_FILE="${WORKFLOW_FILE:-release.yml}"
        export WORKFLOW_LABEL="${WORKFLOW_LABEL:-Release}"
        export MOCK_GH_LOG="$work/gh.log"
        export JQ_BIN="$jq_bin"
        export ORIG_PATH="$orig_path"
        export PATH="$resolve_path"
        while [ "$#" -gt 0 ]; do
            export "$1"
            shift
        done
        "$resolve"
    )
}

read_field() {
    awk -F= -v key="$2" '$1 == key { print substr($0, index($0, "=") + 1); exit }' "$1"
}

expect_mode() {
    local desc=$1
    local want=$2
    local out=$3
    local got
    got=$(read_field "$out" mode)
    if [ "$got" = "$want" ]; then
        note "PASS $desc (mode=$got)"
    else
        fail "$desc (mode=$got, expected $want)"
        printf '%s\n' "--- GITHUB_OUTPUT ---" >&2
        cat "$out" >&2
    fi
}

expect_log() {
    local desc=$1
    local needle=$2
    local file=$3
    if grep -F "$needle" "$file" >/dev/null; then
        note "PASS $desc"
    else
        fail "$desc (missing '$needle')"
        cat "$file" >&2
    fi
}

forbid_log() {
    local desc=$1
    local needle=$2
    local file=$3
    if grep -F "$needle" "$file" >/dev/null; then
        fail "$desc (unexpected '$needle')"
        cat "$file" >&2
    else
        note "PASS $desc"
    fi
}

# Non-PR / periodic.
log="$work/periodic.log"
if run_resolve "$work/periodic.out" \
    EVENT_NAME=schedule EVENT_ACTION= \
    >"$log" 2>&1; then
    expect_mode "non-PR uses full-periodic" full-periodic "$work/periodic.out"
else
    fail "non-PR resolve exited $? "
    cat "$log" >&2
fi

# First PR event (opened): complete PR, no gh query.
: >"$work/gh.log"
log="$work/opened.log"
if run_resolve "$work/opened.out" \
    EVENT_ACTION=opened MOCK_GH_EXIT=1 \
    >"$log" 2>&1; then
    expect_mode "opened PR uses full-pr" full-pr "$work/opened.out"
    if [ -s "$work/gh.log" ]; then
        fail "opened PR queried workflow runs"
    else
        note "PASS opened PR does not query workflow runs"
    fi
else
    fail "opened PR resolve exited $?"
    cat "$log" >&2
fi

# Rewritten history: previous head exists but is not an ancestor.
(
    cd "$repo"
    original_branch=$(git rev-parse --abbrev-ref HEAD)
    git checkout -q --orphan rewritten
    printf 'unrelated\n' >unrelated.txt
    git add unrelated.txt
    git commit -q -m 'rewritten history'
    git rev-parse HEAD >"$work/orphan-sha"
    git checkout -q "$original_branch"
)
log="$work/rewritten.log"
if run_resolve "$work/rewritten.out" \
    PREVIOUS_HEAD="$(cat "$work/orphan-sha")" \
    >"$log" 2>&1; then
    expect_mode "rewritten history uses full-pr-rewritten" full-pr-rewritten "$work/rewritten.out"
    expect_log "rewritten history explains itself" "history was rewritten" "$log"
else
    fail "rewritten history resolve exited $?"
    cat "$log" >&2
fi

# Query failure must not look like "no successful run".
: >"$work/gh.log"
log="$work/query.log"
if ! MOCK_GH_EXIT=1 MOCK_GH_ERR='connection refused' \
    run_resolve "$work/query.out" >"$log" 2>&1; then
    fail "query failure should fall back, not abort"
    cat "$log" >&2
else
    expect_mode "query failure uses full-pr-query-fallback" full-pr-query-fallback "$work/query.out"
    expect_log "query failure names the query" "Could not query" "$log"
    forbid_log "query failure does not claim no successful run" "has no successful" "$log"
fi

run_with_body() {
    local out=$1
    local body=$2
    local log=$3
    shift 3
    : >"$work/gh.log"
    MOCK_GH_BODY="$body" run_resolve "$out" "$@" >"$log" 2>&1
}

# Missing workflow_runs must not look like an empty success list.
log="$work/missing.log"
if run_with_body "$work/missing.out" '{"message":"unexpected response"}' "$log"; then
    expect_mode "missing workflow_runs uses full-pr-parse-fallback" full-pr-parse-fallback "$work/missing.out"
    expect_log "missing workflow_runs names the parse" "Could not parse" "$log"
    forbid_log "missing workflow_runs does not claim no successful run" "has no successful" "$log"
else
    fail "missing workflow_runs should fall back, not abort"
    cat "$log" >&2
fi

log="$work/null.log"
if run_with_body "$work/null.out" '{"workflow_runs":null}' "$log"; then
    expect_mode "null workflow_runs uses full-pr-parse-fallback" full-pr-parse-fallback "$work/null.out"
    forbid_log "null workflow_runs does not claim no successful run" "has no successful" "$log"
else
    fail "null workflow_runs should fall back, not abort"
    cat "$log" >&2
fi

log="$work/badtype.log"
if run_with_body "$work/badtype.out" '{"workflow_runs":"nope"}' "$log"; then
    expect_mode "non-array workflow_runs uses full-pr-parse-fallback" full-pr-parse-fallback "$work/badtype.out"
    forbid_log "non-array workflow_runs does not claim no successful run" "has no successful" "$log"
else
    fail "non-array workflow_runs should fall back, not abort"
    cat "$log" >&2
fi

# jq evaluation error (invalid JSON) is a parse failure, not "no success".
log="$work/malformed.log"
if run_with_body "$work/malformed.out" 'not-json' "$log"; then
    expect_mode "invalid JSON uses full-pr-parse-fallback" full-pr-parse-fallback "$work/malformed.out"
    expect_log "invalid JSON names the parse" "Could not parse" "$log"
    forbid_log "invalid JSON does not claim no successful run" "has no successful" "$log"
else
    fail "invalid JSON should fall back, not abort"
    cat "$log" >&2
fi

# Legitimate empty list: queried, completed, none succeeded.
log="$work/empty.log"
if run_with_body "$work/empty.out" '{"workflow_runs":[]}' "$log"; then
    expect_mode "empty workflow_runs uses full-pr-no-prior-success" full-pr-no-prior-success "$work/empty.out"
    expect_log "empty list says no successful run" "has no successful" "$log"
    got_base=$(read_field "$work/empty.out" base)
    if [ "$got_base" = "$base_sha" ]; then
        note "PASS empty list keeps the PR base"
    else
        fail "empty list base=$got_base, expected $base_sha"
    fi
else
    fail "empty workflow_runs resolve exited $?"
    cat "$log" >&2
fi

# Other SHA succeeded; previous head did not.
log="$work/othersha.log"
if run_with_body "$work/othersha.out" \
    "{\"workflow_runs\":[{\"head_sha\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"conclusion\":\"success\"}]}" \
    "$log"; then
    expect_mode "other SHA success is not this head" full-pr-no-prior-success "$work/othersha.out"
else
    fail "other SHA resolve exited $?"
    cat "$log" >&2
fi

# Mixed conclusions: matching SHA success plus a failure.
log="$work/mixed.log"
if run_with_body "$work/mixed.out" \
    "{\"workflow_runs\":[{\"head_sha\":\"$prev_sha\",\"conclusion\":\"failure\"},{\"head_sha\":\"$prev_sha\",\"conclusion\":\"success\"}]}" \
    "$log"; then
    expect_mode "mixed conclusions still take a success" incremental-validated-head "$work/mixed.out"
else
    fail "mixed conclusions resolve exited $?"
    cat "$log" >&2
fi

# PATH has no working DotSlash; previous head succeeded → incremental.
log="$work/incr.log"
if run_with_body "$work/incr.out" \
    "{\"workflow_runs\":[{\"head_sha\":\"$prev_sha\",\"conclusion\":\"success\"}]}" \
    "$log"; then
    expect_mode "successful prior run uses incremental-validated-head" incremental-validated-head "$work/incr.out"
    got_base=$(read_field "$work/incr.out" base)
    got_head=$(read_field "$work/incr.out" head)
    if [ "$got_base" = "$prev_sha" ]; then
        note "PASS incremental base is the previous head"
    else
        fail "incremental base=$got_base, expected $prev_sha"
    fi
    if grep -F -- '--jq' "$work/gh.log" >/dev/null; then
        note "PASS incremental query used gh --jq"
    else
        fail "incremental query did not pass --jq to gh"
        cat "$work/gh.log" >&2
    fi
    if grep -F "actions/workflows/release.yml/runs" "$work/gh.log" >/dev/null; then
        note "PASS Release query targets release.yml"
    else
        fail "Release query did not ask for release.yml"
        cat "$work/gh.log" >&2
    fi
    if grep -F "actions/workflows/code.yml/runs" "$work/gh.log" >/dev/null; then
        fail "Release query borrowed code.yml"
        cat "$work/gh.log" >&2
    else
        note "PASS Release query does not borrow code.yml"
    fi
    forbid_log "incremental path did not call resolver-PATH dotslash" "dotslash must not be called" "$log"
    forbid_log "incremental path did not call resolver-PATH jq-bin" "jq-bin must not be called" "$log"
else
    fail "incremental resolve exited $?"
    cat "$log" >&2
fi

# Code workflow must query code.yml, not Release.
: >"$work/gh.log"
log="$work/codewf.log"
if run_with_body "$work/codewf.out" \
    "{\"workflow_runs\":[{\"head_sha\":\"$prev_sha\",\"conclusion\":\"success\"}]}" \
    "$log" WORKFLOW_FILE=code.yml WORKFLOW_LABEL=Code; then
    expect_mode "Code query still incremental" incremental-validated-head "$work/codewf.out"
    if grep -F "actions/workflows/code.yml/runs" "$work/gh.log" >/dev/null; then
        note "PASS Code query targets code.yml"
    else
        fail "Code query did not ask for code.yml"
        cat "$work/gh.log" >&2
    fi
    if grep -F "actions/workflows/release.yml/runs" "$work/gh.log" >/dev/null; then
        fail "Code query borrowed release.yml"
        cat "$work/gh.log" >&2
    else
        note "PASS Code query does not borrow release.yml"
    fi
else
    fail "Code workflow resolve exited $?"
    cat "$log" >&2
fi

# Consume resolver output + the production Release path-detection pattern.
release_pattern=$(awk '
    /name: Detect product and release paths/ { in_step = 1 }
    in_step && /grep -Eq \\/ {
        getline
        sub(/^[[:space:]]+'\''/, "")
        sub(/'\''.*$/, "")
        print
        exit
    }
' "$release_yml")
if [ -z "$release_pattern" ]; then
    fail "could not extract Release path-detection pattern from workflow"
else
    note "PASS extracted Release path-detection pattern from workflow"
fi

incr_base=$(read_field "$work/incr.out" base)
incr_head=$(read_field "$work/incr.out" head)
full_base=$base_sha
full_head=$(read_field "$work/empty.out" head)
incr_changed=$(
    cd "$repo"
    git diff --no-renames --name-only "$incr_base...$incr_head"
)
full_changed=$(
    cd "$repo"
    git diff --no-renames --name-only "$full_base...$full_head"
)
if printf '%s\n' "$incr_changed" | grep -q '^src/agency/'; then
    fail "incremental docs-only range still includes src/agency/"
    printf '%s\n' "$incr_changed" >&2
else
    note "PASS incremental docs-only range excludes src/agency/"
fi
if printf '%s\n' "$full_changed" | grep -q '^src/agency/'; then
    note "PASS complete PR range still includes the earlier Skill path"
else
    fail "complete PR range lost src/agency/; fixture is wrong"
    printf '%s\n' "$full_changed" >&2
fi
if printf '%s\n' "$incr_changed" | grep -Eq "$release_pattern"; then
    fail "resolver incremental range still matches the production Release path filter"
    printf '%s\n' "$incr_changed" >&2
else
    note "PASS resolver incremental range does not match the production Release path filter"
fi
if printf '%s\n' "$full_changed" | grep -Eq "$release_pattern"; then
    note "PASS complete PR range still matches the production Release path filter via Skill"
else
    fail "complete PR range no longer matches the production Release path filter"
    printf '%s\n' "$full_changed" >&2
fi

if [ "$failures" -ne 0 ]; then
    echo "check_validation_range: $failures failure(s)" >&2
    exit 1
fi
echo "check_validation_range: OK"
