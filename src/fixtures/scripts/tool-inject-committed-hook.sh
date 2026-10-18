#!/bin/sh
[ "$1" = "committed" ] || exit 0
printf 'injected\n' >> "$HCTL2_TEST_INJECT"
