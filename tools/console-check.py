"""Drive the guest through PS/2 and COM1 and capture its VGA scanout via QMP."""
import argparse
import json
import os
from pathlib import Path
import re
import select
import shlex
import subprocess
import time
import tty

TIMEOUT = 15
ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]")


def read_available(fd, deadline):
    remaining = deadline - time.monotonic()
    if remaining <= 0 or not select.select([fd], [], [], remaining)[0]:
        raise TimeoutError("timed out waiting for QEMU")
    data = os.read(fd, 8192)
    if not data:
        raise EOFError("QEMU disconnected")
    return data


class Guest:
    def __init__(self, command, folder):
        self.folder = folder
        self.tail = bytearray()
        self.serial_log = bytearray()
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self.port = None

    def connect(self):
        assert "QMP" in self.message(time.monotonic() + TIMEOUT)
        self.qmp("qmp_capabilities")
        serial = next(c["filename"] for c in self.qmp("query-chardev") if c["label"] == "serial0")
        assert serial.startswith("pty:"), serial
        self.port = os.open(serial.removeprefix("pty:"), os.O_RDWR | os.O_NOCTTY)
        tty.setraw(self.port)
        self.qmp("cont")

    def message(self, deadline):
        while True:
            while b"\n" not in self.tail:
                self.tail.extend(read_available(self.process.stdout.fileno(), deadline))
            line, _, rest = self.tail.partition(b"\n")
            self.tail = bytearray(rest)
            if not line.startswith(b"char device redirected to "):
                return json.loads(line)

    def qmp(self, name, **arguments):
        message = json.dumps({"execute": name, "arguments": arguments}).encode() + b"\n"
        self.process.stdin.write(message)
        self.process.stdin.flush()
        deadline = time.monotonic() + TIMEOUT
        while True:
            response = self.message(deadline)
            if "return" in response:
                return response["return"]
            if "error" in response:
                raise RuntimeError(f"QMP {name}: {response['error']}")

    def wait_for(self, needle, after=None):
        mark = len(self.serial_log) if after is None else after
        deadline = time.monotonic() + TIMEOUT
        try:
            while needle not in ANSI.sub(b"", self.serial_log[mark:]):
                self.serial_log.extend(read_available(self.port, deadline))
                if len(self.serial_log) >= 128 * 1024:
                    raise RuntimeError("console transcript exceeded the test limit")
        except (TimeoutError, EOFError) as error:
            raise RuntimeError(f"waiting for {needle!r}: {error}") from error

    def ps2(self, text):
        for char in text:
            key = {" ": "spc", "\n": "ret", "\t": "tab", "-": "minus"}.get(char, char)
            self.key(key)

    def key(self, key):
        message = self.qmp("human-monitor-command", **{"command-line": f"sendkey {key} 25"})
        if message:
            raise RuntimeError(message)
        time.sleep(0.045)

    def serial(self, text):
        os.write(self.port, text)

    def screen(self, name):
        path = self.folder / f"{name}.ppm"
        self.qmp("screendump", filename=str(path), format="ppm")
        data = path.read_bytes()
        assert data.startswith(b"P6\n") and len(data) > 640 * 480 * 3, path
        return data

    def close(self):
        if self.port is not None:
            (self.folder / "serial.log").write_bytes(self.serial_log)
            os.close(self.port)
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()


def graphical(guest, resolution):
    guest.wait_for(b"SLC VIDEO " + resolution.encode(), 0)
    home = guest.screen("console")
    for command, marker in [
        ("sy\t\n", b"SYSTEM OVERVIEW"),
        ("help\n", b"command reference"),
        ("echo guest-input\n", b"\r\nguest-input\r\n"),
    ]:
        start = len(guest.serial_log)
        guest.ps2(command)
        guest.wait_for(marker, start)
        if command.startswith("sy"):
            assert home != guest.screen("sysinfo")
            guest.key("esc")
        elif command == "help\n":
            start = len(guest.serial_log)
            guest.ps2("memory\n")
            guest.wait_for(b"[command reference: memory]", start)
            guest.wait_for(b"Summarize the firmware memory map", start)
            assert home != guest.screen("help")
            guest.key("esc")

    start = len(guest.serial_log)
    guest.key("up")
    guest.key("home")
    guest.key("end")
    guest.key("backspace")
    guest.ps2("t\n")
    guest.wait_for(b"\r\nguest-input\r\n", start)
    start = len(guest.serial_log)
    guest.ps2("metrics\n")
    guest.wait_for(b"First frame:", start)

    start = len(guest.serial_log)
    guest.ps2("theme ember\n")
    guest.wait_for(b"Theme set to ember.", start)
    decorated = guest.screen("ember")
    assert decorated != home

    start = len(guest.serial_log)
    guest.ps2("zoom 2\n")
    guest.wait_for(b"Text scale set to 2.", start)
    guest.screen("zoom")
    start = len(guest.serial_log)
    guest.ps2("clear\n")
    guest.wait_for(b"Activity cleared", start)
    assert decorated != guest.screen("cleared")


def recovery(guest):
    guest.wait_for(b"SLC READY serial", 0)
    for command, marker in [
        (b"sysinfo\r", b"SYSTEM OVERVIEW"),
        (b"\x1dhelp cpu\r", b"Read processor information"),
        (b"\x1dfoo\r", b"Unknown command: foo"),
        (b"echo recovery\r", b"\r\nrecovery\r\n"),
        (b"clear\r", b"Activity cleared"),
    ]:
        start = len(guest.serial_log)
        guest.serial(command)
        guest.wait_for(marker, start)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--iso", required=True, type=Path)
    parser.add_argument("--resolution")
    parser.add_argument("--qemu", nargs=argparse.REMAINDER, required=True)
    options = parser.parse_args()
    folder = options.output.resolve()
    folder.mkdir(parents=True, exist_ok=True)
    command = options.qemu + [
        "-machine", "pc-i440fx-7.2", "-accel", "tcg", "-cpu", "qemu64", "-m", "128M",
        "-boot", "d", "-cdrom", str(options.iso.resolve()), "-nic", "none", "-no-reboot",
        "-display", "none", "-monitor", "none", "-serial", "pty", "-qmp", "stdio", "-S",
    ]
    print(shlex.join(command), flush=True)
    guest = Guest(command, folder)
    try:
        guest.connect()
        guest.wait_for(b"SYLLASLOPOS: x86_64 kernel entry ok", 0)
        if options.resolution:
            graphical(guest, options.resolution)
        else:
            recovery(guest)
        print(f"PASS: {'PS/2 and VGA ' + options.resolution if options.resolution else 'COM1 recovery'}")
        return 0
    finally:
        guest.close()


if __name__ == "__main__":
    raise SystemExit(main())
