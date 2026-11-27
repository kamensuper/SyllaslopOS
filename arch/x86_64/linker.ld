OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

KERNEL_BASE = 0xffffffff80000000;
PAGE_SIZE = 0x1000;

PHDRS
{
    entry PT_LOAD FLAGS(5);
    tables PT_LOAD FLAGS(6);
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0x100000;
    .boot : {
        KEEP(*(.multiboot2))
        *(.boot.text .boot.rodata)
    } :entry
    .boot.bss ALIGN(PAGE_SIZE) (NOLOAD) : {
        *(.boot.bss)
    } :tables

    . += KERNEL_BASE;
    .text ALIGN(PAGE_SIZE) : AT(ADDR(.text) - KERNEL_BASE) {
        *(.text .text.*)
    } :text
    .rodata ALIGN(PAGE_SIZE) : AT(ADDR(.rodata) - KERNEL_BASE) {
        *(.rodata .rodata.*)
    } :rodata
    .data ALIGN(PAGE_SIZE) : AT(ADDR(.data) - KERNEL_BASE) {
        *(.data .data.* .got .got.*)
    } :data
    .bss ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.bss) - KERNEL_BASE) {
        *(.bss .bss.* COMMON)
        /* paging.rs unmaps this page, so a stack overflow faults instead of
           silently overwriting the data below the stack. */
        . = ALIGN(PAGE_SIZE);
        __stack_guard = .;
        . += PAGE_SIZE;
        __stack_bottom = .;
        . += 0x10000;
        __stack_top = .;
    } :data

    /DISCARD/ : { *(.eh_frame .eh_frame.* .note.GNU-stack .comment) }
}

ASSERT((__stack_top - KERNEL_BASE) <= 0x200000,
       "the kernel must end inside the first 2 MiB, which paging.rs splits into 4 KiB pages")
