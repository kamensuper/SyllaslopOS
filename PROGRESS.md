# Progress

## M0–M1 · first successor kernel entry

The GitHub baseline was `kamensuper/SyllaslopOS` `main` at
`6a07d543bcd9ade20a7bba5dcca0bac8ac3c`. That tree contained the GRUB template
and documents describing earlier boot work. The source, build recipe, ISO
and monitor test described there were absent from the committed tree. The
separately supplied SyllaslopOS Bible is a design and history reference.

The workspace builds a freestanding Rust ELF and a GRUB BIOS ISO from source.
The first guest milestone is a serial line from 64-bit Rust after the
Multiboot2 argument and kernel stack checks. See [the boot contract](docs/BOOT.md)
for ownership and the mapping scope.

## Next milestone · exceptions and memory

Install a kernel IDT with a controlled fault test, then validate the
Multiboot2 info structure in an explicitly mapped range before parsing its
memory information. Establish the new page protections and track live boot
state before handing any eligible frames to an allocator.

## Verification record · 2026-11-21

Source commit `cdb3fe7d34e3fbab99063205e763664a1bb1202f` was built in two
independent clean clones on Debian 13 x86-64. Both clean builds used
`make -j4 all`; `cmp` confirmed both output files matched byte-for-byte.

| Tool or boot input | Tested identity |
| --- | --- |
| Rust | `1.85.1 (4eb161250)`, target `x86_64-unknown-none` |
| GNU assembler and linker | Debian Binutils `2.44` |
| GRUB host tools and i386-pc modules | Debian `2.12-9+deb13u2`, [source manifest](https://deb.debian.org/debian/pool/main/g/grub2/grub2_2.12-9+deb13u2.dsc) |
| xorriso | `1.5.6` |
| GNU Make and Python | `4.4.1` and `3.12.14` |
| QEMU | Debian `1:10.0.13+ds-0+deb13u1`, TCG, `pc-i440fx-7.2` |
| Firmware | SeaBIOS `1.16.3-2` `bios-256k.bin`, SHA-256 `fc6d2ea888862e6b26d53fb877ab712324726eb8f44eb6baf076c64a31f1c3fb` |

| Output | Size | SHA-256 |
| --- | ---: | --- |
| `build/kernel.elf` | 13,840 bytes | `36c08ca35ab3911628abf2d0a5af9a86caa5dbb2aab277766b917bbe207bde8a` |
| `build/syllaslopos.iso` | 2,723,840 bytes | `6a781a515aa4e3cdacab3cb9af27d15e3a04a4713892f9be4acd713dc1346ad2` |

`readelf -h -lW -SW -rW -dW build/kernel.elf` identified an x86-64 executable
with entry `0x100018`, five LOAD segments (RX, RW, RX, R, RW), and no dynamic
section or relocation records. `nm -u build/kernel.elf` produced no symbols.
`grub-file --is-x86-multiboot2 build/kernel.elf` returned zero.

`make test QEMU_BIOS=/path/to/bios-256k.bin QEMU_MEMORY=128M` passed, then
passed again with `QEMU_MEMORY=256M`. The sandbox additionally set
`QEMU="qemu-system-x86_64 -L /path/to/qemu-firmware"` for the extracted QEMU
and SeaBIOS data. Both serial captures contained:

```text
SYLLASLOPOS: x86_64 kernel entry ok
```

A negative guest run changed QEMU's `-cpu qemu64` to `-cpu qemu32` in the
command printed by `make test`. It produced no serial marker and the checker
returned 1 after 20 seconds. These runs cover the BIOS emulator path. Fedora
host, UEFI firmware and physical hardware checks are pending.
