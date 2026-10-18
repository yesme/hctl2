#!/bin/sh
# `tea api` fixture: a PUT records the account and the permission it was granted,
# unless the fixture is told to lose the response; a collaborator read answers
# from the accounts recorded so far, naming the account only. State lives beside
# the invoked path (`$0.*`).
method="$4"
if [ "$method" = PUT ]; then
    path="$7"
    body="$(cat)"
else
    path="$5"
fi
name="${path##*/}"
if [ "$method" = GET ]; then
    if cut -f1 "$0.grants" 2>/dev/null | grep -qx "$name"; then
        printf 'HTTP/1.1 204 No Content\n' >&2
    else
        printf 'HTTP/1.1 404 Not Found\n' >&2
        printf '{}'
    fi
    exit 0
fi
printf x >>"$0.puts"
if [ ! -f "$0.lose" ]; then
    permission="${body#*\"permission\":\"}"
    permission="${permission%%\"*}"
    printf '%s\t%s\n' "$name" "$permission" >>"$0.grants"
fi
printf 'HTTP/1.1 204 No Content\n' >&2
exit 0
