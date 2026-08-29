#!/usr/bin/env python3
"""Disposable terminal fixture for the tmux P0 control-mode probe."""

from __future__ import annotations

import fcntl
import json
import os
import select
import struct
import sys
import termios
import time
import tty
from pathlib import Path


def write_terminal(payload: bytes) -> None:
    os.write(sys.stdout.fileno(), payload)


def read_reply(suffix: bytes, timeout_seconds: float = 3.0) -> bytes:
    deadline = time.monotonic() + timeout_seconds
    reply = bytearray()
    while time.monotonic() < deadline:
        ready, _, _ = select.select([sys.stdin.fileno()], [], [], 0.1)
        if not ready:
            continue
        chunk = os.read(sys.stdin.fileno(), 256)
        if not chunk:
            break
        reply.extend(chunk)
        if reply.endswith(suffix):
            return bytes(reply)
    raise TimeoutError(f"terminal query ending in {suffix!r} timed out: {bytes(reply)!r}")


def terminal_size() -> tuple[int, int]:
    packed = fcntl.ioctl(sys.stdin.fileno(), termios.TIOCGWINSZ, b"\0" * 8)
    rows, columns, _, _ = struct.unpack("HHHH", packed)
    return rows, columns


def read_command(buffer: bytearray) -> bytes:
    while True:
        for separator in (b"\r", b"\n"):
            if separator in buffer:
                command, remainder = buffer.split(separator, 1)
                buffer[:] = remainder.lstrip(b"\r\n")
                return command
        chunk = os.read(sys.stdin.fileno(), 4096)
        if not chunk:
            return b""
        buffer.extend(chunk)


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: p0_fixture.py QUERY-RESULT.json")

    result_path = Path(sys.argv[1])
    input_fd = sys.stdin.fileno()
    original_mode = termios.tcgetattr(input_fd)
    tty.setraw(input_fd)
    try:
        queries = {}
        for name, request, suffix in (
            ("dsr", b"\x1b[5n", b"n"),
            ("da", b"\x1b[c", b"c"),
            ("decrqm", b"\x1b[?2004$p", b"$y"),
        ):
            write_terminal(request)
            queries[name] = read_reply(suffix).hex()

        result_path.write_text(json.dumps(queries, sort_keys=True) + "\n", encoding="utf-8")
        write_terminal(b"HCTL2-P0 READY\r\n")

        command_buffer = bytearray()
        while True:
            command = read_command(command_buffer)
            if not command:
                return 0
            if command.startswith(b"INPUT:"):
                write_terminal(b"HCTL2-P0 INPUT-SEEN " + command[6:] + b"\r\n")
            elif command == b"SIZE":
                rows, columns = terminal_size()
                write_terminal(f"HCTL2-P0 SIZE {rows}x{columns}\r\n".encode())
            elif command.startswith(b"BURST:"):
                line_count = int(command[6:])
                payload = b"x" * 96
                for index in range(line_count):
                    write_terminal(b"HCTL2-P0 BURST " + str(index).encode() + b" " + payload + b"\r\n")
                write_terminal(b"HCTL2-P0 BURST-END\r\n")
            elif command == b"EXIT17":
                write_terminal(b"HCTL2-P0 EXITING 17\r\n")
                return 17
            else:
                write_terminal(b"HCTL2-P0 UNKNOWN " + command + b"\r\n")
    finally:
        termios.tcsetattr(input_fd, termios.TCSANOW, original_mode)


if __name__ == "__main__":
    raise SystemExit(main())
