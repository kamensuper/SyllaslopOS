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
