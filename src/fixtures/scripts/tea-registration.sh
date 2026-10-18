#!/bin/sh
# Registration-time `tea` fixture: a POST records the call and answers 503 until
# the created marker exists, after which it answers with the prepared repository.
# A subprocess fixture, not an HTTP server: the native tea behavior is also
# exercised by complete-test. State lives beside the invoked path (`$0.*`).
if [ "$4" = POST ]; then
    printf x >>"$0.posts"
    touch "$0.created"
    printf 'HTTP/1.1 503 Service Unavailable\n' >&2
    printf '{}'
elif [ -f "$0.created" ]; then
    printf 'HTTP/1.1 200 OK\n' >&2
    cat "$0.json"
else
    printf 'HTTP/1.1 404 Not Found\n' >&2
    printf '{}'
fi
