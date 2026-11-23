#!/bin/sh
# GitHub read fixture for the CLI repo-registration tests: the account lookup and
# the repository lookup answer the fixed records.
if [ "$GH_TOKEN" != hctl2-fixture-control-token ] || [ -n "$GITHUB_TOKEN$GH_ENTERPRISE_TOKEN$GH_DEBUG" ]; then
  printf 'control credential was not isolated\n' >&2; exit 1
fi
printf '%s\n' "$@" >> "$GH_CONFIG_DIR/commands"
if [ -f "$GH_CONFIG_DIR/reject" ]; then
  printf 'gh: credential rejected (HTTP 401)\n' >&2
  exit 1
fi
case "$6" in
 user) printf '%s\n' '{"id":9,"login":"fixture-owner"}' ;;
 repos/a/b) printf '%s\n' '{"id":12,"full_name":"a/b","clone_url":"https://github.com/a/b.git","has_issues":true,"permissions":{"push":true}}' ;;
 *) exit 1 ;;
esac
