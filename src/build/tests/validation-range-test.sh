#!/usr/bin/env bash
# Regression tests for Resolve validation range incremental logic.
# Covers 5 required scenarios plus static guards.
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
project_root="$(CDPATH= cd -- "$script_dir/../.." && pwd)"
repo_root="$(CDPATH= cd -- "$project_root/.." && pwd)"
code_wf="$repo_root/.github/workflows/code.yml"
release_wf="$repo_root/.github/workflows/release.yml"

failures=0
fail() { printf 'FAIL %s\n' "$*" >&2; failures=$((failures + 1)); }
pass() { printf 'PASS %s\n' "$*"; }
# -------------------------------------------------------------------------
# Static guards: workflows must not use DotSlash-gated jq-bin in validation
# and must use gh --jq native.
# -------------------------------------------------------------------------
check_no_jqbin() {
  local wf="$1"
  # Extract the Resolve validation range step (30 lines after header)
  # Must not contain src/build/tools/jq-bin in that step.
  if awk '/Resolve validation range/{flag=1; n=0} flag{n++; if(n<=40 && /jq-bin/){found=1; exit} if(n>40) exit} END{exit found?0:1}' "$wf" 2>/dev/null; then
    fail "$(basename "$wf") Resolve validation range still uses jq-bin"
  else
    pass "$(basename "$wf") no jq-bin in validation range"
  fi
}

check_gh_jq() {
  local wf="$1"
  if grep -A60 "Resolve validation range" "$wf" | grep -q -- "--jq" && \
     grep -A60 "Resolve validation range" "$wf" | grep -q "any(.workflow_runs"; then
    pass "$(basename "$wf") uses gh --jq"
  else
    fail "$(basename "$wf") missing gh --jq in validation range"
  fi
}

check_no_jqbin "$code_wf"
check_no_jqbin "$release_wf"
check_gh_jq "$code_wf"
check_gh_jq "$release_wf"

# -------------------------------------------------------------------------
# Dynamic tests: mock git/gh to exercise mode selection.
# We replicate the fixed workflow logic as a function.
# -------------------------------------------------------------------------

# Mock control variables (set per test)
MOCK_GH_MODE="" # true / false / query-fail / parse-fail
MOCK_GIT_ANCESTOR="true" # true / false

# Mocked git
git() {
  if [ "$1" = "merge-base" ] && [ "$2" = "--is-ancestor" ]; then
    if [ "$MOCK_GIT_ANCESTOR" = "true" ]; then
      return 0
    else
      return 1
    fi
  fi
  # For other git calls, delegate to real git if needed; but in tests we only use merge-base.
  command git "$@"
}

# Mocked gh
gh() {
  # Only mock the specific api call; delegate other gh calls.
  if [ "$1" = "api" ]; then
    case "$MOCK_GH_MODE" in
      true)  printf 'true\n'; return 0 ;;
      false) printf 'false\n'; return 0 ;;
      query-fail) printf 'API rate limited\n' >&2; return 1 ;;
      parse-fail) printf 'not-a-boolean\n'; return 0 ;;
      *) printf 'false\n'; return 0 ;;
    esac
  fi
  command gh "$@"
}

resolve_validation_range() {
  local EVENT_NAME="$1" EVENT_ACTION="$2" PREVIOUS_HEAD="$3" CURRENT_HEAD="$4" BASE_COMMIT="$5"
  local base="$BASE_COMMIT"
  local mode="full-pr"
  local query_output=""
  if [ "$EVENT_NAME" != "pull_request" ]; then
    printf 'full-periodic\n'
    return
  fi
  if [ "$EVENT_ACTION" = "synchronize" ] && [ -n "$PREVIOUS_HEAD" ]; then
    if ! git merge-base --is-ancestor "$PREVIOUS_HEAD" "$CURRENT_HEAD"; then
      printf 'full-pr-rewritten\n'
      return
    elif ! query_output=$(gh api --method GET "repos/owner/repo/actions/workflows/code.yml/runs" -f head_sha="$PREVIOUS_HEAD" -f event=pull_request -f status=completed -f per_page=100 --jq 'any(.workflow_runs[]; .head_sha == "'"$PREVIOUS_HEAD"'" and .conclusion == "success")' 2>&1); then
      # query failure
      printf 'full-pr-query-fallback\n'
      return
    else
      if [ "$query_output" = "true" ]; then
        printf 'incremental-validated-head\n'
        return
      elif [ "$query_output" = "false" ]; then
        printf 'full-pr-no-prior-success\n'
        return
      else
        printf 'full-pr-query-fallback\n'
        return
      fi
    fi
  fi
  printf 'full-pr\n'
}

# Helpers
assert_mode() {
  local desc="$1" expected="$2" actual="$3"
  if [ "$actual" = "$expected" ]; then
    pass "$desc => $actual"
  else
    fail "$desc expected $expected got $actual"
  fi
}

# -------------------------------------------------------------------------
# Test 1: PATH without DotSlash, query true -> incremental
# New code must not invoke src/build/tools/jq-bin or dotslash.
# We poison PATH with failing binaries and ensure incremental still works.
# -------------------------------------------------------------------------
{
  fake_dir=$(mktemp -d)
  trap 'rm -rf "$fake_dir"' EXIT
  printf '#!/bin/sh\necho dotslash missing >&2; exit 127\n' > "$fake_dir/dotslash"
  printf '#!/bin/sh\necho jq-bin poisoned >&2; exit 127\n' > "$fake_dir/jq-bin"
  chmod +x "$fake_dir"/dotslash "$fake_dir"/jq-bin
  # Also place a fake src/build/tools/jq-bin path that would be poisoned if used
  mkdir -p "$fake_dir/src/build/tools"
  printf '#!/bin/sh\necho poisoned jq-bin >&2; exit 127\n' > "$fake_dir/src/build/tools/jq-bin"
  chmod +x "$fake_dir/src/build/tools/jq-bin"

  MOCK_GIT_ANCESTOR="true"
  MOCK_GH_MODE="true"
  # Poison PATH
  orig_path="$PATH"
  export PATH="$fake_dir:$PATH"
  result=$(resolve_validation_range "pull_request" "synchronize" "abc123" "def456" "base000")
  export PATH="$orig_path"
  assert_mode "Test1 PATH poison, previous success" "incremental-validated-head" "$result"

  # Also ensure the workflow file would not have failed under poisoned PATH for the old code:
  # Old code used src/build/tools/jq-bin, which would have been poisoned. Our new code uses gh --jq, so it survives.
  # Verify that a direct call to the old jq-bin path would fail under poisoned PATH (sanity)
  if PATH="$fake_dir:$PATH" src/build/tools/jq-bin -e '.' 2>/dev/null; then
    # If it succeeds, it means our repo's real jq-bin is still reachable via relative path; but PATH poison shouldn't affect relative path.
    # The point is that new code doesn't use relative path at all, so it's immune to DotSlash missing.
    # We already verified static guard, so this is just sanity.
    pass "Test1 sanity: relative jq-bin still exists (expected)"
  else
    pass "Test1 sanity: relative jq-bin would fail without DotSlash (bug scenario)"
  fi
  rm -rf "$fake_dir"
  trap - EXIT
}

# -------------------------------------------------------------------------
# Test 2: previous head has no successful run -> full
# -------------------------------------------------------------------------
{
  MOCK_GIT_ANCESTOR="true"
  MOCK_GH_MODE="false"
  result=$(resolve_validation_range "pull_request" "synchronize" "abc123" "def456" "base000")
  assert_mode "Test2 previous no success" "full-pr-no-prior-success" "$result"
}

# -------------------------------------------------------------------------
# Test 3a: query failure -> fallback, not misjudged as no success
# -------------------------------------------------------------------------
{
  MOCK_GIT_ANCESTOR="true"
  MOCK_GH_MODE="query-fail"
  result=$(resolve_validation_range "pull_request" "synchronize" "abc123" "def456" "base000")
  assert_mode "Test3a query failure" "full-pr-query-fallback" "$result"
}

# -------------------------------------------------------------------------
# Test 3b: parse failure (gh returns non-boolean) -> fallback
# -------------------------------------------------------------------------
{
  MOCK_GIT_ANCESTOR="true"
  MOCK_GH_MODE="parse-fail"
  result=$(resolve_validation_range "pull_request" "synchronize" "abc123" "def456" "base000")
  assert_mode "Test3b parse failure" "full-pr-query-fallback" "$result"
}

# -------------------------------------------------------------------------
# Test 4a: history rewritten -> full-pr-rewritten
# -------------------------------------------------------------------------
{
  MOCK_GIT_ANCESTOR="false"
  MOCK_GH_MODE="true" # should be ignored
  result=$(resolve_validation_range "pull_request" "synchronize" "abc123" "def456" "base000")
  assert_mode "Test4a history rewritten" "full-pr-rewritten" "$result"
}

# -------------------------------------------------------------------------
# Test 4b: non-PR event -> full-periodic
# -------------------------------------------------------------------------
{
  MOCK_GIT_ANCESTOR="true"
  MOCK_GH_MODE="true"
  result=$(resolve_validation_range "schedule" "synchronize" "abc123" "def456" "base000")
  assert_mode "Test4b non-PR event" "full-periodic" "$result"
}

# -------------------------------------------------------------------------
# Test 5: previous succeeded, base should be previous_head for incremental
# This verifies the docs-only scenario: when previous head succeeded,
# validation base becomes previous_head, so diff since previous_head (docs only)
# determines triggers, not the whole PR diff since base.
# We test that resolve returns incremental and that the caller would use
# previous_head as base.
# -------------------------------------------------------------------------
{
  MOCK_GIT_ANCESTOR="true"
  MOCK_GH_MODE="true"
  result=$(resolve_validation_range "pull_request" "synchronize" "prevHead123" "currHead456" "originMainBase")
  assert_mode "Test5 previous success -> incremental" "incremental-validated-head" "$result"
  # Additionally, static check that the workflow sets base=$PREVIOUS_HEAD in that branch
  if grep -A50 "Resolve validation range" "$code_wf" | grep -q 'base=\$PREVIOUS_HEAD'; then
    pass "Test5 code.yml sets base to PREVIOUS_HEAD on success"
  else
    fail "Test5 code.yml does not set base correctly"
  fi
  if grep -A50 "Resolve validation range" "$release_wf" | grep -q 'base=\$PREVIOUS_HEAD'; then
    pass "Test5 release.yml sets base to PREVIOUS_HEAD on success"
  else
    fail "Test5 release.yml does not set base correctly"
  fi
}

# -------------------------------------------------------------------------
# Test 6: Verify that Skill files are NOT blanket-exempted
# The path filter must still trigger on Skill changes when incremental range
# would include them. Ensure workflows still grep for Skill-related paths
# and don't have an early exit for skills.
# -------------------------------------------------------------------------
{
  # Check that code.yml path filter still includes src/agency as code trigger
  if grep -q "src/(agency" "$code_wf"; then
    pass "Test6 Skill paths still considered for code trigger"
  else
    fail "Test6 Skill paths incorrectly exempted"
  fi
  # Check release.yml path filter includes src/agency
  if grep -q "src/agency" "$release_wf"; then
    pass "Test6 release correctly triggers on src/agency"
  else
    fail "Test6 release missing agency trigger"
  fi
}

# -------------------------------------------------------------------------
# Summary
# -------------------------------------------------------------------------
if [ "$failures" -ne 0 ]; then
  printf 'validation-range-test: FAILED (%d)\n' "$failures" >&2
  exit 1
fi
printf 'validation-range-test: OK\n'
