#!/bin/sh
# GitHub read fixture for the CLI repo-registration tests: the account lookup and
# the repository lookup answer the fixed records.
case "$6" in
 user) printf '%s\n' '{"id":9}' ;;
 repos/a/b) printf '%s\n' '{"id":12,"full_name":"a/b","clone_url":"https://github.com/a/b.git","has_issues":true,"permissions":{"push":true}}' ;;
 *) exit 1 ;;
esac
