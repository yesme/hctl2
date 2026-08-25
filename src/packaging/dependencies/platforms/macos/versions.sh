#!/usr/bin/env bash
# Architecture-independent macOS build-host and linked-library locks.

# shellcheck disable=SC2034 # This file is sourced as a platform lock file.

readonly MACOS_BISON_VERSION="3.8.2"
readonly MACOS_PKGCONF_VERSION="3.0.6"
readonly MACOS_DEPLOYMENT_TARGET="13.0"
readonly MACOS_TMUX_BUILD_RECIPE="5"

readonly MACOS_LIBEVENT_VERSION="2.1.13"
readonly MACOS_LIBEVENT_SOURCE_ASSET="libevent-2.1.13-stable.tar.gz"
readonly MACOS_LIBEVENT_SOURCE_URL="https://github.com/libevent/libevent/releases/download/release-2.1.13-stable/$MACOS_LIBEVENT_SOURCE_ASSET"
readonly MACOS_LIBEVENT_SOURCE_SHA256="f7e9383b8c0baa81b687e5b5eecc01beefaf1b19b64151d95ed61647fe7a315c"

readonly MACOS_NCURSES_VERSION="6.6"
readonly MACOS_NCURSES_SOURCE_ASSET="ncurses-6.6.tar.gz"
readonly MACOS_NCURSES_SOURCE_URL="https://invisible-mirror.net/archives/ncurses/$MACOS_NCURSES_SOURCE_ASSET"
readonly MACOS_NCURSES_SOURCE_SHA256="355b4cbbed880b0381a04c46617b7656e362585d52e9cf84a67e2009b749ff11"

readonly MACOS_UTF8PROC_VERSION="2.11.3"
readonly MACOS_UTF8PROC_SOURCE_ASSET="utf8proc-2.11.3.tar.gz"
readonly MACOS_UTF8PROC_SOURCE_URL="https://github.com/JuliaStrings/utf8proc/archive/refs/tags/v2.11.3.tar.gz"
readonly MACOS_UTF8PROC_SOURCE_SHA256="abfed50b6d4da51345713661370290f4f4747263ee73dc90356299dfc7990c78"
