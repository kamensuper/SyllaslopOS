# Boot contract

## Decision · 2026-11-21

Use the existing repository's GRUB Multiboot2 path for the first kernel entry.
A BIOS El Torito ISO and QEMU TCG provide one small, observable boot path.
GRUB has a host-side header check and the precedent of Slopix. This choice
keeps responsibility for the long-mode transition in the kernel. The Rust
kernel uses the freestanding `x86_64-unknown-none` target, with a static
higher-half code model, aborting panic handler and the SysV calling convention.

## Entry and mapping

1. SeaBIOS starts GRUB from the ISO. The GRUB menu uses `multiboot2` to load
   `/boot/kernel.elf`. The header lives at file offset `0x1000` in the linked
   ELF, within the first 32 KiB required by Multiboot2.
2. GRUB enters `_start` in 32-bit protected mode at physical `0x100018`.
   Assembly masks interrupts, clears DF, checks the magic in EAX and saves EBX
   as the physical boot info address. It checks the CPU's long-mode bit.
3. Assembly initializes four 4 KiB page-table pages from the ELF's zero-filled
   boot BSS and loads its own GDT and CR3. It enables PAE, EFER.LME and paging,
   then far-jumps into 64-bit code through the GDT. The leaf maps physical
   `0..0x200000` at both `0` and `0xffffffff80000000`. The linker constrains
   the entire image and stack to this range.
4. The 64-bit stub sets the data selectors, switches to the 16 KiB kernel
   stack and calls `kernel_main(u32, u64)` with a 16-byte aligned pre-call
   stack. Rust checks the original magic, the nonzero/aligned info address
   and that RSP lies within the kernel stack. It does not read the info
   structure. The success line is emitted after these checks. COM1 uses
   115200 baud, 8N1 and bounded transmit polling.

The ELF has separate RX, RW and read-only load segments. Both bootstrap
virtual aliases use a coarse supervisor-writable, executable 2 MiB leaf.
The entry path keeps its boot GDT, mappings and stack alive. Interrupts stay
masked. The milestone includes no IDT, frame allocator, memory-map parser or
boot memory reclaim. Before reclaim or dropping the low alias, establish and
verify kernel-owned exception handling and the next mapping layout, then
account for every live pointer and table.

## Verification

`make check` runs the GRUB Multiboot2 probe and reports ELF metadata.
`make test` boots a PC i440FX 7.2 machine on QEMU TCG with a `qemu64` CPU.
It requires a full COM1 marker line within 20 seconds and rejects an immediate
reset or exit after the marker. The guest check covers BIOS on this emulator.
The exact tool and firmware versions are in [progress and verification](../PROGRESS.md).

- [GNU Multiboot2 specification](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html)
- [Rust x86_64-unknown-none target](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html)
