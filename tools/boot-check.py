"""Boot the ISO in QEMU and require a complete kernel serial line."""

import argparse
import os
from pathlib import Path
import selectors
import shlex
import subprocess
import time

MARKER = b"SYLLASLOPOS: x86_64 kernel entry ok"
# Printed after the IDT is loaded and the low guard pages are unmapped.
ARMED = b"SYLLASLOPOS: exceptions armed, "
TIMEOUT = 20
RESET_WINDOW = 0.2


def boot(command: list[str], log_path: Path) -> bool:
    print(shlex.join(command), flush=True)
    with log_path.open("wb") as log, subprocess.Popen(
        command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
    ) as guest, selectors.DefaultSelector() as events:
        try:
            assert guest.stdout is not None
            events.register(guest.stdout, selectors.EVENT_READ)
            deadline = time.monotonic() + TIMEOUT
            pending = b""
            seen_entry = False
            while (remaining := deadline - time.monotonic()) > 0:
                if not events.select(remaining):
                    break
                data = os.read(guest.stdout.fileno(), 4096)
                if not data:
                    print(f"FAIL: QEMU ended before the kernel marker (see {log_path})")
                    return False
                log.write(data)
                log.flush()
                *lines, pending = (pending + data).split(b"\n")
                seen_entry |= any(line.rstrip(b"\r") == MARKER for line in lines)
                armed = seen_entry and any(line.startswith(ARMED) for line in lines)
                if armed:
                    # With -no-reboot, a reset ends QEMU. Catch immediate resets
                    # and shutdowns even if COM1 already sent the marker.
                    try:
                        status = guest.wait(timeout=RESET_WINDOW)
                    except subprocess.TimeoutExpired:
                        print(f"PASS: {MARKER.decode()}; exceptions armed (see {log_path})")
                        return True
                    print(f"FAIL: QEMU exited after the kernel marker ({status})")
                    return False
                # Keep the partial line bounded; both markers are short.
                if len(pending) > 256:
                    pending = pending[-256:]
            print(f"FAIL: no kernel marker within {TIMEOUT}s (see {log_path})")
            return False
        finally:
            if guest.poll() is None:
                guest.terminate()
                try:
                    guest.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    guest.kill()
                    guest.wait()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument("command", nargs="+")
    args = parser.parse_args()
    try:
        return 0 if boot(args.command, args.log) else 1
    except OSError as error:
        print(f"FAIL: {error}")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
