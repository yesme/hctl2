#!/bin/sh
# Completion writeback against a Gitea-shaped tea: one comment, then the closed state.
# `$0.card` holds issue 1 and `$0.comments` its thread. `$0.comment-status` or
# `$0.close-status` answers that write with the given HTTP status instead of doing it;
# `$0.comment-lost` records the comment but answers 503.
printf '%s\n' "$*" >> "$0.calls"
case "$*" in
 *'GET repos/owner/repo') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1,"full_name":"owner/repo"}' ;;
 *'GET user') printf 'HTTP/1.1 200 OK\n' >&2; printf '{"id":1}' ;;
 *'GET repos/owner/repo/issues/1') printf 'HTTP/1.1 200 OK\n' >&2; cat "$0.card" ;;
 *'GET repos/owner/repo/issues/1/comments')
   printf 'HTTP/1.1 200 OK\n' >&2
   if [ -f "$0.comments" ]; then cat "$0.comments"; else printf '[]'; fi ;;
 *'GET repos/owner/repo/issues/1/timeline')
   printf 'HTTP/1.1 200 OK\n' >&2; printf '[{"type":"close","user":{"username":"hctl2"}}]' ;;
 *'POST --data @- repos/owner/repo/issues/1/comments')
   body="$(cat)"
   if [ -f "$0.comment-status" ]; then
     printf 'HTTP/1.1 %s Error\n' "$(cat "$0.comment-status")" >&2; printf '{}'; exit 1
   fi
   # The body is exactly {"body":<JSON string>}; keep the string literal as it came.
   text="${body#\{\"body\":}"
   text="${text%\}}"
   printf '[{"id":100,"body":%s}]' "$text" > "$0.comments"
   printf x >> "$0.posts"
   if [ -f "$0.comment-lost" ]; then printf 'HTTP/1.1 503 Error\n' >&2; printf '{}'; exit 1; fi
   printf 'HTTP/1.1 201 Created\n' >&2; printf '{"id":100,"body":%s}' "$text" ;;
 *'PATCH --data @- repos/owner/repo/issues/1')
   cat > "$0.patch"
   if [ -f "$0.close-status" ]; then
     printf 'HTTP/1.1 %s Error\n' "$(cat "$0.close-status")" >&2; printf '{}'; exit 1
   fi
   sed 's/"state":"open"/"state":"closed"/' "$0.card" > "$0.next" && mv "$0.next" "$0.card"
   printf x >> "$0.closes"
   printf 'HTTP/1.1 201 Created\n' >&2; cat "$0.card" ;;
 *) printf 'HTTP/1.1 500 Unexpected\n' >&2; printf '{}' ;;
esac
