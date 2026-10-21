#!/bin/sh
# GitHub Issues fixture for the Task acceptance tests. The product canonicalizes
# `HCTL2_GH` before executing it, so `$0` is this built file, not the test's
# writable link; the test passes its state base path in `HCTL2_GH_STATE`.
# Marker files beside that base steer the writes: `.hide` loses a card creation's
# confirmation; `.comment-deny` / `.close-deny` refuse a comment or closing with 403;
# `.comment-lost` records a comment but loses its response; `.comment-hide` keeps
# comments out of reads. Every other PATCH is refused with 403.
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
input=
if [ "$method" != GET ]; then input="$(cat)"; fi
status='200 OK'
case "$method:$path" in
 POST:*/issues) if [ -f "$base.hide" ]; then status='503 Lost Confirmation'; fi ;;
 POST:*/comments)
  if [ -f "$base.comment-deny" ]; then status='403 Forbidden';
  elif [ -f "$base.comment-lost" ]; then status='503 Lost Confirmation'; fi ;;
 PATCH:*)
  case "$input" in
   *'"state"'*) if [ -f "$base.close-deny" ]; then status='403 Forbidden'; fi ;;
   *) status='403 Forbidden' ;;
  esac ;;
esac
if [ "$header" = yes ]; then printf 'HTTP/1.1 %s\n\n' "$status"; fi
jqbin="$(cat "$base.jq")"
case "$method:$path" in
 GET:user) printf '{"id":5}' ;;
 GET:repos/owner/fixture) printf '{"id":77,"full_name":"owner/fixture","clone_url":"https://github.com/owner/fixture.git","has_issues":true,"permissions":{"push":true,"admin":true}}' ;;
 GET:*page=2*) printf '[]' ;;
 GET:*/issues\?*) if [ -f "$base.created" ] && [ ! -f "$base.hide" ] && [ ! -f "$base.list-lag" ]; then "$jqbin" -s '.' "$base.created"; else printf '[]'; fi ;;
 GET:*/issues/1/comments*) if [ -f "$base.comments" ] && [ ! -f "$base.comment-hide" ]; then cat "$base.comments"; else printf '[]'; fi ;;
 GET:*/issues/1/parent) printf 'null' ;;
 GET:*/issues/1) cat "$base.created" ;;
 GET:*) printf '[]' ;;
 POST:*/issues) printf '%s' "$input" | "$jqbin" '. + {id:1,node_id:"node1",number:1,state:"open",updated_at:"t0",labels:[],comments:0}' > "$base.created"; printf x >> "$base.posts"; cat "$base.created" ;;
 POST:*/issues/1/comments)
  if [ -f "$base.comment-deny" ]; then printf '{"message":"Forbidden"}'; exit 1; fi
  [ -f "$base.comments" ] || printf '[]' > "$base.comments"
  "$jqbin" --argjson c "$input" '. + [{id:(length + 100),body:$c.body,user:{login:"fixture-bot"}}]' "$base.comments" > "$base.comments.next" && mv "$base.comments.next" "$base.comments"
  "$jqbin" '.comments += 1' "$base.created" > "$base.created.next" && mv "$base.created.next" "$base.created"
  printf x >> "$base.comment-posts"
  if [ -f "$base.comment-lost" ]; then exit 1; fi
  "$jqbin" '.[-1]' "$base.comments" ;;
 PATCH:*/issues/1)
  case "$input" in
   *'"state"'*)
    if [ -f "$base.close-deny" ]; then printf '{"message":"Forbidden"}'; exit 1; fi
    "$jqbin" --argjson p "$input" '. + {state:$p.state,closed_by:{login:"fixture-bot"},updated_at:"t1"}' "$base.created" > "$base.created.next" && mv "$base.created.next" "$base.created"
    printf x >> "$base.closes"
    cat "$base.created" ;;
   *) printf '{"message":"Forbidden"}'; exit 1 ;;
  esac ;;
 PATCH:*) printf '{"message":"Forbidden"}'; exit 1 ;;
 *) exit 1 ;;
esac
