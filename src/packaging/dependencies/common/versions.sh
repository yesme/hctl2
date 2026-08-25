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

readonly TUWUNEL_PORT="6167"
readonly VIKUNJA_PORT="3456"
readonly DAGU_PORT="18080"
readonly DAGU_SCHEDULER_PORT="18090"
readonly DAGU_COORDINATOR_PORT="15055"
readonly DAGU_COORDINATOR_HEALTH_PORT="18091"
readonly TMUX_SESSION="hctl2-services"
