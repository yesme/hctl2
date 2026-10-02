# Build-only prebuilt tool; provenance and decision: docs/research/build-tools/zstd.md.
#
# Upstream publishes source and Windows binaries only, so the CLI is built from
# the pinned source archive on the build host (see :zstd-bin in BUCK). The
# compiler and make never reach the user's machine, and the resulting binary
# never ships: it only writes the release archives.
ZSTD_VERSION = "1.5.7"
ZSTD_SHA256 = "eb33e51f49a15e023950cd7825ca74a4a2b43db8354825ac24fc1b7ee09e6fa3"
ZSTD_URL = "https://github.com/facebook/zstd/releases/download/v{0}/zstd-{0}.tar.gz".format(ZSTD_VERSION)

def declare_zstd():
    native.http_archive(
        name = "zstd-src",
        urls = [ZSTD_URL],
        sha256 = ZSTD_SHA256,
        strip_prefix = "zstd-" + ZSTD_VERSION,
        type = "tar.gz",
    )
