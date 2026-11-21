//! CPU exception reporting for the boot CPU.
//!
//! The kernel runs with maskable interrupts off and has no recoverable
//! exceptions yet, so every vector reports its machine state on COM1 and
//! stops. Without these gates a fault escalates to a triple fault and the
//! machine resets with no trace.
use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::ptr::{addr_of, addr_of_mut};

use crate::serial::Com1;

const VECTORS: usize = 32;
const STUB_BYTES: u64 = 16;
const CODE_SELECTOR: u16 = 0x08;
const INTERRUPT_GATE: u8 = 0x8e;

// One fixed-size stub per vector gives every exception the same frame: the
// CPU's error code (or a zero in its place) below the vector number.
global_asm!(
    r#"
.macro trap_stub vector, pushes_error
    .balign 16
    .if \pushes_error == 0
    push 0
    .endif
    push \vector
    jmp trap_common
.endm

.section .text.traps,"ax",@progbits
.balign 16
.global trap_stubs
trap_stubs:
    trap_stub 0, 0
    trap_stub 1, 0
    trap_stub 2, 0
    trap_stub 3, 0
    trap_stub 4, 0
    trap_stub 5, 0
    trap_stub 6, 0
    trap_stub 7, 0
    trap_stub 8, 1
    trap_stub 9, 0
    trap_stub 10, 1
    trap_stub 11, 1
    trap_stub 12, 1
    trap_stub 13, 1
    trap_stub 14, 1
    trap_stub 15, 0
    trap_stub 16, 0
    trap_stub 17, 1
    trap_stub 18, 0
    trap_stub 19, 0
    trap_stub 20, 0
    trap_stub 21, 1
    trap_stub 22, 0
    trap_stub 23, 0
    trap_stub 24, 0
    trap_stub 25, 0
    trap_stub 26, 0
    trap_stub 27, 0
    trap_stub 28, 0
    trap_stub 29, 1
    trap_stub 30, 1
    trap_stub 31, 0

trap_common:
    cld
    mov rdi, rsp
    and rsp, -16
    call trap_report
    ud2
"#
);

extern "C" {
    static trap_stubs: u8;
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Gate {
    offset_low: u16,
    selector: u16,
    stack_table: u8,
    kind: u8,
    offset_middle: u16,
    offset_high: u32,
    reserved: u32,
}

impl Gate {
    const MISSING: Self = Self::new(0, 0);

    const fn new(handler: u64, kind: u8) -> Self {
        Self {
            offset_low: handler as u16,
            selector: CODE_SELECTOR,
            stack_table: 0,
            kind,
            offset_middle: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, packed)]
struct TablePointer {
    limit: u16,
    base: u64,
}

static mut TABLE: [Gate; VECTORS] = [Gate::MISSING; VECTORS];

/// What the stubs and the CPU leave on the stack, lowest address first.
#[repr(C)]
struct Frame {
    vector: u64,
    error: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

/// Call once on the boot CPU, at CPL0 in long mode, with the entry stub's
/// GDT loaded and before any other code reads the interrupt table.
pub unsafe fn init() {
    let table = addr_of_mut!(TABLE).cast::<Gate>();
    let stubs = addr_of!(trap_stubs) as u64;
    for vector in 0..VECTORS {
        let handler = stubs + vector as u64 * STUB_BYTES;
        table.add(vector).write(Gate::new(handler, INTERRUPT_GATE));
    }
    let pointer = TablePointer {
        limit: (size_of::<[Gate; VECTORS]>() - 1) as u16,
        base: table as u64,
    };
    asm!("lidt [{}]", in(reg) addr_of!(pointer), options(readonly, nostack, preserves_flags));
}

pub fn name(vector: u64) -> &'static str {
    match vector {
        0 => "divide error",
        1 => "debug",
        2 => "non-maskable interrupt",
        3 => "breakpoint",
        4 => "overflow",
        5 => "bound range exceeded",
        6 => "invalid opcode",
        7 => "device not available",
        8 => "double fault",
        10 => "invalid TSS",
        11 => "segment not present",
        12 => "stack-segment fault",
        13 => "general protection fault",
        14 => "page fault",
        16 => "x87 floating-point error",
        17 => "alignment check",
        18 => "machine check",
        19 => "SIMD floating-point error",
        20 => "virtualization exception",
        21 => "control protection exception",
        _ => "reserved vector",
    }
}

fn describe(out: &mut impl Write, frame: &Frame, fault_address: Option<u64>) -> core::fmt::Result {
    write!(
        out,
        "\r\nSYLLASLOPOS: exception {} ({}) error=0x{:x}\r\n",
        frame.vector,
        name(frame.vector),
        frame.error
    )?;
    write!(
        out,
        "  rip=0x{:016x} cs=0x{:x} rflags=0x{:x}\r\n  rsp=0x{:016x} ss=0x{:x}\r\n",
        frame.rip, frame.cs, frame.rflags, frame.rsp, frame.ss
    )?;
    if let Some(address) = fault_address {
        write!(out, "  cr2=0x{address:016x}\r\n")?;
    }
    out.write_str("SYLLASLOPOS: halted\r\n")
}

#[no_mangle]
extern "C" fn trap_report(frame: &Frame) -> ! {
    let fault_address = (frame.vector == 14).then(|| {
        let address: u64;
        // SAFETY: reading CR2 at CPL0 has no side effects.
        unsafe { asm!("mov {}, cr2", out(reg) address, options(nomem, nostack, preserves_flags)) };
        address
    });
    // SAFETY: this path never returns to the interrupted COM1 owner.
    let mut serial = unsafe { Com1::steal() };
    let _ = describe(&mut serial, frame, fault_address);
    crate::halt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gates_split_the_handler_address() {
        let gate = Gate::new(0xffff_ffff_8012_3456, INTERRUPT_GATE);
        assert_eq!(size_of::<Gate>(), 16);
        assert_eq!(size_of::<TablePointer>(), 10);
        assert_eq!(
            (gate.offset_low, gate.offset_middle, gate.offset_high),
            (0x3456, 0x8012, 0xffff_ffff)
        );
        assert_eq!((gate.selector, gate.kind, gate.stack_table), (8, 0x8e, 0));
    }

    #[test]
    fn reports_name_the_vector_and_fault_address() {
        let frame = Frame {
            vector: 14,
            error: 2,
            rip: 0xffff_ffff_8000_1000,
            cs: 8,
            rflags: 0x46,
            rsp: 0xffff_ffff_8002_0000,
            ss: 0x10,
        };
        let mut text = String::new();
        describe(&mut text, &frame, Some(0xdead_b000)).unwrap();
        assert!(text.contains("exception 14 (page fault) error=0x2\r\n"));
        assert!(text.contains("rip=0xffffffff80001000 cs=0x8 rflags=0x46"));
        assert!(text.contains("cr2=0x00000000deadb000"));
        assert!(text.ends_with("SYLLASLOPOS: halted\r\n"));
        assert_eq!(name(2), "non-maskable interrupt");
        assert_eq!(name(31), "reserved vector");
    }
}
