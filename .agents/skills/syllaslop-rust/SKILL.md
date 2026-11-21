---
name: syllaslop-rust
description: Review or change SyllaslopOS freestanding Rust, bounded parsers, layout and ABI, unsafe access, platform I/O, and memory ownership.
---

Inspect the current caller and the available input types before changing the contract. Keep pure logic host-testable, and build the actual `x86_64-unknown-none` target before claiming integration.

- Carry units and address domains explicitly. Check arithmetic and bounds before slicing, mapping or pointer construction. Preserve alignment, initialization, lifetime, and exclusive access at unsafe boundaries. Treat a volatile operation as a device access, with synchronization and ordering justified separately.
- Preserve parser limits, malformed-input behavior and error specificity. Use a discriminating failing case and the adjacent successful cases. Keep command formatting, result types and caller behavior stable unless the request changes them.
- Any new CPU/ABI/interrupt assumption needs a named manual and the hardware or emulator scope. Route memory, interrupts, or userspace through the matching row in `docs/skillforge/ROUTING.md` if needed.
