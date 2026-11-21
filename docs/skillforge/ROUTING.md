# Route by subsystem

Read the changed source first and open the named skill when its constraints
are relevant. The Bible volume prefixes below identify folders in the
user's SyllaslopOS Bible checkout. The Bible is optional and lives outside
this repository. Search the selected volume before expanding the scope.
Use its V32 specification index only for the specification being checked.

| Task | Skill | Bible volumes and what to look up |
| --- | --- | --- |
| GRUB, Multiboot2, UEFI design, early assembly, CPUID, ELF/linker | syllaslop-boot | V04 toolchain, V05 handoff, V06 CPU |
| Frames, heaps, mappings, PTEs, TLB, framebuffer MMIO | syllaslop-rust, boot if crossing handoff | V07 frames, V08 paging and WORKED-WALK, V09 allocation, V16 device geometry |
| GDT, IDT, TSS, interrupts, timers, exceptions, ABI | syllaslop-boot or syllaslop-rust | V06 CPU, V10 ENTRY-CHECKLIST, V11 timer and synchronization |
| Locks, atomics, scheduling, context switch | syllaslop-rust | V11 time and concurrency, V06 machine state, V10 interrupt frames |
| Ring 3, syscalls, ELF loading, isolation and SDK | syllaslop-rust | V12 processes, V13 ABI, V21 SDK, V22 apps, V23 security |
| PCI, ACPI, storage controllers, DMA, serial, network drivers | syllaslop-rust | V15 device lifecycle, V14 storage, V19 networking, V05 firmware |
| VFS, FAT32, persistence and binary parsing | syllaslop-rust, verify when needed | V14 CRASH-ORDERING and filesystem invariants, V13 binary formats, V26 validation |
| Framebuffer, fonts, clipping, layout, damage, command UI, accessibility | syllaslop-console | V16 SURFACE-TESTS, V17 interaction, V18 input, V24 measurement |
| PS/2, keyboard state or recovery console | syllaslop-console | V18 input, V15 driver contracts, V25 diagnostics |
| QEMU, GDB, serial, crashes, fuzz or property tests, profiling, reproducibility | syllaslop-verify | V25 failure atlas, V26 verification, V24 measurement, V04 builds |
| Architecture and compatibility decision | Start with the affected row above | V02 project principles, V03 atlas, V27 decisions, V28 dependencies, V31 roadmap |

V10, V11, V12 and V14 describe design territory. This branch uses the current
SLC firmware and console baseline. Confirm subsystem availability in code.
