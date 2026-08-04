#!/usr/bin/env python3
"""Exercise the generated init script in a real Fish 4.1+ PTY."""

from __future__ import annotations

import os
import fcntl
import pty
import select
import signal
import struct
import sys
import tempfile
import termios
import time
from pathlib import Path


ARROW = "".encode()
TIME = "".encode()
CAP = "".encode()
DURATION = "󰔚".encode()
MARKER = b"CLAVIS_FISH_PTY_MARKER"
ERROR_ARROW = b"\x1b[38;2;255;218;214m" + ARROW
SUCCESS_ARROW = b"\x1b[38;2;192;199;213m" + ARROW


def read_until(fd: int, marker: bytes, timeout: float = 8.0) -> bytes:
    data = bytearray()
    deadline = time.monotonic() + timeout
    while marker not in data and time.monotonic() < deadline:
        ready, _, _ = select.select([fd], [], [], 0.1)
        if not ready:
            continue
        try:
            chunk = os.read(fd, 65536)
        except OSError:
            break
        if not chunk:
            break
        data.extend(chunk)
    if marker not in data:
        raise AssertionError(f"timed out waiting for {marker!r}: {bytes(data)!r}")
    return bytes(data)


def send(fd: int, command: str) -> None:
    os.write(fd, command.encode())


def main() -> int:
    binary = Path(sys.argv[1]).resolve()
    with tempfile.TemporaryDirectory(prefix="clavis-fish-pty-") as home:
        environment = os.environ.copy()
        environment.update(
            HOME=home,
            XDG_CONFIG_HOME=f"{home}/.config",
            TERM="dumb",
            COLUMNS="80",
            PATH=f"{binary.parent}:{environment.get('PATH', '')}",
        )
        pid, fd = pty.fork()
        if pid == 0:
            os.chdir(home)
            fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
            os.execvpe("fish", ["fish", "--no-config", "--private", "--interactive"], environment)

        fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))

        try:
            initial = read_until(fd, b"> ")
            send(fd, "clavis-fish-theme init fish | source\n")
            initialized = read_until(fd, ARROW)
            if CAP not in initialized:
                raise AssertionError(f"capsule prompt was not installed: {initialized!r}")
            if DURATION in initialized:
                raise AssertionError(f"initial prompt displayed duration: {initialized!r}")

            # A command submitted after transient mode is enabled should leave
            # a transient arrow/time record before its output and then return a
            # normal two-line prompt.
            send(fd, "printf 'CLAVIS_FISH_PTY_MARKER\\n'\n")
            history = read_until(fd, MARKER) + read_until(fd, ARROW)
            if TIME not in history:
                raise AssertionError(f"transient time was not rendered: {history!r}")
            if MARKER not in history:
                raise AssertionError("submitted command output disappeared")

            send(fd, "false\n")
            failed = read_until(fd, ERROR_ARROW)
            if ERROR_ARROW not in failed:
                raise AssertionError(f"failed arrow did not use error foreground: {failed!r}")

            send(fd, "true\n")
            succeeded = read_until(fd, SUCCESS_ARROW)
            if SUCCESS_ARROW not in succeeded:
                raise AssertionError(f"successful arrow did not use arrow color: {succeeded!r}")
            if "󰔚".encode() not in succeeded:
                raise AssertionError(f"duration was not shown after a command: {succeeded!r}")

            send(fd, "clavis-fish-theme init fish | source\n")
            repeated = read_until(fd, ARROW)
            if CAP not in repeated:
                raise AssertionError("repeated source removed the prompt")

            send(fd, "functions --handlers fish_postexec\n")
            handlers = read_until(fd, b"__clavis_fish_theme_mark_command")
            handlers += read_until(fd, ARROW)
            if handlers.count(b"__clavis_fish_theme_mark_command") != 1:
                raise AssertionError(f"duplicate event handlers: {handlers!r}")

            send(fd, "exit\n")
            os.waitpid(pid, 0)
        finally:
            try:
                os.close(fd)
            except OSError:
                pass
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass

    print("Fish PTY integration passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
