#!/bin/sh
case "$PWD" in *hctl2-herdr-smoke*) echo confined-install-unavailable >&2; exit 9;; esac
echo '2.1.289 (Claude Code)'
