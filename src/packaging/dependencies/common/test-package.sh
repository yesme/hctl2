#!/usr/bin/env bash
# Shared offline install and lifecycle verification.

test_dependency_package() {
    local test_root
    local prefix
    local state_root
    local services

    [[ -n "${PACKAGE_ID:-}" && -n "${ARCHIVE:-}" ]] || \
        die "assemble_dependency_package must run before test_dependency_package"

    test_root="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-package-test.XXXXXX")"
    prefix="$test_root/prefix"
    state_root="$test_root/state"
    services="$prefix/bin/hctl2-services"

    case "$test_root" in
        /*/hctl2-package-test.*) ;;
        *) die "unsafe package test directory: $test_root" ;;
    esac

    PACKAGE_TEST_ROOT_TO_CLEAN="$test_root"
    PACKAGE_TEST_SERVICES="$services"
    PACKAGE_TEST_STATE_ROOT="$state_root"
    readonly PACKAGE_TEST_ROOT_TO_CLEAN PACKAGE_TEST_SERVICES PACKAGE_TEST_STATE_ROOT
    cleanup_package_test() {
        local test_exit=$?

        if [[ -x "$PACKAGE_TEST_SERVICES" ]] &&
            ! HCTL2_STATE_ROOT="$PACKAGE_TEST_STATE_ROOT" "$PACKAGE_TEST_SERVICES" stop >/dev/null 2>&1; then
            printf 'warning: preserving failed package test at %s because managed services did not stop\n' \
                "$PACKAGE_TEST_ROOT_TO_CLEAN" >&2
            return "$test_exit"
        fi
        find "$PACKAGE_TEST_ROOT_TO_CLEAN" -depth -delete
        return "$test_exit"
    }
    trap cleanup_package_test EXIT

    tar -xzf "$ARCHIVE" -C "$test_root"
    grep -F '# HCTL2 使用说明' "$test_root/$PACKAGE_ID/USAGE.md" >/dev/null
    "$test_root/$PACKAGE_ID/install.sh" --prefix "$prefix"
    "$test_root/$PACKAGE_ID/install.sh" --prefix "$prefix"
    "$services" --help | grep -F 'Usage:' >/dev/null
    if "$services" status unexpected >/dev/null 2>&1; then
        die "hctl2-services accepted an argument for status"
    fi
    HCTL2_STATE_ROOT="$state_root" "$services" start
    HCTL2_STATE_ROOT="$state_root" "$services" smoke
    HCTL2_STATE_ROOT="$state_root" "$services" stop

    note "$HCTL2_TARGET_ID offline package install and lifecycle test passed"
}
