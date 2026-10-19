#!/bin/sh
# gh fixture: the ref is an ancestor of the compared commit.
if printf '%s' "$*" | grep '/git/ref/' >/dev/null; then
  echo '{"object":{"sha":"def"}}'
else
  echo '{"status":"ahead"}'
fi
