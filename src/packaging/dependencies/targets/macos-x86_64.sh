#!/usr/bin/env bash
# Locked inputs for the macOS Intel release target.

# shellcheck disable=SC2034 # This file is sourced as a target descriptor.

readonly HCTL2_TARGET_ID="macos-x86_64"
readonly HCTL2_TARGET_OS="macos"
readonly HCTL2_TARGET_ARCH="x86_64"
readonly HCTL2_TARGET_UNAME_S="Darwin"
readonly HCTL2_TARGET_UNAME_M="x86_64"
readonly HCTL2_RUST_TARGET="x86_64-apple-darwin"

# Tuwunel does not publish Darwin artifacts; macOS builds the locked source.
readonly TUWUNEL_ASSET=""
readonly TUWUNEL_URL=""
readonly TUWUNEL_SHA256=""
readonly TUWUNEL_BUILD_INPUT_SHA256="$TUWUNEL_SOURCE_SHA256"

readonly VIKUNJA_ASSET="vikunja-v2.5.0-darwin-10.15-amd64-full.zip"
readonly VIKUNJA_URL="https://github.com/go-vikunja/vikunja/releases/download/v2.5.0/$VIKUNJA_ASSET"
readonly VIKUNJA_SHA256="0df68cadc84984372353d3436dbe7593e499d04382e0a1950d579c1a83727982"
readonly VIKUNJA_BUILD_INPUT_SHA256="$VIKUNJA_SHA256"

readonly DAGU_ASSET="dagu-v2.15.1-darwin-amd64.tar.gz"
readonly DAGU_URL="https://github.com/dagucloud/dagu/releases/download/v2.15.1/dagu_2.15.1_darwin_amd64.tar.gz"
readonly DAGU_SHA256="a7fbed8e194668b9ede73de486f06f706574731ccf58fba1cffa86a1ffe84c18"
readonly DAGU_BUILD_INPUT_SHA256="$DAGU_SHA256"
