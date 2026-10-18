#!/bin/sh
# A user hook that records that it ran: the test proves credentials and tracing
# never reach user hooks by the absence of the marker named in the environment.
# Git chooses the hook's path and arguments, so the per-test marker path is
# passed in the environment rather than written into the body: a script the
# test just opened for writing can answer execve with ETXTBSY while another
# test thread's forked child still holds the inherited write descriptor.
printf hook >"$HCTL2_REPO_HOOK_MARKER"
exit 1
