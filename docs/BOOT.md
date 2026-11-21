# Boot protocol

## Decision: Multiboot2 via GRUB (BIOS, El Torito ISO)

- **Works here today.** Ubuntu 22.04 ships `grub-mkrescue`, `grub-file` and
  the `grub-pc-bin` modules from apt, which the sandbox can install. Limine
  is not in Ubuntu 22.04's archive, so we would have to vendor or download
  binaries. Multiboot2 needs no extra dependency.
- **Verifiable.** `grub-file --is-x86-multiboot2` checks the header at link
  time. The Multiboot2 spec defines the boot state exactly: EAX = 0x36d76289,
  EBX = physical address of the boot information. That gives us a register
  value to assert on.
- **Known path.** Slopix boots the same way, so the behaviour on our QEMU
  setup is known.
- **Cost.** GRUB enters in 32-bit protected mode with paging off. The kernel
  must build its own page tables and enter long mode itself; Milestone 2
  does that in `boot.S`. Limine would hand over in long mode, and we could
  revisit it if UEFI or SMP bring-up favour it later.

Multiboot2 also allows UEFI boot (`grub-efi-amd64-bin`). Only BIOS boot is
built and tested for now.

## Image layout (`arch/x86_64/linker.ld`)

- Physical load base 1 MiB. `.boot` (the Multiboot2 header and the 32-bit
  entry code) is linked at its physical address because it runs before
  paging.
- Everything else is linked in the higher half at
  `0xFFFFFFFF80000000 + phys` (LMA = VMA - base), which `-mcmodel=kernel`
  requires.
- Separate PT_LOAD segments for boot (RWX), text (R-X), rodata (R--) and
  data/bss (RW-).
- A link-time assert keeps the image within the first 1 GiB.

## Toolchain

The host `gcc`/`ld` (x86-64 ELF) with freestanding flags: `-ffreestanding
-nostdinc -fno-pic -fno-pie -mcmodel=kernel -mno-red-zone
-mgeneral-regs-only -mno-sse*`, no stack protector, and only the compiler's
own freestanding headers. The host and target are both x86-64 ELF, and these
flags remove every hosted assumption, so a separate `x86_64-elf` cross
compiler is not needed. The link fails if any symbol is left undefined.
Assembly is GAS `.S` (preprocessed); nasm is not used.

## Boot sequence (current)

1. SeaBIOS loads the El Torito image, and GRUB starts with `timeout=0`.
2. GRUB loads `/boot/kernel.elf` with `multiboot2` and passes the cmdline
   from `tools/mkiso.sh`.
3. `_start` (32-bit) runs `cli; cld; hlt` in a loop. **Milestone 1 stops
   here.**

## Verification

`make test` runs `tests/qemu_halt_check.py`. It boots the ISO headless
(`-display none`, TCG, `-no-reboot`) and polls `info registers` over the
QEMU HMP monitor. It passes only when the CPU reports `HLT=1` with EIP
inside `_start`'s symbol range and EAX = the Multiboot2 bootloader magic. It
fails on a timeout or an early QEMU exit. Serial-based assertions replace
this from Milestone 3, once the kernel has output.
