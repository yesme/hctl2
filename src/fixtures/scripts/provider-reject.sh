#!/bin/sh
case "$*" in
 *'GET repos/owner/repo') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1,"full_name":"owner/repo"}' ;;
 *'GET user') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1}' ;;
 *'GET repos/owner/repo/issues/1') printf 'HTTP/1.1 200 OK\n' >&2; cat "$0.card" ;;
 *PATCH*) cat > "$0.input"; printf 'HTTP/1.1 %s Error\n' "$(cat "$0.status")" >&2; printf '{}' ;;
 *) printf 'HTTP/1.1 500 Unexpected\n' >&2; printf '{}' ;;
esac
