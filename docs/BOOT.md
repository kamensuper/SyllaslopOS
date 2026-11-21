# Boot contract

SyllaslopOS uses GRUB Multiboot2 with a BIOS El Torito ISO. The entry stub
initializes x86-64 long mode and calls Rust using SysV. The Rust target stays
`x86_64-unknown-none` with a static higher-half code model and an aborting
panic handler that reports on COM1. QEMU TCG and SeaBIOS are the exercised
firmware path.

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
Before publishing general RAM, add guarded stacks, per-section
protections, and a record of the live boot mappings and references.

## Faults and diagnostics

`kernel/traps.rs` loads an IDT with gates for vectors 0 to 31 as soon as
COM1 is ready. No exception is recoverable yet: each one writes its vector,
error code, RIP, RSP, RFLAGS, and CR2 for a page fault to COM1, then halts.
A Rust panic writes its source location and message the same way. Both end
with `SYLLASLOPOS: halted`.

The gates use the interrupted stack because there is no TSS. A kernel stack
overflow therefore still ends in a triple fault; reporting it needs a TSS
with an IST stack for the double-fault gate.

A rejected Multiboot2 blob, framebuffer, or keyboard controller is logged
on COM1 with the check that refused it before the console starts.

## Framebuffer aperture

The renderer accepts direct RGB with 24 or 32 bits per pixel. It checks extent,
pitch, 32-bit alignment, channel widths and overlap, and the byte span of the
padded framebuffer. Early BIOS graphics are scoped to the physical
`[0xc0000000, 0x100000000)` PCI window and must not overlap non-reserved
ranges in the firmware map after 2 MiB rounding. At most 64 MiB are mapped
into the dedicated virtual window at `0xffffff0000000000`. The leaves use
uncached memory and disable instruction fetch on supporting CPUs. This
aperture has no alias in the low RAM window. The kernel draws directly and
uses the reported pitch. Invalid modes and the `serial` boot argument
select COM1 recovery.

- [Multiboot2 boot information format](https://www.gnu.org/software/grub/manual/multiboot2/html_node/Boot-information-format.html)
- [Rust bare-metal x86-64 target](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html)
