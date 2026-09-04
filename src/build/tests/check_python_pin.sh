#!/usr/bin/env bash
# Adversarial checks for the pinned host Python (#157 / PR #165).
# Portable: bash 3.2. Does not nest a Buck2 daemon.
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
python_pin="${HOST_PYTHON3:-$script_dir/../tools/host-bin/python3}"
launcher="${BUCK2_LAUNCHER:-$script_dir/../../buck2}"
research="${PYTHON_RESEARCH:-}"

if [ -z "$research" ]; then
    research="$(CDPATH= cd -- "$script_dir/../../.." && pwd)/docs/research/build-tools/python-build-standalone.md"
fi

failures=0
fail() {
    printf 'FAIL %s\n' "$*" >&2
    failures=$((failures + 1))
}
note() { printf '%s\n' "$*"; }

[[ -f "$python_pin" ]] || { echo "missing pinned python: $python_pin" >&2; exit 1; }
[[ -f "$launcher" ]] || { echo "missing launcher: $launcher" >&2; exit 1; }

# Helper scripts must not use PATH `python3`: CI poisons PATH to prove the pin,
# and this test also puts a fake python3 first.
if [ -x /usr/bin/python3 ]; then
    helper_python=/usr/bin/python3
elif [ -x /usr/local/bin/python3 ]; then
    helper_python=/usr/local/bin/python3
else
    echo "missing helper python3 at /usr/bin/python3" >&2
    exit 1
fi

# --- 1. pinned interpreter is the one that runs ---------------------------
version="$("$python_pin" -c 'import sys; print(sys.version.split()[0])')"
if [ "$version" = "3.12.14" ]; then
    note "PASS pinned interpreter reports Python 3.12.14"
else
    fail "pinned interpreter version is $version, expected 3.12.14"
fi

executable="$("$python_pin" -c 'import sys; print(sys.executable)')"
prefix="$("$python_pin" -c 'import sys; print(sys.prefix)')"
inner="$prefix/bin/python3"
if head -n 1 "$python_pin" | grep -F '/usr/bin/env dotslash' >/dev/null &&
    [ "$executable" != "$inner" ] && [ -x "$inner" ]
then
    note "PASS DotSlash cache interpreter is executable and differs from the trampoline"
else
    fail "DotSlash cache interpreter is unavailable or is the trampoline: $executable (inner=$inner)"
fi
case "$prefix" in
    */Caches/dotslash/* | */.cache/dotslash/* | */dotslash/*)
        note "PASS sys.prefix is the DotSlash cache extract"
        ;;
    *)
        fail "sys.prefix is not a DotSlash cache path: $prefix"
        ;;
esac

fake="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-poison-python.XXXXXX")"
trap 'rm -rf "$fake"' EXIT
printf '%s\n' '#!/bin/sh' 'echo POISONED_PYTHON >&2' 'exit 1' >"$fake/python3"
chmod +x "$fake/python3"
poisoned_version="$(PATH="$fake:$PATH" "$python_pin" -c 'import sys; print(sys.version.split()[0])')"
if [ "$poisoned_version" = "3.12.14" ]; then
    note "PASS PATH poison does not divert the pinned interpreter"
else
    fail "PATH poison changed the interpreter: $poisoned_version"
fi
if PATH="$fake:$PATH" command -v python3 | grep -F "$fake/python3" >/dev/null; then
    note "PASS poison python3 is first on PATH"
else
    fail "poison python3 was not first on PATH"
fi

# --- broken pin: current-platform digest must fail closed -----------------
broken="$fake/broken-python3"
"$helper_python" - "$python_pin" "$broken" <<'PY'
from pathlib import Path
import re
import sys

src, dst = Path(sys.argv[1]), Path(sys.argv[2])
text = src.read_text()

def flip(match):
    digest = match.group(1)
    tail = "0" if digest[-1] != "0" else "1"
    return '"digest": "%s%s"' % (digest[:-1], tail)

new, count = re.subn(r'"digest": "([0-9a-f]{64})"', flip, text)
if count != 3:
    sys.stderr.write("expected to rewrite 3 digests, got %s\n" % count)
    sys.exit(2)
dst.write_text(new)
dst.chmod(0o755)
PY
set +e
"$broken" -c 'print(1)' >"$fake/broken.out" 2>"$fake/broken.err"
broken_rc=$?
set -e
if [ "$broken_rc" -ne 0 ] && grep -E 'incorrect digest|failed to verify artifact' "$fake/broken.err" >/dev/null; then
    note "PASS corrupting the current-platform digest fails in DotSlash"
else
    fail "broken digest did not fail in DotSlash (exit $broken_rc)"
    cat "$fake/broken.err" >&2 || true
fi

if grep -E 'host-bin/python3.*2>/dev/null \|\| true' "$launcher" >/dev/null; then
    fail "src/buck2 still swallows a broken pin with || true"
else
    note "PASS src/buck2 no longer swallows a broken pin"
fi
if grep -F 'pinned host Python at build/tools/host-bin/python3 is unusable' "$launcher" >/dev/null; then
    note "PASS src/buck2 fail-closed error is present"
else
    fail "src/buck2 is missing the fail-closed error for a broken pin"
fi
if grep -F 'print(sys.prefix + "/bin/python3")' "$launcher" >/dev/null; then
    note "PASS src/buck2 injects the shared DotSlash cache interpreter"
else
    fail "src/buck2 does not resolve Python from the shared DotSlash cache"
fi

# --- 2. supply-chain lock -------------------------------------------------
expected_linux=72748da13197c1fb161e3afeef20a6a385ff24f2165e6e2758e47008e7faba4c
expected_macos_arm=81a359f1cfadd4da11766534c5913791cea55f26e1bb902cacd2a531bb1e4b2b
expected_macos_x86=65b195c9cedc1fef6767f044f9822069adbd1bd9204d424ece4628776fdc04bb

digest_of() {
    "$helper_python" - "$python_pin" "$1" <<'PY'
from pathlib import Path
import json, sys
text = Path(sys.argv[1]).read_text()
start = text.find("{")
payload = json.loads(text[start:])
print(payload["platforms"][sys.argv[2]]["digest"])
PY
}

got_linux="$(digest_of linux-x86_64)"
got_macos_arm="$(digest_of macos-aarch64)"
got_macos_x86="$(digest_of macos-x86_64)"
if [ "$got_linux" = "$expected_linux" ] &&
    [ "$got_macos_arm" = "$expected_macos_arm" ] &&
    [ "$got_macos_x86" = "$expected_macos_x86" ]; then
    note "PASS DotSlash digests match the python-build-standalone 20260901 table"
else
    fail "DotSlash digest mismatch: linux=$got_linux arm=$got_macos_arm x86=$got_macos_x86"
fi

official_json="$fake/release.json"
if command -v gh >/dev/null 2>&1 &&
    gh release view 20260901 --repo astral-sh/python-build-standalone --json assets >"$official_json" 2>/dev/null
then
    "$helper_python" - "$official_json" "$expected_linux" "$expected_macos_arm" "$expected_macos_x86" <<'PY'
import json
import sys
from pathlib import Path

data = json.loads(Path(sys.argv[1]).read_text())
want = {
    "cpython-3.12.14+20260901-x86_64-unknown-linux-gnu-install_only_stripped.tar.gz": sys.argv[2],
    "cpython-3.12.14+20260901-aarch64-apple-darwin-install_only_stripped.tar.gz": sys.argv[3],
    "cpython-3.12.14+20260901-x86_64-apple-darwin-install_only_stripped.tar.gz": sys.argv[4],
}
assets = {row["name"]: row.get("digest", "") for row in data["assets"]}
missing = []
for name, digest in want.items():
    got = assets.get(name, "")
    hexdigest = got.split(":", 1)[-1]
    if hexdigest != digest:
        missing.append("%s official=%s pin=%s" % (name, got, digest))
if missing:
    raise SystemExit("\n".join(missing))
PY
    note "PASS official GitHub release 20260901 digests match the pin"
else
    note "SKIP live GitHub digest check (gh unavailable)"
fi

if [ -f "$research" ]; then
    if grep -F "$expected_linux" "$research" >/dev/null &&
        grep -F "$expected_macos_arm" "$research" >/dev/null &&
        grep -F "$expected_macos_x86" "$research" >/dev/null; then
        note "PASS research file lists the same three SHA-256 values"
    else
        fail "research file is missing one of the pinned SHA-256 values"
    fi
fi

# Compiler injection replica of src/buck2. On Darwin the pin must be the
# /usr/bin xcrun shim, not the Xcode.app toolchain binary.
if [ "$(uname -s)" = Darwin ]; then
    if [ -x /usr/bin/clang ]; then
        cc_bin=/usr/bin/clang
        cxx_bin=/usr/bin/clang++
        ar_bin=/usr/bin/ar
    else
        cc_bin=$(xcrun --find clang)
        cxx_bin=$(xcrun --find clang++)
        ar_bin=$(xcrun --find ar)
    fi
    if [ "$cc_bin" = /usr/bin/clang ] &&
        [ "$cxx_bin" = /usr/bin/clang++ ] &&
        [ "$ar_bin" = /usr/bin/ar ]; then
        note "PASS macOS cc/cxx/ar resolve to /usr/bin shims"
    else
        fail "macOS compiler paths are $cc_bin $cxx_bin $ar_bin"
    fi
    xcode_clang=$(xcrun --find clang)
    case "$xcode_clang" in
        */Xcode.app/Contents/Developer/Toolchains/*)
            if [ "$cc_bin" != "$xcode_clang" ]; then
                note "PASS injected clang is not the Xcode.app toolchain binary"
            else
                fail "injected clang is the Xcode.app toolchain binary: $cc_bin"
            fi
            ;;
        *)
            note "PASS xcrun clang is $xcode_clang"
            ;;
    esac
else
    cc_bin=$(command -v clang || command -v cc || true)
    case "$cc_bin" in
        /*)
            note "PASS Linux cc resolves to $cc_bin"
            ;;
        *)
            fail "Linux cc did not resolve to an absolute path: $cc_bin"
            ;;
    esac
fi

if grep -F 'hctl2.python=$py_bin' "$launcher" >/dev/null &&
    grep -F 'hctl2.cc=$cc_bin' "$launcher" >/dev/null; then
    note "PASS launcher still injects hctl2.python / hctl2.cc"
else
    fail "launcher lost hctl2.python or hctl2.cc injection"
fi

if [ "$failures" -ne 0 ]; then
    echo "check_python_pin: FAILED ($failures)" >&2
    exit 1
fi
echo "check_python_pin: OK"
