#!/bin/sh
# Bootstrap-time Gitea admin fixture: the first `admin user list` answers with the
# error a HALF-migrated database gives, then the ready marker makes it succeed.
# State lives beside the invoked path (`$0.*`).
sub="$7"
case "$sub" in
list)
    if [ -f "$0.ready" ]; then
        printf 'ID\tUsername\tEmail\tIsActive\tIsAdmin\n'
        exit 0
    fi
    : >"$0.ready"
    printf 'Command error: SQL logic error: no such table: user (1)\n' >&2
    exit 1
    ;;
create)
    exit 0
    ;;
generate-access-token)
    printf '%s\n' 0123456789abcdef0123456789abcdef01234567
    ;;
esac
exit 0
