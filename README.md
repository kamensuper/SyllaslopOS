# SyllaslopOS

An independent x86-64 hobby operating system, successor to Slopix. The
long-term goal is a real kernel (memory management, scheduling, syscalls,
userspace, filesystems) with a desktop modelled architecturally on
Syllable/AtheOS. The name is a joke; the engineering is not.

**Current state: Milestone 1.** The image boots via GRUB (Multiboot2) in QEMU
and halts in the 32-bit entry stub. There is no C kernel and no output yet.
See [PROGRESS.md](PROGRESS.md).

## Setup (Ubuntu 22.04 / Debian-like)

    bash tools/setup-toolchain.sh  # idempotent; uses sudo when not root

Installs gcc/binutils, make, QEMU, GRUB (i386-pc), xorriso, mtools, gdb, python3.

## Build, run, test

    make          # build/kernel.elf and build/syllaslopos.iso
    make run      # boot in QEMU, serial on stdio
    make debug    # as run, paused, gdb stub on :1234
    make test     # headless automated boot check (exit status 0 = pass)
    make clean

All output goes to `build/` (gitignored).

## Layout

    arch/x86_64/   boot entry (GAS) and linker script
    boot/grub/     grub.cfg template
    tools/         toolchain setup, ISO builder
    tests/         QEMU test harnesses
    docs/          BOOT.md (boot protocol), REFERENCES.md (references and licensing)

## License

MIT, see [LICENSE](LICENSE). Read [docs/REFERENCES.md](docs/REFERENCES.md)
before using any outside code.
