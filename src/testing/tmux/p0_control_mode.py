#!/usr/bin/env python3
"""Run the disposable tmux P0 probe against an installed distribution binary."""

from __future__ import annotations

import argparse
import json
import os
import platform
import queue
import re
import shlex
import shutil
import stat
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path


CONTROL_OUTPUT = re.compile(rb"^%output (?P<pane>%[0-9]+) (?P<data>.*)\n$")
OCTAL_ESCAPE = re.compile(rb"\\([0-7]{3})")


def run_tmux(tmux: Path, socket: Path, *arguments: str, check: bool = True) -> str:
    completed = subprocess.run(
        [str(tmux), "-S", str(socket), *arguments],
        check=check,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return completed.stdout.strip()


def wait_until(predicate, description: str, timeout_seconds: float = 8.0) -> None:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(0.05)
    raise TimeoutError(f"timed out waiting for {description}")


def decode_control_output(data: bytes) -> bytes:
    decoded = OCTAL_ESCAPE.sub(lambda match: bytes([int(match.group(1), 8)]), data)
    return decoded.replace(b"\\\\", b"\\")


class ControlClient:
    def __init__(self, tmux: Path, socket: Path, session: str) -> None:
        self.process = subprocess.Popen(
            [str(tmux), "-S", str(socket), "-C", "attach-session", "-t", session],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            bufsize=0,
        )
        if self.process.stdin is None or self.process.stdout is None:
            raise RuntimeError("tmux control client did not expose pipes")
        self._output = bytearray()
        self._output_lock = threading.Lock()
        self._fast_queue: queue.Queue[bytes | None] = queue.Queue(maxsize=4096)
        self._slow_queue: queue.Queue[bytes] = queue.Queue(maxsize=2)
        self._results: queue.Queue[tuple[bool, list[bytes]]] = queue.Queue()
        self._command_lock = threading.Lock()
        self.slow_drops = 0
        self.protocol_lines = 0
        self.fast_output = bytearray()
        self._reader = threading.Thread(target=self._read, daemon=True)
        self._fast_observer = threading.Thread(target=self._consume_fast, daemon=True)
        self._reader.start()
        self._fast_observer.start()
        succeeded, output = self._results.get(timeout=5)
        if not succeeded:
            raise RuntimeError(f"initial control attach failed: {output!r}")

    def _read(self) -> None:
        assert self.process.stdout is not None
        command_output: list[bytes] | None = None
        for line in iter(self.process.stdout.readline, b""):
            self.protocol_lines += 1
            if line.startswith(b"%begin "):
                if command_output is not None:
                    raise RuntimeError("nested tmux control response")
                command_output = []
                continue
            if line.startswith((b"%end ", b"%error ")):
                if command_output is None:
                    raise RuntimeError(f"tmux control response ended without beginning: {line!r}")
                self._results.put((line.startswith(b"%end "), command_output))
                command_output = None
                continue
            match = CONTROL_OUTPUT.match(line)
            if match is not None:
                payload = decode_control_output(match.group("data"))
                with self._output_lock:
                    self._output.extend(payload)
                self._fast_queue.put(payload)
                try:
                    self._slow_queue.put_nowait(payload)
                except queue.Full:
                    self.slow_drops += 1
                continue
            if command_output is not None and not line.startswith(b"%"):
                command_output.append(line.rstrip(b"\n"))
        self._fast_queue.put(None)

    def _consume_fast(self) -> None:
        while True:
            payload = self._fast_queue.get()
            if payload is None:
                return
            self.fast_output.extend(payload)

    def _write_command(self, command: str) -> None:
        if self.process.stdin is None:
            raise RuntimeError("tmux control client stdin is closed")
        self.process.stdin.write(command.encode() + b"\n")
        self.process.stdin.flush()

    def command(self, command: str) -> list[str]:
        with self._command_lock:
            self._write_command(command)
            try:
                succeeded, output = self._results.get(timeout=8)
            except queue.Empty as error:
                raise TimeoutError(f"tmux control command timed out: {command}") from error
        decoded = [line.decode(errors="replace") for line in output]
        if not succeeded:
            raise RuntimeError(f"tmux control command failed: {command}: {decoded!r}")
        return decoded

    def send_fixture_command(self, pane_id: str, command: str) -> None:
        self.command(f"send-keys -t {pane_id} -l -- {shlex.quote(command)}")
        self.command(f"send-keys -t {pane_id} Enter")

    def contains(self, marker: bytes) -> bool:
        with self._output_lock:
            return marker in self._output

    def close(self) -> None:
        if self.process.poll() is None:
            self._write_command("detach-client")
        self.process.wait(timeout=5)
        self._reader.join(timeout=5)
        self._fast_observer.join(timeout=5)
        if self.process.returncode != 0:
            stderr = b"" if self.process.stderr is None else self.process.stderr.read()
            raise RuntimeError(f"tmux control client failed: {stderr.decode(errors='replace')}")


def parse_identity(value: str) -> dict[str, str]:
    fields = value.split("|")
    if len(fields) != 7:
        raise AssertionError(f"unexpected pane identity: {value!r}")
    return dict(zip(("session", "window", "pane", "pid", "width", "height", "dead"), fields))


def process_exists(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tmux", required=True, type=Path)
    parser.add_argument("--keep-failed-root", action="store_true")
    arguments = parser.parse_args()

    tmux = arguments.tmux.resolve(strict=True)
    fixture = Path(__file__).with_name("p0_fixture.py").resolve(strict=True)
    probe_root = Path(tempfile.mkdtemp(prefix="hctl2-tmux-p0."))
    os.chmod(probe_root, 0o700)
    socket = probe_root / "owner.sock"
    query_result = probe_root / "queries.json"
    session = "hctl2-p0"
    control: ControlClient | None = None
    started_at = time.monotonic()
    succeeded = False

    try:
        version = subprocess.run(
            [str(tmux), "-V"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        ).stdout.strip()
        fixture_command = shlex.join([sys.executable, str(fixture), str(query_result)])
        run_tmux(
            tmux,
            socket,
            "-f",
            "/dev/null",
            "new-session",
            "-d",
            "-s",
            session,
            "-n",
            "runtime",
            fixture_command,
        )
        run_tmux(tmux, socket, "set-option", "-t", session, "remain-on-exit", "on")
        run_tmux(tmux, socket, "set-option", "-t", session, "status", "off")

        socket_mode = stat.S_IMODE(socket.stat().st_mode)
        root_mode = stat.S_IMODE(probe_root.stat().st_mode)
        if root_mode != 0o700 or socket_mode & 0o077:
            raise AssertionError(
                f"owner socket is exposed: root={root_mode:o}, socket={socket_mode:o}"
            )

        identity_format = (
            "#{session_id}|#{window_id}|#{pane_id}|#{pane_pid}|"
            "#{pane_width}|#{pane_height}|#{pane_dead}"
        )
        wait_until(query_result.exists, "detached terminal query responses")
        queries = json.loads(query_result.read_text(encoding="utf-8"))
        decoded_queries = {name: bytes.fromhex(value) for name, value in queries.items()}
        if decoded_queries["dsr"] != b"\x1b[0n":
            raise AssertionError(f"unexpected DSR response: {decoded_queries['dsr']!r}")
        if not re.fullmatch(rb"\x1b\[\?[0-9;]+c", decoded_queries["da"]):
            raise AssertionError(f"unexpected DA response: {decoded_queries['da']!r}")
        if not re.fullmatch(rb"\x1b\[\?2004;[0-4]\$y", decoded_queries["decrqm"]):
            raise AssertionError(f"unexpected DECRQM response: {decoded_queries['decrqm']!r}")

        control = ControlClient(tmux, socket, session)
        identity_before = parse_identity(
            "\n".join(control.command(f"list-panes -t {session} -F {shlex.quote(identity_format)}"))
        )
        wait_until(
            lambda: len(control.command("list-clients -F '#{client_flags}'")) > 0,
            "writable control client",
        )
        client_flags = "\n".join(control.command("list-clients -F '#{client_flags}'"))
        if len(client_flags.splitlines()) != 1:
            raise AssertionError(f"expected exactly one owner control client: {client_flags!r}")
        if "read-only" in client_flags:
            raise AssertionError(f"owner control client is read-only: {client_flags}")

        token = "input-7f15d6"
        control.send_fixture_command(identity_before["pane"], f"INPUT:{token}")
        wait_until(lambda: control is not None and control.contains(token.encode()), "control output")

        control.command(f"resize-window -t {identity_before['window']} -x 111 -y 37")
        control.send_fixture_command(identity_before["pane"], "SIZE")
        wait_until(lambda: control is not None and control.contains(b"HCTL2-P0 SIZE 37x111"), "PTY resize")

        control.send_fixture_command(identity_before["pane"], "BURST:12000")
        wait_until(lambda: control is not None and control.contains(b"HCTL2-P0 BURST-END"), "burst output", 20)
        if control.slow_drops == 0:
            raise AssertionError("slow observer queue did not reach its bounded limit")
        wait_until(lambda: b"HCTL2-P0 BURST-END" in control.fast_output, "fast observer fanout", 5)
        if control.fast_output.count(b"HCTL2-P0 BURST ") != 12000:
            raise AssertionError("fast observer did not receive every burst record")
        if b"HCTL2-P0 BURST 0 " not in control.fast_output or b"HCTL2-P0 BURST 11999 " not in control.fast_output:
            raise AssertionError("fast observer burst boundaries are incomplete")
        control.send_fixture_command(identity_before["pane"], "INPUT:after-burst")
        wait_until(lambda: control is not None and control.contains(b"INPUT-SEEN after-burst"), "post-burst input")
        if control.command("display-message -p PONG") != ["PONG"]:
            raise AssertionError("tmux server stopped answering during slow-observer pressure")

        first_control_bytes = len(control.fast_output)
        first_protocol_lines = control.protocol_lines
        slow_drops = control.slow_drops
        control.close()
        control = None

        control = ControlClient(tmux, socket, session)
        identity_after_reconnect = parse_identity(
            "\n".join(control.command(f"list-panes -t {session} -F {shlex.quote(identity_format)}"))
        )
        for field in ("session", "window", "pane", "pid"):
            if identity_after_reconnect[field] != identity_before[field]:
                raise AssertionError(f"{field} changed after control reconnect")

        control.send_fixture_command(identity_before["pane"], "EXIT17")

        def pane_exited() -> bool:
            current = parse_identity(
                "\n".join(
                    control.command(f"list-panes -t {session} -F {shlex.quote(identity_format)}")
                )
            )
            return current["dead"] == "1"

        wait_until(pane_exited, "pane exit")
        exit_identity = parse_identity(
            "\n".join(control.command(f"list-panes -t {session} -F {shlex.quote(identity_format)}"))
        )
        for field in ("session", "window", "pane"):
            if exit_identity[field] != identity_before[field]:
                raise AssertionError(f"{field} changed when the pane exited")
        exit_status = "\n".join(
            control.command(
                f"display-message -p -t {identity_before['pane']} '#{{pane_dead_status}}'"
            )
        )
        if exit_status != "17":
            raise AssertionError(f"unexpected pane exit status: {exit_status!r}")
        wait_until(lambda: not process_exists(int(identity_before["pid"])), "fixture process cleanup")

        control.close()
        control = None
        summary = {
            "binary": str(tmux),
            "client_flags": client_flags,
            "control_bytes": first_control_bytes,
            "control_protocol_lines": first_protocol_lines,
            "elapsed_seconds": round(time.monotonic() - started_at, 3),
            "exit_status": int(exit_status),
            "host": f"{platform.system()} {platform.machine()}",
            "ids": {
                "session": identity_before["session"],
                "window": identity_before["window"],
                "pane": identity_before["pane"],
            },
            "queries": {name: repr(value) for name, value in decoded_queries.items()},
            "reconnect_ids_stable": True,
            "root_mode": f"{root_mode:04o}",
            "slow_observer_dropped_chunks": slow_drops,
            "socket_mode": f"{socket_mode:04o}",
            "version": version,
            "window_size": "37x111",
        }
        print(json.dumps(summary, indent=2, sort_keys=True))
        succeeded = True
        return 0
    finally:
        if control is not None:
            try:
                control.close()
            except Exception:
                pass
        if socket.exists():
            run_tmux(tmux, socket, "kill-server", check=False)
        if succeeded or not arguments.keep_failed_root:
            shutil.rmtree(probe_root, ignore_errors=True)
        else:
            print(f"preserved failed probe root: {probe_root}", file=sys.stderr)


if __name__ == "__main__":
    raise SystemExit(main())
