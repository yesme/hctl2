#!/bin/sh
# gh fixture: every check run and commit status for the commit succeeded.
if printf '%s' "$*" | grep check-runs >/dev/null; then
  echo '{"check_runs":[{"status":"completed","conclusion":"success"}]}'
else
  echo '{"state":"success","statuses":[{"state":"success"}]}'
fi
