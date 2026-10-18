#!/bin/sh
# Process fixture for the foundation command tests. The first argument names the
# behaviour; later arguments are that behaviour's own. A built file is executed
# here instead of a script written by the test: a file that the test just opened
# for writing can answer execve with ETXTBSY while another test thread's forked
# child still holds the inherited write descriptor (Linux execve(2)).
set -u
case "${1:-}" in
sleep-exec) exec sleep "$2" ;;
sleep-hold)
    sleep "$2" &
    exit 0
    ;;
sleep-hold-exec)
    sleep "$2" &
    exec sleep "$3"
    ;;
echo-stdin)
    cat >"$2"
    cat "$2"
    ;;
pid-then-cat)
    echo $$ >"$2"
    exec cat
    ;;
refuse)
    echo refused >&2
    exit 3
    ;;
*)
    echo "unknown command fixture: ${1:-}" >&2
    exit 2
    ;;
esac
