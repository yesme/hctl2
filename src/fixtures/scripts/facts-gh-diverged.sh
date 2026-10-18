#!/bin/sh
# gh fixture: the ref and the compared commit have diverged.
if printf '%s' "$*" | grep '/git/ref/' >/dev/null; then
  echo '{"object":{"sha":"def"}}'
else
  echo '{"status":"diverged"}'
fi
