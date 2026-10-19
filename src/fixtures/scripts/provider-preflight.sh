#!/bin/sh
printf '%s\n' "$*" >> "$0.calls"
case "$*" in
 *'GET repos/owner/repo') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1,"full_name":"owner/repo","permissions":{"admin":false}}' ;;
 *'GET user') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1}' ;;
 *'GET repos/owner/repo/issues/1')
   if [ -f "$0.card" ]; then printf 'HTTP/1.1 200 OK\n' >&2; cat "$0.card";
   else printf 'HTTP/1.1 503 Error\n' >&2; printf '{}'; fi ;;
 *) printf 'HTTP/1.1 500 Unexpected\n' >&2; printf '{}' ;;
esac
