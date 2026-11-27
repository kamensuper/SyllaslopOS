# Boot contract

SyllaslopOS uses GRUB Multiboot2 with a BIOS El Torito ISO. The entry stub
initializes x86-64 long mode and calls Rust using SysV. The Rust target stays
`x86_64-unknown-none` with a static higher-half code model and an aborting
panic handler. QEMU TCG and SeaBIOS are the exercised firmware path.

## Entry and boot memory

1. GRUB loads `/boot/kernel.elf`. Its Multiboot2 header is at file offset
   `0x1000` and includes an optional framebuffer request. The ISO selects
   the mode after the GRUB `multiboot2` command processes that request.
2. Assembly verifies the boot magic in EAX, preserves the physical information
   address from EBX, enables PAE, and establishes its own GDT and CR3.
   A higher-half stack of 64 KiB and a 16-byte aligned pre-call RSP establish
   the Rust entry boundary. EFER.NXE is enabled on supporting processors.
3. The first 32 MiB are visible at VA zero and at `0xffffffff80000000`.
   Those temporary 2 MiB supervisor leaves cover entry, the stack, active
   tables, and bootloader data. ELF load segments independently separate RX,
   RW and read-only areas. The loader bytes are immutable under this kernel.
4. Rust checks the stack and handoff before emitting the original COM1
   entry marker. It validates the Multiboot blob within the low window
   with a 1 MiB cap, alignment, tag bounds and a trailing end record.
   Missing or malformed optional information leaves COM1 available.

The boot aliases remain writable and executable. The kernel keeps all
boot tables and loader bytes reserved. Maskable interrupts remain off.

## Exceptions and guard pages

5. Right after the entry marker, `interrupts.rs` loads a kernel GDT with a
   64-bit TSS and an IDT for vectors 0-31. Double fault, NMI and machine
   check use IST stacks of 32 KiB each. `#BP` returns; every other vector
   ends in `crash.rs`, which writes a register dump to COM1, paints it on
   the framebuffer when one is attached, and halts.
6. `paging.rs` then replaces the first 2 MiB leaf, shared by the low and
   higher-half aliases, with a 4 KiB page table. Physical pages below
   64 KiB (or below the boot information page, if GRUB put it lower) and
   the page under the 64 KiB kernel stack (`__stack_guard`) are not
   present. A null dereference raises `#PF`. A stack overflow raises `#PF`
   that cannot push its frame, which escalates to `#DF` on IST1. The
   linker asserts that the kernel ends inside this first 2 MiB.
7. COM1 then reports `SYLLASLOPOS: exceptions armed, ...`, which
   `make test` requires after the entry marker.

Still to do before publishing general RAM: per-section protections
(read-only text and rodata, NX data), dropping the low identity alias,
and a record of the live boot mappings and references.

## Framebuffer aperture

The renderer accepts direct RGB with 24 or 32 bits per pixel. It checks extent,
pitch, 32-bit alignment, channel widths and overlap, and the byte span of the
padded framebuffer. Early BIOS graphics are scoped to the physical
`[0xc0000000, 0x100000000)` PCI window and must not overlap non-reserved
ranges in the firmware map after 2 MiB rounding. At most 64 MiB are mapped
into the dedicated virtual window at `0xffffff0000000000`. The leaves use
write-combining memory, through PAT entry 7, which the kernel programs to
WC (no boot mapping selects entry 7). CPUs without PAT fall back to
uncached memory. The console issues `sfence` after each frame. The leaves also
disable instruction fetch on supporting CPUs. This
aperture has no alias in the low RAM window. The kernel draws directly and
uses the reported pitch. Invalid modes and the `serial` boot argument
select COM1 recovery.

- [Multiboot2 boot information format](https://www.gnu.org/software/grub/manual/multiboot2/html_node/Boot-information-format.html)
- [Rust bare-metal x86-64 target](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html)
