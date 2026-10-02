#!/bin/sh
# CI gate for Clippy. root//:clippy runs Clippy for every first-party target and
# writes each run's diagnostics to a report file, but the run itself succeeds no
# matter what the report says, so nothing downstream would ever notice. This
# test is the only reader: one non-empty report fails the build, and its
# contents are printed so the lint name and location are visible in the log.
#
# Generated code is exempt by the owner's ruling, and the exemption lives at the
# code-generation boundary rather than here: crates/proto/src/lib.rs allows
# those lints on everything its include! pulls in, so the proto reports come
# back empty and need no special case below.
set -eu

reports_root=${CLIPPY_REPORTS:?CLIPPY_REPORTS must point at the Clippy reports}

# Each report is the filegroup's named output, so it arrives at <crate>/<key>
# and no two reports share a path.
total=$(find "$reports_root" -type f | wc -l | tr -d ' ')
if [ "$total" -eq 0 ]; then
    echo "no Clippy report under $reports_root; the gate would pass vacuously" >&2
    exit 1
fi

dirty=$(find "$reports_root" -type f -size +0 | sort)
if [ -n "$dirty" ]; then
    # Report paths come from target and filegroup keys, which carry no whitespace.
    for report in $dirty; do
        printf 'Clippy diagnostics in %s:\n' "${report#"$reports_root"/}" >&2
        cat "$report" >&2
        echo >&2
    done
    printf 'Clippy reported diagnostics in %s of %s reports; the gate fails until they are gone.\n' \
        "$(printf '%s\n' "$dirty" | wc -l | tr -d ' ')" "$total" >&2
    exit 1
fi

printf '%s Clippy reports, all empty.\n' "$total"
