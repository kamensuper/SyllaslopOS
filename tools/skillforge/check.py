#!/usr/bin/env python3
"""Lint project skills and optionally query a fresh local Codex app server."""
import argparse
import json
import os
from pathlib import Path
import queue
import re
import subprocess
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[2]
METADATA = re.compile(r"\A---\nname: ([a-z0-9-]+)\ndescription: ([^\n]+)\n---\n")


def lint():
    skills = {}
    for path in sorted((ROOT / ".agents/skills").glob("*/SKILL.md")):
        body = path.read_text()
        match = METADATA.match(body)
        if not match:
            raise ValueError(f"invalid frontmatter: {path.relative_to(ROOT)}")
        name, description = match.groups()
        if name != path.parent.name or not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", name):
            raise ValueError(f"invalid skill name: {name}")
        if len(name) > 64 or not 1 <= len(description) <= 1024 or not body[match.end():].strip():
            raise ValueError(f"invalid skill size: {name}")
        skills[name] = path
        print(f"{name}: {len(body.encode())} bytes; metadata {len(match[0].encode())} bytes")
    if not skills:
        raise ValueError("no skills found")
    return skills


def discover(skills):
    with tempfile.TemporaryDirectory(prefix="slc-codex-") as profile, tempfile.TemporaryDirectory(prefix="slc-outside-") as outside:
        # The temporary Codex profile is process-local and has no credentials.
        env = os.environ | {"CODEX_HOME": profile, "CODEX_SQLITE_HOME": profile}
        process = subprocess.Popen(
            ["codex", "app-server", "--listen", "stdio://"],
            cwd=ROOT, env=env, text=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        )
        messages = queue.Queue()

        def read_messages():
            for line in process.stdout:
                messages.put(line)
            messages.put(None)

        threading.Thread(target=read_messages, daemon=True).start()

        def send(method, params=None, ident=None):
            record = {"method": method}
            if params is not None:
                record["params"] = params
            if ident is not None:
                record["id"] = ident
            process.stdin.write(json.dumps(record) + "\n")
            process.stdin.flush()

        def receive(ident):
            deadline = time.monotonic() + 12
            while True:
                line = messages.get(timeout=max(0, deadline - time.monotonic()))
                if line is None:
                    raise RuntimeError("Codex app server exited")
                record = json.loads(line)
                if record.get("id") == ident:
                    if "error" in record:
                        raise RuntimeError(record["error"])
                    return record["result"]

        cwds = [str(ROOT), str(ROOT / "kernel/console"), outside]
        try:
            send("initialize", {"clientInfo": {"name": "slc-skill-check", "version": "1"}}, 1)
            receive(1)
            send("initialized")
            send("skills/list", {"cwds": cwds, "forceReload": True}, 2)
            rows = {row["cwd"]: row for row in receive(2)["data"]}
            for cwd in cwds:
                row = rows[cwd]
                mine = {skill["name"]: skill for skill in row["skills"] if skill["name"].startswith("syllaslop-")}
                expected = {} if cwd == outside else skills
                if row.get("errors") or set(mine) != set(expected):
                    raise RuntimeError(f"unexpected skill discovery at {cwd}: {row.get('errors')}, {list(mine)}")
                for name, path in expected.items():
                    item = mine[name]
                    if not item["enabled"] or item["scope"] != "repo" or Path(item["path"]) != path:
                        raise RuntimeError(f"unexpected metadata for {name}: {item}")
                print(f"{cwd}: {len(mine)} project skills discovered")
        finally:
            process.terminate()
            process.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--discover", action="store_true")
    args = parser.parse_args()
    skills = lint()
    if args.discover:
        discover(skills)


if __name__ == "__main__":
    main()
