# Dependency packaging

This directory is the first implementation of HCTL2's external-dependency supply chain. The end
user receives one offline installation archive, not a bootstrap recipe. Network access, checksum
verification, archive extraction, and compilation are build-machine responsibilities.

## Build flow

`versions.sh` is the lock file for the four selected dependencies. `bootstrap.sh` downloads and
verifies official release artifacts, uses the published Linux binaries for Tuwunel, Vikunja, and
Dagu, and builds tmux because upstream does not publish a Linux binary. Missing tmux build headers
and bison are downloaded at versions pinned in the lock file and extracted into the build cache;
nothing is installed with sudo. The resulting package records both those deb versions and the
compiler/binutils/glibc/make/pkg-config versions used by the builder.

`build-package.sh` then assembles one versioned payload containing:

- all four executables and tmux's non-system shared libraries;
- the tracked `hctl2-services` lifecycle entry and its start/stop/status/smoke implementation;
- dependency versions, commits, asset digests, licenses, and copyright material;
- pinned source archives for all four dependencies, including corresponding source for
  GPL-licensed Dagu and AGPL-licensed Vikunja;
- an offline installer and a full payload checksum manifest.

Build on Ubuntu x86_64:

```bash
src/packaging/dependencies/build-package.sh
```

Run the full build, offline install, idempotent reinstall, startup, smoke, and shutdown path in an
isolated temporary prefix:

```bash
src/packaging/dependencies/test-package.sh
```

The ignored output is `src/dist/hctl2-0.0.0-linux-x86_64.tar.gz` plus its SHA-256 file. Release CI
must build on the oldest supported glibc baseline; a local build proves the package pipeline but
does not by itself establish the final Linux compatibility floor.

Build inputs default to `${XDG_CACHE_HOME:-$HOME/.cache}/hctl2/dependencies`. Set an absolute
`HCTL2_BUILD_CACHE` to reuse or isolate another verified build cache.

The release matrix produces one package per supported platform, so an end user still downloads
one artifact. This slice implements and verifies Linux x86_64. The macOS arm64 builder will use
the official Dagu and Vikunja binaries and compile pinned tmux and Tuwunel sources; that native
Tuwunel build must pass the same installed-payload test on macOS before the target can be offered.

## User flow

The installer never downloads or compiles anything:

```bash
tar -xzf hctl2-0.0.0-linux-x86_64.tar.gz
cd hctl2-0.0.0-linux-x86_64
./install.sh
~/.local/bin/hctl2-services start
~/.local/bin/hctl2-services smoke
```

Installation defaults to `$HOME/.local`; `--prefix` selects another absolute prefix. Persistent
data, secrets, logs, sockets, and PIDs stay outside the versioned installation under
`${XDG_STATE_HOME:-$HOME/.local/state}/hctl2`. An absolute `HCTL2_STATE_ROOT` provides isolated
state for development and tests. Installing a package does not start processes automatically.

## Runtime policy

| Component | Version | Local endpoint |
| --- | --- | --- |
| Tuwunel | 1.9.0 | `http://127.0.0.1:6167` |
| Vikunja | 2.5.0 | `http://127.0.0.1:3456` |
| Dagu | 2.15.1 | `http://127.0.0.1:18080` |
| tmux | 3.7c | owner-only socket under the HCTL2 state root |

All listeners bind to loopback. This local Tuwunel configuration disables federation and room
encryption, as required for HCTL Rooms. Dagu authentication is disabled only on its loopback
listener. Vikunja creates a random local secret at first start.

These scripts prove packaging and the first startup seam. When the public Rust CLI exists,
`hctl2 start` will call the same lifecycle layer; users will not gain a second operational API.
