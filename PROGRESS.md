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

## Next milestone · memory ownership

The IDT, fault tests and guard pages are in place (see SLC 0.3 below).
Next: validate the Multiboot2 info structure in an explicitly mapped range
before parsing its memory information, apply per-section page protections,
drop the low identity alias, and track live boot state before handing any
eligible frames to an allocator. Then PIC/APIC setup and input IRQs.

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

## Minimal console redesign · 2026-11-21

The redesign was built from local source commit
`2c22fc76d41b7df6c6c89a8c1affba7feb374a5e`, tree
`447cc2602ef00c4eb68e7300aab4a9f1d4d49c55`. It follows a research pass
through the pinned OpenCode v2.3.2 CLI source and a comparison of two
implemented VGA compositions. The rationale and source links are in
[OpenCode v2 research](docs/OPENCODE-V2-RESEARCH.md).

| Evidence | Result |
| --- | --- |
| Original `make test` boot check | Passed with the same serial entry marker |
| Host tests | 13 passed, including layout bounds and reveal endpoints |
| 1024×768 and 800×600 VGA | Real PS/2 command editing, completion, help, history, system information, zoom, clear and theme all passed |
| Motion | QMP sampled 8 distinct frames at 1024×768 and 7 at 800×600, including the settled states. Subsequent idle frames were identical. `motion off` produced identical immediate and settled captures. |
| Recovery | COM1 commands passed in both a recovery ISO and the default ISO booted with `-vga none` |
| Independent clean build | ELF and default ISO matched byte for byte |

The idle screenshot has only a text wordmark, prompt, placeholder and caret.
The exact flat background occupies 99.8791% at 1024×768 and 99.8019% at
800×600. The previous default idle used its darkest background at 23.99%.
On the compact viewport the fixed-center composition holds nine default-scale
activity rows. The selected layout holds eighteen. Both are genuine guest
captures in the deliverable.

The last prompt update before the `metrics` command wrote 7,032 framebuffer
bytes at both sizes. Previously the same command path reported 340,536 bytes
at 1024×768 and 263,480 at 800×600. These values measure pixel writes
including clearing and overdraw. They do not measure input latency or VM
CPU consumption. The renderer stops scheduling frames after each reveal.
The kernel still polls input while idle.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `build/kernel.elf` | 141,504 | `fe40426f554652e3a45cf5913ec58548ef70cfcad71a9cadbb35e7d24a82b8fe` |
| `build/syllaslopos.iso` | 2,852,864 | `01141c4a095570a7b7a106bbb0d95d0abacb4440926c57408c9b7ec37d0fa43d` |
| `build/compact/syllaslopos.iso` | 2,852,864 | `784e1a98a4fec97d9a14648afb1c1f10d9b8068a672c667ef8d36803d393cbda` |
| `build/recovery/syllaslopos.iso` | 2,852,864 | `a57c32f8eeab20346d112830e42e14e82a151e552c4da9bbd970548bdcddd8f1` |

`make -j4 test test-console` ran with the QEMU and SeaBIOS identities
recorded above. For the no-VGA case, run the existing recovery probe with
`--qemu qemu-system-x86_64 -vga none` against the default ISO. This boot
and timer path is verified on the BIOS QEMU guest.

## SLC 0.2 · interactive command tools

All transition and animation clock code was removed. The native console
adds a ranked command finder (Ctrl+P), searchable boot-local history
(Ctrl+R), inline completion, checked integer arithmetic and base formatting,
and a paged view of the Multiboot physical memory map.

`make -j4 test test-console` passed with 15 host tests, real PS/2 input
at 1024×768 and 800×600, and COM1 recovery. The test opens the palette,
stages a command, reads actual guest memory ranges, accepts a completion,
computes a result, and recalls a filtered command. The immediate and
later system-info captures match. The default ISO booted with VGA disabled
and passed the same serial recovery checks.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `build/kernel.elf` | 151,704 | `fc7367cc007514af793aa7257cac980536098a926689414c11ec600adfc0fd0d` |
| `build/syllaslopos.iso` | 2,863,104 | `7e29d75ac3bf67ce4845f0f583867abb54b84a465251fc5199bc720dfea48597` |

## SLC 0.3 · exceptions, guard pages, vector text, CI

**Status: written, not yet built or booted.** This change was prepared
without a Rust toolchain or QEMU. Run `make -j4 test-host test
test-faults test-console` (or let CI do it) and record the results and
artifact hashes here before treating it as verified.

| Area | Change |
| --- | --- |
| Exceptions | Kernel GDT and TSS, IDT for vectors 0-31, IST stacks for #DF, NMI and #MC. `#BP` returns; other vectors print a register dump to COM1 and on a red crash screen, then halt. |
| Panics | The panic handler reports file, line, column and message through the same crash path. |
| Guard pages | The first 2 MiB leaf is split into 4 KiB pages. Low memory up to 64 KiB and the page under the kernel stack are unmapped. |
| Fault test | `fault int3` resumes. `fault ud`, `de`, `gp`, `null` and `stack` must produce the matching report. `make test-faults` boots once per kind. |
| Framebuffer | PAT entry 7 is programmed to write-combining for the aperture, with `sfence` after each frame. UC fallback without PAT. |
| Text | Bitmap masks replaced with DejaVu Sans Mono outlines rasterized at each scale (16 vertical samples, exact horizontal coverage, cached per glyph). Zoom 2 is no longer pixel-doubled. |
| Boot check | `make test` also requires `SYLLASLOPOS: exceptions armed, ...` after the entry marker. |
| CI | `.github/workflows/ci.yml` builds and runs host tests, the boot check, fault checks and console checks on Ubuntu 24.04, and uploads images, logs and screenshots. |

Expected changes to recorded numbers: kernel size and `.bss` grow (glyph
cache of about 135 KiB, three 32 KiB IST stacks, a 4 KiB guard and page
table). Framebuffer write counts in `metrics` change because glyph
coverage differs from the old masks.
