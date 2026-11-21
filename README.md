# SyllaslopOS

An independent x86-64 hobby OS, successor to Slopix. The project is building
toward an independent kernel and a desktop informed by Syllable/AtheOS.

The first boot path is GRUB Multiboot2 on BIOS. A 32-bit bootstrap enters long
mode, switches to a kernel stack and calls a freestanding Rust entry. On success,
the guest prints `SYLLASLOPOS: x86_64 kernel entry ok` to COM1 and halts.
See [progress and verification](PROGRESS.md) and the [boot contract](docs/BOOT.md).

## Build and boot

Use an x86-64 Linux host with [rustup](https://rustup.rs/) installed. The
repository selects Rust 1.85.1 and the `x86_64-unknown-none` target. Debian
host packages:

```sh
sudo apt install binutils make grub-common grub-pc-bin xorriso qemu-system-x86 python3
rustup show
make -j
make test
```

On Fedora, the GRUB commands use the `grub2-` prefix. Install
`binutils make grub2-tools grub2-tools-extra grub2-pc-modules xorriso qemu-system-x86-core python3`
using `dnf`, then run:

```sh
make -j GRUB_FILE=grub2-file GRUB_MKRESCUE=grub2-mkrescue
make test GRUB_FILE=grub2-file GRUB_MKRESCUE=grub2-mkrescue
```

`make` writes `build/kernel.elf` and `build/syllaslopos.iso`. `make test`
checks the ELF, prints both artifacts' SHA-256 digests, boots the ISO and
captures output in `build/boot.log`. `make run` keeps QEMU open on stdio.
`make debug` pauses QEMU and listens for GDB on `127.0.0.1:1234`.
`make clean` removes the generated files. Use `GRUB_PLATFORM` to override
the BIOS module directory, `QEMU_BIOS` to select an exact firmware binary,
and `QEMU_MEMORY` to change guest RAM.

## Layout

- `arch/x86_64/` owns the assembly entry, bootstrap mappings and ELF layout.
- `kernel/` contains the Rust entry and COM1 code.
- `boot/grub/` defines the BIOS ISO boot menu.
- `tools/boot-check.py` drives the bounded QEMU test.

The kernel source is MIT-licensed. The ISO incorporates GRUB under its
upstream license. See [references and licensing](docs/REFERENCES.md).
