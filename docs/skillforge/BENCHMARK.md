# Skillforge benchmark

## Trial design

Four fresh agents in the same collaboration environment each received two
small isolated fixtures and the same acceptance contract. The source copies
came from SLC commit `d58adb7f3dfe109c327bba17f6d49f22058a4a53`. Fault
seeding changed binary operator associativity and let a silent, successful
child exit count as a passing boot. The actual kernel and boot checker
were outside the trial directories.

A worked from the sources. B loaded the Rust skill for the parser and used
direct work for the oracle. C added the verification skill and a focused
self-review. D had the same skills as C, added a written plan and separate
specification and code-quality passes, then brought in a fresh reviewer.
D is a deliberately small heavyweight arm. It does not measure the full
Superpowers per-task fan-out or compare different model tiers.

The grader runs eight shared Rust cases and six fake guest cases, with
child cleanup, output logging and time bounds. The unchanged seeded control
failed three Rust tests (two shared and one original) and two guest cases.
The benchmark agent could see the kernel's original unit test but had no
access to the independent grader during the trial. QEMU was exercised
separately on the actual branch.

## Quality and cost evidence

| Arm | Shared Rust / fake guest | Extra CR / noisy early exit | Solver edit rounds | Solver tool calls / inner operations | Time from fixture seed to first complete source edit |
| --- | --- | --- | ---: | ---: | ---: |
| A baseline | 8/8, 6/6 | fail / fail | 1 | 6 / 9 | ~46 s |
| B lightweight | 8/8, 6/6 | pass / fail | 1 | 6 / 11 | ~63 s |
| C balanced | 8/8, 6/6 | pass / fail | 1 | 7 / 12 | ~86 s |
| D heavyweight | 8/8, 6/6 | pass / pass after independent review | 2 | 20 / 34, plus the reviewer | ~183 s, ~441 s to reviewed source |

Tool counts are solver reports. Inner operations count shell and patch
operations inside tool calls. D also had an independent agent dispatch and
a repair handoff. Source-edit times are filesystem timestamps measured
from fixture preparation. They include launch skew and, for D's reviewed
result, parent handoff time. They exclude some final checks and report
writing. Treat them as latency proxies. Replaying the final patches took
roughly two seconds per shared grading run on this host.

The two edge cases were added **after the initial implementations** to probe
differences and the review finding. B and C independently tightened malformed
marker handling. D's reviewer demonstrated a false PASS when a child
printed the marker and filled the output pipe before it could exit. The
original probe stopped reading during its reset window, so the child
blocked. D drained the pipe during that same bounded window and added a
regression test. Early-exit verification then passed. The finding remains
open in the production probe because this branch changes the skill system.

| Dimension | Weight | Retrospective rubric |
| --- | ---: | --- |
| Correctness | 35 | Shared acceptance cases all pass |
| Regression prevention | 20 | 10 for shared negative and normal cases, plus 5 per exploratory edge |
| Debugging effectiveness | 15 | Both failures reproduced and fixes localized to the cause |
| Architectural compatibility | 15 | Existing parser and probe entry points, checked arithmetic and deadlines kept |
| Maintainability | 10 | Focused production-source diff and scoped test evidence |
| Documentation | 5 | Reproduction and verification commands reported with their scope |

Applying this rubric yields **A 90 / B 95 / C 95 / D 100**. The score records
the edge discoveries and exact acceptance scope, and is not a blind or
statistical comparison. No evaluated shared case regressed. The trial
covers one sample per arm, two tasks, shared host load and subjective rubric
dimensions. A repeated replay checks the scorer and patch artifacts, not
agent-to-agent variance. Input/output tokens, cached tokens, reasoning
counters and dollar billing were unavailable from the collaboration tool.

## Activation budget

| Text | Measured bytes | Estimated tokens at 3–5 bytes/token |
| --- | ---: | ---: |
| Always-on root instructions and four frontmatters | 1,353 | 270–451 |
| One skill file, including its frontmatter | 941–1,132 | 188–378 |
| Two loaded skill files, as used by C and D | 2,251 | 450–751 |
| Optional subsystem routing index | 2,254 | 451–752 |
| Bible Markdown in inspected local checkout, all 49 pages | 133,407 | 26,681–44,469 |

B loaded 1,132 skill bytes, C/D loaded 2,251, and A loaded zero. These
are authored context bytes only. Client wrappers, read repetition,
reasoning, tool I/O, planning, reviewer context, and cache reads and writes
are outside that estimate. For upstream optional references, one Superpowers
root-cause document measured 5,321 bytes, the reviewer prompt 6,449, and
the task-review prompt 9,175. They cost about 1.1–1.8k, 1.3–2.2k, and
1.8–3.1k estimated tokens if opened. None are activated by project skills.

## Replay and interpretation

From the repository with Python and rustc available:

```sh
trial_dir=$(mktemp -d)
python3 tools/skillforge/benchmark.py prepare "$trial_dir"
for arm in a b c d; do
    git -C "$trial_dir/$arm" apply --unidiff-zero "$PWD/tools/skillforge/trials/$arm.patch"
    python3 tools/skillforge/benchmark.py grade "$trial_dir/$arm"
    python3 tools/skillforge/benchmark.py grade "$trial_dir/$arm" --exploratory
done
```

Exploratory failures in A/B/C are expected. The saved patches record fixture
source edits only. Run a new comparison with fresh prepared fixture folders,
the same task text, equal budgets and agents isolated from this report and
the grader. Capture per-agent timing and token usage if the next harness
exposes them. Add hardware-facing fixtures before generalizing to page-table
or interrupt work. Only grade local trial code that has been reviewed for
side effects.

Choose fast for well-bounded repairs. B and C scored equally in this pilot,
so open the verification skill when it answers a real evidence question.
Choose balanced for ordinary subsystem changes and escalate when a false
positive, corruption or isolation failure would materially affect subsequent
development. The noisy-exit finding justifies one independent review of
such an oracle. This trial offers no evidence that multiple routine reviews
are worth their cost.
