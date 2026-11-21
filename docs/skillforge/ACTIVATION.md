# Skill activation

Codex CLI can discover repository skills from `.agents/skills/` when the
working directory is inside this checkout. Start a session from this repo,
or request a skill explicitly with `$syllaslop-boot`, `$syllaslop-rust`,
`$syllaslop-console`, or `$syllaslop-verify`. Clients with no repo skill
discovery can use `AGENTS.md` and open the selected `SKILL.md` as a local
file. Account-level skill installation is unnecessary.

| Tier | Choose based on | Work and evidence |
| --- | --- | --- |
| Fast | Local, reversible changes with a clear success predicate | Read the affected source and tests. Load one skill if useful. Fix and run the smallest discriminating checks. |
| Balanced | Ordinary subsystem work or unclear failure modes | Load the matching domain skill. Add verification guidance when diagnosing or combining evidence. Record a short plan, run targeted plus integration checks, and inspect one focused diff. |
| Critical | Unsafe ownership, page tables, interrupt frames, isolation, concurrency or storage integrity | Write invariants and failure modes. Add negative tests, inspect the machine boundary and use independent review where it can catch a materially different error. Require scoped evidence before integration. |

A low-risk task stays fast when focused tests cover the contract. Escalate
on an actual hardware assumption, ambiguous evidence or potential corruption.
Read [ROUTING.md](ROUTING.md) for task-local reference pointers. Check
installed skill metadata and the local client with
`python3 tools/skillforge/check.py --discover`. This check uses a temporary
state directory and makes no model request. It requires Codex CLI with the
`skills/list` app-server method. Omit `--discover` to lint files only.

## Observed in this environment

Codex CLI 0.172.0-alpha.2 reported all four as enabled `repo` skills with
exact paths both from the checkout root and from `kernel/console`. From a
directory outside the checkout it reported zero project skills.

A fresh agent given metadata only and a theme-clipping task selected
`syllaslop-console`, read exactly its one file, and identified the QEMU
scanout checks for both resolutions. Benchmark arms B–D used the specified
local guide files. The work environment's nested CLI could start its
metadata server but could not start a model turn under allowed sandbox
modes. Model-driven implicit activation in the nested client remains
unverified. The metadata and fresh-agent read tests cover discovery and
targeted loading independently.
