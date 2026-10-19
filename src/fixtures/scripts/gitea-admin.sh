#!/bin/sh
# Native Gitea admin CLI fixture: `admin user list` reads the accounts recorded so
# far, `admin user create` prints a password and then records one. State lives
# beside the invoked path (`$0.*`), which the test points at a writable link.
sub="$7"
if [ "$sub" = list ]; then
    printf 'ID\tUsername\tEmail\tIsActive\n'
    cat "$0.users" 2>/dev/null
    exit 0
fi
if [ "$sub" = create ]; then
    name="$9"
    printf "generated random password is '%s'\n" 'fixture-initial-password'
    if [ -f "$0.fail" ]; then
        exit 1
    fi
    printf x >>"$0.creates"
    printf '2   %s   %s@localhost   active   public\n' "$name" "$name" >>"$0.users"
    exit 0
fi
exit 1
