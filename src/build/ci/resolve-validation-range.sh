#!/usr/bin/env bash
# Resolve validation range for incremental CI.
# Shared implementation for .github/workflows/code.yml and release.yml.
# Reads env: BASE_COMMIT, CURRENT_HEAD, EVENT_ACTION, EVENT_NAME,
# PREVIOUS_HEAD, GITHUB_REPOSITORY, WORKFLOW_FILE (default code.yml),
# GITHUB_OUTPUT (required in CI, optional in tests).
set -euo pipefail

workflow_file="${WORKFLOW_FILE:-${1:-code.yml}}"
case "$workflow_file" in
  code.yml|release.yml) ;;
  *code*|*release*) ;;
  *) workflow_file="code.yml" ;;
esac

# Allow caller to set output file explicitly; default to GITHUB_OUTPUT or stdout.
output_file="${GITHUB_OUTPUT:-}"
if [ -z "$output_file" ] || [ "$output_file" = "/dev/stdout" ]; then
  # Tests may set GITHUB_OUTPUT to temp file; if unset, write to stdout via fd.
  output_file=""
fi

write_output() {
  local key="$1" val="$2"
  if [ -n "$output_file" ]; then
    printf '%s=%s\n' "$key" "$val" >> "$output_file"
  else
    printf '%s=%s\n' "$key" "$val"
  fi
}

if [ "${EVENT_NAME:-}" != "pull_request" ]; then
  write_output "base" "HEAD"
  write_output "head" "HEAD"
  write_output "mode" "full-periodic"
  echo "Validation mode: full-periodic"
  echo "Validation range: HEAD...HEAD"
  exit 0
fi

if [ -z "${BASE_COMMIT:-}" ] || [ -z "${CURRENT_HEAD:-}" ]; then
  echo "BASE_COMMIT and CURRENT_HEAD must be set" >&2
  exit 2
fi

candidate=$(git rev-parse --verify 'HEAD^{commit}')
base="$BASE_COMMIT"
mode="full-pr"

if [ "${EVENT_ACTION:-}" = "synchronize" ] && [ -n "${PREVIOUS_HEAD:-}" ]; then
  if ! git merge-base --is-ancestor "$PREVIOUS_HEAD" "$CURRENT_HEAD"; then
    echo "::notice::The PR history was rewritten; validate the complete PR diff."
    mode="full-pr-rewritten"
  elif ! query_output=$(gh api --method GET \
    "repos/${GITHUB_REPOSITORY}/actions/workflows/${workflow_file}/runs" \
    -f head_sha="$PREVIOUS_HEAD" \
    -f event=pull_request \
    -f status=completed \
    -f per_page=100 \
    --jq 'any(.workflow_runs[]; .head_sha == "'"$PREVIOUS_HEAD"'" and .conclusion == "success")' 2>&1); then
    # Distinguish genuine API/query failure from filter evaluation failure
    # for honest diagnostics; both fall back to full validation.
    echo "::warning::Could not query the previous run for ${workflow_file} ($query_output); validate the complete PR diff."
    mode="full-pr-query-fallback"
  else
    if [ "$query_output" = "true" ]; then
      base="$PREVIOUS_HEAD"
      mode="incremental-validated-head"
    elif [ "$query_output" = "false" ]; then
      # Distinguish "no successful run" (legitimate false) from transport failures
      # already handled above; this branch is only a false boolean from jq.
      if [ "$workflow_file" = "release.yml" ]; then
        echo "::notice::The previous PR head has no successful Release run; validate the complete PR diff."
      else
        echo "::notice::The previous PR head has no successful Code run; validate the complete PR diff."
      fi
      mode="full-pr-no-prior-success"
    else
      echo "::warning::Could not parse the previous run response for ${workflow_file} ($query_output); validate the complete PR diff."
      mode="full-pr-query-fallback"
    fi
  fi
fi

write_output "base" "$base"
write_output "head" "$candidate"
write_output "mode" "$mode"
echo "Validation mode: $mode"
echo "Validation range: $base...$candidate"
