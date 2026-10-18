#!/bin/sh
# Bootstrap-time `tea` fixture: the first identity read answers 401 and creates the
# `ours` marker; later reads answer with the account written beside it. State lives
# beside the invoked path (`$0.*`).
if [ -f "$0.ours" ]; then
    printf 'HTTP/1.1 200 OK\n' >&2
    printf '{"login":"%s","is_admin":true}\n' "$(cat "$0.user")"
    exit 0
fi
: >"$0.ours"
printf 'HTTP/1.1 401 Unauthorized\n' >&2
printf '{}\n'
exit 0
