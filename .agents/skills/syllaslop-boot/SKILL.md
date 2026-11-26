---
name: syllaslop-boot
description: Work on SyllaslopOS BIOS boot, GRUB and Multiboot2, x86-64 entry, ELF layout, linker scripts, early mappings and boot handoff.
---

Read `arch/x86_64/boot.S`, `arch/x86_64/linker.ld`, `boot/grub/grub.cfg` and the relevant part of `docs/BOOT.md`.

- Track physical load addresses separately from linked virtual addresses. Check Multiboot2 header placement and alignment, entry mapping, stack alignment and each load segment's permissions.
- Follow every live boot-info, page-table, descriptor and stack reference before changing aliases or reclaiming memory. Treat framebuffer ranges as device memory with a separate ownership and caching contract.
- Confirm the ELF with `grub-file --is-x86-multiboot2` and `readelf -lW`. Run `make test` for the BIOS path and `make test-console` if framebuffer handoff or recovery changes. Track the first missing serial milestone when boot fails.
- Use the boot or memory row in `docs/skillforge/ROUTING.md` only if design details are needed. Treat future UEFI and interrupt designs as separate work.
