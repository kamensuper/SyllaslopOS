//! Kernel GDT, TSS and IDT for the 32 CPU exception vectors.
//!
//! Every exception enters a common assembly stub that saves the general
//! registers and calls `syllaslop_exception`. A breakpoint (#BP) is a trap
//! and returns to the instruction after `int3`. Every other vector is fatal
//! and produces a crash report on COM1 and the framebuffer.
//!
//! Double fault, NMI and machine check run on dedicated IST stacks, so a
//! kernel stack overflow (a #PF on the guard page that cannot push its own
//! frame) still reaches a working handler as #DF.

use core::arch::{asm, global_asm};
use core::cell::UnsafeCell;
use core::mem::size_of;

pub const KERNEL_CODE: u16 = 0x08;
pub const KERNEL_DATA: u16 = 0x10;
const TSS_SELECTOR: u16 = 0x18;
const IST_BYTES: usize = 32 * 1024;
const INTERRUPT_GATE: u8 = 0x8e;
const VECTORS: usize = 32;

/// Register state saved by the stubs, lowest address first.
#[repr(C)]
pub struct Frame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub vector: u64,
    pub error: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

// Vectors 8, 10-14, 17, 21, 29 and 30 push an error code. The others get a
// zero so that every frame has the same layout. Before the call, 22 quad
// words sit above the 16-byte aligned interrupt RSP, which keeps the SysV
// call alignment.
global_asm!(
    r#"
.pushsection .text.exceptions,"ax",@progbits
.macro SYLLASLOP_STUB vector, code
syllaslop_isr_\vector:
.if \code == 0
    pushq $0
.endif
    pushq $\vector
    jmp syllaslop_isr_common
.endm

SYLLASLOP_STUB 0, 0
SYLLASLOP_STUB 1, 0
SYLLASLOP_STUB 2, 0
SYLLASLOP_STUB 3, 0
SYLLASLOP_STUB 4, 0
SYLLASLOP_STUB 5, 0
SYLLASLOP_STUB 6, 0
SYLLASLOP_STUB 7, 0
SYLLASLOP_STUB 8, 1
SYLLASLOP_STUB 9, 0
SYLLASLOP_STUB 10, 1
SYLLASLOP_STUB 11, 1
SYLLASLOP_STUB 12, 1
SYLLASLOP_STUB 13, 1
SYLLASLOP_STUB 14, 1
SYLLASLOP_STUB 15, 0
SYLLASLOP_STUB 16, 0
SYLLASLOP_STUB 17, 1
SYLLASLOP_STUB 18, 0
SYLLASLOP_STUB 19, 0
SYLLASLOP_STUB 20, 0
SYLLASLOP_STUB 21, 1
SYLLASLOP_STUB 22, 0
SYLLASLOP_STUB 23, 0
SYLLASLOP_STUB 24, 0
SYLLASLOP_STUB 25, 0
SYLLASLOP_STUB 26, 0
SYLLASLOP_STUB 27, 0
SYLLASLOP_STUB 28, 0
SYLLASLOP_STUB 29, 1
SYLLASLOP_STUB 30, 1
SYLLASLOP_STUB 31, 0

syllaslop_isr_common:
    pushq %rax
    pushq %rbx
    pushq %rcx
    pushq %rdx
    pushq %rsi
    pushq %rdi
    pushq %rbp
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    movq %rsp, %rdi
    cld
    call syllaslop_exception
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rbp
    popq %rdi
    popq %rsi
    popq %rdx
    popq %rcx
    popq %rbx
    popq %rax
    addq $16, %rsp
    iretq
.popsection

.pushsection .rodata.exceptions,"a",@progbits
.balign 8
.global syllaslop_isr_table
syllaslop_isr_table:
    .quad syllaslop_isr_0, syllaslop_isr_1, syllaslop_isr_2, syllaslop_isr_3
    .quad syllaslop_isr_4, syllaslop_isr_5, syllaslop_isr_6, syllaslop_isr_7
    .quad syllaslop_isr_8, syllaslop_isr_9, syllaslop_isr_10, syllaslop_isr_11
    .quad syllaslop_isr_12, syllaslop_isr_13, syllaslop_isr_14, syllaslop_isr_15
    .quad syllaslop_isr_16, syllaslop_isr_17, syllaslop_isr_18, syllaslop_isr_19
    .quad syllaslop_isr_20, syllaslop_isr_21, syllaslop_isr_22, syllaslop_isr_23
    .quad syllaslop_isr_24, syllaslop_isr_25, syllaslop_isr_26, syllaslop_isr_27
    .quad syllaslop_isr_28, syllaslop_isr_29, syllaslop_isr_30, syllaslop_isr_31
.popsection
"#,
    options(att_syntax)
);

extern "C" {
    static syllaslop_isr_table: [u64; VECTORS];
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Gate {
    offset_low: u16,
    selector: u16,
    ist: u8,
    kind: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl Gate {
    const MISSING: Self = Self {
        offset_low: 0,
        selector: 0,
        ist: 0,
        kind: 0,
        offset_mid: 0,
        offset_high: 0,
        reserved: 0,
    };

    fn new(handler: u64, ist: u8) -> Self {
        Self {
            offset_low: handler as u16,
            selector: KERNEL_CODE,
            ist,
            kind: INTERRUPT_GATE,
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, packed(4))]
struct Tss {
    reserved0: u32,
    rsp: [u64; 3],
    reserved1: u64,
    ist: [u64; 7],
    reserved2: u64,
    reserved3: u16,
    iomap: u16,
}

#[repr(C, packed)]
struct Pointer {
    limit: u16,
    base: u64,
}

#[repr(C, align(16))]
struct Tables {
    gdt: [u64; 5],
    tss: Tss,
    idt: [Gate; 256],
}

#[repr(C, align(16))]
struct Stacks([[u8; IST_BYTES]; 3]);

struct Cell<T>(UnsafeCell<T>);

// SAFETY: `install` is the only writer, once, on the boot CPU before any
// other code can observe these tables. The CPU reads them afterwards.
unsafe impl<T> Sync for Cell<T> {}

static TABLES: Cell<Tables> = Cell(UnsafeCell::new(Tables {
    gdt: [0; 5],
    tss: Tss {
        reserved0: 0,
        rsp: [0; 3],
        reserved1: 0,
        ist: [0; 7],
        reserved2: 0,
        reserved3: 0,
        iomap: 0,
    },
    idt: [Gate::MISSING; 256],
}));
static STACKS: Cell<Stacks> = Cell(UnsafeCell::new(Stacks([[0; IST_BYTES]; 3])));

/// Build and load the kernel GDT, TSS and IDT.
///
/// Call once on the boot CPU, at CPL0, with interrupts masked and the
/// higher-half kernel mapping active.
pub unsafe fn install() {
    let tables = &mut *TABLES.0.get();
    let stacks = STACKS.0.get() as u64;
    let top = |index: u64| stacks + (index + 1) * IST_BYTES as u64;

    tables.tss = Tss {
        reserved0: 0,
        rsp: [0; 3],
        reserved1: 0,
        // IST1 double fault, IST2 NMI, IST3 machine check.
        ist: [top(0), top(1), top(2), 0, 0, 0, 0],
        reserved2: 0,
        reserved3: 0,
        iomap: size_of::<Tss>() as u16,
    };
    let base = core::ptr::addr_of!(tables.tss) as u64;
    let limit = (size_of::<Tss>() - 1) as u64;
    tables.gdt = [
        0,
        0x00af_9b00_0000_ffff, // 64-bit code, DPL0
        0x00cf_9300_0000_ffff, // writable data, DPL0
        (limit & 0xffff)
            | ((base & 0xff_ffff) << 16)
            | (0x89 << 40) // present, available 64-bit TSS
            | (((limit >> 16) & 0xf) << 48)
            | (((base >> 24) & 0xff) << 56),
        base >> 32,
    ];
    for (vector, gate) in tables.idt.iter_mut().take(VECTORS).enumerate() {
        let ist = match vector {
            8 => 1,
            2 => 2,
            18 => 3,
            _ => 0,
        };
        *gate = Gate::new(syllaslop_isr_table[vector], ist);
    }

    let gdt = Pointer {
        limit: (size_of::<[u64; 5]>() - 1) as u16,
        base: tables.gdt.as_ptr() as u64,
    };
    asm!(
        "lgdt [{gdt}]",
        "push {code}",
        "lea {tmp}, [rip + 55f]",
        "push {tmp}",
        "retfq",
        "55:",
        "mov ds, {data:x}",
        "mov es, {data:x}",
        "mov ss, {data:x}",
        "ltr {tss:x}",
        gdt = in(reg) &gdt,
        code = in(reg) u64::from(KERNEL_CODE),
        data = in(reg) u32::from(KERNEL_DATA),
        tss = in(reg) u32::from(TSS_SELECTOR),
        tmp = out(reg) _,
    );
    let idt = Pointer {
        limit: (size_of::<[Gate; 256]>() - 1) as u16,
        base: tables.idt.as_ptr() as u64,
    };
    asm!("lidt [{}]", in(reg) &idt, options(readonly, nostack, preserves_flags));
}

#[no_mangle]
extern "C" fn syllaslop_exception(frame: &mut Frame) {
    if frame.vector == 3 {
        // #BP is a trap: the saved RIP already follows the int3.
        return;
    }
    crate::crash::exception(frame)
}

// The CPU defines these layouts; check them at compile time.
const _: () = assert!(size_of::<Gate>() == 16);
const _: () = assert!(size_of::<Tss>() == 104);
const _: () = assert!(size_of::<Pointer>() == 10);
const _: () = assert!(size_of::<Frame>() == 22 * 8);
