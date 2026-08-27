#!/usr/bin/env bash
# Platform-independent dependency versions and source pins.

# shellcheck disable=SC2034 # This file is sourced as a cross-script lock file.

readonly TUWUNEL_VERSION="1.9.0"
readonly TUWUNEL_RUST_TOOLCHAIN="1.95.0"
readonly TUWUNEL_SOURCE_COMMIT="5b3669144219d5d4c0774743c84191b476f1b54f"
readonly TUWUNEL_SOURCE_ASSET="tuwunel-$TUWUNEL_SOURCE_COMMIT-source.tar.gz"
readonly TUWUNEL_SOURCE_URL="https://github.com/matrix-construct/tuwunel/archive/$TUWUNEL_SOURCE_COMMIT.tar.gz"
readonly TUWUNEL_SOURCE_SHA256="e3a1844fe969660e4d418e9a1013718d4ec92f9289b0c7c122ef7eccee91bafa"

readonly VIKUNJA_VERSION="2.5.0"
readonly VIKUNJA_SOURCE_COMMIT="ef2200e9429c5cc42f5c1811433418bfcc72b3aa"
readonly VIKUNJA_SOURCE_ASSET="vikunja-$VIKUNJA_SOURCE_COMMIT-source.tar.gz"
readonly VIKUNJA_SOURCE_URL="https://github.com/go-vikunja/vikunja/archive/$VIKUNJA_SOURCE_COMMIT.tar.gz"
readonly VIKUNJA_SOURCE_SHA256="b4ff02d3484321613eeca34a84d4e64d955e28ed03bdfe3535b431e03e0929d6"

readonly DAGU_VERSION="2.15.1"
readonly DAGU_SOURCE_COMMIT="532c512944b2e5eb8991b5bc7cbeafa74fd5b47a"
readonly DAGU_SOURCE_ASSET="dagu-$DAGU_SOURCE_COMMIT-source.tar.gz"
readonly DAGU_SOURCE_URL="https://github.com/dagucloud/dagu/archive/$DAGU_SOURCE_COMMIT.tar.gz"
readonly DAGU_SOURCE_SHA256="9f6e4a4e5e2cb63e0c46e9b5e7b4d4a4a1c3edc452433dbd60cb83e75538ce0a"

readonly TMUX_VERSION="3.7c"
readonly TMUX_ASSET="tmux-3.7c.tar.gz"
readonly TMUX_URL="https://github.com/tmux/tmux/releases/download/3.7c/tmux-3.7c.tar.gz"
readonly TMUX_SHA256="7c60cae9a0e25288e2e24750aafc9e8800fc7fd4555e447e1b29ee4201cfb3bf"
readonly TMUX_SOURCE_COMMIT="e476c1230b958df0cb12977517d24b3dc931375b"

readonly CINNY_VERSION="4.12.6"
readonly CINNY_SOURCE_COMMIT="33f4ba3674fa4f57e048e81b28f8426defc03eac"
readonly CINNY_ASSET="cinny-v${CINNY_VERSION}.tar.gz"
readonly CINNY_URL="https://github.com/cinnyapp/cinny/releases/download/v${CINNY_VERSION}/${CINNY_ASSET}"
readonly CINNY_SHA256="d478bf11c6101b9c218257772ed6de38f9b1d3ed156f019d14f180cb30592595"
readonly CINNY_SOURCE_ASSET="cinny-$CINNY_SOURCE_COMMIT-source.tar.gz"
readonly CINNY_SOURCE_URL="https://github.com/cinnyapp/cinny/archive/$CINNY_SOURCE_COMMIT.tar.gz"
readonly CINNY_SOURCE_SHA256="df400a9fe206e6fedb87701d6854d05035d9f06dd97558b7b0bb484c8ced8d36"

readonly STATIC_WEB_SERVER_VERSION="2.44.0"
readonly STATIC_WEB_SERVER_SOURCE_COMMIT="27aa3450b6bf70bd1fa553b2197b53032affcba1"
readonly STATIC_WEB_SERVER_SOURCE_ASSET="static-web-server-$STATIC_WEB_SERVER_SOURCE_COMMIT-source.tar.gz"
readonly STATIC_WEB_SERVER_SOURCE_URL="https://github.com/static-web-server/static-web-server/archive/$STATIC_WEB_SERVER_SOURCE_COMMIT.tar.gz"
readonly STATIC_WEB_SERVER_SOURCE_SHA256="fa04fc2ed8d8ff6cce6e1001cb80746bc7f0b9dc635324fb5b72d87ed4b9603f"

readonly TUWUNEL_PORT="6167"
readonly VIKUNJA_PORT="3456"
readonly DAGU_PORT="18080"
readonly DAGU_SCHEDULER_PORT="18090"
readonly DAGU_COORDINATOR_PORT="15055"
readonly DAGU_COORDINATOR_HEALTH_PORT="18091"
readonly CINNY_PORT="6168"
readonly TMUX_SESSION="hctl2-services"
