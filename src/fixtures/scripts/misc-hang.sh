#!/bin/sh
# A command that never returns, so a caller's deadline is observable. The sleep
# marker is the only per-test value and it is an argument, so this body stays
# static: tests link this built file instead of writing a script they execute.
# A file the test just opened for writing can answer execve with ETXTBSY while
# another test thread's forked child still holds the inherited write descriptor.
exec sleep "$1"
