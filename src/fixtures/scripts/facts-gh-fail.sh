#!/bin/sh
# gh fixture: a check run failed while the commit statuses are still pending.
if printf '%s' "$*" | grep check-runs >/dev/null; then
  echo '{"check_runs":[{"status":"completed","conclusion":"failure"}]}'
else
  echo '{"state":"pending","statuses":[]}'
fi
