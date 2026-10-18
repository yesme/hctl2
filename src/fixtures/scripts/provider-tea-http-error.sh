#!/bin/sh
case "$*" in
 *"GET user") printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1}' ;;
 *"GET repos/owner/repo") printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1,"full_name":"owner/repo"}' ;;
 *POST*) cat > "$0.input"; printf x >> "$0.posts"; cp "$0.result" "$0.created"; printf 'HTTP/1.1 503 Error\n' >&2; printf '{}' ;;
 *page=2*) printf 'HTTP/1.1 200 OK\n' >&2; printf '[]' ;;
 *) printf 'HTTP/1.1 200 OK\n' >&2; if [ -f "$0.created" ]; then cat "$0.created"; else printf '[]'; fi ;;
esac
