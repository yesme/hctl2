#!/usr/bin/env bash
# Locked inputs for the macOS Apple Silicon release target.

# shellcheck disable=SC2034 # This file is sourced as a target descriptor.

readonly HCTL2_TARGET_ID="macos-aarch64"
readonly HCTL2_TARGET_OS="macos"
readonly HCTL2_TARGET_ARCH="aarch64"
readonly HCTL2_TARGET_UNAME_S="Darwin"
readonly HCTL2_TARGET_UNAME_M="arm64"
readonly HCTL2_RUST_TARGET="aarch64-apple-darwin"

# Tuwunel does not publish Darwin artifacts; macOS builds the locked source.
readonly TUWUNEL_ASSET=""
readonly TUWUNEL_URL=""
readonly TUWUNEL_SHA256=""
readonly TUWUNEL_BUILD_INPUT_SHA256="$TUWUNEL_SOURCE_SHA256"

readonly VIKUNJA_ASSET="vikunja-v2.5.0-darwin-10.15-arm64-full.zip"
readonly VIKUNJA_URL="https://github.com/go-vikunja/vikunja/releases/download/v2.5.0/$VIKUNJA_ASSET"
readonly VIKUNJA_SHA256="9c77cb6afddc3191696696f624830620361bf12a0016dd41c4028a7428651d91"
readonly VIKUNJA_BUILD_INPUT_SHA256="$VIKUNJA_SHA256"

readonly DAGU_ASSET="dagu-v2.15.1-darwin-arm64.tar.gz"
readonly DAGU_URL="https://github.com/dagucloud/dagu/releases/download/v2.15.1/dagu_2.15.1_darwin_arm64.tar.gz"
readonly DAGU_SHA256="e7343a08bfb42091c2a177cc1a75f7df5a4fc9c256735dfc769b3bed18f2a319"
readonly DAGU_BUILD_INPUT_SHA256="$DAGU_SHA256"

readonly TMUX_ASSET="tmux-3.7c-macos-arm64.tar.gz"
readonly TMUX_URL="https://github.com/tmux/tmux-builds/releases/download/v${TMUX_VERSION}/${TMUX_ASSET}"
readonly TMUX_SHA256="0a763dd0380aa980d239509654da1bc7455843706a3c050f6709c8cd2e13d12d"
readonly TMUX_BUILD_INPUT_SHA256="$TMUX_SHA256"

readonly STATIC_WEB_SERVER_ASSET="static-web-server-v${STATIC_WEB_SERVER_VERSION}-aarch64-apple-darwin.tar.gz"
readonly STATIC_WEB_SERVER_URL="https://github.com/static-web-server/static-web-server/releases/download/v${STATIC_WEB_SERVER_VERSION}/${STATIC_WEB_SERVER_ASSET}"
readonly STATIC_WEB_SERVER_SHA256="5f22e1d0072a0f5bf8bb32468885d6ba7bc9df470d37f16bdd71f9697832964f"
readonly STATIC_WEB_SERVER_BUILD_INPUT_SHA256="$STATIC_WEB_SERVER_SHA256"
