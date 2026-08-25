# HCTL2 offline Linux package

This archive is an offline-installable HCTL2 development package for Linux x86_64. It contains
the pinned Tuwunel, Vikunja, Dagu, and tmux executables, tmux's non-system shared libraries,
license material, pinned upstream source (including corresponding source for the GPL/AGPL
components), and the tracked lifecycle scripts used to run the four dependencies.

Install for the current user:

```bash
./install.sh
~/.local/bin/hctl2-services start
~/.local/bin/hctl2-services smoke
```

The default persistent state directory is `${XDG_STATE_HOME:-$HOME/.local/state}/hctl2`. Set an
absolute `HCTL2_STATE_ROOT` to isolate an installation. All service listeners bind to loopback.

This is the first packaging slice. Future HCTL2 binaries and Workbench assets will enter the same
versioned payload; the public `hctl2 start` command will own this lifecycle once the CLI exists.
