#!/bin/sh
# GitHub Issues fixture for the Task acceptance tests. The product canonicalizes
# `HCTL2_GH` before executing it, so `$0` is this built file, not the test's
# writable link; the test passes its state base path in `HCTL2_GH_STATE`.
base="${HCTL2_GH_STATE:?HCTL2_GH_STATE must be set to the test state directory}"
method=GET
header=no
for arg do
  if [ "$arg" = --include ]; then header=yes; fi
  if [ "$previous" = --method ]; then method="$arg"; fi
  previous="$arg"
  path="$arg"
done
case "$path" in
 *'type=issues'*) printf 'HTTP/1.1 422 Validation Failed\n\n{"message":"Validation Failed","errors":[{"field":"type","code":"invalid","value":"issues"}]}'; exit 1 ;;
esac
if [ "$header" = yes ]; then
  if [ "$method" = POST ] && [ -f "$base.hide" ]; then printf 'HTTP/1.1 503 Lost Confirmation\n\n';
  elif [ "$method" = PATCH ]; then printf 'HTTP/1.1 403 Forbidden\n\n';
  else printf 'HTTP/1.1 200 OK\n\n'; fi
fi
case "$method:$path" in
 GET:user) printf '{"id":5}' ;;
 GET:repos/owner/fixture) printf '{"id":77,"full_name":"owner/fixture","clone_url":"https://github.com/owner/fixture.git","has_issues":true,"permissions":{"push":true,"admin":true}}' ;;
 GET:*page=2*) printf '[]' ;;
 GET:*/issues\?*) if [ -f "$base.created" ] && [ ! -f "$base.hide" ] && [ ! -f "$base.list-lag" ]; then jqbin="$(cat "$base.jq")"; "$jqbin" -s '.' "$base.created"; else printf '[]'; fi ;;
 GET:*/issues/1/parent) if [ "$header" = yes ]; then :; fi; printf 'null' ;;
 GET:*/issues/1) cat "$base.created" ;;
 GET:*) printf '[]' ;;
 POST:*/issues) jqbin="$(cat "$base.jq")"; "$jqbin" '. + {id:1,node_id:"node1",number:1,state:"open",updated_at:"t0",labels:[],comments:0}' > "$base.created"; printf x >> "$base.posts"; cat "$base.created" ;;
 PATCH:*) cat >/dev/null; printf '{"message":"Forbidden"}'; exit 1 ;;
 *) exit 1 ;;
esac
