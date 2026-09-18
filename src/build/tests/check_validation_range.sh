#!/usr/bin/env bash
# Regression checks for Code/Release incremental revalidation (#257 follow-up).
# Portable: bash 3.2. Does not require DotSlash or jq-bin.
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
resolve="${RESOLVE_VALIDATION_RANGE:-$script_dir/../ci/resolve-validation-range}"
code_yml="${CODE_WORKFLOW:-$script_dir/../../../.github/workflows/code.yml}"
release_yml="${RELEASE_WORKFLOW:-$script_dir/../../../.github/workflows/release.yml}"

failures=0
fail() {
    printf 'FAIL %s\n' "$*" >&2
    failures=$((failures + 1))
}
note() { printf '%s\n' "$*"; }

[[ -x "$resolve" ]] || { echo "missing resolver: $resolve" >&2; exit 1; }
[[ -f "$code_yml" ]] || { echo "missing Code workflow: $code_yml" >&2; exit 1; }
[[ -f "$release_yml" ]] || { echo "missing Release workflow: $release_yml" >&2; exit 1; }

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
printf '%s\n' "${MOCK_GH_RESULT:-false}"
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

# Parse failure: gh returns a non-boolean.
log="$work/parse.log"
if MOCK_GH_RESULT='{"workflow_runs":[]}' \
    run_resolve "$work/parse.out" >"$log" 2>&1; then
    expect_mode "parse failure uses full-pr-parse-fallback" full-pr-parse-fallback "$work/parse.out"
    expect_log "parse failure names the parse" "Could not parse" "$log"
    forbid_log "parse failure does not claim no successful run" "has no successful" "$log"
else
    fail "parse failure should fall back, not abort"
    cat "$log" >&2
fi

# Previous head completed without success.
log="$work/nosuccess.log"
if MOCK_GH_RESULT=false \
    run_resolve "$work/nosuccess.out" >"$log" 2>&1; then
    expect_mode "no prior success uses full-pr-no-prior-success" full-pr-no-prior-success "$work/nosuccess.out"
    expect_log "no prior success says so" "has no successful" "$log"
    got_base=$(read_field "$work/nosuccess.out" base)
    if [ "$got_base" = "$base_sha" ]; then
        note "PASS no prior success keeps the PR base"
    else
        fail "no prior success base=$got_base, expected $base_sha"
    fi
else
    fail "no prior success resolve exited $?"
    cat "$log" >&2
fi

# PATH has no working DotSlash; previous head succeeded → incremental.
log="$work/incr.log"
if MOCK_GH_RESULT=true \
    run_resolve "$work/incr.out" >"$log" 2>&1; then
    expect_mode "successful prior run uses incremental-validated-head" incremental-validated-head "$work/incr.out"
    got_base=$(read_field "$work/incr.out" base)
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
    forbid_log "incremental path did not call dotslash" "dotslash must not be called" "$log"
    forbid_log "incremental path did not call jq-bin" "jq-bin must not be called" "$log"
else
    fail "incremental resolve exited $?"
    cat "$log" >&2
fi

# Docs-only follow-up must not re-select the earlier Skill path.
incr_changed=$(
    cd "$repo"
    git diff --no-renames --name-only "$prev_sha...$head_sha"
)
full_changed=$(
    cd "$repo"
    git diff --no-renames --name-only "$base_sha...$head_sha"
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
if printf '%s\n' "$incr_changed" | grep -Eq \
    '^(src/(third-party|packaging)/|src/build/(empty_cell|execution|platforms|rules|toolchains)/|src/build/tools/(bazel-remote-bin|buck2-bin|buck2-cache|dotslash\.env|install-dotslash)$|src/(BUCK|Cargo\.lock|Cargo\.toml|buck2|rust-toolchain\.toml)$|\.buckconfig$|\.buckroot$|BUCK$|src/agency/|\.github/workflows/release\.yml$)'; then
    fail "incremental docs-only range would still trigger complete package tests"
    printf '%s\n' "$incr_changed" >&2
else
    note "PASS incremental docs-only range does not match the Release path filter"
fi
if printf '%s\n' "$full_changed" | grep -Eq \
    '^(src/(third-party|packaging)/|src/build/(empty_cell|execution|platforms|rules|toolchains)/|src/build/tools/(bazel-remote-bin|buck2-bin|buck2-cache|dotslash\.env|install-dotslash)$|src/(BUCK|Cargo\.lock|Cargo\.toml|buck2|rust-toolchain\.toml)$|\.buckconfig$|\.buckroot$|BUCK$|src/agency/|\.github/workflows/release\.yml$)'; then
    note "PASS complete PR range still matches the Release path filter via Skill"
else
    fail "complete PR range no longer matches the Release path filter"
    printf '%s\n' "$full_changed" >&2
fi

if [ "$failures" -ne 0 ]; then
    echo "check_validation_range: $failures failure(s)" >&2
    exit 1
fi
echo "check_validation_range: OK"
