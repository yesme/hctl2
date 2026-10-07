#!/usr/bin/env bash
# Adversarial checks for the pinned host Python (#157 / PR #165).
# Portable: bash 3.2. Does not nest a Buck2 daemon.
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
python_pin="${HOST_PYTHON3:-$script_dir/../tools/host-bin/python3}"
launcher="${BUCK2_LAUNCHER:-$script_dir/../../buck2}"
gh_pin="${GH_PIN:-$script_dir/../tools/gh-bin}"
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
poisoned_inner_version="$(PATH="$fake:$PATH" "$inner" -c 'import sys; print(sys.version.split()[0])')"
if [ "$poisoned_inner_version" = "3.12.14" ]; then
    note "PASS injected sys.prefix interpreter reports Python 3.12.14 under PATH poison"
else
    fail "injected sys.prefix interpreter version is $poisoned_inner_version, expected 3.12.14"
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

# The pinned gh, not whatever the host has: an old distro gh omits `digest`.
official_json="$fake/release.json"
if [ -f "$gh_pin" ] &&
    "$gh_pin" release view 20260901 --repo astral-sh/python-build-standalone --json assets >"$official_json" 2>/dev/null
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
    note "SKIP live GitHub digest check (pinned gh unavailable or not signed in)"
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
    cc_bin=$(command -v clang || true)
    case "$cc_bin" in
        /*)
            note "PASS Linux cc resolves to $cc_bin"
            ;;
        *)
            fail "Linux clang did not resolve to an absolute path: $cc_bin"
            ;;
    esac
fi

# Linux takes the prelude's clang + lld as is. No cc/c++ fallback in the
# launcher, and a missing toolchain stops the build with an install hint.
if grep -E 'command -v (cc|c\+\+)( |\))' "$launcher" >/dev/null; then
    fail "src/buck2 still falls back to cc/c++ on Linux"
else
    note "PASS src/buck2 has no cc/c++ fallback"
fi
if grep -F 'Linux builds need clang, clang++ and lld' "$launcher" >/dev/null &&
    grep -F -- '-print-prog-name=ld.lld' "$launcher" >/dev/null; then
    note "PASS src/buck2 stops a Linux build without clang, clang++ and lld"
else
    fail "src/buck2 is missing the Linux clang/lld check"
fi
if grep -F 'macOS builds need a working clang from the Command Line Tools' "$launcher" >/dev/null &&
    grep -F 'xcode-select --install' "$launcher" >/dev/null &&
    grep -F 'A full Xcode.app is not required' "$launcher" >/dev/null; then
    note "PASS src/buck2 asks for the Command Line Tools, not a full Xcode"
else
    fail "src/buck2 is missing the Command Line Tools hint"
fi
if [ "$(uname -s)" = Darwin ]; then
    # Copies stay next to src/buck2 so the launcher still finds the pinned
    # Python. The buck2 binary is replaced with exit 0 so this test does not
    # start a daemon. A failing xcodebuild must not block a working clang.
    printf '%s\n' '#!/bin/sh' 'exit 0' >"$fake/buck2-ok"
    printf '%s\n' '#!/bin/sh' 'exit 1' >"$fake/clang-bad"
    printf '%s\n' '#!/bin/sh' 'echo xcodebuild requires Xcode >&2' 'exit 1' >"$fake/xcodebuild"
    chmod +x "$fake/buck2-ok" "$fake/clang-bad" "$fake/xcodebuild"
    probe="$script_dir/../../buck2.macos-probe"
    bad="$probe-bad"
    sed "s|^buck2_bin=.*|buck2_bin=$fake/buck2-ok|" "$launcher" >"$probe"
    sed -e "s|/usr/bin/clang++|$fake/clang-bad|g" \
        -e "s|/usr/bin/clang|$fake/clang-bad|g" \
        -e "s|/usr/bin/ar|$fake/clang-bad|g" \
        "$probe" >"$bad"
    chmod +x "$probe" "$bad"
    clt_err="$fake/clt-missing.txt"
    if HCTL2_BUCK2_CACHE=0 "$bad" build --help >"$fake/clt-missing.out" 2>"$clt_err"; then
        fail "src/buck2 build continued when clang cannot run"
    elif grep -F 'xcode-select --install' "$clt_err" >/dev/null &&
        grep -F 'Command Line Tools' "$clt_err" >/dev/null &&
        grep -F 'A full Xcode.app is not required' "$clt_err" >/dev/null; then
        note "PASS src/buck2 build stops when clang cannot run and names the Command Line Tools"
    else
        fail "src/buck2 build did not name the Command Line Tools when clang cannot run"
    fi
    if PATH="$fake:$PATH" HCTL2_BUCK2_CACHE=0 "$probe" build --help \
        >"$fake/clt-only.out" 2>"$fake/clt-only.err"; then
        note "PASS src/buck2 build accepts a working clang when xcodebuild is absent"
    else
        fail "src/buck2 build rejected a working clang because xcodebuild failed"
    fi
    rm -f "$probe" "$bad"
fi

# --- the Linux guard is a behaviour, not a string ---------------------------
# The grep above only proves the hint is still in the launcher. Drive the Linux
# branch for real instead: a `uname` that says Linux, and a `clang` that answers
# a bare `ld.lld` (what clang answers when lld is not next to it), so the
# guard's own condition holds. Then require every verb it claims to guard to
# stop with the install hint *before the exec*, and the one it does not claim
# to reach the exec and stay clean.
# A "non-zero exit" assertion is not enough on its own: a guard that prints the
# hint and then falls through also exits non-zero, because the exec below it
# fails on a missing binary. So the probe carries a buck2-bin stand-in that
# records that it was reached and exits 0, and a guarded verb counts as stopped
# only when that marker is absent. --version must reach the stand-in, which
# proves the marker would have shown up had the guard let the exec through.
# $launcher arrives in the sandbox as a bare export_file with no siblings, so
# give the probe the small tree the launcher expects to be sitting in.
guard_root="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-guard-probe.XXXXXX")"
trap 'rm -rf "$fake" "$guard_root"' EXIT
mkdir -p "$guard_root/src/build/tools/host-bin" "$guard_root/bin"
cp "$launcher" "$guard_root/src/buck2"
cp "$python_pin" "$guard_root/src/build/tools/host-bin/python3"
chmod +x "$guard_root/src/build/tools/host-bin/python3"
probe_launcher="$guard_root/src/buck2"
cat > "$guard_root/src/build/tools/buck2-bin" <<'SH'
#!/bin/sh
# Stands in for the real Buck2. Succeeds, so the probe can tell "the guard
# stopped here" apart from "the exec fell over on a missing binary".
: > "$GUARD_PROBE_MARKER"
echo "buck2 stand-in reached the exec"
exit 0
SH
chmod +x "$guard_root/src/build/tools/buck2-bin"
cat > "$guard_root/bin/uname" <<'SH'
#!/bin/sh
echo Linux
SH
cat > "$guard_root/bin/clang" <<'SH'
#!/bin/sh
case "$*" in
    # No lld next to this clang: clang hands back the bare name, not a path.
    *-print-prog-name=ld.lld*) echo "ld.lld" ;;
    *) echo "clang version 18.1.3" ;;
esac
SH
cat > "$guard_root/bin/clang++" <<'SH'
#!/bin/sh
echo "clang version 18.1.3"
SH
chmod +x "$guard_root/bin/uname" "$guard_root/bin/clang" "$guard_root/bin/clang++"

guard_hint='Linux builds need clang, clang++ and lld'
guard_bad=0
guard_rc=0
for verb in build test run; do
    marker="$guard_root/reached-$verb"
    guard_out="$(PATH="$guard_root/bin:$PATH" GUARD_PROBE_MARKER="$marker" \
        HCTL2_BUCK2_CACHE=0 sh "$probe_launcher" "$verb" --help 2>&1)" \
        && guard_rc=0 || guard_rc=$?
    if [ -e "$marker" ]; then
        guard_bad=1
        note "guard probe: '$verb' reached the Buck2 exec instead of stopping"
    fi
    if [ "$guard_rc" -eq 0 ]; then
        guard_bad=1
        note "guard probe: '$verb' exited 0 with no clang and no lld"
    fi
    case "$guard_out" in
        *"$guard_hint"*) ;;
        *)
            guard_bad=1
            note "guard probe: '$verb' exited $guard_rc without the install hint: $guard_out"
            ;;
    esac
done
# The guard is scoped to build/run/test. A verb it does not claim must not
# collect the build hint, and it must still reach the exec — that positive
# control is what makes the three "did not reach the exec" checks above mean
# anything. Its exit code is not this assertion's business.
marker="$guard_root/reached-version"
guard_out="$(PATH="$guard_root/bin:$PATH" GUARD_PROBE_MARKER="$marker" \
    HCTL2_BUCK2_CACHE=0 sh "$probe_launcher" --version 2>&1)" || true
if [ -e "$marker" ]; then
    note "PASS the probe can see the exec, so the three checks above are real"
else
    guard_bad=1
    note "guard probe: '--version' never reached the Buck2 stand-in; \
the probe cannot tell a stopped guard from a failed exec"
fi
case "$guard_out" in
    *"$guard_hint"*)
        guard_bad=1
        note "guard probe: '--version' collected the build hint"
        ;;
esac
if [ "$guard_bad" -eq 0 ]; then
    note "PASS the Linux guard stops build/test/run before the exec and leaves --version alone"
else
    fail "the Linux guard does not behave the way its message claims"
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
