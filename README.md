# SyllaslopOS

An independent x86-64 hobby OS, successor to Slopix. Syllaslop Console
(`slc`) opens at boot. It provides a graphical command prompt, searchable
help, command history, and views of the running system.

## Build and run

Use an x86-64 Linux host with [rustup](https://rustup.rs/). The repository
selects Rust 1.85.1 and the `x86_64-unknown-none` target.

```sh
sudo apt install binutils make grub-common grub-pc-bin xorriso qemu-system-x86 qemu-system-gui python3
rustup show
make -j
make run
```

On Fedora, install `binutils make grub2-tools grub2-tools-extra grub2-pc-modules
xorriso qemu-system-x86-core qemu-ui-gtk python3` and use the GRUB tool names
`GRUB_FILE=grub2-file GRUB_MKRESCUE=grub2-mkrescue` when invoking `make`.
`make run QEMU_DISPLAY=sdl` selects an installed SDL backend.

| Command | Result |
| --- | --- |
| `make` | Builds `build/kernel.elf` and `build/syllaslopos.iso` |
| `make run` | Opens the VGA console and mirrors command results through COM1 |
| `make run-serial` | Boots headlessly with COM1 on stdio |
| `make test` | Checks the ELF and original serial boot marker in QEMU |
| `make test-console` | Runs host tests, drives PS/2 and COM1 in the guest and captures VGA screenshots |
| `make debug` | Pauses QEMU for GDB on `127.0.0.1:1234` |

A separate image configured for text-mode recovery can be built and run with
`make BUILD=build/recovery GRUB_DEFAULT=serial run-serial`. For an 800×600
image, use `make BUILD=build/compact GRUB_DEFAULT=compact run`.

## Use the console

Type `help` for the command registry or press F1 to open the searchable
reference. F2 opens system information, F3 opens CPU details and F4 opens
the memory view. Tab completes commands and supported arguments. Up and Down
recall previous commands. Use Home, End and Delete to edit, Ctrl+L to clear
activity, and Esc to close a view. On COM1, Ctrl+] closes a view.

Try `bootinfo`, `devices`, `history`, `theme ember`, `theme mono`,
`zoom 2` and `metrics`. See [the console guide](docs/CONSOLE.md) for the
input model, verification, and limits, and [the boot contract](docs/BOOT.md)
for memory ownership and firmware requirements.

The kernel source is MIT-licensed. The ISO incorporates GRUB under its
upstream licensing. The bundled bitmap masks are derived from DejaVu.
See [references and licenses](docs/REFERENCES.md).
