# Shared environment for fixtures that drive the real Agency over a cold
# session. Hashing a whole installed payload, and paging two 2 MiB outputs, take
# longer than the 5 s default request budget on a loaded release runner, so the
# fixture declares the 30 s cold-session readiness budget for every client it
# starts, including clients created inside control's preservation path. It is a
# ceiling, not a delay: no production default changes and nothing retries.
HEAVY_AGENCY_RPC_ENV = {"HCTL2_AGENCY_REQUEST_TIMEOUT_MS": "30000"}
