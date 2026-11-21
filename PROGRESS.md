# Progress

**Last verified milestone:** M1, repository and build foundation, with a
bootable image (2026-11-21).

**Current phase:** First development phase. M1 is done. M2 is next.

## Working features
- Reproducible build: `make` produces `build/kernel.elf` (Multiboot2,
  checked with `grub-file`) and `build/syllaslopos.iso` (GRUB, BIOS).
- The image boots in QEMU (TCG, `-machine pc`). GRUB hands off to `_start`,
  which halts with EAX = 0x36d76289 (the Multiboot2 magic).
- `make test` checks this automatically, through register state over the
  QEMU monitor.

## Known limitations
- No C kernel yet, and no output of any kind (no serial, no VGA text).
- `_start` is a 32-bit `cli; hlt` stub. Long mode is not entered yet.
- BIOS boot only. UEFI has not been tested.
- The tests use TCG only. Real hardware has not been tested.

## Test commands
    bash tools/setup-toolchain.sh   # once per sandbox rebuild
    make test                  # boots ISO headless; PASS line + exit 0

Last known-good boot output:
`PASS: CPU halted at IP=0x10001b inside _start [0x100018,0x10001d), EAX=0x36d76289`

## Next dependency-ready task
M2, kernel entry: build the boot page tables (identity map plus higher
half), enter long mode, set up the stack, clear .bss, call `kmain(magic,
mbi)` in C, validate the Multiboot2 magic and the info structure (size,
alignment, tag walk), then halt in a controlled way.
