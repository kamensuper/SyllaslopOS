//! Fatal crash reports for CPU exceptions and Rust panics.
//!
//! The report goes to COM1 first, because the serial path has the fewest
//! dependencies, and then to the framebuffer when one is attached. A fault
//! inside the crash path itself halts immediately instead of recursing.

use core::arch::asm;
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::interrupts::Frame;
use crate::paging::{Guards, KERNEL_BASE};
use crate::serial::Com1;

const LINES: usize = 24;
const WIDTH: usize = 96;
const PAGE: u64 = 4096;

static CRASHED: AtomicBool = AtomicBool::new(false);
static NULL_END: AtomicU64 = AtomicU64::new(0);
static STACK_GUARD: AtomicU64 = AtomicU64::new(0);

/// Remember the guard ranges so page faults can be explained.
pub fn set_guards(guards: Guards) {
    NULL_END.store(guards.null_end, Ordering::Relaxed);
    STACK_GUARD.store(guards.stack_guard, Ordering::Relaxed);
}

/// Fixed-size text buffer, so reporting needs no heap and little stack.
struct Report {
    text: [[u8; WIDTH]; LINES],
    len: [usize; LINES],
    line: usize,
}

impl Report {
    const fn new() -> Self {
        Self {
            text: [[0; WIDTH]; LINES],
            len: [0; LINES],
            line: 0,
        }
    }

    fn lines(&self) -> impl Iterator<Item = &[u8]> + '_ {
        let open = self.line < LINES && self.len[self.line] > 0;
        let count = self.line.min(LINES) + usize::from(open);
        (0..count).map(|index| &self.text[index][..self.len[index]])
    }
}

impl Write for Report {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                self.line = (self.line + 1).min(LINES);
                continue;
            }
            if self.line >= LINES {
                continue;
            }
            let len = &mut self.len[self.line];
            if *len < WIDTH {
                self.text[self.line][*len] = if byte.is_ascii_graphic() || byte == b' ' {
                    byte
                } else {
                    b'?'
                };
                *len += 1;
            }
        }
        Ok(())
    }
}

pub fn exception(frame: &Frame) -> ! {
    let mut report = Report::new();
    let _ = describe(&mut report, frame);
    finish(&report)
}

pub fn panic(info: &PanicInfo<'_>) -> ! {
    let mut report = Report::new();
    let _ = match info.location() {
        Some(at) => writeln!(
            report,
            "SYLLASLOPOS: PANIC at {}:{}:{}",
            at.file(),
            at.line(),
            at.column()
        ),
        None => writeln!(report, "SYLLASLOPOS: PANIC"),
    };
    let _ = writeln!(report, "{}", info.message());
    let _ = control_registers(&mut report);
    finish(&report)
}

fn finish(report: &Report) -> ! {
    if CRASHED.swap(true, Ordering::AcqRel) {
        crate::halt();
    }
    // SAFETY: this path never returns to the console that owns COM1.
    let mut serial = unsafe { Com1::steal() };
    serial.write(b"\r\n");
    for line in report.lines() {
        serial.write(line);
        serial.write(b"\r\n");
    }
    serial.write(b"SYLLASLOPOS: halted\r\n");
    // SAFETY: as above, nothing resumes the framebuffer's owner.
    unsafe { crate::graphics::crash_screen(report.lines()) };
    crate::halt()
}

fn describe(out: &mut Report, f: &Frame) -> fmt::Result {
    let (mnemonic, name) = exception_name(f.vector);
    writeln!(
        out,
        "SYLLASLOPOS: CPU EXCEPTION #{mnemonic} {name} (vector {})",
        f.vector
    )?;
    if pushes_error_code(f.vector) {
        write!(out, "error {:#018x}", f.error)?;
        if f.vector == 14 {
            write!(
                out,
                "  {} {} {}{}{}",
                if f.error & 1 != 0 { "protection" } else { "not-present" },
                if f.error & 2 != 0 { "write" } else { "read" },
                if f.error & 4 != 0 { "user" } else { "supervisor" },
                if f.error & 8 != 0 { " reserved-bit" } else { "" },
                if f.error & 16 != 0 { " fetch" } else { "" },
            )?;
        }
        writeln!(out)?;
    }
    if matches!(f.vector, 8 | 14) {
        let cr2 = read_cr2();
        writeln!(out, "cr2   {cr2:#018x}  {}", classify(cr2))?;
    }
    if f.vector == 8 {
        writeln!(out, "note  the saved RIP of a double fault is architecturally undefined")?;
    }
    writeln!(
        out,
        "rip   {:#018x}  cs {:#06x}  rflags {:#010x}",
        f.rip, f.cs, f.rflags
    )?;
    writeln!(out, "rsp   {:#018x}  ss {:#06x}", f.rsp, f.ss)?;
    writeln!(out, "rax {:#018x}  rbx {:#018x}  rcx {:#018x}", f.rax, f.rbx, f.rcx)?;
    writeln!(out, "rdx {:#018x}  rsi {:#018x}  rdi {:#018x}", f.rdx, f.rsi, f.rdi)?;
    writeln!(out, "rbp {:#018x}  r8  {:#018x}  r9  {:#018x}", f.rbp, f.r8, f.r9)?;
    writeln!(out, "r10 {:#018x}  r11 {:#018x}  r12 {:#018x}", f.r10, f.r11, f.r12)?;
    writeln!(out, "r13 {:#018x}  r14 {:#018x}  r15 {:#018x}", f.r13, f.r14, f.r15)?;
    control_registers(out)
}

fn control_registers(out: &mut Report) -> fmt::Result {
    let (cr0, cr3, cr4): (u64, u64, u64);
    // SAFETY: reading control registers has no side effects at CPL0.
    unsafe {
        asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
        asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
        asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
    }
    writeln!(out, "cr0 {cr0:#018x}  cr3 {cr3:#018x}  cr4 {cr4:#018x}")
}

fn read_cr2() -> u64 {
    let cr2: u64;
    // SAFETY: reading CR2 has no side effects at CPL0.
    unsafe { asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags)) };
    cr2
}

fn classify(address: u64) -> &'static str {
    let guard = STACK_GUARD.load(Ordering::Relaxed);
    if address < NULL_END.load(Ordering::Relaxed) {
        "null pointer guard"
    } else if guard != 0 && (guard..guard + PAGE).contains(&address) {
        "kernel stack guard (stack overflow)"
    } else if address >= KERNEL_BASE {
        "kernel image"
    } else if (0x0000_8000_0000_0000..0xffff_8000_0000_0000).contains(&address) {
        "non-canonical"
    } else {
        ""
    }
}

fn pushes_error_code(vector: u64) -> bool {
    matches!(vector, 8 | 10..=14 | 17 | 21 | 29 | 30)
}

fn exception_name(vector: u64) -> (&'static str, &'static str) {
    match vector {
        0 => ("DE", "divide error"),
        1 => ("DB", "debug"),
        2 => ("NMI", "non-maskable interrupt"),
        3 => ("BP", "breakpoint"),
        4 => ("OF", "overflow"),
        5 => ("BR", "bound range exceeded"),
        6 => ("UD", "invalid opcode"),
        7 => ("NM", "device not available"),
        8 => ("DF", "double fault"),
        9 => ("CSO", "coprocessor segment overrun"),
        10 => ("TS", "invalid TSS"),
        11 => ("NP", "segment not present"),
        12 => ("SS", "stack-segment fault"),
        13 => ("GP", "general protection"),
        14 => ("PF", "page fault"),
        16 => ("MF", "x87 floating-point error"),
        17 => ("AC", "alignment check"),
        18 => ("MC", "machine check"),
        19 => ("XM", "SIMD floating-point error"),
        20 => ("VE", "virtualization exception"),
        21 => ("CP", "control protection"),
        28 => ("HV", "hypervisor injection"),
        29 => ("VC", "VMM communication"),
        30 => ("SX", "security exception"),
        _ => ("??", "reserved vector"),
    }
}
