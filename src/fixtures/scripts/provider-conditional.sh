#!/bin/sh
printf '%s\n' "$*" >> "$0.calls"
case "$*" in
 *limited*) printf 'HTTP/2.0 429 Too Many Requests\nRetry-After: 3600\n\n{}' ;;
 *If-None-Match*) printf 'HTTP/2.0 304 Not Modified\n\n' ;;
 *) printf 'HTTP/2.0 200 OK\nETag: "v1"\n\n{"id":1}' ;;
esac
