#!/bin/sh
# Exercise the real coverage checker with fixed uquery answers and copies of
# the real Release workflow. No nested Buck daemon, builds or network access.
set -eu

coverage=${CLIPPY_COVERAGE:?}
release_workflow=${RELEASE_WORKFLOW:?}
jq_bin=${JQ_BIN:?}
work=$(mktemp -d "${TMPDIR:-/tmp}/hctl2-clippy-coverage-test.XXXXXX")
trap 'rm -rf -- "$work"' EXIT
checkout="$work/checkout"
mkdir -p "$checkout/src/build/ci" "$checkout/src/build/tools" "$checkout/.github/workflows"
cp "$coverage" "$checkout/src/build/ci/clippy-coverage"
ln -s "$jq_bin" "$checkout/src/build/tools/jq-bin"
export FIXTURE_GRAPH="$work/graph.json" FIXTURE_JQ="$jq_bin"
export FAIL_QUERY= MALFORMED_QUERY=

cat >"$work/base.json" <<'JSON'
{
  "root//:clippy": {
    "fixture_kind": "filegroup",
    "srcs": {"facts": "root//crates/facts:clippy"}
  },
  "root//crates/facts:clippy": {
    "fixture_kind": "filegroup",
    "srcs": {"library": "root//crates/facts:facts[clippy.txt]"}
  },
  "root//packaging/release:room-cli-clippy": {
    "fixture_kind": "filegroup",
    "srcs": {"room": "root//packaging/release:room-cli-test[clippy.txt]"}
  },
  "root//build/tests:clippy_clean_test": {
    "env": {"CLIPPY_REPORTS": "$(location root//:clippy)"},
    "test": "root//build/tests/check_clippy_clean.sh",
    "labels": ["ci:fast"]
  },
  "root//packaging/release:room-cli-clippy-clean-test": {
    "env": {"CLIPPY_REPORTS": "$(location root//packaging/release:room-cli-clippy)"},
    "test": "root//build/tests:check-clippy-clean.sh",
    "labels": ["ci:release"]
  }
}
JSON
cat >"$checkout/src/buck2" <<'SH'
#!/bin/sh
set -eu
[ "$1" = uquery ] || exit 2
if [ "$#" -eq 2 ]; then
    [ "$2" = 'kind("filegroup", root//...)' ] || exit 2
    [ "$FAIL_QUERY" != all ] || { echo 'fixture graph query failed' >&2; exit 70; }
    "$FIXTURE_JQ" -r 'to_entries[] | select(.value.fixture_kind == "filegroup") | .key' "$FIXTURE_GRAPH"
elif [ "$#" -eq 4 ] && [ "$2" = --output-attribute ]; then
    [ "$FAIL_QUERY" != "$4" ] || { echo 'fixture attribute query failed' >&2; exit 70; }
    if [ "$MALFORMED_QUERY" = "$4" ]; then echo '{invalid'; exit 0; fi
    "$FIXTURE_JQ" -e --arg target "$4" 'has($target)' "$FIXTURE_GRAPH" >/dev/null || exit 1
    "$FIXTURE_JQ" --arg target "$4" '{($target): .[$target]}' "$FIXTURE_GRAPH"
else
    echo 'unexpected fixture Buck invocation' >&2
    exit 2
fi
SH
chmod +x "$checkout/src/buck2"

root_gate='root//build/tests:clippy_clean_test'
release_gate='root//packaging/release:room-cli-clippy-clean-test'
failures=0
cases=0
reset() {
    cp "$work/base.json" "$FIXTURE_GRAPH"
    cp "$release_workflow" "$checkout/.github/workflows/release.yml"
    FAIL_QUERY= MALFORMED_QUERY=
}
mutate_graph() {
    "$jq_bin" --arg root "$root_gate" --arg release "$release_gate" --arg gate "${gate:-}" "$1" "$FIXTURE_GRAPH" >"$work/next.json"
    mv "$work/next.json" "$FIXTURE_GRAPH"
}
expect() { # name success/failure diagnostic [forbidden diagnostic]
    cases=$((cases + 1))
    status=0
    sh "$checkout/src/build/ci/clippy-coverage" >"$work/output" 2>&1 || status=$?
    if { [ "$2" = success ] && [ "$status" -eq 0 ]; } || { [ "$2" = failure ] && [ "$status" -ne 0 ]; }; then
        if grep -Fq -- "$3" "$work/output" && { [ "$#" -lt 4 ] || ! grep -Fq -- "$4" "$work/output"; }; then
            printf 'PASS %s\n' "$1"
            return
        fi
    fi
    failures=$((failures + 1))
    printf 'FAIL %s (exit %s, expected %s and %s)\n' "$1" "$status" "$2" "$3" >&2
    cat "$work/output" >&2
}

reset
expect baseline success '3 Clippy reports'
for gate in "$root_gate" "$release_gate"; do
    reset
    mutate_graph 'del(.[$gate])'
    expect "missing $gate" failure 'cannot query declared gate'
    reset
    mutate_graph '.[$gate].env |= {NOT_CLIPPY_REPORTS: .CLIPPY_REPORTS}'
    expect "wrong env key $gate" failure 'CLIPPY_REPORTS is <unset>'
    reset
    mutate_graph '.[$gate].env.CLIPPY_REPORTS = "$(location root//crates/facts:clippy)"'
    expect "wrong report $gate" failure 'does not read'
    reset
    mutate_graph '.[$gate].test = "root//build/tests/check_clippy_clean_fixtures.sh"'
    expect "wrong executable $gate" failure 'runs root//build/tests/check_clippy_clean_fixtures.sh'
done
reset
mutate_graph '.[$root].env.EXTRA = "unrelated" | .[$release].env.EXTRA = "unrelated"'
expect 'unrelated env does not change coverage' success '3 Clippy reports'
reset
mutate_graph '.[$root].labels = ["ci:platform", "ci:fast"]'
expect 'extra labels are accepted' success '3 Clippy reports'
reset
mutate_graph '.[$root].labels = ["ci:slow"]'
expect 'Code gate must be selectable' failure 'no ci:fast label'
reset
mutate_graph 'del(.[$release].labels)'
expect 'Release scheduling is not inferred from labels' success '3 Clippy reports'
reset
mutate_graph '.["root//:clippy"].srcs = {}'
expect 'omitted package report' failure 'Clippy report with no verified reader'
reset
mutate_graph '.["root//crates/new:clippy"] = {fixture_kind: "filegroup", srcs: {}}'
expect 'new unlisted report' failure 'root//crates/new:clippy'
reset
mutate_graph '.["root//:clippy"].srcs.gone = "root//crates/gone:clippy"'
expect 'stale aggregate member' failure 'not a Clippy report in the graph'
reset
mutate_graph 'with_entries(select(.value.fixture_kind != "filegroup"))'
expect 'empty graph does not pass' failure 'would pass vacuously'
reset
FAIL_QUERY=all
expect 'graph query failure' failure 'fixture graph query failed'
for report in 'root//:clippy' 'root//packaging/release:room-cli-clippy'; do
    reset
    FAIL_QUERY=$report
    expect "member query failure $report" failure 'cannot query report members'
    reset
    MALFORMED_QUERY=$report
    expect "malformed member answer $report" failure 'cannot query report members'
done
reset
mutate_graph '.["root//:clippy"].srcs = null'
expect 'missing srcs is not an empty member set' failure 'missing or invalid attribute'
reset
MALFORMED_QUERY=$release_gate
expect 'malformed gate answer' failure 'parse error'

# These mutate a copy of the real workflow, so deleting either actual argument
# is tested with all BUCK gates and labels still intact.
for occurrence in 1 2; do
    reset
    awk -v occurrence="$occurrence" -v gate="$release_gate" '
        $0 == "          " gate { seen++; if (seen == occurrence) next }
        { print }
    ' "$release_workflow" >"$checkout/.github/workflows/release.yml"
    expect "missing Release argument $occurrence" failure 'must both schedule'
done
reset
sed '/^          root\/\/packaging\/release:room-cli-clippy-clean-test$/s/^          /          # /' "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect 'a comment cannot prove scheduling' failure 'cannot verify Release workflow shape'
reset
sed 's/\.\/buck2 test --build-default-info/\.\/buck2 build --build-default-info/g' "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect 'building is not running a gate' failure 'must both schedule'
reset
sed "s/event_name != 'pull_request'/event_name == 'pull_request'/g" "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect 'both event paths must be covered' failure 'must both schedule'
reset
sed '/^          root\/\/packaging\/release:room-cli-clippy-clean-test$/s/$/ --exclude ci:release/' "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect 'an exclusion cannot masquerade as scheduling' failure 'cannot verify Release workflow shape'

# The credential path is part of the owned shape: without the env key, or
# without the test-executor argument that forwards it, the demo3 GitHub path
# would skip silently on every runner instead of exercising the canary.
reset
awk '/^          HCTL2_CANARY_GITHUB_TOKEN: / { next } { print }' \
    "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect 'an env block without the canary key is unknown' failure 'cannot verify Release workflow shape' 'must both schedule'
reset
awk '/^          -- --env / { next } { print }' \
    "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect 'dropping the test-executor credential is unknown' failure 'cannot verify Release workflow shape' 'must both schedule'

# Harmless YAML changes remain unsupported, but must not be diagnosed as a
# proven scheduling omission. Keep the narrow assertion rather than growing
# it into another YAML parser.
reset
awk '
    function flush() { if (folded) { print "        run: " body; folded = 0 } }
    $0 == "        run: >-" { folded = 1; body = ""; next }
    folded && /^          / { sub(/^          /, ""); body = body (body == "" ? "" : " ") $0; next }
    { flush(); print }
    END { flush() }
' "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect 'single-line run is unknown, not absent' failure 'cannot verify Release workflow shape' 'must both schedule'
reset
awk '
    /^      - / { wanted = ($0 ~ /- name: Test complete offline install and lifecycle/) }
    wanted && /^          / { print "  " $0; next }
    { print }
' "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect '12-space body is unknown, not absent' failure 'cannot verify Release workflow shape' 'must both schedule'
reset
awk '
    /^      - name: Test complete offline install and lifecycle/ {
        print; print "        env:"; print "          COVERAGE_FORMAT_PROBE: present"; next
    }
    { print }
' "$release_workflow" >"$checkout/.github/workflows/release.yml"
expect 'extra env is unknown, not absent' failure 'cannot verify Release workflow shape' 'must both schedule'
reset
expect 'restored workflow' success '3 Clippy reports'

[ "$failures" -eq 0 ] || { printf '%s of %s fixture checks failed\n' "$failures" "$cases" >&2; exit 1; }
printf '%s Clippy coverage fixture checks passed\n' "$cases"
