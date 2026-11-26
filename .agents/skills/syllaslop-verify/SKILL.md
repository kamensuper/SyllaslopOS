---
name: syllaslop-verify
description: Diagnose SyllaslopOS failures and review evidence for build, unit, ELF, QEMU, GDB, serial, screenshot and reproducibility claims.
---

Reproduce on the smallest path that can distinguish the suspected failure. Capture the exact command, expectation and first observed divergence. Inspect the nearest producer and consumer before patching.

- Treat build, loader, end-to-end guest and hardware evidence separately. Record relevant tool versions and artifact identity. Use the existing Make targets and inspect their success predicates.
- For a boot probe, require the complete expected line after launch. Exercise silence, partial or misleading output, early exit, timeout and a live success. Preserve child cleanup and finite termination. A printed marker followed by immediate exit must fail.
- Prefer one focused review against explicit invariants. Add independent review for high-consequence changes when it tests a genuinely distinct failure mode. Run fresh affected checks before reporting success.
- See `docs/skillforge/ACTIVATION.md` for evidence and escalation by tier.
