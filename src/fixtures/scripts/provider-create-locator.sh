#!/bin/sh
method=GET
for arg do
 if [ "$previous" = --method ]; then method="$arg"; fi
 previous="$arg"
 path="$arg"
done
printf '%s %s\n' "$method" "$path" >> "$0.calls"
if [ "$method:$path" = GET:repos/owner/repo/issues/1 ]; then
 printf 'HTTP/1.1 %s Status\n\n' "$(cat "$0.status")"
 cat "$0.readback"
 exit 0
fi
printf 'HTTP/1.1 200 OK\n\n'
case "$method:$path" in
 GET:user) printf '{"id":1}' ;;
 GET:repos/owner/repo) printf '{"id":1,"full_name":"owner/repo"}' ;;
 GET:*/issues\?*) printf '[]' ;;
 POST:*/issues) cat >/dev/null; cat "$0.card" ;;
 *) exit 1 ;;
esac
