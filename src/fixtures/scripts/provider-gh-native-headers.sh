#!/bin/sh
printf 'HTTP/2.0 200 OK\n\n'
cat "$0.json"
