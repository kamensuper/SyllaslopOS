# Progress

## M0–M1 · first successor kernel entry

The GitHub baseline was `kamensuper/SyllaslopOS` `main` at
`6a07d543bcd9ade20a7bba5dcca0bac8ac3c8a3c`. That tree contained the GRUB template
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

## SLC · native console verification · 2026-11-21

The graphical console was built from local source commit
`71be8b0346250262082a1447658a8e66e47a935d` and tree
`e495ebf8a309c7e173ec756bee1a570fce7c8eef`, based on `ac0d8ee`.
The original boot artifacts and entry marker were reproduced before editing.
The build used the same compiler, GRUB, QEMU and SeaBIOS versions above.

| Check | Result |
| --- | --- |
| `make test` | Multiboot2 ELF accepted, original entry marker observed |
| `make test-host` | 12 passed, including malformed boot tags, framebuffer bounds, key decoding, layout, history and command parsing |
| `make test-console` | Real PS/2 editing and commands at 1024×768 and 800×600, plus COM1 commands in text-mode recovery |
| Default ISO with `-vga none` | COM1 fallback accepted commands without forcing the recovery boot option |
| Independent clean build | `cmp` matched the kernel ELF and default ISO byte for byte |

The guest exercised completion into `sysinfo`, searchable `help`, typed
`echo`, history recall and editing, theme and zoom changes, and `clear`.
The verifier captured PPM scanout with QMP `screendump`, paired with guest
COM1 output. The screenshot files retain those pixels as PNG.

To repeat, use the [console verification commands](docs/CONSOLE.md). The
extracted emulator additionally needed
`QEMU="qemu-system-x86_64 -L /path/to/qemu-firmware"` and
`QEMU_BIOS=/path/to/bios-256k.bin`. The automatic no-VGA test used:

```sh
python3 tools/console-check.py --output build/screens-no-vga \
  --iso build/syllaslopos.iso --qemu qemu-system-x86_64 -vga none
```

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `build/kernel.elf` | 166,088 | `ee4d00f54d7758080cf577682c4f7408d8c3dd7fb847ba137f4c7e0f774bf38f` |
| `build/syllaslopos.iso` | 2,877,440 | `54d666d159fcb8ace6c39b665064a513dc0f95ab389aef9fb08d4599a644e725` |
| `build/compact/syllaslopos.iso` | 2,877,440 | `fdf72129ed02dc58eebdf1d903029d2a1af3b86a0a11cc509e1efb28b23b64d7` |
| `build/recovery/syllaslopos.iso` | 2,877,440 | `06ab2db5be711abe8bfca12c7b1e69638b8554c207d0e02504ad7461d6df45eb` |

ELF entry `0x100030`, five load segments (RX, RW, RX, R, RW), no relocation
records and no undefined symbols. `size` reported 142,842 bytes of text and
read-only data, 648 bytes of initialized data and 90,112 bytes of BSS,
including a 64 KiB boot stack. Total linked section sizes were 233,602 bytes.
The firmware framebuffer is a device aperture outside this total.

The guest's `metrics` counted 7,270,564 bytes written for the first 1024×768
frame and 340,536 for the last prompt redraw before the metrics command
(about 4.7% of the first frame's writes). The compact equivalents were
4,330,304 and 263,480 bytes. These are actual volatile write volumes,
including overdraw. They are not host CPU timing or display latency.

Known constraints remain in the [console guide](docs/CONSOLE.md). Input polling
keeps the boot CPU awake. Power command confirmation is implemented; these
guest checks did not exercise reset or halt. The next milestone is an IDT
and fault diagnostics, followed by input IRQs, reliable timing, and protected
memory ownership.
