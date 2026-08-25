#!/usr/bin/env bash
# Pinned dependency supply-chain inputs.

# shellcheck disable=SC2034 # This file is sourced as a cross-script lock file.

# Pinned dependency inputs for the Ubuntu x86_64 package.
readonly TUWUNEL_VERSION="1.9.0"
readonly TUWUNEL_ASSET="tuwunel-v1.9.0-linux-amd64.deb"
readonly TUWUNEL_URL="https://github.com/matrix-construct/tuwunel/releases/download/v1.9.0/v1.9.0-release-all-x86_64-v1-linux-gnu-tuwunel.deb"
readonly TUWUNEL_SHA256="1d5d9d326e16b52a3e8b13d6e847fc1b661803a415c189d90b2755bef236c613"
readonly TUWUNEL_SOURCE_COMMIT="5b3669144219d5d4c0774743c84191b476f1b54f"
readonly TUWUNEL_SOURCE_ASSET="tuwunel-$TUWUNEL_SOURCE_COMMIT-source.tar.gz"
readonly TUWUNEL_SOURCE_URL="https://github.com/matrix-construct/tuwunel/archive/$TUWUNEL_SOURCE_COMMIT.tar.gz"
readonly TUWUNEL_SOURCE_SHA256="e3a1844fe969660e4d418e9a1013718d4ec92f9289b0c7c122ef7eccee91bafa"

readonly VIKUNJA_VERSION="2.5.0"
readonly VIKUNJA_ASSET="vikunja-v2.5.0-linux-amd64-full.zip"
readonly VIKUNJA_URL="https://github.com/go-vikunja/vikunja/releases/download/v2.5.0/vikunja-v2.5.0-linux-amd64-full.zip"
readonly VIKUNJA_SHA256="8843de18f5f297bac83db010a54064a45033f82cffdf53421f6ce39f12a8ad98"
readonly VIKUNJA_SOURCE_COMMIT="ef2200e9429c5cc42f5c1811433418bfcc72b3aa"
readonly VIKUNJA_SOURCE_ASSET="vikunja-$VIKUNJA_SOURCE_COMMIT-source.tar.gz"
readonly VIKUNJA_SOURCE_URL="https://github.com/go-vikunja/vikunja/archive/$VIKUNJA_SOURCE_COMMIT.tar.gz"
readonly VIKUNJA_SOURCE_SHA256="b4ff02d3484321613eeca34a84d4e64d955e28ed03bdfe3535b431e03e0929d6"

readonly DAGU_VERSION="2.15.1"
readonly DAGU_ASSET="dagu-v2.15.1-linux-amd64.tar.gz"
readonly DAGU_URL="https://github.com/dagucloud/dagu/releases/download/v2.15.1/dagu_2.15.1_linux_amd64.tar.gz"
readonly DAGU_SHA256="cfadd9606af9ff74d0ea8aef91fd89a831b3c63cf99a95c456fe2fcb3bb0471e"
readonly DAGU_SOURCE_COMMIT="532c512944b2e5eb8991b5bc7cbeafa74fd5b47a"
readonly DAGU_SOURCE_ASSET="dagu-$DAGU_SOURCE_COMMIT-source.tar.gz"
readonly DAGU_SOURCE_URL="https://github.com/dagucloud/dagu/archive/$DAGU_SOURCE_COMMIT.tar.gz"
readonly DAGU_SOURCE_SHA256="9f6e4a4e5e2cb63e0c46e9b5e7b4d4a4a1c3edc452433dbd60cb83e75538ce0a"

readonly TMUX_VERSION="3.7c"
readonly TMUX_ASSET="tmux-3.7c.tar.gz"
readonly TMUX_URL="https://github.com/tmux/tmux/releases/download/3.7c/tmux-3.7c.tar.gz"
readonly TMUX_SHA256="7c60cae9a0e25288e2e24750aafc9e8800fc7fd4555e447e1b29ee4201cfb3bf"
readonly TMUX_SOURCE_COMMIT="e476c1230b958df0cb12977517d24b3dc931375b"
# shellcheck disable=SC2016 # The dynamic loader, not Bash, expands $ORIGIN.
readonly TMUX_RELATIVE_RUNPATH='$ORIGIN/../../lib/hctl2/vendor'
readonly TMUX_BUILD_BISON_APT_VERSION="2:3.8.2+dfsg-1build4"
readonly TMUX_BUILD_LIBEVENT_APT_VERSION="2.1.12-stable-10build2"
readonly TMUX_BUILD_NCURSES_APT_VERSION="6.6+20251231-1"

readonly TUWUNEL_PORT="6167"
readonly VIKUNJA_PORT="3456"
readonly DAGU_PORT="18080"
readonly DAGU_SCHEDULER_PORT="18090"
readonly DAGU_COORDINATOR_PORT="15055"
readonly DAGU_COORDINATOR_HEALTH_PORT="18091"
readonly TMUX_SESSION="hctl2-services"
