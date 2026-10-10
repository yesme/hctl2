#!/usr/bin/env bash
# Two targeted cases over demo3_unpack_payload, on fixtures of a few hundred
# bytes: an archive that is valid but carries trailing padding, and an archive
# whose every compressed byte is intact but whose zstd frame fails its own
# checksum. The first must unpack; the second must be refused.
#
# The second is the case a shape that reads only tar's status gets wrong. The
# decoder detects the mismatch at the end of the frame and by then has written
# nothing at all, so tar sees an empty stream, reports no error and exits 0, and
# the install directory stays empty while the run goes on to start services.
#
# Nothing here reads the offline payload: the fixtures are made with the same
# pinned zstd, so this test costs kilobytes rather than the 635 MB its siblings
# in this package cost.

set -Eeuo pipefail

: "${HCTL2_DEMO3_COMMON:?Buck must provide HCTL2_DEMO3_COMMON}"
# shellcheck source=demo3.sh
source "$HCTL2_DEMO3_COMMON"

zstd_bin="$HCTL2_ZSTD_ROOT/bin/zstd"
work="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-demo3-unpack.XXXXXX")"
trap 'rm -rf -- "$work"' EXIT

bytes() { wc -c <"$1" | tr -d '[:space:]'; }

stage="$work/stage"
mkdir -p "$stage/bin"
printf 'payload\n' >"$stage/bin/hctl2-services"
tar -cf "$work/plain.tar" -C "$stage" .
"$zstd_bin" -q -c "$work/plain.tar" >"$work/good.tar.zst"

# Padding after the end-of-archive marker is not a defect: tar writes whole
# blocks, and anything that concatenated or padded the stream leaves zeros
# there. It is also the input that made the decoder die of SIGPIPE once tar
# stopped reading. Piped rather than written out, so the 40 MiB never lands on
# the runner's disk.
{ cat "$work/plain.tar"; head -c 41943040 /dev/zero; } |
    "$zstd_bin" -q -c >"$work/padded.tar.zst"

# Flip the frame's trailing content checksum: every compressed byte still
# decodes, and only the decoder knows the result is not what was written.
cp "$work/good.tar.zst" "$work/badsum.tar.zst"
last=$(( $(bytes "$work/badsum.tar.zst") - 1 ))
printf '\377' |
    dd of="$work/badsum.tar.zst" bs=1 seek="$last" count=1 conv=notrunc 2>/dev/null
# Same length and one byte different is what makes this a corrupt checksum
# rather than a truncated stream: truncation would fail on tar's side too, and
# the case below would then prove nothing about the decoder's status.
if cmp -s "$work/good.tar.zst" "$work/badsum.tar.zst"; then
    die "the corrupt-checksum fixture came out byte-identical to the good one"
fi
if [[ "$(bytes "$work/good.tar.zst")" != "$(bytes "$work/badsum.tar.zst")" ]]; then
    die "the corrupt-checksum fixture changed length, so it is not a checksum case"
fi

failures=0

unpack_case() { # <label> <archive> <unpacked|refused>
    local label="$1" archive="$2" expect="$3"
    local dest="$work/out-$label" status=0 verdict
    mkdir -p "$dest"
    demo3_unpack_payload "$work/$archive" "$dest" 2>"$work/$label.err" || status=$?
    if [[ "$expect" == unpacked ]]; then
        if [[ "$status" -eq 0 && -f "$dest/bin/hctl2-services" ]]; then
            verdict=PASS
        else
            verdict=FAIL
        fi
    elif [[ "$status" -ne 0 ]]; then
        verdict=PASS
    else
        verdict=FAIL
    fi
    [[ "$verdict" == PASS ]] || failures=$((failures + 1))
    printf 'demo3-unpack: %-4s %-34s exit=%s payload=%s\n' "$verdict" "$label" \
        "$status" \
        "$( [[ -f "$dest/bin/hctl2-services" ]] && printf yes || printf no )"
    if [[ -s "$work/$label.err" ]]; then
        sed 's/^/demo3-unpack:   stderr| /' "$work/$label.err" | head -n 3
    fi
}

unpack_case padded  padded.tar.zst unpacked
unpack_case badsum  badsum.tar.zst refused

if [[ "$failures" -ne 0 ]]; then
    die "$failures of 2 unpack cases failed"
fi
note "both unpack cases hold"
