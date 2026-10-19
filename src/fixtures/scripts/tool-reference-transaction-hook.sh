#!/bin/sh
if [ "$1" = prepared ]; then
    while read -r old new ref; do
        if [ "$ref" = refs/heads/main ]; then exit 1; fi
    done
fi
exit 0
