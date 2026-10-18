#!/bin/sh
# Packaged services stub: records its arguments beside the invoked path and
# succeeds, so a test can assert the lifecycle calls the supervisor made.
printf '%s\n' "$*" >>"$0.calls"
exit 0
