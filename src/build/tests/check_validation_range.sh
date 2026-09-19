#!/usr/bin/env bash
# Regression checks for the "Resolve validation range" step that
# .github/workflows/code.yml and release.yml share (incremental PR
# revalidation, docs/research/build-tools/github-actions-incremental-validation.md).
#
# The step bodies are extracted from the workflow files and executed against a
# throwaway Git repository with a stand-in `gh` on PATH. DotSlash is kept off
# PATH on purpose: the step runs in the path-filter job before DotSlash is
# installed, so it must not depend on any src/build/tools trampoline. The
# stand-in gh evaluates the step's --jq filter with the pinned jq, the way the
# real gh evaluates it with its built-in jq, and never touches the network.
# Portable: bash 3.2.
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
repo_root="$(CDPATH= cd -- "$script_dir/../../.." && pwd)"
code_workflow="${CODE_WORKFLOW:-$repo_root/.github/workflows/code.yml}"
release_workflow="${RELEASE_WORKFLOW:-$repo_root/.github/workflows/release.yml}"
jq_bin="${JQ_BIN:-$script_dir/../tools/jq-bin}"
affected_targets="${AFFECTED_TARGETS:-$script_dir/../ci/affected-targets}"

[[ -f "$code_workflow" ]] || { echo "missing workflow: $code_workflow" >&2; exit 1; }
[[ -f "$release_workflow" ]] || { echo "missing workflow: $release_workflow" >&2; exit 1; }
[[ -f "$jq_bin" ]] || { echo "missing pinned jq: $jq_bin" >&2; exit 1; }
[[ -f "$affected_targets" ]] || { echo "missing target selector: $affected_targets" >&2; exit 1; }
command -v dotslash >/dev/null || { echo "dotslash must be on PATH for the stand-in gh (the step itself runs without it)" >&2; exit 1; }

failures=0
fail() {
    printf 'FAIL %s\n' "$*" >&2
    failures=$((failures + 1))
}
note() { printf '%s\n' "$*"; }

work="$(mktemp -d)"
trap 'chmod -R u+w "$work" 2>/dev/null; rm -rf "$work"' EXIT
original_path=$PATH

# --- extract the step bodies from the workflow files -----------------------
extract_step() {
    awk -v step="$2" '
        $0 == "      - name: " step { in_step = 1; next }
        in_step && /^      - name: / { exit }
        in_step && $0 == "        run: |" { in_run = 1; next }
        in_run {
            if ($0 ~ /^          / || $0 == "") { sub(/^          /, ""); print; next }
            exit
        }' "$1"
}

mkdir -p "$work/steps"
extract_step "$code_workflow" "Resolve validation range" > "$work/steps/code-range.sh"
extract_step "$release_workflow" "Resolve validation range" > "$work/steps/release-range.sh"
extract_step "$code_workflow" "Detect first-party / CI paths" > "$work/steps/code-paths.sh"
extract_step "$release_workflow" "Detect product and release paths" > "$work/steps/release-paths.sh"
for body in code-range release-range code-paths release-paths; do
    file="$work/steps/$body.sh"
    if [ ! -s "$file" ] || ! grep -q GITHUB_OUTPUT "$file"; then
        echo "could not extract the $body step body from the workflow" >&2
        exit 1
    fi
    if ! bash -n "$file"; then
        fail "$body step body does not parse"
    fi
done
for body in code-range release-range; do
    if grep -v '^ *#' "$work/steps/$body.sh" | grep -Eq 'src/build/tools/|jq-bin'; then
        fail "$body step body still runs a repository tool before DotSlash is installed"
    else
        note "PASS $body step body runs no src/build/tools trampoline"
    fi
done

# --- throwaway repository: base -> c1 (skill change) -> c2 (docs only) ------
# Git ignores the developer's global config (signing, hooks, credential
# helpers); HOME stays real so DotSlash reuses its cache for the stand-in gh.
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
repo="$work/repo"
origin="$work/origin.git"
mkdir -p "$repo"
git -c init.defaultBranch=main init -q "$repo"
git -C "$repo" config user.email test@example.invalid
git -C "$repo" config user.name test
git -C "$repo" config commit.gpgsign false
mkdir -p "$repo/docs/design" "$repo/src/agency/skills/hctl2-shaping"
printf '/docs/** hctl-doc=design\n' > "$repo/.gitattributes"
echo base > "$repo/docs/design/a.md"
echo base > "$repo/src/agency/skills/hctl2-shaping/SKILL.md"
git -C "$repo" add -A
git -C "$repo" commit -q -m base
base_commit="$(git -C "$repo" rev-parse HEAD)"
git init -q --bare "$origin"
git -C "$repo" remote add origin "$origin"
git -C "$repo" push -q origin main
git -C "$repo" checkout -q -b pr
echo skill > "$repo/src/agency/skills/hctl2-shaping/SKILL.md"
git -C "$repo" commit -q -am 'skill change'
c1="$(git -C "$repo" rev-parse HEAD)"
echo docs > "$repo/docs/design/a.md"
git -C "$repo" commit -q -am 'docs only'
c2="$(git -C "$repo" rev-parse HEAD)"
git -C "$repo" checkout -q -b rewritten "$base_commit"
echo other > "$repo/docs/design/a.md"
git -C "$repo" commit -q -am 'rewritten lineage'
c3="$(git -C "$repo" rev-parse HEAD)"
git -C "$repo" checkout -q pr

# --- stand-in gh and a PATH without DotSlash --------------------------------
bin="$work/bin"
mkdir -p "$bin"
# `type -P` resolves external binaries only: `command -v echo` would answer the
# builtin's bare name and produce a self-referencing link. GNU xargs execs
# `echo` from PATH, so the external echo must be reachable too.
for tool in bash git grep awk sed tr mktemp rm cat sort uniq xargs echo printf env dirname basename wc head tail cut date mkdir ls uname; do
    real="$(type -P "$tool" || true)"
    [ -n "$real" ] && ln -s "$real" "$bin/$tool"
done
cat > "$bin/gh" <<'GH'
#!/usr/bin/env bash
# Stand-in for the runner's GitHub CLI. It checks that the step asks exactly
# for this workflow's completed pull_request runs on the previous head, then
# answers `gh api ... --jq FILTER` from the fixture chosen per workflow file by
# evaluating FILTER with the pinned jq (gh evaluates it with its built-in jq),
# or reproduces the CLI's error output. It never touches the network.
set -euo pipefail
method=""
path=""
filter=""
head_sha=""
event=""
status=""
per_page=""
prev=""
first=1
for arg in "$@"; do
    if [ "$first" = 1 ]; then
        [ "$arg" = api ] || { echo "stand-in gh: unexpected subcommand $arg" >&2; exit 2; }
        first=0
        prev=$arg
        continue
    fi
    case "$prev" in
        --method) method=$arg ;;
        --jq) filter=$arg ;;
        -f)
            case "$arg" in
                head_sha=*) head_sha=${arg#head_sha=} ;;
                event=*) event=${arg#event=} ;;
                status=*) status=${arg#status=} ;;
                per_page=*) per_page=${arg#per_page=} ;;
                *) echo "stand-in gh: unexpected field $arg" >&2; exit 2 ;;
            esac
            ;;
        *)
            case "$arg" in
                --method | --jq | -f) ;;
                -*) echo "stand-in gh: unexpected flag $arg" >&2; exit 2 ;;
                *) path=$arg ;;
            esac
            ;;
    esac
    prev=$arg
done
case "$path" in
    "repos/${GITHUB_REPOSITORY}/actions/workflows/code.yml/runs") workflow=CODE ;;
    "repos/${GITHUB_REPOSITORY}/actions/workflows/release.yml/runs") workflow=RELEASE ;;
    *) echo "stand-in gh: unexpected request path '$path'" >&2; exit 2 ;;
esac
[ "$method" = GET ] || { echo "stand-in gh: unexpected method '$method'" >&2; exit 2; }
[ "$head_sha" = "${PREVIOUS_HEAD:-}" ] || { echo "stand-in gh: head_sha '$head_sha' is not the previous head" >&2; exit 2; }
[ "$event" = pull_request ] || { echo "stand-in gh: unexpected event filter '$event'" >&2; exit 2; }
[ "$status" = completed ] || { echo "stand-in gh: unexpected status filter '$status'" >&2; exit 2; }
printf 'api GET %s head_sha=%s event=%s status=%s per_page=%s jq=%s\n' \
    "$path" "$head_sha" "$event" "$status" "$per_page" "${filter:+set}" >> "$FAKE_GH_LOG"
fixture=$FAKE_GH_FIXTURE
if [ "$workflow" = CODE ] && [ -n "${FAKE_GH_FIXTURE_CODE:-}" ]; then fixture=$FAKE_GH_FIXTURE_CODE; fi
if [ "$workflow" = RELEASE ] && [ -n "${FAKE_GH_FIXTURE_RELEASE:-}" ]; then fixture=$FAKE_GH_FIXTURE_RELEASE; fi
case "$FAKE_GH_CASE" in
    http-404)
        printf '{"message":"Not Found","status":"404"}'
        echo "gh: Not Found (HTTP 404)" >&2
        exit 1
        ;;
    bad-credentials)
        echo "gh: Bad credentials (HTTP 401)" >&2
        exit 1
        ;;
    network)
        echo "error connecting to api.github.com" >&2
        echo "check your internet connection or https://githubstatus.com" >&2
        exit 1
        ;;
    weird-answer)
        echo null
        exit 0
        ;;
    unknown-error)
        echo "something went sideways while talking to the API" >&2
        exit 1
        ;;
    gojq-error-prefix)
        # gh's built-in jq reports the filter's own error() as "error: ...".
        echo "error: unexpected workflow_runs type: object" >&2
        exit 1
        ;;
    fixture)
        # Without --jq the CLI prints the response body; this is what the
        # unfixed step relied on before piping the body into jq-bin.
        if [ -z "$filter" ]; then
            cat "$fixture"
            exit 0
        fi
        if ! PATH="$FAKE_GH_TOOL_PATH" "$FAKE_GH_JQ" -r "$filter" < "$fixture" 2>"$FAKE_GH_LOG.jq"; then
            # gh's built-in jq prints the bare message ("cannot iterate over:
            # null"); strip jq's "jq: error (at <stdin>:0): " prefix to match.
            sed -E 's/^jq: error( \([^)]*\))?: //' "$FAKE_GH_LOG.jq" >&2
            exit 1
        fi
        ;;
    *)
        echo "stand-in gh: unknown case $FAKE_GH_CASE" >&2
        exit 2
        ;;
esac
GH
chmod +x "$bin/gh"
poisoned_path="$bin"
if PATH="$poisoned_path" command -v dotslash >/dev/null 2>&1; then
    echo "test setup error: dotslash is still reachable on the poisoned PATH" >&2
    exit 1
fi
# The path-detect step joins targets with `xargs`, which execs echo from PATH.
if [ "$(printf 'a b\n' | PATH="$poisoned_path" xargs 2>&1)" != "a b" ]; then
    echo "test setup error: xargs cannot run echo on the poisoned PATH" >&2
    exit 1
fi

fixtures="$work/fixtures"
mkdir -p "$fixtures"
run_json() { # head_sha conclusion
    printf '{"head_sha":"%s","event":"pull_request","status":"completed","conclusion":"%s"}' "$1" "$2"
}
printf '{"total_count":1,"workflow_runs":[%s]}' "$(run_json "$c1" success)" > "$fixtures/success.json"
printf '{"total_count":2,"workflow_runs":[%s,%s]}' "$(run_json "$c1" failure)" "$(run_json "$c1" success)" > "$fixtures/mixed.json"
printf '{"total_count":1,"workflow_runs":[%s]}' "$(run_json "$c1" failure)" > "$fixtures/failure.json"
printf '{"total_count":0,"workflow_runs":[]}' > "$fixtures/empty.json"
printf '{"total_count":1,"workflow_runs":[%s]}' "$(run_json 0000000000000000000000000000000000000000 success)" > "$fixtures/other-sha.json"
printf '{"message":"unexpected body"}' > "$fixtures/shapeless.json"
printf '{"total_count":0,"workflow_runs":{}}' > "$fixtures/object.json"

# --- running a step body ----------------------------------------------------
output="$work/output.txt"
log="$work/step.log"
gh_log="$work/gh.log"
runner_tmp="$work/runner-tmp"
mkdir -p "$runner_tmp"

run_step() { # body event_name event_action previous_head gh_case fixture
    : > "$output"
    : > "$gh_log"
    (
        cd "$repo" && \
        PATH="$poisoned_path" GITHUB_OUTPUT="$output" RUNNER_TEMP="$runner_tmp" \
        GITHUB_REPOSITORY=yesme/hctl2 GH_TOKEN=stand-in \
        FAKE_GH_LOG="$gh_log" FAKE_GH_CASE="$5" FAKE_GH_FIXTURE="$6" \
        FAKE_GH_FIXTURE_CODE="${STEP_FIXTURE_CODE:-}" FAKE_GH_FIXTURE_RELEASE="${STEP_FIXTURE_RELEASE:-}" \
        FAKE_GH_JQ="$jq_bin" FAKE_GH_TOOL_PATH="$original_path" \
        EVENT_NAME="$2" EVENT_ACTION="$3" BASE_COMMIT="$base_commit" \
        CURRENT_HEAD="$c2" PREVIOUS_HEAD="$4" BASE_REF=main \
        HEAD_COMMIT="${STEP_HEAD_COMMIT:-}" VALIDATION_BASE="${STEP_VALIDATION_BASE:-}" \
        "$BASH" "$work/steps/$1.sh"
    ) > "$log" 2>&1
}

output_value() { sed -n "s/^$1=//p" "$output" | tail -n 1; }
gh_calls() { wc -l < "$gh_log" | tr -d ' '; }

expect() { # label actual expected
    if [ "$2" = "$3" ]; then
        note "PASS $1: $2"
    else
        fail "$1: got '$2', expected '$3'"
        sed 's/^/    | /' "$log" >&2
    fi
}
expect_log() { # label pattern
    if grep -Eq -- "$2" "$log"; then
        note "PASS $1"
    else
        fail "$1: log lacks '$2'"
        sed 's/^/    | /' "$log" >&2
    fi
}
expect_no_log() { # label pattern
    if grep -Eq -- "$2" "$log"; then
        fail "$1: log unexpectedly contains '$2'"
        sed 's/^/    | /' "$log" >&2
    else
        note "PASS $1"
    fi
}
expect_gh_request() { # label pattern (matched against the stand-in's request log)
    if grep -Eq -- "$2" "$gh_log"; then
        note "PASS $1"
    else
        fail "$1: no request matching '$2'"
        sed 's/^/    | /' "$gh_log" "$log" >&2
    fi
}

for workflow in code release; do
    body="$workflow-range"

    # 1. no DotSlash on PATH, query works, previous head succeeded -> incremental
    if run_step "$body" pull_request synchronize "$c1" fixture "$fixtures/success.json"; then
        expect "$workflow: prior success selects incremental mode" "$(output_value mode)" incremental-validated-head
        expect "$workflow: incremental base is the previous head" "$(output_value base)" "$c1"
        expect "$workflow: head is the checked-out commit" "$(output_value head)" "$c2"
        expect "$workflow: one query" "$(gh_calls)" 1
        expect_gh_request "$workflow: asks for this workflow's completed pull_request runs on the previous head" \
            "^api GET repos/yesme/hctl2/actions/workflows/$workflow\.yml/runs head_sha=$c1 event=pull_request status=completed per_page=100 jq=set\$"
        expect_no_log "$workflow: the stand-in accepted the request shape" "stand-in gh: unexpected"
        expect_no_log "$workflow: no DotSlash error" "dotslash"
        expect_no_log "$workflow: success is not reported as missing" "has no successful"
    else
        fail "$workflow: step exited non-zero on the incremental case"; sed 's/^/    | /' "$log" >&2
    fi

    # a failed run beside a successful one still counts as success
    run_step "$body" pull_request synchronize "$c1" fixture "$fixtures/mixed.json" || true
    expect "$workflow: mixed conclusions with one success -> incremental" "$(output_value mode)" incremental-validated-head

    # 2. previous head failed, or has no record -> complete PR diff
    run_step "$body" pull_request synchronize "$c1" fixture "$fixtures/failure.json" || true
    expect "$workflow: prior failure -> full PR diff" "$(output_value mode)" full-pr-no-prior-success
    expect "$workflow: prior failure keeps the PR base" "$(output_value base)" "$base_commit"
    expect_log "$workflow: prior failure names the previous head" "no successful .* run"

    run_step "$body" pull_request synchronize "$c1" fixture "$fixtures/empty.json" || true
    expect "$workflow: no record -> full PR diff" "$(output_value mode)" full-pr-no-prior-success

    run_step "$body" pull_request synchronize "$c1" fixture "$fixtures/other-sha.json" || true
    expect "$workflow: success on another sha does not count" "$(output_value mode)" full-pr-no-prior-success

    # 3. query failure and evaluation failure are diagnosed, never reported as "no success"
    run_step "$body" pull_request synchronize "$c1" http-404 "" || true
    expect "$workflow: HTTP error -> query fallback" "$(output_value mode)" full-pr-query-fallback
    expect "$workflow: HTTP error keeps the PR base" "$(output_value base)" "$base_commit"
    expect_log "$workflow: HTTP error is quoted" "Could not query .*HTTP 404"
    expect_no_log "$workflow: HTTP error is not 'no success'" "has no successful"

    run_step "$body" pull_request synchronize "$c1" bad-credentials "" || true
    expect "$workflow: auth error -> query fallback" "$(output_value mode)" full-pr-query-fallback
    expect_log "$workflow: auth error is quoted" "HTTP 401"

    run_step "$body" pull_request synchronize "$c1" network "" || true
    expect "$workflow: connection error -> query fallback" "$(output_value mode)" full-pr-query-fallback
    expect_log "$workflow: connection error is quoted" "Could not query .*connecting to"

    run_step "$body" pull_request synchronize "$c1" fixture "$fixtures/shapeless.json" || true
    expect "$workflow: unreadable answer -> parse fallback" "$(output_value mode)" full-pr-parse-fallback
    expect_log "$workflow: unreadable answer is quoted" "Could not evaluate .*unexpected workflow_runs type: null"
    expect_no_log "$workflow: unreadable answer is not 'no success'" "has no successful"

    run_step "$body" pull_request synchronize "$c1" fixture "$fixtures/object.json" || true
    expect "$workflow: workflow_runs that is an object, not an array -> parse fallback" "$(output_value mode)" full-pr-parse-fallback
    expect_log "$workflow: the type check names the anomaly" "Could not evaluate .*unexpected workflow_runs type: object"
    expect_no_log "$workflow: an object is not reported as 'no success'" "has no successful"

    run_step "$body" pull_request synchronize "$c1" gojq-error-prefix "" || true
    expect "$workflow: gh's 'error: ' form -> parse fallback" "$(output_value mode)" full-pr-parse-fallback
    expect_log "$workflow: gh's 'error: ' form is quoted" "Could not evaluate .*error: unexpected workflow_runs type"

    run_step "$body" pull_request synchronize "$c1" weird-answer "" || true
    expect "$workflow: non-count answer -> parse fallback" "$(output_value mode)" full-pr-parse-fallback
    expect_log "$workflow: non-count answer is quoted" "answered 'null' instead of a count"

    run_step "$body" pull_request synchronize "$c1" unknown-error "" || true
    expect "$workflow: unrecognised error -> full PR diff" "$(output_value mode)" full-pr-query-fallback
    expect_log "$workflow: unrecognised error is reported as undetermined and quoted" "Could not determine .*something went sideways"
    expect_no_log "$workflow: unrecognised error is not 'no success'" "has no successful"
    expect_no_log "$workflow: unrecognised error is not called an evaluation failure" "Could not evaluate"

    # 4. rewritten history, first open, missing previous head, non-PR runs
    run_step "$body" pull_request synchronize "$c3" fixture "$fixtures/success.json" || true
    expect "$workflow: rewritten history -> full PR diff" "$(output_value mode)" full-pr-rewritten
    expect "$workflow: rewritten history never queries" "$(gh_calls)" 0

    run_step "$body" pull_request opened "" fixture "$fixtures/success.json" || true
    expect "$workflow: opened -> full PR diff" "$(output_value mode)" full-pr
    expect "$workflow: opened never queries" "$(gh_calls)" 0

    run_step "$body" pull_request synchronize "" fixture "$fixtures/success.json" || true
    expect "$workflow: synchronize without previous head -> full PR diff" "$(output_value mode)" full-pr

    run_step "$body" schedule "" "" fixture "$fixtures/success.json" || true
    expect "$workflow: scheduled run -> periodic full check" "$(output_value mode)" full-periodic
    expect "$workflow: scheduled run validates HEAD" "$(output_value base)" HEAD
    expect "$workflow: scheduled run never queries" "$(gh_calls)" 0
done

# each workflow consults only its own history: Release success must not let Code go incremental, and vice versa
STEP_FIXTURE_CODE="$fixtures/failure.json"
STEP_FIXTURE_RELEASE="$fixtures/success.json"
export STEP_FIXTURE_CODE STEP_FIXTURE_RELEASE
run_step code-range pull_request synchronize "$c1" fixture "" || true
expect "code: Release success alone does not make Code incremental" "$(output_value mode)" full-pr-no-prior-success
run_step release-range pull_request synchronize "$c1" fixture "" || true
expect "release: its own success still selects incremental" "$(output_value mode)" incremental-validated-head
STEP_FIXTURE_CODE="$fixtures/success.json"
STEP_FIXTURE_RELEASE="$fixtures/failure.json"
export STEP_FIXTURE_CODE STEP_FIXTURE_RELEASE
run_step release-range pull_request synchronize "$c1" fixture "" || true
expect "release: Code success alone does not make Release incremental" "$(output_value mode)" full-pr-no-prior-success
run_step code-range pull_request synchronize "$c1" fixture "" || true
expect "code: its own success still selects incremental" "$(output_value mode)" incremental-validated-head
unset STEP_FIXTURE_CODE STEP_FIXTURE_RELEASE

# 5. docs-only commit after a Skill change: the incremental range excludes the Skill file
STEP_HEAD_COMMIT="$c2"
STEP_VALIDATION_BASE="$c1"
export STEP_HEAD_COMMIT STEP_VALIDATION_BASE
run_step release-paths pull_request synchronize "$c1" fixture "" || true
expect "release: docs-only increment after a Skill change skips the complete package" "$(output_value release)" false
run_step code-paths pull_request synchronize "$c1" fixture "" || true
expect "code: docs-only increment after a Skill change needs no Buck matrix" "$(output_value code)" false
expect "code: docs-only increment runs the docs check" "$(output_value docs)" true

# the same commit against the PR base still selects the Skill change (the range matters)
STEP_VALIDATION_BASE="$base_commit"
export STEP_VALIDATION_BASE
run_step release-paths pull_request synchronize "$c1" fixture "" || true
expect "release: the complete PR diff still includes the Skill change" "$(output_value release)" true
run_step code-paths pull_request synchronize "$c1" fixture "" || true
expect "code: the complete PR diff still includes the Skill change" "$(output_value code)" true

# A compressor pin or tool declaration change must trigger the full release.
for tool_file in xz.bzl BUCK; do
    previous_head="$(git -C "$repo" rev-parse HEAD)"
    mkdir -p "$repo/src/build/tools"
    echo pin > "$repo/src/build/tools/$tool_file"
    git -C "$repo" add "src/build/tools/$tool_file"
    git -C "$repo" commit -q -m "xz tool fixture: $tool_file"
    STEP_VALIDATION_BASE="$previous_head"
    STEP_HEAD_COMMIT="$(git -C "$repo" rev-parse HEAD)"
    export STEP_VALIDATION_BASE STEP_HEAD_COMMIT
    run_step release-paths pull_request synchronize "$previous_head" fixture "" || true
    expect "release: $tool_file change selects the complete package" "$(output_value release)" true
done

# 6. Every full selection must execute standalone library tests, not just compile their Clippy.
# Run the actual selector's early policy-change path in the throwaway repository.
mkdir -p "$repo/src/build/ci" "$runner_tmp/hctl2-btd"
cp "$affected_targets" "$repo/src/build/ci/affected-targets"
git -C "$repo" add src/build/ci/affected-targets
git -C "$repo" commit -q -m 'target selection policy fixture'
policy_head="$(git -C "$repo" rev-parse HEAD)"
sh "$repo/src/build/ci/affected-targets" --base "$c2" --head "$policy_head" \
    --output "$runner_tmp/hctl2-btd/selected.txt" --mode-output "$runner_tmp/hctl2-btd/mode.txt"
if grep -Fxq 'root//crates/...' "$runner_tmp/hctl2-btd/selected.txt"; then
    note 'PASS policy-change target selection includes standalone crate tests'
else
    fail 'policy-change target selection omits standalone crate tests'
fi

extract_step "$code_workflow" 'Resolve Buck target selection' > "$work/steps/code-targets.sh"
for selection in full-policy-change full-btd-fallback full-periodic; do
    event=pull_request
    btd_outcome=success
    case "$selection" in
        full-btd-fallback) btd_outcome=failure ;;
        full-periodic) event=schedule ;;
    esac
    : > "$output"
    EVENT_NAME="$event" BTD_OUTCOME="$btd_outcome" RUNNER_TEMP="$runner_tmp" \
        GITHUB_OUTPUT="$output" "$BASH" "$work/steps/code-targets.sh" > "$log" 2>&1
    expect "$selection: mode preserved" "$(output_value mode)" "$selection"
    if output_value targets | tr ' ' '\n' | grep -Fxq 'root//crates/...'; then
        note "PASS $selection: workflow executes standalone crate tests"
    else
        fail "$selection: workflow omits standalone crate tests"
    fi
done

if [ "$failures" -ne 0 ]; then
    echo "check_validation_range: FAILED ($failures)" >&2
    exit 1
fi
echo "check_validation_range: OK"
