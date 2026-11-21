.set MULTIBOOT2_HEADER_MAGIC, 0xe85250d6
.set MULTIBOOT2_BOOT_MAGIC, 0x36d76289
.set CODE64_SELECTOR, 0x08
.set DATA_SELECTOR, 0x10

.section .multiboot2,"a",@progbits
.balign 8
header_start:
    .long MULTIBOOT2_HEADER_MAGIC
    .long 0                        # i386 Multiboot2 entry ABI
    .long header_end - header_start
    .long -(MULTIBOOT2_HEADER_MAGIC + (header_end - header_start))
    .short 0, 0
    .long 8                        # end tag
header_end:

.section .boot.text,"ax",@progbits
.code32
.global _start
.type _start, @function
_start:
    cli
    cld
    cmpl $MULTIBOOT2_BOOT_MAGIC, %eax
    jne .Lhalt32
    movl %eax, %ebp
    movl %ebx, %esi

    movl $0x80000000, %eax
    cpuid
    cmpl $0x80000001, %eax
    jb .Lhalt32
    movl $0x80000001, %eax
    cpuid
    testl $(1 << 29), %edx         # long mode
    jz .Lhalt32

    # The ELF loader zeroes these NOBITS pages. Build two aliases of the
    # same 2 MiB frame: VA 0 and VA 0xffffffff80000000.
    movl $low_pdpt + 3, pml4
    movl $high_pdpt + 3, pml4 + 511 * 8
    movl $page_directory + 3, low_pdpt
    movl $page_directory + 3, high_pdpt + 510 * 8
    movl $0x83, page_directory    # present | writable | large page

    lgdt gdt_pointer
    movl $(1 << 5), %eax          # PAE, four-level paging
    movl %eax, %cr4
    movl $pml4, %eax
    movl %eax, %cr3

    movl $0xc0000080, %ecx        # EFER
    rdmsr
    orl $(1 << 8), %eax           # LME
    wrmsr

    movl %cr0, %eax
    orl $((1 << 31) | (1 << 16)), %eax  # PG | WP
    movl %eax, %cr0
    ljmp $CODE64_SELECTOR, $.Lentry64

.Lhalt32:
    hlt
    jmp .Lhalt32

.code64
.Lentry64:
    movw $DATA_SELECTOR, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %ss
    xorw %ax, %ax
    movw %ax, %fs
    movw %ax, %gs

    movabs $__stack_top, %rsp
    movl %ebp, %edi               # SysV argument 1: Multiboot2 magic
    movl %esi, %esi               # SysV argument 2: zero-extended physical address
    xorl %ebp, %ebp
    movabs $kernel_main, %rax
    call *%rax
.Lhalt64:
    hlt
    jmp .Lhalt64
.size _start, . - _start

.section .boot.rodata,"a",@progbits
.balign 8
gdt:
    .quad 0
    .quad 0x00af9b000000ffff       # long-mode code, accessed
    .quad 0x00cf93000000ffff       # writable data, accessed
gdt_end:
gdt_pointer:
    .word gdt_end - gdt - 1
    .long gdt

.section .boot.bss,"aw",@nobits
.balign 4096
pml4:
    .skip 4096
low_pdpt:
    .skip 4096
high_pdpt:
    .skip 4096
page_directory:
    .skip 4096

.section .note.GNU-stack,"",@progbits
