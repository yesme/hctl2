#!/usr/bin/env bash
# Locked inputs for the Linux x86_64 release target.

# shellcheck disable=SC2034 # This file is sourced as a target descriptor.

readonly HCTL2_TARGET_ID="linux-x86_64"
readonly HCTL2_TARGET_OS="linux"
readonly HCTL2_TARGET_ARCH="x86_64"
readonly HCTL2_TARGET_UNAME_S="Linux"
readonly HCTL2_TARGET_UNAME_M="x86_64"
readonly HCTL2_RUST_TARGET="x86_64-unknown-linux-gnu"

readonly TUWUNEL_ASSET="tuwunel-v1.9.0-linux-amd64.deb"
readonly TUWUNEL_URL="https://github.com/matrix-construct/tuwunel/releases/download/v1.9.0/v1.9.0-release-all-x86_64-v1-linux-gnu-tuwunel.deb"
readonly TUWUNEL_SHA256="1d5d9d326e16b52a3e8b13d6e847fc1b661803a415c189d90b2755bef236c613"
readonly TUWUNEL_BUILD_INPUT_SHA256="$TUWUNEL_SHA256"

readonly VIKUNJA_ASSET="vikunja-v2.5.0-linux-amd64-full.zip"
readonly VIKUNJA_URL="https://github.com/go-vikunja/vikunja/releases/download/v2.5.0/vikunja-v2.5.0-linux-amd64-full.zip"
readonly VIKUNJA_SHA256="8843de18f5f297bac83db010a54064a45033f82cffdf53421f6ce39f12a8ad98"
readonly VIKUNJA_BUILD_INPUT_SHA256="$VIKUNJA_SHA256"

readonly DAGU_ASSET="dagu-v2.15.1-linux-amd64.tar.gz"
readonly DAGU_URL="https://github.com/dagucloud/dagu/releases/download/v2.15.1/dagu_2.15.1_linux_amd64.tar.gz"
readonly DAGU_SHA256="cfadd9606af9ff74d0ea8aef91fd89a831b3c63cf99a95c456fe2fcb3bb0471e"
readonly DAGU_BUILD_INPUT_SHA256="$DAGU_SHA256"

readonly TMUX_ASSET="tmux-3.7c-linux-x86_64.tar.gz"
readonly TMUX_URL="https://github.com/tmux/tmux-builds/releases/download/v${TMUX_VERSION}/${TMUX_ASSET}"
readonly TMUX_SHA256="cc56bd1cc873eb6089c615f0496b072385bda8a6d944069f38564a5d49c128aa"
readonly TMUX_BUILD_INPUT_SHA256="$TMUX_SHA256"

readonly STATIC_WEB_SERVER_ASSET="static-web-server-v${STATIC_WEB_SERVER_VERSION}-x86_64-unknown-linux-musl.tar.gz"
readonly STATIC_WEB_SERVER_URL="https://github.com/static-web-server/static-web-server/releases/download/v${STATIC_WEB_SERVER_VERSION}/${STATIC_WEB_SERVER_ASSET}"
readonly STATIC_WEB_SERVER_SHA256="804bc0c31c78385ac04e9a36f3c2aa3d3170eb77d66807c2a1660c56b2026bb1"
readonly STATIC_WEB_SERVER_BUILD_INPUT_SHA256="$STATIC_WEB_SERVER_SHA256"
