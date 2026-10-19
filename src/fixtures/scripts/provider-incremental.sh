#!/bin/sh
printf '%s\n' "$*" >> "$0.calls"
printf 'HTTP/1.1 200 OK\n' >&2
case "$*" in
 *'GET repos/owner/repo') printf '{"id":1,"full_name":"owner/repo","internal_tracker":{"enable_issue_dependencies":false}}' ;;
 *'GET user') printf '{"id":1}' ;;
 *'issues?'*'page=1') cat "$0.issues1" ;;
 *'issues?'*'page=2') cat "$0.issues2" ;;
 *) printf '[]' ;;
esac
