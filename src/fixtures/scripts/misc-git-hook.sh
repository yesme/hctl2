#!/bin/sh
# A user hook that records that it ran: the test proves credentials and tracing
# never reach user hooks by the absence of `$0.ran`. The marker sits beside the
# invoked hook, so the proof does not depend on any environment variable
# reaching the hook: a future sanitizer that clears the environment (the
# direction isolation is going) must not turn a hook that ran into a pass.
printf hook >"$0.ran"
exit 1
