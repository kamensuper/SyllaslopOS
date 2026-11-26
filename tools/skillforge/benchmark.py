#!/usr/bin/env python3
"""Prepare isolated trials and score production-source copies with a shared oracle."""
import argparse
import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from time import perf_counter

ROOT = Path(__file__).resolve().parents[2]
GRAMMAR = "keep arithmetic precedence, left associativity, bounded recursion and checked errors"
NEGATIVE = "reject exit success, silence, partial output and lookalikes without a complete marker"
CASES = (
    ('left_assoc', '100 - 12 - 3 * -2', 'Ok(94)'),
    ('same_precedence', '100 / 5 / 2', 'Ok(10)'),
    ('precedence', '2 + 3 * 4', 'Ok(14)'),
    ('parentheses_and_base', '0xff & (2 + 3 * 4) | 0b1', 'Ok(15)'),
    ('division_by_zero', '1 / 0', 'Err("division by zero")'),
    ('overflow', '9223372036854775807 + 1', 'Err("integer overflow")'),
    ('malformed', '0b12', 'Err("unexpected token")'),
    ('depth_bound', '(' * 20 + '1' + ')' * 20, 'Err("expression nested too deeply")'),
)
MARKER = "SYLLASLOPOS: x86_64 kernel entry ok"
GUESTS = {
    "silence_exits_zero": ("pass", False),
    "partial_then_exit": (f"sys.stdout.write({MARKER!r})", False),
    "prefix_lookalike": (f"print('noise ' + {MARKER!r}, flush=True); time.sleep(4)", False),
    "silence_timeout": ("time.sleep(4)", False),
    "marker_then_exit": (f"print({MARKER!r}, flush=True)", False),
    "fragmented_live_marker": (
        f"print({MARKER[:12]!r}, end='', flush=True); time.sleep(.04); "
        f"print({MARKER[12:]!r}, flush=True); time.sleep(4)", True),
}

# Added after the first round. This suite measures the value of edge discovery,
# independently of the primary acceptance suite above.
EXPLORATORY = {
    "double_cr": (f"print({MARKER!r} + '\\r\\r', flush=True); time.sleep(4)", False),
    "burst_before_early_exit": (
        f"os.write(1, {MARKER.encode() + bytes([10])!r}); "
        "sys.stdout.buffer.write(b'X' * 1048576); sys.stdout.flush(); sys.exit(7)",
        False,
    ),
}


def replace_once(text, old, new):
    if text.count(old) != 1:
        raise ValueError(f"fixture seed expected one occurrence of {old!r}")
    return text.replace(old, new, 1)


def prepare(root):
    # Derive every arm from the same source bytes. Hold this preparation and
    # grading utility outside the trial directories during a measured run.
    parser = (ROOT / "kernel/console/numbers.rs").read_text()
    parser = replace_once(parser, "self.expr(priority + 1,", "self.expr(priority,")
    probe = (ROOT / "tools/boot-check.py").read_text()
    probe = replace_once(probe, 'before the kernel marker (see {log_path})")\n                    return False',
                         'before the kernel marker (see {log_path})")\n                    return guest.wait() == 0')
    for arm in ("a", "b", "c", "d"):
        path = root / arm
        path.mkdir(parents=True)
        (path / "numbers.rs").write_text(parser)
        (path / "boot-check.py").write_text(probe)
        if arm in "bcd":
            copy_skill(path, "rust")
        if arm in "cd":
            copy_skill(path, "verify")
    print(f"Prepared four trials: {GRAMMAR}; {NEGATIVE}.")


def copy_skill(path, kind):
    skill = ROOT / ".agents/skills" / f"syllaslop-{kind}" / "SKILL.md"
    (path / f"{kind}-guide.md").write_bytes(skill.read_bytes())


def grade(path, compiler, exploratory):
    compiler = str(Path(shutil.which(compiler) or compiler).resolve())
    start = perf_counter()
    with tempfile.TemporaryDirectory(prefix="slc-score-") as temp:
        tmp = Path(temp)
        (tmp / "numbers.rs").write_bytes((path / "numbers.rs").read_bytes())
        tests = [f'#[test] fn oracle_{name}() {{ assert_eq!(numbers::evaluate("{expr}"), {expect}); }}'
                 for name, expr, expect in CASES]
        (tmp / "test.rs").write_text("mod numbers;\n" + "\n".join(tests))
        output = tmp / "numbers-test"
        build = subprocess.run([compiler, "--edition=2021", "--test", "test.rs", "-o", str(output)],
                               cwd=tmp, capture_output=True, text=True, timeout=20)
        if build.returncode:
            print(build.stderr)
            return 1
        run = subprocess.run([output], capture_output=True, text=True, timeout=10)
        print(run.stdout.strip())
        if run.stderr:
            print(run.stderr.strip())

        spec = importlib.util.spec_from_file_location("probe", path / "boot-check.py")
        probe = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(probe)
        probe.TIMEOUT = 0.65
        good = True
        guests = EXPLORATORY if exploratory else GUESTS
        for name, (code, expected) in guests.items():
            log = tmp / (name + ".log")
            pid_file = tmp / (name + ".pid")
            setup = (
                "import os, sys, time; from pathlib import Path; "
                "Path(sys.argv[1]).write_text(str(os.getpid())); "
            )
            tick = perf_counter()
            got = probe.boot([sys.executable, "-c", setup + code, str(pid_file)], log)
            took = perf_counter() - tick
            pid = int(pid_file.read_text())
            reaped = not Path(f"/proc/{pid}").exists()
            logged = log.is_file() and (name != "fragmented_live_marker" or MARKER.encode() in log.read_bytes())
            passed = got is expected and reaped and logged and took < 3
            good = good and passed
            print(f"{name}: {'PASS' if passed else 'FAIL'} verdict={got} reaped={reaped} log={logged} {took:.3f}s")
    print(f"oracle_seconds={perf_counter() - start:.3f}")
    return int(not good or run.returncode != 0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("prepare", "grade"))
    parser.add_argument("directory", type=Path)
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--exploratory", action="store_true")
    args = parser.parse_args()
    sys.dont_write_bytecode = True
    if args.action == "prepare":
        prepare(args.directory)
        return 0
    return grade(args.directory, args.rustc, args.exploratory)


if __name__ == "__main__":
    raise SystemExit(main())
